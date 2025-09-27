/// Modular P2P Chat Client
/// 
/// This module provides a clean, modular implementation of the P2P chat client
/// broken down into focused components for better maintainability.

pub mod constants;
pub mod display;
pub mod history;
pub mod events;
pub mod commands;

use self::constants::*;
use self::display::DisplayManager;
use self::history::MessageHistory;
use self::events::EventHandler;
use self::commands::CommandHandler;

use shared::{P2PNode, P2PNodeConfig, P2PEvent};
use shared::p2p::discovery::{DiscoveryMethod, DEFAULT_MULTICAST_ADDR};
use std::net::SocketAddr;
use std::io::{self, Write};
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, error};

/// Main P2P Chat Client
pub struct P2PChatClient {
    node: P2PNode,
    event_rx: mpsc::Receiver<P2PEvent>,
    username: String,
    running: bool,
    prompt: String,
    history: MessageHistory,
    connected_peers: std::collections::HashMap<String, String>, // peer_id -> username
    peer_addresses: std::collections::HashMap<String, SocketAddr>, // peer_id -> address
}

impl P2PChatClient {
    /// Create a new P2P chat client
    pub async fn new(
        username: String,
        listen_port: Option<u16>,
        bootstrap_peers: Vec<SocketAddr>,
        enable_tls: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let listen_addr = if let Some(port) = listen_port {
            if port == 0 {
                "127.0.0.1:0".parse()? // Random port
            } else {
                format!("127.0.0.1:{}", port).parse()?
            }
        } else {
            "127.0.0.1:0".parse()? // Random port
        };

        // Configure P2P node
        let config = P2PNodeConfig {
            username: username.clone(),
            listen_addr,
            enable_tls,
            discovery_methods: vec![
                DiscoveryMethod::Multicast {
                    multicast_addr: DEFAULT_MULTICAST_ADDR.parse()?,
                    interface: None,
                },
            ],
            bootstrap_peers,
            connection_timeout_secs: 30,
            heartbeat_interval_secs: 60,
            max_connections: 50,
        };

        let (mut node, event_rx) = P2PNode::new(config).await?;

        // Start the node
        node.start().await?;

        // Create prompt with actual listening address and color
        let listen_addr = node.listen_addr().await;
        let prompt = format!("{}{}@{}$ {}", COLOR_CYAN, username, listen_addr, COLOR_RESET);

        Ok(Self {
            node,
            event_rx,
            username,
            running: true,
            prompt,
            history: MessageHistory::new(100), // Keep last 100 messages
            connected_peers: std::collections::HashMap::new(),
            peer_addresses: std::collections::HashMap::new(),
        })
    }

    /// Start the chat client
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Clear screen and setup UI
        print!("\x1b[2J\x1b[H"); // Clear screen and move cursor to top
        
        // Display initial UI
        self.display_startup_info().await;
        DisplayManager::draw_chat_area();
        DisplayManager::draw_input_area(&self.prompt);

        // Run the main event loop
        self.run_event_loop().await?;
        
        // Cleanup
        self.shutdown().await;
        Ok(())
    }

    /// Display startup information
    async fn display_startup_info(&self) {
        let listen_addr = self.node.listen_addr().await;
        
        println!("🚀 P2P Chat Client Started!");
        
        // Wait a moment then clear terminal for clean display
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        print!("\x1b[2J\x1b[H"); // Clear screen and move cursor to top
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("👤 Username: {} | 🔗 Connected to: Waiting... | 🌐 Port: {} | 🔒 TLS: Enabled", 
                 self.username, listen_addr.port());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    /// Update header with connected peer information
    async fn update_header(&self) {
        let listen_addr = self.node.listen_addr().await;
        let connected_to = if self.connected_peers.is_empty() {
            "Waiting...".to_string()
        } else {
            let usernames: Vec<_> = self.connected_peers.values().cloned().collect();
            usernames.join(", ")
        };
        
        // Save current cursor position
        print!("\x1b[s");
        
        // Move to header line and update
        print!("\x1b[2;1H"); // Move to line 2
        print!("\x1b[K"); // Clear line
        println!("👤 Username: {} | 🔗 Connected to: {} | 🌐 Port: {} | 🔒 TLS: Enabled", 
                 self.username, connected_to, listen_addr.port());
        
        // Restore cursor position
        print!("\x1b[u");
        io::stdout().flush().unwrap();
    }

    /// Main event loop
    async fn run_event_loop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use simple line-based input instead of raw mode
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        while self.running {
            tokio::select! {
                // Handle P2P events
                event = self.event_rx.recv() => {
                    match event {
                        Some(event) => {
                            // Update connected peers tracking
                            match &event {
                                P2PEvent::PeerConnected { peer_id, addr, username: peer_username } => {
                                    // Store actual username and address
                                    self.connected_peers.insert(peer_id.clone(), peer_username.clone());
                                    self.peer_addresses.insert(peer_id.clone(), *addr);
                                    self.update_header().await;
                                }
                                P2PEvent::PeerDisconnected { peer_id, .. } => {
                                    self.connected_peers.remove(peer_id);
                                    self.peer_addresses.remove(peer_id);
                                    self.update_header().await;
                                }
                                _ => {}
                            }
                            
                            EventHandler::handle_event(event, &self.username, &self.history, &self.peer_addresses);
                            DisplayManager::clear_input_area(&self.prompt);
                        }
                        None => {
                            error!("Event channel closed");
                            // Force terminate when event channel closes (indicates network failure)
                            force_cleanup_terminal("Network connection lost");
                        }
                    }
                }
                
                // Handle user input
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let input = line.trim().to_string();
                            line.clear();
                            
                            if !input.is_empty() {
                                if !CommandHandler::handle_command(&input, &mut self.node, &self.username, &self.history).await? {
                                    break;
                                }
                                // Clear input area after command
                                DisplayManager::clear_input_area(&self.prompt);
                            }
                        }
                        Err(e) => {
                            error!("Error reading input: {}", e);
                            break;
                        }
                    }
                }
            }

            if !self.running {
                break;
            }
        }

        Ok(())
    }

    /// Shutdown the client
    async fn shutdown(&mut self) {
        self.running = false;
        info!("Shutting down P2P chat client");
        
        // Add shutdown message to history
        let formatted_message = format!(
            "{}*** 👋 Chat client shutting down...{}",
            COLOR_YELLOW, COLOR_RESET
        );
        self.history.add_message(formatted_message);
        self.history.refresh_display();
        
        // Move cursor to bottom
        println!("\n{}*** Goodbye! 👋{}", COLOR_BOLD, COLOR_RESET);
    }
}
