//! Command handlers for CLI operations

use super::{Cli, Commands};
use crate::ui::InteractiveMenu;
use colored::*;
use shared::config::{DEFAULT_LOG_LEVEL, FIXED_PORT, FALLBACK_PORT_START, FALLBACK_PORT_END, 
                     DEFAULT_HOST_LOCALHOST, MULTICAST_ADDR, CONNECTION_TIMEOUT, 
                     HEARTBEAT_INTERVAL, MAX_CONNECTIONS};
use std::env;
use std::process::Command;
use identity_gen;

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
        Some(Commands::GenerateKey { username, expires_days }) => {
            println!("{}", "🔐 Starting Identity Generation...".bright_magenta().bold());
            handle_generate_key(username, expires_days).await
        }
        Some(Commands::List) => {
            handle_list_identities().await
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

/// Handle identity generation command
async fn handle_generate_key(_username: Option<String>, _expires_days: Option<i64>) -> Result<(), Box<dyn std::error::Error>> {
    show_identity_generation_welcome();
    
    // Use the identity-gen library directly
    match identity_gen::run_identity_generator().await {
        Ok(_) => {
            // Don't show success message, just exit cleanly
            // The identity-gen library will show the identity details
        }
        Err(e) => {
            show_generation_error(&e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// Show welcome screen for identity generation
fn show_identity_generation_welcome() {
    // Clear screen for better presentation
    print!("\x1B[2J\x1B[1;1H");
    
    // Clean header
    println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                    CRYSTALS-Dilithium Identity Generator                     ║".bright_cyan().bold());
    println!("{}", "║                         Post-Quantum Cryptographic Security                  ║".bright_cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
    
    println!("{}", "Starting identity generation process...".bright_cyan().bold());
    println!("{}", "─".repeat(80).dimmed());
    println!();
}

/// Show success message after generation
fn show_generation_success() {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_green());
    println!("{}", "║                            GENERATION SUCCESSFUL!                           ║".bright_green().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_green());
    println!();
    
    println!("{}", "Your CRYSTALS-Dilithium identity has been created successfully!".bright_green().bold());
    println!();
    
    // Next steps
    println!("{}", "Next Steps:".bright_yellow().bold());
    println!("  • List all identities: {}", "cargo run -- list".cyan());
    println!("  • Start P2P chat: {}", "cargo run -- p2p -u <username> -p <port>".cyan());
    println!();
    
    // Important notes
    println!("{}", "Important Notes:".bright_red().bold());
    println!("  • Keep your password safe - it's required to use this identity");
    println!("  • Share your fingerprint for identity verification");
    println!("  • Identity saved to ~/.terminal-chat/identities/");
    println!();
    
    println!("{}", "Your identity is ready for secure P2P communication!".bright_magenta().bold());
}

/// Show error message if generation fails
fn show_generation_error(error: &identity_gen::IdentityError) {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_red());
    println!("{}", "║                              GENERATION FAILED                              ║".bright_red().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_red());
    println!();
    
    println!("{} {}", "Error:".bright_red().bold(), error.to_string().bright_white());
    println!();
    
    // Troubleshooting tips
    println!("{}", "Troubleshooting Tips:".bright_yellow().bold());
    println!("  • Check if ~/.terminal-chat/ directory is writable");
    println!("  • Ensure you have sufficient disk space");
    println!("  • Try running with different username");
    println!("  • Check if identity already exists");
    println!();
    
    println!("{}", "Try again or contact support if the problem persists.".dimmed());
}

/// Handle list identities command
async fn handle_list_identities() -> Result<(), Box<dyn std::error::Error>> {
    // Clear screen for better presentation
    print!("\x1B[2J\x1B[1;1H");
    
    // Clean header
    println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_blue());
    println!("{}", "║                           CRYPTOGRAPHIC IDENTITIES                           ║".bright_blue().bold());
    println!("{}", "║                            Your Digital Identity Vault                       ║".bright_blue());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_blue());
    println!();
    
    match identity_gen::list_identities() {
        Ok(identities) => {
            if identities.is_empty() {
                show_no_identities_found();
            } else {
                show_identities_list(&identities).await?;
            }
        }
        Err(e) => {
            println!("{} {}", "❌ Failed to list identities:".bright_red().bold(), e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// Show message when no identities are found
fn show_no_identities_found() {
    println!("{}", "No identities found in your vault.".bright_yellow().bold());
    println!();
    
    println!("{}", "Getting Started:".bright_cyan().bold());
    println!("  • Create your first identity: {}", "cargo run -- generate-key".cyan());
    println!("  • Specify username: {}", "cargo run -- generate-key -u <username>".cyan());
    println!("  • Set expiration: {}", "cargo run -- generate-key -u <username> -e 365".cyan());
    println!();
    
    println!("{}", "Your identity will be securely stored and ready for P2P communication!".dimmed());
}

/// Show list of identities with clean formatting
async fn show_identities_list(identities: &[(String, std::path::PathBuf)]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} {}", "Found".bright_white(), format!("{} identities", identities.len()).bright_cyan().bold());
    println!("{}", "─".repeat(80).dimmed());
    println!();
    
    for (i, (username, path)) in identities.iter().enumerate() {
        match identity_gen::load_identity(username) {
            Ok(identity) => {
                let status = if identity.is_expired() {
                    "EXPIRED".bright_red().bold()
                } else {
                    "ACTIVE".bright_green().bold()
                };
                
                println!("{}. {} [{}]", i + 1, username.bright_cyan().bold(), status);
                
                // Show details with clean formatting
                println!("   Fingerprint: {}", identity.fingerprint.bright_white());
                println!("   Algorithm: {}", identity.algorithm.bright_white());
                println!("   Created: {}", identity.created_at.format("%Y-%m-%d %H:%M UTC").to_string().bright_white());
                
                if let Some(expires) = identity.expires_at {
                    let expires_str = expires.format("%Y-%m-%d %H:%M UTC").to_string();
                    if identity.is_expired() {
                        println!("   Expired: {}", expires_str.bright_red());
                    } else {
                        println!("   Expires: {}", expires_str.bright_white());
                    }
                } else {
                    println!("   Expires: {}", "Never".bright_green());
                }
                
                println!("   File: {}", path.display().to_string().dimmed());
                
                if i < identities.len() - 1 {
                    println!();
                }
                println!();
            }
            Err(_) => {
                println!("{}. {} [CORRUPTED]", i + 1, username.bright_red().bold());
                println!("   File: {}", path.display().to_string().bright_red());
                println!();
            }
        }
    }
    
    // Show usage tips
    println!("{}", "─".repeat(80).dimmed());
    println!("{}", "Quick Actions:".bright_yellow().bold());
    println!("  • Generate new identity: {}", "cargo run -- generate-key".cyan());
    println!("  • Start P2P chat: {}", "cargo run -- p2p -u <username> -p <port>".cyan());
    
    Ok(())
}
