//! Terminal Chat Launcher Binary
//! 
//! Main entry point binary that delegates to the launcher library.

use std::process;

#[tokio::main]
async fn main() {
    // Delegate to the launcher library's main function
    if let Err(e) = launcher::main().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
