//! P2P Core Library
//! 
//! Provides P2P chat functionality as a library that can be used by other components.

pub mod client;
pub mod ui;

pub use client::core::{P2PChatClient, QuitReason};

use std::net::SocketAddr;

/// Create and run a P2P chat client
pub async fn run_p2p_chat(
    username: String,
    listen_host: Option<String>,
    listen_port: Option<u16>,
    bootstrap_peers: Vec<SocketAddr>,
    enable_tls: bool,
) -> Result<QuitReason, Box<dyn std::error::Error + Send + Sync>> {
    let mut client = P2PChatClient::new(username, listen_host, listen_port, bootstrap_peers, enable_tls).await?;
    
    // Run the client and get the result
    let result = client.start().await;
    let quit_reason = client.get_quit_reason();
    
    // Handle any errors but don't propagate them to avoid terminal issues
    if let Err(e) = result {
        // Only log IO errors, don't print to stderr to avoid terminal corruption
        if e.to_string().contains("read interrupted") || e.to_string().contains("IO error") {
            // Silently ignore IO interruption errors during shutdown
        } else {
            eprintln!("Chat client error: {}", e);
        }
    }
    
    // Give a moment for final cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(quit_reason)
}
