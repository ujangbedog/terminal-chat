//! Command handlers for CLI operations

use super::{Cli, Commands};
use crate::ui::InteractiveMenu;
use colored::*;
use std::env;
use std::process::Command;

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
            println!("{}", "🚀 Starting P2P Chat Mode...".bright_cyan().bold());
            
            // Convert to the format expected by the existing P2P client
            let mut args = vec![
                "p2p-core".to_string(),
                "-u".to_string(),
                username,
                "--host".to_string(),
                host,
            ];

            if let Some(p) = port {
                args.push("-p".to_string());
                args.push(p.to_string());
            }

            for peer in bootstrap {
                args.push("-b".to_string());
                args.push(peer.to_string());
            }

            if no_tls {
                env::set_var("TLS_ENABLED", "false");
            }

            // Call external p2p-core binary
            run_chat_client(&args).await
        }
        Some(Commands::Menu) | None => {
            // Interactive menu mode
            println!("{}", "🎯 Starting Interactive Menu...".bright_green().bold());
            let mut menu = InteractiveMenu::new();
            menu.show().await
        }
        Some(Commands::Config { show }) => {
            if show {
                show_config();
            }
            Ok(())
        }
    }
}

/// Show current configuration
fn show_config() {
    println!("{}", "📋 Current Configuration".bright_yellow().bold());
    println!("{}", "─".repeat(50).dimmed());
    
    let default_host = env::var("DEFAULT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let default_port = env::var("DEFAULT_PORT").unwrap_or_else(|_| "8080".to_string());
    let tls_enabled = env::var("TLS_ENABLED").unwrap_or_else(|_| "true".to_string());
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "error".to_string());

    println!("🏠 Default Host: {}", default_host.bright_white());
    println!("🔌 Default Port: {}", default_port.bright_white());
    println!("🔒 TLS Enabled: {}", tls_enabled.bright_white());
    println!("📝 Log Level: {}", log_level.bright_white());
    
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", "💡 Tip: Copy .env.example to .env to customize these settings".dimmed());
}

/// Run external p2p-core binary
async fn run_chat_client(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "🚀 Launching P2P Chat Client...".bright_cyan().bold());
    
    let mut cmd = Command::new("cargo");
    cmd.arg("run")
       .arg("-p")
       .arg("p2p-core")
       .arg("--bin")
       .arg("p2p-core")
       .arg("--");
    
    // Add all arguments except the first one (program name)
    for arg in args.iter().skip(1) {
        cmd.arg(arg);
    }
    
    let status = cmd.status()?;
    
    if !status.success() {
        return Err("Chat client failed to start".into());
    }
    
    Ok(())
}
