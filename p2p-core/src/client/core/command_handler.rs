//! Command handling for P2P chat client

use crate::ui::{ChatUI, MessageType};
use super::super::history::MessageHistory;
use std::collections::HashMap;
use std::net::SocketAddr;

/// Handles chat commands
pub struct CommandHandler;

impl CommandHandler {
    /// Handle chat commands
    pub async fn handle_command(
        command: &str,
        chat_ui: &mut ChatUI,
        connected_peers: &HashMap<String, String>,
        peer_addresses: &HashMap<String, SocketAddr>,
        history: &MessageHistory,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"/help") => {
                Self::show_help(chat_ui).await?;
            }
            Some(&"/quit") | Some(&"/exit") => {
                chat_ui.add_message(
                    "System".to_string(),
                    "👋 Goodbye! Shutting down...".to_string(),
                    MessageType::SystemMessage,
                )?;
                return Ok(false);
            }
            Some(&"/peers") => {
                Self::show_peers(chat_ui, connected_peers, peer_addresses).await?;
            }
            Some(&"/clear") => {
                chat_ui.refresh_display()?;
            }
            Some(&"/history") => {
                Self::show_history(chat_ui, history).await?;
            }
            Some(cmd) => {
                chat_ui.add_message(
                    "System".to_string(),
                    format!("❓ Unknown command: {}. Type /help for available commands.", cmd),
                    MessageType::SystemMessage,
                )?;
            }
            None => {}
        }
        
        Ok(true)
    }

    /// Show help information
    async fn show_help(chat_ui: &mut ChatUI) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let help_messages = vec![
            "📖 Available Commands:",
            "/help     - Show this help message",
            "/peers    - List connected peers", 
            "/history  - Show message history",
            "/clear    - Clear chat display",
            "/quit     - Exit the chat",
            "",
            "💡 Tips:",
            "• Just type your message and press Enter to send",
            "• Messages are sent to all connected peers",
            "• Use Ctrl+C to force quit anytime",
        ];
        
        for msg in help_messages {
            chat_ui.add_message(
                "System".to_string(),
                msg.to_string(),
                MessageType::SystemMessage,
            )?;
        }
        
        Ok(())
    }

    /// Show connected peers
    async fn show_peers(
        chat_ui: &mut ChatUI,
        connected_peers: &HashMap<String, String>,
        peer_addresses: &HashMap<String, SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if connected_peers.is_empty() {
            chat_ui.add_message(
                "System".to_string(),
                "👥 No peers currently connected".to_string(),
                MessageType::SystemMessage,
            )?;
        } else {
            chat_ui.add_message(
                "System".to_string(),
                format!("👥 Connected Peers ({}):", connected_peers.len()),
                MessageType::SystemMessage,
            )?;
            
            for (peer_id, username) in connected_peers {
                let addr = peer_addresses.get(peer_id)
                    .map(|a| format!(" ({})", a))
                    .unwrap_or_default();
                
                chat_ui.add_message(
                    "System".to_string(),
                    format!("  • {}{}", username, addr),
                    MessageType::SystemMessage,
                )?;
            }
        }
        
        Ok(())
    }

    /// Show message history
    async fn show_history(
        chat_ui: &mut ChatUI,
        _history: &MessageHistory,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get recent messages from history (placeholder for now)
        let messages = vec!["Message history feature coming soon...".to_string()];
        
        if messages.is_empty() {
            chat_ui.add_message(
                "System".to_string(),
                "📜 No message history available".to_string(),
                MessageType::SystemMessage,
            )?;
        } else {
            chat_ui.add_message(
                "System".to_string(),
                "📜 Recent Message History:".to_string(),
                MessageType::SystemMessage,
            )?;
            
            for msg in messages {
                chat_ui.add_message(
                    "System".to_string(),
                    format!("  {}", msg),
                    MessageType::SystemMessage,
                )?;
            }
        }
        
        Ok(())
    }
}
