//! Terminal Chat Application
//! 
//! Main entry point for the Terminal Chat application.
//! This delegates to the launcher module which provides the interactive menu
//! and CLI interface for the P2P chat system.

use std::process;

#[tokio::main]
async fn main() {
    // Delegate to the launcher's main function
    if let Err(e) = launcher::main().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
