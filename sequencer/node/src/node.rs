use crate::config::Export as _;
use crate::config::{Committee, ConfigError, Parameters, Secret};
use consensus::{Block, Consensus};
use crypto::SignatureService;

use log::info;
use mempool::{Mempool, MempoolMessage, TransactionType};
use rpc_endpoint::new_server;

use store::Store;
use tokio::sync::mpsc::{channel, Receiver};

use std::process::Command;

/// The default channel capacity for this module.
pub const CHANNEL_CAPACITY: usize = 1_000;

/// Default port offset for RPC endpoint
const RPC_PORT_OFFSET: u16 = 1000;

pub struct Node {
    pub commit: Receiver<Block>,
    pub store: Store,
    pub external_store: sequencer::store::Store,
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
            sequencer::store::Store::new(store_path, sequencer::store::EngineType::Sled);
        // let _ = external_store
        //     .clone()
        //     .add_transaction("id_1".as_bytes().to_vec(), "tx_1".as_bytes().to_vec());

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
        })
    }

    pub fn print_key_file(filename: &str) -> Result<(), ConfigError> {
        Secret::new().write(filename)
    }

    pub async fn analyze_block(&mut self) {
        while let Some(block) = self.commit.recv().await {
            // This is where we can further process committed block.
            for p in block.payload {
                let tx_batch = self.store.read(p.to_vec()).await.unwrap().unwrap();
                info!("Batch is {} bytes long", tx_batch.len());

                let list_of_tx: MempoolMessage =
                    bincode::deserialize(&tx_batch).expect("Error trying to deserialize batch");
                //info!("There are {} transactions in {:?} ", list_of_tx.len(), p);

                match list_of_tx {
                    MempoolMessage::Batch(batch_txs) => {
                        info!(
                            "Batch message confirmed, with {} transactions!",
                            batch_txs.len()
                        );

                        for (i, m) in batch_txs.into_iter().enumerate() {
                            let transaction_type: TransactionType =
                                bincode::deserialize(&m[9..]).unwrap();
                            info!(
                                "Message {i} in {:?} is of tx_type {:?}",
                                p, transaction_type
                            );

                            match transaction_type {
                                TransactionType::ExecuteFibonacci(_) => {
                                    let res = Command::new("../cairo_native/target/release/cli")
                                        .arg("run")
                                        .arg("-f")
                                        .arg("fib::fib::main")
                                        .arg("../cairo_programs/fib.cairo")
                                        .arg("--available-gas")
                                        .arg("900000000")
                                        .output()
                                        .expect("Failed to execute process");
                                    info!("Output: {}", String::from_utf8_lossy(&res.stdout));
                                }
                            }
                        }
                    }
                    MempoolMessage::BatchRequest(_, _) => {
                        info!("Batch Request message confirmed")
                    }
                }
            }
        }
    }
}
