//! Command line argument parsing for P2P core

use std::net::SocketAddr;
use shared::config::{DEFAULT_HOST_LOCALHOST, FIXED_PORT, find_available_port};

/// Parsed command line arguments
pub struct P2PArgs {
    pub username: String,
    pub final_host: String,
    pub final_port: u16,
    pub bootstrap_peers: Vec<SocketAddr>,
    pub enable_tls: bool,
}

/// Parse command line arguments
pub fn parse_args(args: &[String]) -> Result<Option<P2PArgs>, Box<dyn std::error::Error>> {
    // Use hardcoded configuration values
    let default_host = DEFAULT_HOST_LOCALHOST;
    let default_port = FIXED_PORT;
    
    // Parse command line arguments
    let mut username = "Anonymous".to_string();
    let mut listen_port: Option<u16> = None;
    let mut bootstrap_peers: Vec<SocketAddr> = vec![];
    let mut custom_host: Option<String> = None;
    let enable_tls = true; // Always true
    
    let mut i = 1; // Skip program name only
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 < args.len() {
                    username = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --username requires a value");
                    return Ok(None);
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    listen_port = Some(args[i + 1].parse()?);
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value");
                    return Ok(None);
                }
            }
            "--host" => {
                if i + 1 < args.len() {
                    custom_host = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --host requires a value");
                    return Ok(None);
                }
            }
            "--bootstrap" | "-b" => {
                if i + 1 < args.len() {
                    let addr: SocketAddr = args[i + 1].parse()?;
                    bootstrap_peers.push(addr);
                    i += 2;
                } else {
                    eprintln!("Error: --bootstrap requires a value");
                    return Ok(None);
                }
            }
            "--help" | "-h" => {
                super::print_help();
                return Ok(None);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                super::print_help();
                return Ok(None);
            }
        }
    }
    
    // Validate username
    if username.trim().is_empty() {
        eprintln!("Error: Username cannot be empty");
        return Ok(None);
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
    
    Ok(Some(P2PArgs {
        username,
        final_host,
        final_port,
        bootstrap_peers,
        enable_tls,
    }))
}
