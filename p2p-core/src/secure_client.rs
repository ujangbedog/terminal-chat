//! Secure P2P client with session key management

use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

use shared::crypto::{
    SessionManager, HandshakeManager, MessageCrypto, EncryptedMessage, 
    HandshakeData, PeerInfo, PlainMessage, MessageSequenceManager
};
use crate::ui::ChatUI;
use crate::P2PNode;

/// Secure P2P client with session key management
pub struct SecureP2PChatClient {
    /// Our identity information
    username: String,
    fingerprint: String,
    public_key: Vec<u8>,
    
    /// P2P networking
    node: P2PNode,
    
    /// Session management
    session_manager: SessionManager,
    handshake_manager: HandshakeManager,
    sequence_manager: MessageSequenceManager,
    
    /// UI
    chat_ui: ChatUI,
    
    /// Connected peers (fingerprint -> username)
    connected_peers: HashMap<String, String>,
    /// Peer addresses (fingerprint -> address)
    peer_addresses: HashMap<String, SocketAddr>,
    
    /// Running state
    running: bool,
}

impl SecureP2PChatClient {
    /// Create a new secure P2P chat client
    pub fn new(
        username: String,
        fingerprint: String,
        public_key: Vec<u8>,
        node: P2PNode,
    ) -> Self {
        let handshake_manager = HandshakeManager::new(
            username.clone(),
            fingerprint.clone(),
            public_key.clone(),
        );
        
        let chat_ui = ChatUI::new(username.clone());
        
        Self {
            username,
            fingerprint,
            public_key,
            node,
            session_manager: SessionManager::new(),
            handshake_manager,
            sequence_manager: MessageSequenceManager::new(),
            chat_ui,
            connected_peers: HashMap::new(),
            peer_addresses: HashMap::new(),
            running: false,
        }
    }
    
    /// Start the secure chat client
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.running = true;
        
        // Initialize UI
        self.chat_ui.initialize()?;
        self.chat_ui.add_message("🔐 Secure P2P Chat initialized with post-quantum cryptography");
        self.chat_ui.add_message(&format!("🆔 Your identity: {} ({})", self.username, &self.fingerprint[..8]));
        
        // Start P2P node
        let (event_tx, mut event_rx) = mpsc::channel(100);
        self.node.start(event_tx).await?;
        
        // Start input handler
        let (input_tx, mut input_rx) = mpsc::channel(100);
        let input_handle = tokio::spawn(async move {
            Self::handle_input(input_tx).await;
        });
        
