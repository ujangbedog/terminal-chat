//! P2P Chat Client
//! 
//! Pure P2P chat functionality without CLI interface.
//! This binary is called by the launcher/CLI when P2P chat is needed.

mod client;
mod ui;
mod cli;

use client::core::P2PChatClient;
use client::constants::force_cleanup_terminal;
use shared::config::DEFAULT_LOG_LEVEL;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use hardcoded configuration instead of .env file
    let _log_level = DEFAULT_LOG_LEVEL;
    
    // Initialize tracing with all logs disabled to avoid UI interference
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("off".parse()?) // Disable all logs completely
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Setup Ctrl+C handler for clean terminal cleanup
    ctrlc::set_handler(move || {
        force_cleanup_terminal("P2P Chat interrupted");
    }).expect("Error setting Ctrl+C handler");

    // Parse command line arguments and start P2P client
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments using the modular CLI
    match cli::parse_args(&args)? {
        Some(parsed_args) => {
            // Create and start P2P client
            let mut client = P2PChatClient::new(
                parsed_args.username,
                Some(parsed_args.final_host),
                Some(parsed_args.final_port),
                parsed_args.bootstrap_peers,
                parsed_args.enable_tls,
            ).await.map_err(|e| format!("Failed to create P2P client: {}", e))?;
            
            client.start().await
                .map_err(|e| format!("Failed to start P2P client: {}", e))?;
        }
        None => {
            // Help was shown or there was an error, exit gracefully
        }
    }

    Ok(())
}
