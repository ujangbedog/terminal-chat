mod ui;
mod client;

use ui::display_header;
use client::P2PChatClient;
use client::constants::force_cleanup_terminal;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Get log level from environment or default to error
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "error".to_string());
    
    // initialize tracing with configurable log level
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(format!("p2p_chat={}", log_level).parse()?)
                .add_directive(format!("shared={}", log_level).parse()?),
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

    display_header();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    run_p2p_client(&args).await?;

    Ok(())
}

async fn run_p2p_client(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting P2P Chat");
    
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
            println!("   Example: cargo run --bin p2p-chat -- -u {} -p 8080", username);
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
    println!("\n📖 P2P Chat Help");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Usage: p2p-chat [OPTIONS]");
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
    println!("  p2p-chat -u Alice");
    println!("  p2p-chat -u Bob -p 8080");
    println!("  p2p-chat -u Charlie --host 0.0.0.0 -p 9000");
    println!("  p2p-chat -u David -b 192.168.1.100:8080");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
