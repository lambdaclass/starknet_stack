use anyhow::Result;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use rpc::StarknetRpcApiServer;
use sequencer::store::Store;
use starknet_backend::StarknetBackend;

pub mod rpc;
pub mod starknet_backend;

pub async fn new_server(port: u16, store: Store) -> Result<ServerHandle> {
    let server = ServerBuilder::default()
        .build(format!("0.0.0.0:{}", port))
        .await?;
    let server_handle = server.start(StarknetBackend { store }.into_rpc())?;

    Ok(server_handle)
}
