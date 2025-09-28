/// Command handling for P2P chat client
use crate::client::constants::*;
use crate::client::history::MessageHistory;
use shared::{P2PNode, P2PMessage};
use tracing::{info, error};

/// Command handler for chat commands and user input
pub struct CommandHandler;

impl CommandHandler {
    /// Handle user commands and messages
    pub async fn handle_command(
        input: &str,
        node: &mut P2PNode,
        username: &str,
        history: &MessageHistory,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let trimmed = input.trim();
        
        // Handle special commands
        if trimmed.starts_with('/') {
            return Self::handle_special_command(trimmed, node, history).await;
        }
        
        // Handle regular chat message
        if !trimmed.is_empty() {
            Self::send_chat_message(trimmed, node, username, history).await?;
        }
        
        Ok(true) // Continue running
    }
    
    /// Handle special commands (starting with /)
    async fn handle_special_command(
        command: &str,
        node: &mut P2PNode,
        history: &MessageHistory,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match command {
            "/quit" | "/exit" => {
                info!("User requested quit");
                let formatted_message = format!(
                    "{}*** 👋 Disconnecting from P2P network...{}",
                    COLOR_YELLOW, COLOR_RESET
                );
                history.add_message(formatted_message);
                history.refresh_display();
                return Ok(false); // Stop running
            }
            
            "/peers" => {
                let stats = node.get_stats().await;
                let formatted_message = format!(
                    "{}*** 👥 Connected peers: {}{}",
                    COLOR_CYAN, stats.connected_peers, COLOR_RESET
                );
                history.add_message(formatted_message);
                history.refresh_display();
            }
            
            "/stats" => {
                let stats = node.get_stats().await;
                let formatted_message = format!(
                    "{}*** 📊 Network Stats - Peers: {}, Messages Sent: {}, Messages Received: {}{}",
                    COLOR_CYAN, stats.connected_peers, stats.total_messages_sent, stats.total_messages_received, COLOR_RESET
                );
                history.add_message(formatted_message);
                history.refresh_display();
            }
            
            "/help" => {
                let help_messages = vec![
                    format!("{}*** 📖 Available Commands:{}", COLOR_BOLD, COLOR_RESET),
                    format!("{}  /peers  - Show connected peers{}", COLOR_DIM, COLOR_RESET),
                    format!("{}  /stats  - Show network statistics{}", COLOR_DIM, COLOR_RESET),
                    format!("{}  /help   - Show this help{}", COLOR_DIM, COLOR_RESET),
                    format!("{}  /quit   - Exit the application{}", COLOR_DIM, COLOR_RESET),
                    format!("{}  Type any message to send to all peers{}", COLOR_DIM, COLOR_RESET),
                ];
                
                for msg in help_messages {
                    history.add_message(msg);
                }
                history.refresh_display();
            }
            
            _ => {
                let formatted_message = format!(
                    "{}*** ❓ Unknown command: {}. Type /help for available commands{}",
                    COLOR_YELLOW, command, COLOR_RESET
                );
                history.add_message(formatted_message);
                history.refresh_display();
            }
        }
        
        Ok(true) // Continue running
    }
    
    /// Send a chat message to all peers
    async fn send_chat_message(
        content: &str,
        node: &mut P2PNode,
        username: &str,
        history: &MessageHistory,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Send message to network (P2PNode will create the P2PMessage internally)
        if let Err(e) = node.send_chat_message(content.to_string()).await {
            error!("Failed to send message: {}", e);
            let formatted_message = format!(
                "{}*** ❌ Failed to send message: {}{}",
                COLOR_YELLOW, e, COLOR_RESET
            );
            history.add_message(formatted_message);
            history.refresh_display();
            return Err(e);
        }
        
        // Add our own message to history
        let formatted_message = format!(
            "{}{}@{} > {}{}",
            COLOR_GREEN, username, node.listen_addr().await, content, COLOR_RESET
        );
        
        // Add to history
        history.add_message(formatted_message);
        
        // Refresh the entire chat display
        history.refresh_display();
        
        Ok(())
    }
}
