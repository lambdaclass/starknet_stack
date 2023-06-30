use anyhow::Result;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use rpc::StarknetRpcApiServer;
use starknet_backend::StarknetBackend;
use tracing::info;
use tracing_subscriber::util::SubscriberInitExt;

pub mod rpc;
pub mod starknet_backend;

const _RPC_PORT: u16 = 9994;

//#[tokio::main]
async fn _start_new_server() {
    tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Build and init the subscriber
        .finish()
        .init();

    let handle = new_server(_RPC_PORT).await;

    match handle {
        Ok(handle) => {
            info!("RPC Server started, running on port {}", _RPC_PORT);
            handle.stopped().await;
        }
        Err(e) => println!("Error creating RPC server: {}", e),
    };
}

pub async fn new_server(port: u16) -> Result<ServerHandle> {
    let server = ServerBuilder::default()
        .build(format!("127.0.0.1:{}", port))
        .await?;
    let server_handle = server.start(StarknetBackend {}.into_rpc())?;

    Ok(server_handle)
}
