//! P2P Chat Client
//! 
//! Pure P2P chat functionality without CLI interface.
//! This binary is called by the launcher/CLI when P2P chat is needed.

mod client;
mod ui;

use client::core::P2PChatClient;
use client::constants::force_cleanup_terminal;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Get log level from environment or default to error
    let _log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "error".to_string());
    
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
    
    // Get default values from environment variables
    let default_host = env::var("DEFAULT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let default_port = env::var("DEFAULT_PORT")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<u16>()
        .unwrap_or(0);
    
    // Parse command line arguments
    let mut username = "Anonymous".to_string();
    let mut listen_port: Option<u16> = None;
    let mut bootstrap_peers: Vec<SocketAddr> = vec![];
    let mut custom_host: Option<String> = None;
    let enable_tls = env::var("TLS_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
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
    
    // Determine final host and port
    let final_host = custom_host.unwrap_or(default_host);
    
    // If no port specified and no bootstrap peers, suggest using a specific port
    let final_port = if let Some(port) = listen_port {
        port
    } else if bootstrap_peers.is_empty() {
        // This is likely a bootstrap node, suggest using a specific port
        if default_port == 0 {
            println!("💡 Tip: For bootstrap nodes, use -p <PORT> to specify a port that other peers can connect to");
            println!("   Example: p2p-core -u {} -p 8080", username);
        }
        default_port
    } else {
        // This is a regular client connecting to bootstrap, use random port
        0
    };
    
    // Create and start P2P client
    let mut client = P2PChatClient::new(username, Some(final_host), Some(final_port), bootstrap_peers, enable_tls).await
        .map_err(|e| format!("Failed to create P2P client: {}", e))?;
    client.start().await
        .map_err(|e| format!("Failed to start P2P client: {}", e))?;
    
    Ok(())
}

fn print_help() {
    println!("\n📖 P2P Chat Client Help");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Usage: p2p-core [OPTIONS]");
    println!("\nOptions:");
    println!("  -u, --username <NAME>     Set username (required)");
    println!("  -p, --port <PORT>         Set listening port (overrides .env DEFAULT_PORT)");
    println!("      --host <HOST>         Set listening host (overrides .env DEFAULT_HOST)");
    println!("  -b, --bootstrap <IP:PORT> Add bootstrap peer (can be used multiple times)");
    println!("  -h, --help                Show this help");
    println!("\nConfiguration:");
    println!("  Copy .env.example to .env and modify default settings");
    println!("  Command line options override .env file settings");
    println!("\nNote: TLS encryption is configurable via .env file.");
    println!("\nExamples:");
    println!("  p2p-core -u Alice");
    println!("  p2p-core -u Bob -p 8080");
    println!("  p2p-core -u Charlie --host 0.0.0.0 -p 9000");
    println!("  p2p-core -u David -b 192.168.1.100:8080");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
