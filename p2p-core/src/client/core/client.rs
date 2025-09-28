//! Main P2P Chat Client implementation

use crate::ui::{ChatUI, MessageType};
use super::super::constants::*;
use super::super::history::MessageHistory;
use super::{EventHandler, CommandHandler};

use shared::{P2PNode, P2PNodeConfig, P2PEvent};
use shared::p2p::discovery::{DiscoveryMethod, DEFAULT_MULTICAST_ADDR};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use colored::*;

/// P2P Chat Client with beautiful UI
pub struct P2PChatClient {
    node: P2PNode,
    event_rx: mpsc::Receiver<P2PEvent>,
    username: String,
    running: bool,
    chat_ui: ChatUI,
    history: MessageHistory,
    connected_peers: HashMap<String, String>, // peer_id -> username
    peer_addresses: HashMap<String, SocketAddr>, // peer_id -> address
}

impl P2PChatClient {
    /// Create a new P2P chat client
    pub async fn new(
        username: String,
        listen_host: Option<String>,
        listen_port: Option<u16>,
        bootstrap_peers: Vec<SocketAddr>,
        enable_tls: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let host = listen_host.unwrap_or_else(|| "127.0.0.1".to_string());
        let port = listen_port.unwrap_or(0);
        
        let listen_addr = if port == 0 {
            format!("{}:0", host).parse()? // Random port
        } else {
            format!("{}:{}", host, port).parse()?
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

        // Create beautiful chat UI
        let chat_ui = ChatUI::new(username.clone(), 100)?;

        Ok(Self {
            node,
            event_rx,
            username,
            running: true,
            chat_ui,
            history: MessageHistory::new(100),
            connected_peers: HashMap::new(),
            peer_addresses: HashMap::new(),
        })
    }

    /// Start the chat client
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Show welcome screen
        self.chat_ui.show_welcome()?;
        
        // Show connection progress
        self.chat_ui.show_connection_progress("Initializing P2P connection...").await?;
        
        // Initialize the beautiful chat interface
        self.chat_ui.initialize()?;
        
        // Add welcome message
        let listen_addr = self.node.listen_addr().await;
        self.chat_ui.add_message(
            "System".to_string(),
            format!("🚀 P2P Chat started! Listening on {}", listen_addr),
            MessageType::SystemMessage,
        )?;
        
        // Add help message
        self.chat_ui.add_message(
            "System".to_string(),
            "💡 Type '/help' for commands, '/quit' to exit".to_string(),
            MessageType::SystemMessage,
        )?;

        // Run the main event loop
        self.run_event_loop().await?;
        
        // Cleanup
        self.shutdown().await;
        Ok(())
    }

    /// Main event loop with beautiful UI
    async fn run_event_loop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create a channel for input handling
        let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<String>(100);
        
        // Spawn input handling task
        let input_tx_clone = input_tx.clone();
        tokio::spawn(async move {
            loop {
                let input = tokio::task::spawn_blocking(|| {
                    use std::io::{stdin, BufRead};
                    let stdin = stdin();
                    let mut line = String::new();
                    match stdin.lock().read_line(&mut line) {
                        Ok(_) => Some(line.trim().to_string()),
                        Err(_) => None,
                    }
                }).await;
                
                match input {
                    Ok(Some(line)) => {
                        if input_tx_clone.send(line).await.is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });
        
        // Position cursor initially
        self.chat_ui.position_cursor_for_input()?;
        
        while self.running {
            tokio::select! {
                // Handle P2P events
                event = self.event_rx.recv() => {
                    match event {
                        Some(event) => {
                            EventHandler::handle_p2p_event(
                                event,
                                &mut self.chat_ui,
                                &mut self.connected_peers,
                                &mut self.peer_addresses,
                            ).await?;
                        }
                        None => {
                            error!("Event channel closed");
                            self.chat_ui.add_message(
                                "System".to_string(),
                                "❌ Network connection lost".to_string(),
                                MessageType::ErrorMessage,
                            )?;
                            break;
                        }
                    }
                }
                
                // Handle user input
                input = input_rx.recv() => {
                    match input {
                        Some(input) => {
                            if !self.handle_user_input(&input).await? {
                                break;
                            }
                        }
                        None => {
                            error!("Input channel closed");
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

    /// Handle user input with command processing
    async fn handle_user_input(&mut self, input: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let input = input.trim();
        
        // Clear input area first (this clears the typed text)
        self.chat_ui.clear_input_area()?;
        
        if input.is_empty() {
            return Ok(true);
        }
        
        // Handle commands
        if input.starts_with('/') {
            return CommandHandler::handle_command(
                input,
                &mut self.chat_ui,
                &self.connected_peers,
                &self.peer_addresses,
                &self.history,
            ).await;
        }
        
        // Regular message - send to all connected peers
        if self.connected_peers.is_empty() {
            self.chat_ui.add_message(
                "System".to_string(),
                "⚠️  No peers connected. Your message was not sent.".to_string(),
                MessageType::SystemMessage,
            )?;
            return Ok(true);
        }
        
        // Display message locally first
        self.chat_ui.add_message(
            self.username.clone(),
            input.to_string(),
            MessageType::UserMessage,
        )?;
        
        // Send chat message to network
        if let Err(e) = self.node.send_chat_message(input.to_string()).await {
            warn!("Failed to send message: {}", e);
            self.chat_ui.add_message(
                "System".to_string(),
                format!("⚠️  Failed to send message: {}", e),
                MessageType::ErrorMessage,
            )?;
        }
        
        // Add to history
        let formatted_message = format!("{}: {}", self.username, input);
        self.history.add_message(formatted_message);
        
        Ok(true)
    }

    /// Shutdown the client
    async fn shutdown(&mut self) {
        self.running = false;
        info!("Shutting down P2P chat client");
        
        self.chat_ui.add_message(
            "System".to_string(),
            "🔌 Chat client shutting down...".to_string(),
            MessageType::SystemMessage,
        ).ok();
        
        // Give a moment for the message to display
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        println!("\n{}*** Thanks for using P2P Terminal Chat! 👋 ***{}", COLOR_BOLD.bright_green(), COLOR_RESET);
    }
}
