//! Command handlers for CLI operations

use super::{Cli, Commands};
use crate::ui::InteractiveMenu;
use colored::*;
use shared::config::{DEFAULT_LOG_LEVEL, FIXED_PORT, FALLBACK_PORT_START, FALLBACK_PORT_END, 
                     DEFAULT_HOST_LOCALHOST, MULTICAST_ADDR, CONNECTION_TIMEOUT, 
                     HEARTBEAT_INTERVAL, MAX_CONNECTIONS};
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

            // TLS is always enabled in hardcoded config, ignore no_tls flag
            if no_tls {
                println!("{}", "⚠️  Warning: TLS is always enabled for security. --no-tls flag ignored.".bright_yellow());
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
    println!("{}", "─".repeat(60).dimmed());
    
    println!("🏠 Default Host: {}", DEFAULT_HOST_LOCALHOST.bright_white());
    println!("🔌 Fixed Port: {}", FIXED_PORT.to_string().bright_white());
    println!("🔄 Fallback Ports: {}-{}", FALLBACK_PORT_START.to_string().bright_white(), FALLBACK_PORT_END.to_string().bright_white());
    println!("🔒 TLS: {} (Always Enabled)", "true".bright_green());
    println!("📝 Log Level: {}", DEFAULT_LOG_LEVEL.bright_white());
    println!("🌐 Multicast: {}", MULTICAST_ADDR.bright_white());
    println!("⏱️  Connection Timeout: {}s", CONNECTION_TIMEOUT.to_string().bright_white());
    println!("💓 Heartbeat Interval: {}s", HEARTBEAT_INTERVAL.to_string().bright_white());
    println!("👥 Max Connections: {}", MAX_CONNECTIONS.to_string().bright_white());
    
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "💡 Configuration is now hardcoded for security and simplicity".dimmed());
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
