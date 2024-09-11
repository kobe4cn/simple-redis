use simple_redis::{network::stream_handler, Backend};
use tracing::{info, warn};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:6379";
    info!("Simple Redis Server started at {}", addr);
    let backend = Backend::new();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    loop {
        let (stream, raddr) = listener.accept().await?;
        let backend_clone = backend.clone();
        tokio::spawn(async move {
            match stream_handler(stream, backend_clone).await {
                Ok(_) => {
                    info!("Connection from  {} exited", raddr);
                }
                Err(e) => {
                    warn!("Error handling client:{}", e);
                }
            }
        });
    }
}
