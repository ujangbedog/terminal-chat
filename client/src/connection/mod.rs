use shared::config;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::info;

/// connect to the chat server
pub async fn connect_to_server() -> Result<Framed<TcpStream, LinesCodec>, Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config::DEFAULT_SERVER_ADDR, config::DEFAULT_SERVER_PORT);
    info!("Connecting to server at {}", addr);
    
    let stream = TcpStream::connect(&addr).await?;
    let framed = Framed::new(stream, LinesCodec::new());
    
    info!("Connected to server successfully");
    Ok(framed)
}
