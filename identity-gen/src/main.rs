mod error;
mod identity;
mod crypto;
mod file_manager;
mod cli;

use clap::Parser;
use colored::*;

use cli::{Cli, CliHandler};
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize colored output
    colored::control::set_override(true);
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Handle any errors gracefully
    if let Err(e) = CliHandler::run(cli) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
    
    Ok(())
}
