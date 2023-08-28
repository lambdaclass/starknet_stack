use anyhow::bail;
use anyhow::{Context, Result};
use bytes::BufMut as _;
use bytes::BytesMut;
use cairo_felt::{felt_str, Felt252};
use clap::Parser;
use env_logger::Env;
use futures::future::join_all;
use futures::sink::SinkExt as _;
use log::{info, warn};
use rand::seq::SliceRandom;
use rand::Rng;
use rpc_endpoint::rpc::InvokeTransaction;
use rpc_endpoint::rpc::Transaction;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::time::{interval, sleep, Duration, Instant};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Parser)]
#[clap(
    author,
    version,
    about,
    long_about = "Benchmark client for HotStuff nodes."
)]
struct Cli {
    /// The network address of the node where to send txs.
    #[clap(value_parser, value_name = "ADDR")]
    target: SocketAddr,
    /// The nodes timeout value.
    #[clap(short, long, value_parser, value_name = "INT")]
    timeout: u64,
    /// The size of each transaction in bytes.
    #[clap(short, long, value_parser, value_name = "INT")]
    size: usize,
    /// The rate (txs/s) at which to send the transactions.
    #[clap(short, long, value_parser, value_name = "INT")]
    rate: u64,
    /// Network addresses that must be reachable before starting the benchmark.
    #[clap(short, long, value_parser, value_name = "[Addr]", multiple = true)]
    nodes: Vec<SocketAddr>,
    /// Running time of the client in seconds.
    #[clap(short, long, value_parser, value_name = "INT")]
    running_time: Option<u8>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Node address: {}", cli.target);
    info!("Transactions size: {} B", cli.size);
    info!("Transactions rate: {} tx/s", cli.rate);
    let client = Client {
        target: cli.target,
        size: cli.size,
        rate: cli.rate,
        timeout: cli.timeout,
        nodes: cli.nodes,
    };

    // Wait for all nodes to be online and synchronized.
    client.wait().await;

    // Start the benchmark.
    client
        .send(cli.running_time)
        .await
        .context("Failed to submit transactions")
}

struct Client {
    target: SocketAddr,
    size: usize,
    rate: u64,
    timeout: u64,
    nodes: Vec<SocketAddr>,
}

