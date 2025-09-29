//! P2P command handlers

use colored::*;
use std::process::Command;
use std::net::SocketAddr;

/// Handle P2P chat command
pub async fn handle_p2p_command(
    username: String,
    port: Option<u16>,
    host: String,
    bootstrap: Vec<SocketAddr>,
    no_tls: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
