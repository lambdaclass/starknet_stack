use crate::config::{Committee, ConfigError, Parameters, Secret};
use crate::config::{ExecutionParameters, Export as _};
use cairo_felt::Felt252;
use consensus::{Block, Consensus};
use crypto::SignatureService;
use execution_engine::starknet_in_rust_engine::StarknetState;
use log::{error, info};
use mempool::{Mempool, MempoolMessage};
use num_bigint::BigUint;
use rpc_endpoint::rpc::{
    self, InvokeTransaction, InvokeTransactionReceipt, MaybePendingTransactionReceipt, Transaction,
    TransactionReceipt,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use store::Store;
use tokio::sync::mpsc::{channel, Receiver};

/// The default channel capacity for this module.
pub const CHANNEL_CAPACITY: usize = 1_000;

const ROUND_TIMEOUT_FOR_EMPTY_BLOCKS: u64 = 1500;

enum ExecutionEngine {
    StarknetInRust(Box<StarknetState>),
}

impl ExecutionEngine {
    fn handle_invoke(&mut self, calldata: Vec<Felt252>) -> Result<Vec<Felt252>, String> {
        match self {
            ExecutionEngine::StarknetInRust(sir_engine) => {
                sir_engine.invoke(calldata).map_err(|x| x.to_string())
            }
        }
    }
}

pub struct HotstuffNode {
    pub commit: Receiver<Block>,
    pub store: Store,
    pub external_store: sequencer::store::Store,
    pub mempool_transaction_endpoint: SocketAddr,
    execution_program: ExecutionEngine,
    last_committed_round: u64,
    pub transaction_endpoint: SocketAddr,
}

impl HotstuffNode {
    pub async fn new(
        committee_file: &str,
        key_file: &str,
        store_path: &str,
        parameters: Option<String>,
        external_store: sequencer::store::Store,
    ) -> Result<Self, ConfigError> {
        let (tx_commit, rx_commit) = channel(CHANNEL_CAPACITY);
        let (tx_consensus_to_mempool, rx_consensus_to_mempool) = channel(CHANNEL_CAPACITY);
        let (tx_mempool_to_consensus, rx_mempool_to_consensus) = channel(CHANNEL_CAPACITY);

        // Read the committee and secret key from file.
        let committee = Committee::read(committee_file)?;
        let secret = Secret::read(key_file)?;
        let name = secret.name;
        let secret_key = secret.secret;

        // Load default parameters if none are specified.
        let parameters = match parameters {
            Some(filename) => Parameters::read(&filename)?,
            None => Parameters::default(),
        };

        // Make the data store.
        let store = Store::new(store_path).expect("Failed to create store");

        // Init the execution engine according to the parameters sent
        let execution_engine = match parameters.execution {
            ExecutionParameters::CairoVM => {
                unimplemented!(
                    "Cairo VM was disabled in favor of deciding VMs within Starknet in Rust"
                )
            }
            ExecutionParameters::CairoNative => {
                unimplemented!(
                    "Cairo Native was disabled in favor of deciding VMs within Starknet in Rust"
                )
            }
            ExecutionParameters::StarknetInRust => {
                ExecutionEngine::StarknetInRust(Box::new(StarknetState::new_for_tests()))
            }
        };

        // Run the signature service.
        let signature_service = SignatureService::new(secret_key);

        // Make a new mempool.
        Mempool::spawn(
            name,
            committee.clone().mempool,
            parameters.mempool,
            store.clone(),
            rx_consensus_to_mempool,
            tx_mempool_to_consensus,
        );

        // Run the consensus core.
        Consensus::spawn(
            name,
            committee.clone().consensus,
            parameters.consensus,
            signature_service,
            store.clone(),
            rx_mempool_to_consensus,
            tx_consensus_to_mempool,
            tx_commit,
        );

        info!("Hotstuff node {} successfully booted", name);
        Ok(Self {
            commit: rx_commit,
            store,
            mempool_transaction_endpoint: committee
                .mempool
                .mempool_address(&name)
                .expect("Error retrieving our own mempool parameters while initializing"),
            transaction_endpoint: committee
                .mempool
                .transactions_address(&name)
                .expect("Error retrieving our own mempool parameters while initializing"),
            external_store,
            execution_program: execution_engine,
            last_committed_round: 0u64,
        })
    }

    pub fn print_key_file(filename: &str) -> Result<(), ConfigError> {
        Secret::new().write(filename)
    }

    // TODO: This is application code and as such it should not depend on consensus node code, so it should be factored out to an `on_block_commit()` call
    pub async fn analyze_block(&mut self) {
        while let Some(block) = self.commit.recv().await {
            let mut transactions = vec![];

            // This is where we can further process committed block.
            for p in block.payload {
                let tx_batch = self.store.read(p.to_vec()).await.unwrap().unwrap();
                info!("Batch is {} bytes long", tx_batch.len());

                let list_of_tx: MempoolMessage =
                    bincode::deserialize(&tx_batch).expect("Error trying to deserialize batch");

                match list_of_tx {
                    MempoolMessage::Batch(batch_txs) => {
                        info!(
                            "Batch message confirmed, with {} transactions!",
                            batch_txs.len()
                        );

                        for (i, tx_bytes) in batch_txs.into_iter().enumerate() {
                            // Consensus codebase uses the first 9 bytes to track the transaction like this:
                            //
                            // - First byte can be 0 or 1 and represents whether it's a benchmarked tx or standard tx
                            // - Next 8 bytes represent a transaction ID
                            //
                            // If it's a benchmarked tx, it then gets tracked in logs to compute metrics
                            // So we need to strip that section in order to get the starknet transaction to execute
                            #[cfg(feature = "benchmark")]
                            let tx_bytes = &tx_bytes[9..];

                            #[allow(clippy::needless_borrow)]
                            let starknet_tx = rpc::Transaction::from_bytes(&tx_bytes);

                            info!(
                                "Message {i} in {:?} is of tx_type {:?}, executing",
                                p, starknet_tx
                            );

                            match &starknet_tx {
                                Transaction::Invoke(InvokeTransaction::V1(tx)) => {
                                    info!(
                                        "tx hash serialized: {}, decimal {} (hex {})",
                                        serde_json::to_string(&tx.transaction_hash).unwrap(),
                                        &tx.transaction_hash,
                                        &tx.transaction_hash.to_str_radix(16)
                                    );

                                    let execution_result =
                                        self.execution_program.handle_invoke(tx.calldata.clone());

                                    if execution_result.is_ok() {
                                        info!(
                                            "Execution output is: {:?}",
                                            execution_result.unwrap()
                                        );
                                        let _ = self
                                            .external_store
                                            .add_transaction(starknet_tx.clone());
                                    } else {
                                        error!(
                                            "Error running transaction: {}",
                                            execution_result.unwrap_err()
                                        );
                                    }
                                }
                                _ => todo!(),
                            }

                            transactions.push(starknet_tx);
                        }
                    }
                    MempoolMessage::BatchRequest(_, _) => {
                        info!("Batch Request message confirmed")
                    }
                }
            }
            if !transactions.is_empty()
                || (block.round - self.last_committed_round) > ROUND_TIMEOUT_FOR_EMPTY_BLOCKS
            {
                info!("About to store block from round {}", block.round);
                self.last_committed_round = block.round;
                self.create_and_store_new_block(transactions);
            }
        }
    }

    fn create_and_store_new_block(&mut self, transactions: Vec<Transaction>) {
        let height = self
            .external_store
            .get_height()
            .expect("Height value not found")
            + 1;

        let status = rpc_endpoint::rpc::BlockStatus::AcceptedOnL2;
        // TODO: store deserialization should be managed in store logic.
        let parent_block = self.external_store.get_block_by_height(height - 1);

        let parent_hash = parent_block.map_or(Felt252::new(0), |maybe_block| {
            maybe_block.map_or(Felt252::new(0), |block| match block {
                rpc::MaybePendingBlockWithTxs::Block(block) => block.block_hash,
                _ => Felt252::new(0),
            })
        });
        let new_root = Felt252::new(938938281);

        let timestamp: u128 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Timestamp failed")
            .as_secs()
            .into();

        let sequencer_address = Felt252::new(12039102);

        // TODO: This is quick and dirty hashing,
        //       Block hashing should be done in it's own module
        let mut state = DefaultHasher::new();
        status.hash(&mut state);
        parent_hash.hash(&mut state);
        height.hash(&mut state);
        new_root.hash(&mut state);
        sequencer_address.hash(&mut state);
        transactions.iter().for_each(|tx| match &tx {
            Transaction::Invoke(InvokeTransaction::V1(invoke_tx)) => invoke_tx.hash(&mut state),
            _ => todo!(),
        });
        let block_hash = Felt252::new(state.finish());

        let block_with_txs = rpc::MaybePendingBlockWithTxs::Block(rpc::BlockWithTxs {
            status,
            block_hash: block_hash.clone(),
            parent_hash,
            block_number: height,
            new_root,
            timestamp,
            sequencer_address,
            transactions: transactions.clone(),
        });

        _ = self.external_store.add_block(block_with_txs);

        _ = self.external_store.set_height(height);

        transactions.iter().for_each(|tx| match tx {
            Transaction::Invoke(InvokeTransaction::V1(invoke_tx)) => {
                let tx_receipt: InvokeTransactionReceipt = InvokeTransactionReceipt {
                    transaction_hash: invoke_tx.transaction_hash.clone(),
                    actual_fee: invoke_tx.max_fee.clone(),
                    status: rpc::TransactionStatus::AcceptedOnL2,
                    block_hash: block_hash.clone(),
                    block_number: height,
                    messages_sent: vec![],
                    events: vec![],
                };

                _ = self.external_store.add_transaction_receipt(
                    MaybePendingTransactionReceipt::Receipt(TransactionReceipt::Invoke(tx_receipt)),
                );
            }
            _ => todo!(),
        });
    }
}

fn _get_input_value_cairo_native(n: usize) -> Vec<u32> {
    let mut digits = BigUint::from(n).to_u32_digits();
    digits.resize(8, 0);
    digits
}
