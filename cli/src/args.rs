//! Command-line argument definitions using clap

use clap::{Parser, Subcommand};
use std::net::SocketAddr;

/// Terminal Chat Client - A modern P2P chat application
#[derive(Parser)]
#[command(name = "terminal-chat")]
#[command(about = "A modern terminal-based P2P chat application")]
#[command(version = "0.1.0")]
#[command(author = "Terminal Chat Team")]
#[command(long_about = None)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start a P2P chat session
    P2p {
        /// Username for the chat session
        #[arg(short, long)]
        username: String,

        /// Port to listen on
        #[arg(short, long)]
        port: Option<u16>,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Bootstrap peer addresses to connect to
        #[arg(short, long)]
        bootstrap: Vec<SocketAddr>,

        /// Disable TLS encryption
        #[arg(long)]
        no_tls: bool,
    },
    /// Interactive menu mode (default)
    Menu,
    /// Show configuration
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
