mod client;
mod state;
mod handler;

use handler::handle_client;
use shared::config;
use state::SharedState;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing with better defaults
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse()?)
                .add_directive("chat_server=info".parse()?),
        )
        .init();

    let addr = format!("{}:{}", config::DEFAULT_SERVER_ADDR, config::DEFAULT_SERVER_PORT);
    let listener = TcpListener::bind(&addr).await?;
    
    println!("=== Simple Chat Server ===");
    println!("Server listening on: {}", addr);
    println!("Waiting for clients to connect...");
    println!("Press Ctrl+C to stop the server");
    println!("{}", "=".repeat(50));
    
    info!("Chat server listening on {}", addr);

    let state = Arc::new(Mutex::new(SharedState::new()));

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let client_id = Uuid::new_v4();
                let state_clone = Arc::clone(&state);
                
                println!("[CONNECTION] New client connected from: {}", addr);
                info!("New connection from {}", addr);
                
                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_id, stream, state_clone).await {
                        error!("Error handling client {}: {}", client_id, e);
                    }
                });
            }
            Err(e) => {
                println!("[ERROR] Failed to accept connection: {}", e);
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}
