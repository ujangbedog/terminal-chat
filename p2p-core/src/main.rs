//! P2P Chat Client
//! 
//! Pure P2P chat functionality without CLI interface.
//! This binary is called by the launcher/CLI when P2P chat is needed.

mod client;
mod ui;

use client::core::P2PChatClient;
use client::constants::force_cleanup_terminal;
use shared::config::{DEFAULT_LOG_LEVEL, DEFAULT_HOST_LOCALHOST, FIXED_PORT, TLS_ENABLED, find_available_port};
use std::env;
use std::net::SocketAddr;

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

    // Header will be displayed by the enhanced UI
    
    // Parse command line arguments and start P2P client
    let args: Vec<String> = env::args().collect();
    run_p2p_client(&args).await?;

    Ok(())
}

async fn run_p2p_client(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    
    // Use hardcoded configuration values
    let default_host = DEFAULT_HOST_LOCALHOST;
    let default_port = FIXED_PORT;
    
    // Parse command line arguments
    let mut username = "Anonymous".to_string();
    let mut listen_port: Option<u16> = None;
    let mut bootstrap_peers: Vec<SocketAddr> = vec![];
    let mut custom_host: Option<String> = None;
    let enable_tls = TLS_ENABLED; // Always true
    
    let mut i = 1; // Skip program name only
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 < args.len() {
                    username = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --username requires a value");
                    return Ok(());
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    listen_port = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value");
                    return Ok(());
                }
            }
            "--host" => {
                if i + 1 < args.len() {
                    custom_host = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --host requires a value");
                    return Ok(());
                }
            }
            "--bootstrap" | "-b" => {
                if i + 1 < args.len() {
                    let addr: SocketAddr = args[i + 1].parse()?;
                    bootstrap_peers.push(addr);
                    i += 2;
                } else {
                    eprintln!("Error: --bootstrap requires a value");
                    return Ok(());
                }
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                return Ok(());
            }
        }
    }
    
    // Validate username
    if username.trim().is_empty() {
        eprintln!("Error: Username cannot be empty");
        return Ok(());
    }
    
    // Determine final host
    let final_host = custom_host.unwrap_or_else(|| default_host.to_string());
    
    // Determine final port using the new fixed port system
    let final_port = if let Some(port) = listen_port {
        // Port explicitly specified via command line
        port
    } else {
        // Use automatic port selection: try fixed port first, then fallback range
        match find_available_port(&final_host) {
            Ok(port) => {
                if port == FIXED_PORT {
                    println!("🔌 Using fixed port: {}", port);
                } else {
                    println!("🔌 Fixed port {} unavailable, using fallback port: {}", FIXED_PORT, port);
                }
                port
            }
            Err(e) => {
                eprintln!("❌ Error finding available port: {}", e);
                return Err(e);
            }
        }
    };
    
    // Create and start P2P client
    let mut client = P2PChatClient::new(username, Some(final_host), Some(final_port), bootstrap_peers, enable_tls).await
        .map_err(|e| format!("Failed to create P2P client: {}", e))?;
    client.start().await
        .map_err(|e| format!("Failed to start P2P client: {}", e))?;
    
    Ok(())
}

fn print_help() {
    use shared::config::*;
    
    println!("\n📖 P2P Chat Client Help");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Usage: p2p-core [OPTIONS]");
    println!("\nOptions:");
    println!("  -u, --username <NAME>     Set username (required)");
    println!("  -p, --port <PORT>         Set listening port (default: auto-select from {}-{})", FIXED_PORT, FALLBACK_PORT_END);
    println!("      --host <HOST>         Set listening host (default: {})", DEFAULT_HOST_LOCALHOST);
    println!("  -b, --bootstrap <IP:PORT> Add bootstrap peer (can be used multiple times)");
    println!("  -h, --help                Show this help");
    println!("\nConfiguration:");
    println!("  🔌 Fixed Port: {} (with fallback range {}-{})", FIXED_PORT, FALLBACK_PORT_START, FALLBACK_PORT_END);
    println!("  🔒 TLS: Always enabled for security");
    println!("  🌐 Default Host: {} (localhost)", DEFAULT_HOST_LOCALHOST);
    println!("\nExamples:");
    println!("  p2p-core -u Alice                              # Create new chat room");
    println!("  p2p-core -u Bob --host 0.0.0.0               # Allow external connections");
    println!("  p2p-core -u Charlie -b 192.168.1.100:40000   # Connect to existing peer");
    println!("  p2p-core -u David -p 40005                   # Use specific port");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
