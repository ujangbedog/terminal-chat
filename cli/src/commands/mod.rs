//! Command handlers for CLI operations
//! 
//! This module provides a clean separation of different command types
//! for better maintainability and organization.

pub mod p2p;
pub mod config;
pub mod identity;
pub mod menu;

use super::{Cli, Commands};
use std::env;

/// Handle the parsed CLI command
pub async fn handle_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging level based on verbose flag
    if cli.verbose {
        env::set_var("LOG_LEVEL", "debug");
    }

    match cli.command {
        Some(Commands::P2p { 
            username, 
            port, 
            host, 
            bootstrap, 
            no_tls 
        }) => {
            p2p::handle_p2p_command(username, port, host, bootstrap, no_tls).await
        }
        Some(Commands::Menu) | None => {
            menu::handle_menu_command().await
        }
        Some(Commands::Config { show }) => {
            config::handle_config_command(show).await
        }
        Some(Commands::GenerateKey { username, expires_days }) => {
            identity::handle_generate_key(username, expires_days).await
        }
        Some(Commands::List) => {
            identity::handle_list_identities().await
        }
    }
}
