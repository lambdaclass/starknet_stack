mod config;
mod hotstuff_node;

use crate::config::Export as _;
use crate::config::{Committee, Secret};
use crate::hotstuff_node::HotstuffNode;
use clap::{Parser, Subcommand};
use consensus::Committee as ConsensusCommittee;
use env_logger::Env;
use futures::future::join_all;
use log::error;
use mempool::Committee as MempoolCommittee;
use rpc_endpoint::starknet_backend::StarknetBackend;
use std::fs;
use tokio::task::JoinHandle;

/// Default port offset for RPC endpoint
const RPC_PORT_OFFSET: u16 = 1000;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on.
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// The command to execute.
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a new keypair.
    Keys {
        /// The file where to print the new key pair.
        #[clap(short, long, value_parser, value_name = "FILE")]
        filename: String,
    },
    /// Run a single node.
    Run {
        /// The file containing the node keys.
        #[clap(short, long, value_parser, value_name = "FILE")]
        keys: String,
        /// The file containing committee information.
        #[clap(short, long, value_parser, value_name = "FILE")]
        committee: String,
        /// Optional file containing the node parameters.
        #[clap(short, long, value_parser, value_name = "FILE")]
        parameters: Option<String>,
        /// The path where to create the data store.
        #[clap(short, long, value_parser, value_name = "PATH")]
        store: String,
    },
    /// Deploy a local testbed with the specified number of nodes.
    Deploy {
        #[clap(short, long, value_parser = clap::value_parser!(u16).range(4..))]
        nodes: u16,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };
    let mut logger = env_logger::Builder::from_env(Env::default().default_filter_or(log_level));
    #[cfg(feature = "benchmark")]
    logger.format_timestamp_millis();
    logger.init();

    match cli.command {
        Command::Keys { filename } => {
            if let Err(e) = HotstuffNode::print_key_file(&filename) {
                error!("{}", e);
            }
        }
        Command::Run {
            keys,
            committee,
            parameters,
            store,
        } => {
            let external_store =
                sequencer::store::Store::new(&store, sequencer::store::EngineType::Sled)
                    .expect("Failed to create sequencer store");

            match HotstuffNode::new(
                &committee,
                &keys,
                &store,
                parameters,
                external_store.clone(),
            )
            .await
            {
                Ok(mut node) => {
                    let rpc_port = node.mempool_transaction_endpoint.port() + RPC_PORT_OFFSET;

                    // Start RPC backend module
                    StarknetBackend::spawn(external_store, rpc_port);

                    tokio::spawn(async move {
                        node.analyze_block().await;
                    })
                    .await
                    .expect("Failed to analyze committed blocks");
                }
                Err(e) => error!("{}", e),
            }
        }
        Command::Deploy { nodes } => match deploy_testbed(nodes) {
            Ok(handles) => {
                let _ = join_all(handles).await;
            }
            Err(e) => error!("Failed to deploy testbed: {}", e),
        },
    }
}

fn deploy_testbed(nodes: u16) -> Result<Vec<JoinHandle<()>>, Box<dyn std::error::Error>> {
    let keys: Vec<_> = (0..nodes).map(|_| Secret::new()).collect();

    // Print the committee file.
    let epoch = 1;
    let mempool_committee = MempoolCommittee::new(
        keys.iter()
            .enumerate()
            .map(|(i, key)| {
                let name = key.name;
                let stake = 1;
                let front = format!("127.0.0.1:{}", 25_000 + i).parse().unwrap();
                let mempool = format!("127.0.0.1:{}", 25_100 + i).parse().unwrap();
                (name, stake, front, mempool)
            })
            .collect(),
        epoch,
    );
    let consensus_committee = ConsensusCommittee::new(
        keys.iter()
            .enumerate()
            .map(|(i, key)| {
                let name = key.name;
                let stake = 1;
                let addresses = format!("127.0.0.1:{}", 25_200 + i).parse().unwrap();
                (name, stake, addresses)
            })
            .collect(),
        epoch,
    );
    let committee_file = "committee.json";
    let _ = fs::remove_file(committee_file);
    Committee {
        mempool: mempool_committee,
        consensus: consensus_committee,
    }
    .write(committee_file)?;

    // Write the key files and spawn all nodes.
    keys.iter()
        .enumerate()
        .map(|(i, keypair)| {
            let key_file = format!("node_{}.json", i);
            let _ = fs::remove_file(&key_file);
            keypair.write(&key_file)?;

            let store_path = format!("db_{}", i);
            let _ = fs::remove_dir_all(&store_path);

            Ok(tokio::spawn(async move {
                let external_store =
                    sequencer::store::Store::new(&store_path, sequencer::store::EngineType::Sled)
                        .expect("Failed to create sequencer store");

                match HotstuffNode::new(
                    committee_file,
                    &key_file,
                    &store_path,
                    None,
                    external_store.clone(),
                )
                .await
                {
                    Ok(mut node) => {
                        let rpc_port = node.mempool_transaction_endpoint.port() + RPC_PORT_OFFSET;

                        // Start RPC backend module
                        StarknetBackend::spawn(external_store, rpc_port);
                        // Sink the commit channel.
                        while node.commit.recv().await.is_some() {}
                    }
                    Err(e) => error!("{}", e),
                }
            }))
        })
        .collect::<Result<_, Box<dyn std::error::Error>>>()
}
