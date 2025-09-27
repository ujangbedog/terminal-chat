/// Event handling for P2P chat client
use crate::client::constants::*;
use crate::client::history::MessageHistory;
use shared::{P2PEvent, P2PMessage};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::{info, error, debug};

/// Event handler for P2P events
pub struct EventHandler;

impl EventHandler {
    /// Handle incoming P2P events
    pub fn handle_event(
        event: P2PEvent, 
        _username: &str, 
        history: &MessageHistory,
        peer_addresses: &HashMap<String, SocketAddr>
    ) {
        match event {
            P2PEvent::PeerConnected { peer_id, addr, username: peer_username } => {
                let formatted_message = format!(
                    "{}*** 🟢 {}@{} connected from {}{}",
                    COLOR_GREEN, peer_username, addr, addr, COLOR_RESET
                );
                info!("Peer connected: {} ({})", peer_username, peer_id);
                history.add_message(formatted_message);
                history.refresh_display();
            }
            
            P2PEvent::PeerDisconnected { peer_id, reason } => {
                let formatted_message = format!(
                    "{}*** 🔴 Peer {} disconnected: {}{}",
                    COLOR_YELLOW, &peer_id[..8], reason, COLOR_RESET
                );
                info!("Peer disconnected: {} (reason: {})", peer_id, reason);
                history.add_message(formatted_message);
                history.refresh_display();
            }
            
            P2PEvent::MessageReceived { message, from_peer } => {
                match message {
                    P2PMessage::ChatMessage { username: sender_username, content, .. } => {
                        // Try to get the actual address from peer_addresses mapping
                        let sender_addr = peer_addresses.get(&from_peer)
                            .map(|addr| addr.to_string())
                            .unwrap_or_else(|| from_peer.clone());
                            
                        let formatted_message = format!(
                            "{}{}@{} > {}{}",
                            COLOR_CYAN, sender_username, sender_addr, content, COLOR_RESET
                        );
                        debug!("Received chat message from {}: {}", sender_username, content);
                        history.add_message(formatted_message);
                        history.refresh_display();
                    }
                    _ => {
                        debug!("Received non-chat message: {:?}", message);
                    }
                }
            }
            
            P2PEvent::TopologyChanged { .. } => {
                debug!("Network topology changed");
                // Could add UI notification here if needed
            }
            
            P2PEvent::PeersDiscovered { peers } => {
                let formatted_message = format!(
                    "{}*** 🔍 Discovered {} new peer(s){}",
                    COLOR_CYAN, peers.len(), COLOR_RESET
                );
                info!("Discovered {} new peers", peers.len());
                history.add_message(formatted_message);
                history.refresh_display();
            }
            
            P2PEvent::Error { error, .. } => {
                let formatted_message = format!(
                    "{}*** ❌ P2P Error: {}{}",
                    COLOR_YELLOW, error, COLOR_RESET
                );
                error!("P2P Error: {}", error);
                history.add_message(formatted_message);
                history.refresh_display();
            }
        }
    }
}