        // Main event loop
        while self.running {
            tokio::select! {
                // Handle P2P events
                Some(event) = event_rx.recv() => {
                    if let Err(e) = self.handle_p2p_event(event).await {
                        error!("Error handling P2P event: {}", e);
                    }
                }
                
                // Handle user input
                Some(input) = input_rx.recv() => {
                    if let Err(e) = self.handle_user_input(&input).await {
                        error!("Error handling user input: {}", e);
                    }
                }
                
                // Cleanup expired sessions periodically
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                    self.session_manager.cleanup_expired();
                }
            }
        }
        
        // Cleanup
        input_handle.abort();
        self.shutdown().await;
        
        Ok(())
    }
    
    /// Handle P2P events
    async fn handle_p2p_event(
        &mut self,
        event: shared::P2PEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event {
            shared::P2PEvent::PeerConnected { peer_id, address } => {
                info!("Peer connected: {} from {}", peer_id, address);
                self.peer_addresses.insert(peer_id.clone(), address);
                
                // Initiate handshake
                match self.handshake_manager.initiate_handshake(&peer_id) {
                    Ok(handshake_data) => {
                        self.send_handshake(handshake_data).await?;
                    }
                    Err(e) => {
                        warn!("Failed to initiate handshake with {}: {}", peer_id, e);
                    }
                }
            }
            
            shared::P2PEvent::PeerDisconnected { peer_id } => {
                info!("Peer disconnected: {}", peer_id);
                self.handle_peer_disconnect(&peer_id).await;
            }
            
            shared::P2PEvent::MessageReceived { peer_id, data } => {
                self.handle_received_data(&peer_id, data).await?;
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle received data (handshake or encrypted message)
    async fn handle_received_data(
        &mut self,
        peer_id: &str,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Try to deserialize as handshake first
        if let Ok(handshake_data) = serde_json::from_slice::<HandshakeData>(&data) {
            self.handle_handshake(peer_id, handshake_data).await?;
            return Ok(());
        }
        
        // Try to deserialize as encrypted message
        if let Ok(encrypted_message) = serde_json::from_slice::<EncryptedMessage>(&data) {
            self.handle_encrypted_message(peer_id, encrypted_message).await?;
            return Ok(());
        }
        
        warn!("Received unknown data format from peer: {}", peer_id);
        Ok(())
    }
    
    /// Handle handshake from peer
    async fn handle_handshake(
        &mut self,
        peer_id: &str,
        handshake_data: HandshakeData,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing handshake from peer: {}", peer_id);
        
        match self.handshake_manager.process_handshake(handshake_data) {
            Ok((session_key, response)) => {
                // Store session key
                let peer_fingerprint = session_key.peer_fingerprint().to_string();
                self.session_manager.add_session(peer_fingerprint.clone(), session_key);
                
                // Store peer info
                self.connected_peers.insert(peer_fingerprint.clone(), handshake_data.peer_info.username.clone());
                
                // Send response if needed
                if let Some(response_data) = response {
                    self.send_handshake(response_data).await?;
                }
                
                // Notify UI
                self.chat_ui.add_message(&format!(
                    "🤝 Secure session established with {} ({})",
                    handshake_data.peer_info.username,
                    &peer_fingerprint[..8]
                ));
                
                info!("Secure session established with peer: {}", peer_fingerprint);
            }
            Err(e) => {
                warn!("Handshake failed with peer {}: {}", peer_id, e);
                self.handshake_manager.mark_failed(peer_id, e.to_string());
            }
        }
        
        Ok(())
    }
    
    /// Handle encrypted message from peer
    async fn handle_encrypted_message(
        &mut self,
        peer_id: &str,
        encrypted_message: EncryptedMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let sender_fingerprint = &encrypted_message.sender_fingerprint;
        
        // Verify message integrity
        MessageCrypto::verify_message_integrity(&encrypted_message, sender_fingerprint, 300)?;
        
        // Validate sequence
        self.sequence_manager.validate_sequence(sender_fingerprint, encrypted_message.sequence)?;
        
        // Get session key
        let session_key = match self.session_manager.get_session(sender_fingerprint) {
            Some(key) => key,
            None => {
                warn!("No session key for peer: {}", sender_fingerprint);
                return Ok(());
            }
        };
        
        // Decrypt message
        match MessageCrypto::decrypt_message(session_key, &encrypted_message) {
            Ok(plain_message) => {
                // Display message in UI
                match encrypted_message.message_type {
                    shared::crypto::MessageType::Text => {
                        self.chat_ui.add_message(&format!(
                            "💬 {}: {}",
                            plain_message.sender,
                            plain_message.content
                        ));
                    }
                    shared::crypto::MessageType::System => {
                        self.chat_ui.add_message(&format!(
                            "ℹ️  {}",
                            plain_message.content
                        ));
                    }
                    _ => {
                        // Handle other message types
                    }
                }
            }
            Err(e) => {
                warn!("Failed to decrypt message from {}: {}", sender_fingerprint, e);
            }
        }
        
        Ok(())
    }
    
    /// Handle user input
    async fn handle_user_input(
        &mut self,
        input: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input = input.trim();
        
        if input.is_empty() {
            return Ok(());
        }
        
        // Handle commands
        if input.starts_with('/') {
            return self.handle_command(input).await;
        }
        
        // Send message to all connected peers
        self.send_message_to_all(input).await?;
        
        Ok(())
    }
    
    /// Send message to all connected peers
    async fn send_message_to_all(
        &mut self,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let plain_message = MessageCrypto::create_text_message(
            self.username.clone(),
            content.to_string(),
        );
        
        // Send to each peer with active session
        for peer_fingerprint in self.session_manager.active_peers() {
            if let Some(session_key) = self.session_manager.get_session(&peer_fingerprint) {
                let sequence = self.sequence_manager.next_sequence();
                
                match MessageCrypto::encrypt_message(session_key, &plain_message, sequence) {
                    Ok(encrypted_message) => {
                        let data = serde_json::to_vec(&encrypted_message)?;
                        self.node.send_message(&peer_fingerprint, data).await?;
                    }
                    Err(e) => {
                        warn!("Failed to encrypt message for peer {}: {}", peer_fingerprint, e);
                    }
                }
            }
        }
        
        // Show in our UI
        self.chat_ui.add_message(&format!("💬 {}: {}", self.username, content));
        
        Ok(())
    }
    
    /// Send handshake data to peer
    async fn send_handshake(
        &mut self,
        handshake_data: HandshakeData,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = serde_json::to_vec(&handshake_data)?;
        // Send to the peer (implementation depends on P2P layer)
        // For now, we'll broadcast to all connected peers
        self.node.broadcast_message(data).await?;
        Ok(())
    }
    
    /// Handle peer disconnect
    async fn handle_peer_disconnect(&mut self, peer_id: &str) {
        // Remove session
        if let Some(_) = self.session_manager.remove_session(peer_id) {
            info!("Removed session for disconnected peer: {}", peer_id);
        }
        
        // Remove from connected peers
        if let Some(username) = self.connected_peers.remove(peer_id) {
            self.chat_ui.add_message(&format!("👋 {} disconnected", username));
        }
        
        // Remove address
        self.peer_addresses.remove(peer_id);
        
        // Reset sequence
        self.sequence_manager.reset_peer_sequence(peer_id);
    }
    
    /// Handle commands
    async fn handle_command(
        &mut self,
        command: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"/quit") | Some(&"/exit") => {
                self.chat_ui.add_message("👋 Disconnecting from secure chat...");
                self.running = false;
            }
            Some(&"/sessions") => {
                self.show_session_info().await;
            }
            Some(&"/peers") => {
                self.show_peer_info().await;
            }
            Some(&"/help") => {
                self.show_help().await;
            }
            _ => {
                self.chat_ui.add_message("❌ Unknown command. Type /help for available commands.");
            }
        }
        
        Ok(())
    }
    
    /// Show session information
    async fn show_session_info(&mut self) {
        self.chat_ui.add_message("🔐 Active Sessions:");
        
        if self.session_manager.session_count() == 0 {
            self.chat_ui.add_message("   No active sessions");
            return;
        }
        
        for peer_fingerprint in self.session_manager.active_peers() {
            if let Some(username) = self.connected_peers.get(&peer_fingerprint) {
                self.chat_ui.add_message(&format!(
                    "   {} ({}) - Session active",
                    username,
                    &peer_fingerprint[..8]
                ));
            }
        }
    }
    
    /// Show peer information
    async fn show_peer_info(&mut self) {
        self.chat_ui.add_message("👥 Connected Peers:");
        
        if self.connected_peers.is_empty() {
            self.chat_ui.add_message("   No peers connected");
            return;
        }
        
        for (fingerprint, username) in &self.connected_peers {
            let has_session = self.session_manager.has_session(fingerprint);
            let status = if has_session { "🔐 Secure" } else { "⚠️  Handshaking" };
            
            self.chat_ui.add_message(&format!(
                "   {} ({}) - {}",
                username,
                &fingerprint[..8],
                status
            ));
        }
    }
    
    /// Show help
    async fn show_help(&mut self) {
        self.chat_ui.add_message("📚 Available Commands:");
        self.chat_ui.add_message("   /help      - Show this help");
        self.chat_ui.add_message("   /sessions  - Show active secure sessions");
        self.chat_ui.add_message("   /peers     - Show connected peers");
        self.chat_ui.add_message("   /quit      - Exit chat");
    }
    
    /// Shutdown the client
    async fn shutdown(&mut self) {
        self.chat_ui.add_message("🔌 Shutting down secure chat client...");
        
        // Stop P2P node
        self.node.stop().await;
        
        // Clear sessions
        self.session_manager = SessionManager::new();
        
        // Cleanup UI
        self.chat_ui.cleanup();
    }
    
    /// Handle input in separate task
    async fn handle_input(input_tx: mpsc::Sender<String>) {
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        
        for line in stdin.lock().lines() {
            match line {
                Ok(input) => {
                    if input_tx.send(input).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
}
