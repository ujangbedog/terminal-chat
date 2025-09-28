//! Terminal Chat Launcher Library
//! 
//! Provides the main entry point function that can be called from the root binary.

use cli::{Cli, handle_command};
use shared::constants::force_cleanup_terminal;
use std::env;

/// Main launcher function that can be called from external binaries
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Get log level from environment or default to error
    let _log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "error".to_string());
    
    // Initialize tracing with minimal logging to avoid UI interference
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("off".parse()?) // Disable all logs
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Setup Ctrl+C handler for clean terminal cleanup
    ctrlc::set_handler(move || {
        force_cleanup_terminal("Program interrupted");
    }).expect("Error setting Ctrl+C handler");

    // Parse CLI arguments using clap and handle commands
    let cli = Cli::parse_args();
    handle_command(cli).await?;

    Ok(())
}
