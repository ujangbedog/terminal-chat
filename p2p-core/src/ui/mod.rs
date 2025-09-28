//! UI module for P2P chat client
//! 
//! Contains all user interface components including display, input handling,
//! and message management for the terminal-based chat interface.

pub mod display;
pub mod input;
pub mod messages;

pub use display::DisplayManager;
pub use input::InputHandler;
pub use messages::{MessageType, MessageManager};

use crossterm::{
    terminal::{self, Clear, ClearType},
    cursor::MoveTo,
    execute,
};
use std::io;

/// Main chat UI coordinator
pub struct ChatUI {
    username: String,
    terminal_width: u16,
    terminal_height: u16,
    chat_area_height: u16,
    connected_peers: Vec<String>,
    display_manager: DisplayManager,
    input_handler: InputHandler,
    message_manager: MessageManager,
}

impl ChatUI {
    /// Create new chat UI
    pub fn new(username: String, max_messages: usize) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (width, height) = terminal::size()?;
        let chat_area_height = height.saturating_sub(8); // Reserve space for header and input
        
        Ok(Self {
            username: username.clone(),
            terminal_width: width,
            terminal_height: height,
            chat_area_height,
            connected_peers: Vec::new(),
            display_manager: DisplayManager::new(width, height),
            input_handler: InputHandler::new(username.clone()),
            message_manager: MessageManager::new(max_messages),
        })
    }

    /// Initialize the chat interface
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Clear screen
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        
        self.display_manager.draw_header(&self.username, &self.connected_peers)?;
        self.display_manager.draw_chat_area(self.chat_area_height, &self.message_manager.get_messages())?;
        self.display_manager.draw_input_area(&self.username, self.chat_area_height)?;
        
        Ok(())
    }

    /// Add a new message to the chat
    pub fn add_message(&mut self, sender: String, content: String, message_type: MessageType) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.message_manager.add_message(sender, content, message_type);
        
        // Refresh display immediately
        self.refresh_display()?;
        
        // Reposition cursor to input area
        self.input_handler.position_cursor_for_input(self.chat_area_height, self.terminal_width)?;
        
        Ok(())
    }

    /// Update connected peers list
    pub fn update_connected_peers(&mut self, peers: Vec<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.connected_peers = peers;
        self.display_manager.draw_header(&self.username, &self.connected_peers)?;
        Ok(())
    }

    /// Refresh the entire display
    pub fn refresh_display(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Update terminal size in case it changed
        if let Ok((width, height)) = terminal::size() {
            self.terminal_width = width;
            self.terminal_height = height;
            self.chat_area_height = height.saturating_sub(8);
            self.display_manager.update_size(width, height);
        }
        
        self.display_manager.draw_header(&self.username, &self.connected_peers)?;
        self.display_manager.draw_chat_area(self.chat_area_height, &self.message_manager.get_messages())?;
        self.display_manager.draw_input_area(&self.username, self.chat_area_height)?;
        Ok(())
    }

    /// Position cursor for input
    pub fn position_cursor_for_input(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.input_handler.position_cursor_for_input(self.chat_area_height, self.terminal_width)
    }
    
    /// Clear input area after sending message
    pub fn clear_input_area(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.input_handler.clear_input_area(self.chat_area_height, self.terminal_width)
    }

    /// Show connection progress
    pub async fn show_connection_progress(&self, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.display_manager.show_connection_progress(message).await
    }

    /// Show welcome screen
    pub fn show_welcome(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.display_manager.show_welcome()
    }

}
