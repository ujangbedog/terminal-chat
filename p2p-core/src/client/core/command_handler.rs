//! Command handling for P2P chat client

use crate::ui::{ChatUI, MessageType};
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
        is_owner: bool,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"/help") => {
                Self::show_help(chat_ui).await?;
            }
            Some(&"/quit") | Some(&"/exit") => {
                // Show appropriate goodbye message
                if is_owner {
                    chat_ui.add_message(
                        "System".to_string(),
                        "👋 Owner disconnecting. Goodbye!".to_string(),
                        MessageType::SystemMessage,
                    )?;
                } else {
                    chat_ui.add_message(
                        "System".to_string(),
                        "👋 Goodbye! Exiting program...".to_string(),
                        MessageType::SystemMessage,
                    )?;
                }
                
                // Brief delay for message display
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Clear terminal before exit
                use crossterm::{execute, terminal::{Clear, ClearType}, cursor::MoveTo};
                use std::io;
                execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0)).ok();
                
                // Exit program directly - both owner and peer
                std::process::exit(0);
            }
            Some(&"/peers") => {
                Self::show_peers(chat_ui, connected_peers, peer_addresses).await?;
            }
            Some(&"/clear") => {
                chat_ui.clear_chat()?;
            }
            Some(&"/stats") => {
                Self::show_stats(chat_ui, connected_peers, peer_addresses).await?;
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
            "/stats    - Show detailed peer statistics",
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

    /// Show detailed peer statistics
    async fn show_stats(
        chat_ui: &mut ChatUI,
        connected_peers: &HashMap<String, String>,
        peer_addresses: &HashMap<String, SocketAddr>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if connected_peers.is_empty() {
            chat_ui.add_message(
                "System".to_string(),
                "📊 No peers currently connected".to_string(),
                MessageType::SystemMessage,
            )?;
            return Ok(());
        }

        chat_ui.add_message(
            "System".to_string(),
            "📊 Detailed Peer Statistics:".to_string(),
            MessageType::SystemMessage,
        )?;
        
        chat_ui.add_message(
            "System".to_string(),
            format!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"),
            MessageType::SystemMessage,
        )?;

        for (peer_id, username) in connected_peers {
            let addr = peer_addresses.get(peer_id);
            
            chat_ui.add_message(
                "System".to_string(),
                format!("🔗 Peer ID: {}", &peer_id[..8]), // Show first 8 chars of peer ID
                MessageType::ConnectionInfo,
            )?;
            
            chat_ui.add_message(
                "System".to_string(),
                format!("👤 Username: {}", username),
                MessageType::ConnectionInfo,
            )?;
            
            if let Some(socket_addr) = addr {
                chat_ui.add_message(
                    "System".to_string(),
                    format!("🌐 Host: {}", socket_addr.ip()),
                    MessageType::ConnectionInfo,
                )?;
                
                chat_ui.add_message(
                    "System".to_string(),
                    format!("🔌 Port: {}", socket_addr.port()),
                    MessageType::ConnectionInfo,
                )?;
                
                chat_ui.add_message(
                    "System".to_string(),
                    format!("📍 Full Address: {}", socket_addr),
                    MessageType::ConnectionInfo,
                )?;
            } else {
                chat_ui.add_message(
                    "System".to_string(),
                    "❓ Address: Unknown".to_string(),
                    MessageType::SystemMessage,
                )?;
            }
            
            chat_ui.add_message(
                "System".to_string(),
                "─────────────────────────────────────────────────────────────────────────────".to_string(),
                MessageType::SystemMessage,
            )?;
        }
        
        chat_ui.add_message(
            "System".to_string(),
            format!("📈 Total Connected Peers: {}", connected_peers.len()),
            MessageType::SystemMessage,
        )?;
        
        Ok(())
    }
}
