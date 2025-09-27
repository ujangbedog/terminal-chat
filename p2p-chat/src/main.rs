mod ui;
mod client;

use ui::display_header;
use client::P2PChatClient;
use client::constants::force_cleanup_terminal;
use std::env;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing with minimal output (only errors)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("p2p_chat=error".parse()?)
                .add_directive("shared=error".parse()?),
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
    
    // Parse command line arguments
    let mut username = "Anonymous".to_string();
    let mut listen_port: Option<u16> = None;
    let mut bootstrap_peers: Vec<SocketAddr> = vec![];
    let enable_tls = true; // Always use TLS for security
    
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
    
    // Create and start P2P client
    let mut client = P2PChatClient::new(username, listen_port, bootstrap_peers, enable_tls).await
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
    println!("  -p, --port <PORT>         Set listening port (optional, random if not set)");
    println!("  -b, --bootstrap <IP:PORT> Add bootstrap peer (can be used multiple times)");
    println!("  -h, --help                Show this help");
    println!("\nNote: TLS encryption is always enabled for security.");
    println!("\nExamples:");
    println!("  p2p-chat -u Alice");
    println!("  p2p-chat -u Bob -p 8080");
    println!("  p2p-chat -u Charlie -b 192.168.1.100:8080");
    println!("  p2p-chat -u David -b 127.0.0.1:8080 -b 127.0.0.1:8081");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
