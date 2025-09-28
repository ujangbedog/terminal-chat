//! Event handling for P2P chat client

use crate::ui::{ChatUI, MessageType};
use shared::P2PEvent;
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::{info, error};
use colored::*;

/// Handles P2P events for the chat client
pub struct EventHandler;

impl EventHandler {
    /// Handle P2P events with beautiful display
    pub async fn handle_p2p_event(
        event: P2PEvent,
        chat_ui: &mut ChatUI,
        connected_peers: &mut HashMap<String, String>,
        peer_addresses: &mut HashMap<String, SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event {
            P2PEvent::PeerConnected { peer_id, addr, username: peer_username } => {
                // Store peer info
                connected_peers.insert(peer_id.clone(), peer_username.clone());
                peer_addresses.insert(peer_id.clone(), addr);
                
                // Update UI
                let peer_list: Vec<String> = connected_peers.values().cloned().collect();
                chat_ui.update_connected_peers(peer_list)?;
                
                // Add connection message
                chat_ui.add_message(
                    "System".to_string(),
                    format!("🔗 {} connected from {}", peer_username.bright_green(), addr),
                    MessageType::ConnectionInfo,
                )?;
                
                info!("Peer connected: {} ({})", peer_username, addr);
            }
            
            P2PEvent::PeerDisconnected { peer_id, reason } => {
                // Get username before removing
                let peer_username = connected_peers.get(&peer_id).cloned().unwrap_or("Unknown".to_string());
                
                // Remove peer info
                connected_peers.remove(&peer_id);
                let addr = peer_addresses.remove(&peer_id);
                
                // Update UI
                let peer_list: Vec<String> = connected_peers.values().cloned().collect();
                chat_ui.update_connected_peers(peer_list)?;
                
                // Add disconnection message
                let addr_str = addr.map(|a| format!(" ({})", a)).unwrap_or_default();
                chat_ui.add_message(
                    "System".to_string(),
                    format!("🔌 {} disconnected: {}{}", peer_username.bright_red(), reason, addr_str),
                    MessageType::ConnectionInfo,
                )?;
                
                info!("Peer disconnected: {} ({})", peer_username, reason);
            }
            
            P2PEvent::MessageReceived { message, from_peer: _ } => {
                // Extract message content
                if let shared::message::P2PMessage::ChatMessage { username, content, .. } = &message {
                    // Add message to chat
                    chat_ui.add_message(
                        username.clone(),
                        content.clone(),
                        MessageType::UserMessage,
                    )?;
                    
                    info!("Message from {}: {}", username, content);
                }
            }
            
            P2PEvent::PeersDiscovered { peers } => {
                chat_ui.add_message(
                    "System".to_string(),
                    format!("🔍 Discovered {} new peers", peers.len()),
                    MessageType::SystemMessage,
                )?;
            }
            
            P2PEvent::TopologyChanged { connected_peers: topology_peers } => {
                // Update peer list from topology change
                let peer_names: Vec<String> = topology_peers.iter()
                    .map(|p| p.username.clone())
                    .collect();
                
                chat_ui.update_connected_peers(peer_names)?;
                
                chat_ui.add_message(
                    "System".to_string(),
                    format!("🌐 Network topology updated. {} peers connected", topology_peers.len()),
                    MessageType::SystemMessage,
                )?;
            }
            
            P2PEvent::Error { error, peer_id } => {
                let error_msg = if let Some(pid) = peer_id {
                    format!("Error from {}: {}", pid, error)
                } else {
                    format!("Error: {}", error)
                };
                
                chat_ui.add_message(
                    "System".to_string(),
                    error_msg.clone(),
                    MessageType::ErrorMessage,
                )?;
                error!("P2P Error: {}", error_msg);
            }
        }
        
        Ok(())
    }
}
