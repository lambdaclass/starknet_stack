use crate::config::{Committee, ConfigError, Parameters, Secret};
use crate::config::{ExecutionParameters, Export as _};
use cairo_felt::Felt252;
use consensus::{Block, Consensus};
use crypto::SignatureService;
use execution_engine::cairovm_engine::CairoVMEngine;
use execution_engine::starknet_in_rust_engine::StarknetState;
use log::{error, info};
use mempool::{Mempool, MempoolMessage};
use num_bigint::BigUint;
use rpc_endpoint::new_server;
use rpc_endpoint::rpc::{
    self, InvokeTransaction, InvokeTransactionReceipt, MaybePendingTransactionReceipt, Transaction,
    TransactionReceipt,
};
use std::collections::hash_map::DefaultHasher;
use std::convert::TryInto;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use store::Store;
use tokio::sync::mpsc::{channel, Receiver};

/// The default channel capacity for this module.
pub const CHANNEL_CAPACITY: usize = 1_000;

/// Default port offset for RPC endpoint
const RPC_PORT_OFFSET: u16 = 1000;
const ROUND_TIMEOUT_FOR_EMPTY_BLOCKS: u64 = 1500;

enum ExecutionEngine {
    Cairo(Box<CairoVMEngine>),
    StarknetInRust(Box<StarknetState>),
}

impl ExecutionEngine {
    fn execute_fibonacci(&mut self, n: usize) {
        let ret_msg = match self {
            ExecutionEngine::Cairo(execution_program) => execution_program.execute_fibonacci(n),
            ExecutionEngine::StarknetInRust(state) => format!("{:?}", state.execute_fibonacci(n)),
        };
        info!("{}", ret_msg)
    }

    fn execute_factorial(&mut self, n: usize) {
        let ret_msg = match self {
            ExecutionEngine::Cairo(execution_program) => execution_program.execute_factorial(n),
            ExecutionEngine::StarknetInRust(state) => format!("{:?}", state.execute_factorial(n)),
        };
        info!("{}", ret_msg)
    }

    fn execute_erc20(&mut self, initial_supply: Felt252, symbol: Felt252, contract_address: Felt252) {
        let _ret_msg = match self {
            ExecutionEngine::Cairo(_execution_program) => {
                todo!("Cairo VM does not support ERC20 transactions")
            }
            ExecutionEngine::StarknetInRust(state) => format!("{:?}", state.execute_erc20(initial_supply, symbol, contract_address)),
        };
    }
}

pub struct Node {
    pub commit: Receiver<Block>,
    pub store: Store,
    pub external_store: sequencer::store::Store,
    execution_program: ExecutionEngine,
    last_committed_round: u64,
}

impl Node {
    pub async fn new(
        committee_file: &str,
        key_file: &str,
        store_path: &str,
        parameters: Option<String>,
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
        let external_store =
            sequencer::store::Store::new(store_path, sequencer::store::EngineType::Sled)
                .expect("Failed to create sequencer store");

        // Init the execution engine according to the parameters sent
        let execution_engine = match parameters.execution {
            ExecutionParameters::CairoVM => {
                // Load the casm programs as bytes
                let fib_casm_program: Vec<u8> =
                    include_bytes!("../../cairo_programs/fib_contract.casm").to_vec();
                let fact_casm_program: Vec<u8> =
                    include_bytes!("../../cairo_programs/fact_contract.casm").to_vec();

                let cairovm_engine = CairoVMEngine::new(fib_casm_program, fact_casm_program);

                // Read casm program bytes as CasmContractClass
                ExecutionEngine::Cairo(Box::new(cairovm_engine))
            }
            ExecutionParameters::CairoNative => {
                todo!("we have to get rid of this")
            }
            ExecutionParameters::StarknetInRust => {
                ExecutionEngine::StarknetInRust(Box::new(StarknetState::new_for_tests()))
            },
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

        let external_store_clone = external_store.clone();
        tokio::spawn(async move {
            let port = committee
                .mempool
                .mempool_address(&name)
                .expect("Our public key is not in the committee")
                .port()
                + RPC_PORT_OFFSET;

            let handle = new_server(port, external_store_clone).await;

            match handle {
                Ok(handle) => {
                    info!("RPC Server started, running on port {}", port);
                    handle.stopped().await;
                }
                Err(e) => println!("Error creating RPC server: {}", e),
            };
        });

        info!("Node {} successfully booted", name);
        Ok(Self {
            commit: rx_commit,
            store,
            external_store,
            execution_program: execution_engine,
            last_committed_round: 0u64,
        })
    }

    pub fn print_key_file(filename: &str) -> Result<(), ConfigError> {
        Secret::new().write(filename)
    }

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

                                    // first call data == Felt252::new(0) means we want to execute fibonacci
                                    // first call data == Felt252::new(1) means we want to execute factorial
                                    // first call data == Felt252::new(2) means we want to execute ERC20
                                    let first_felt: u64 = tx
                                        .calldata
                                        .first()
                                        .expect("Calldata in transaction was not correctly set")
                                        .to_le_digits()[0];

                                    match first_felt {
                                        0 => {
                                            let program_input = tx
                                                .calldata
                                                .get(1)
                                                .expect("calldata was not correctly set");
                                            let n: usize =
                                                program_input.to_le_digits()[0].try_into().unwrap();
                                            self.execution_program.execute_fibonacci(n);
                                        }
                                        1 => {
                                            let program_input = tx
                                                .calldata
                                                .get(1)
                                                .expect("calldata was not correctly set");
                                            let n: usize =
                                                program_input.to_le_digits()[0].try_into().unwrap();
                                            self.execution_program.execute_factorial(n);
                                        }
                                        2 => {
                                            self.execution_program.execute_erc20(
                                                tx.calldata[1].clone(),
                                                tx.calldata[2].clone(),
                                                tx.calldata[3].clone(),
                                            );
                                        }
                                        _ => error!("Transaction contains invalid calldata"),
                                    };

                                    let _ =
                                        self.external_store.add_transaction(starknet_tx.clone());
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
