use anyhow::Result;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use rpc::StarknetRpcApiServer;
use starknet_backend::StarknetBackend;
use store::{EngineType, Store};
use tracing::info;
use tracing_subscriber::util::SubscriberInitExt;

mod rpc;
mod starknet_backend;
mod store;

const RPC_PORT: u16 = 9994;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Build and init the subscriber
        .finish()
        .init();

    let handle = new_server().await;

    //TODO remove once store is on use
    #[allow(unused_variables, unused_mut)]
    let mut store = Store::new("store", EngineType::Sled);

    // Example usage:
    // Remove once it has its own tests
    //let _ = store.add_program("id_1".as_bytes().to_vec(), "program_1".as_bytes().to_vec());
    //let _ = store.add_transaction("id_1".as_bytes().to_vec(), "tx_1".as_bytes().to_vec());
    //println!("Store: {store:?}, {}", String::from_utf8_lossy(&store.get_program("id_1".as_bytes().to_vec()).unwrap()));

    match handle {
        Ok(handle) => {
            info!("RPC Server started, running on port {}", RPC_PORT);
            handle.stopped().await;
        }
        Err(e) => println!("Error creating RPC server: {}", e.to_string()),
    };
}

pub async fn new_server() -> Result<ServerHandle> {
    let server = ServerBuilder::default()
        .build(format!("127.0.0.1:{}", RPC_PORT))
        .await?;
    let server_handle = server.start(StarknetBackend {}.into_rpc())?;

    Ok(server_handle)
}