impl Client {
    pub async fn send(&self, running_time_seconds: Option<u8>) -> Result<()> {
        const PRECISION: u64 = 20; // Sample precision.
        const BURST_DURATION: u64 = 1000 / PRECISION;

        // The transaction size must be at least 16 bytes to ensure all txs are different.
        if self.size < 16 {
            return Err(anyhow::Error::msg(
                "Transaction size must be at least 9 bytes",
            ));
        }

        // Connect to the mempool.
        let stream = TcpStream::connect(self.target)
            .await
            .context(format!("failed to connect to {}", self.target))?;

        // Submit all transactions.
        let burst = self.rate / PRECISION;
        let mut tx = BytesMut::with_capacity(self.size);
        let mut counter = 0;
        let mut r: u64 = rand::thread_rng().gen();
        let mut transport = Framed::new(stream, LengthDelimitedCodec::new());
        let interval = interval(Duration::from_millis(BURST_DURATION));
        tokio::pin!(interval);

        // NOTE: This log entry is used to compute performance.
        info!("Start sending transactions");
        let starting_time = Instant::now();
        let mut internal_counter: u64;

        'main: loop {
            interval.as_mut().tick().await;
            let now = Instant::now();
            internal_counter = 0;

            for x in 0..burst {
                let invoke_transaction = Transaction::new_invoke(
                    counter + internal_counter,
                    Self::create_execution_call_data(),
                );

                if let Transaction::Invoke(InvokeTransaction::V1(transaction)) = &invoke_transaction
                {
                    if x == counter % burst {
                        info!(
                            "Sending sample transaction {} - Transaction ID: 0x{}",
                            counter,
                            transaction.transaction_hash.to_str_radix(16)
                        );

                        // NOTE: This log entry is used to compute performance.

                        tx.put_u8(0u8); // Sample txs start with 0.
                        tx.put_u64(counter); // This counter identifies the tx.
                    } else {
                        r += 1;

                        tx.put_u8(1u8); // Standard txs start with 1.
                        tx.put_u64(r); // Ensures all clients send different txs.
                    };
                    for b in invoke_transaction.as_bytes() {
                        tx.put_u8(b);
                    }
                } else {
                    bail!("Expected transaction to be InvokeTransaction::V1");
                };

                let bytes = tx.split().freeze();

                if let Err(e) = transport.send(bytes).await {
                    warn!("Failed to send transaction: {}", e);
                    break 'main;
                }
                internal_counter += 1;
            }
            if now.elapsed().as_millis() > BURST_DURATION as u128 {
                // NOTE: This log entry is used to compute performance.
                warn!("Transaction rate too high for this client");
            }
            if let Some(time) = running_time_seconds {
                if starting_time.elapsed().as_secs() > time as u64 {
                    info!("Sent {} transactions to node", internal_counter + counter);

                    return Ok(());
                }
            }
            counter += 1;
        }
        Ok(())
    }

    pub async fn wait(&self) {
        // First wait for all nodes to be online.
        info!("Waiting for all nodes to be online...");
        join_all(self.nodes.iter().cloned().map(|address| {
            tokio::spawn(async move {
                while TcpStream::connect(address).await.is_err() {
                    sleep(Duration::from_millis(10)).await;
                }
            })
        }))
        .await;

        // Then wait for the nodes to be synchronized.
        info!("Waiting for all nodes to be synchronized...");
        sleep(Duration::from_millis(2 * self.timeout)).await;
    }

    pub fn create_execution_call_data() -> Vec<Felt252> {
        pub enum ExecutionType {
            Fibonacci,
            Factorial,
            ERC20,
        }

        let options = [
            ExecutionType::Fibonacci,
            ExecutionType::Factorial,
            ExecutionType::ERC20,
        ];

        let n: u16 = rand::thread_rng().gen();
        //let rand_program_input: u16 = rand::thread_rng();
        match options.choose(&mut rand::thread_rng()).unwrap() {
            ExecutionType::Fibonacci => {
                let selector = felt_str!(
                    "112e35f48499939272000bd72eb840e502ca4c3aefa8800992e8defb746e0c9",
                    16
                );
                vec![Felt252::new(0), selector, Felt252::new((n % 10000) + 1)]
            }
            ExecutionType::Factorial => {
                let selector = felt_str!(
                    "213cda0181d4bd6d07f2e467ddf45a1d971e14ca1bcd4c83949a6d830a15b7f",
                    16
                );
                vec![Felt252::new(1), selector, Felt252::new((n % 2000) + 1)]
            }
            ExecutionType::ERC20 => {
                let selector = felt_str!(
                    "83afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e",
                    16
                );
                let initial_supply = Felt252::new((n % 5000) + 1);
                let token_symbol = Felt252::new(512);
                let contract_address = Felt252::new(rand::thread_rng().gen::<u128>());
                // execution type felt, initial_supply, token symbol, contract address
                vec![
                    Felt252::new(2),
                    selector,
                    initial_supply,
                    token_symbol,
                    contract_address,
                ]
            }
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::BufMut;
    use bytes::BytesMut;
    use rand::Rng;
    use rpc_endpoint::rpc::Transaction;

    use crate::Client;

    #[test]
    fn test_serialize_transaction() {
        let burst = 12;
        let size = 1000;
        let mut tx = BytesMut::with_capacity(size);
        let counter = 0;
        let small_r: u8 = rand::thread_rng().gen();
        let _r: u64 = small_r.into();

        for x in 0..burst {
            if x == counter % burst {
                tx.put_u8(0u8); // Sample txs start with 0.
            } else {
                tx.put_u8(1u8); // Standard txs start with 1.
            };
            let starknet_tx =
                Transaction::new_invoke(762716321, Client::create_execution_call_data());
            for b in starknet_tx.as_bytes() {
                tx.put_u8(b);
            }
            tx.resize(size, 0u8);
            let bytes = tx.split().freeze();

            let _ret = Transaction::from_bytes(&bytes);
        }
    }
}
