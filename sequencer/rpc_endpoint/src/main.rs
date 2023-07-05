use rpc_endpoint::new_server;
use tracing::log::info;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Build and init the subscriber
        .finish()
        .init();

    let handle = new_server(5000).await;

    match handle {
        Ok(handle) => {
            info!("RPC Server started, running on port {}", 5000);
            handle.stopped().await;
        }
        Err(e) => println!("Error creating RPC server: {}", e),
    };
}
