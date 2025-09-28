//! Message management for chat UI

use std::collections::VecDeque;

/// Chat message structure for display
#[derive(Clone)]
pub struct ChatMessage {
    pub timestamp: String,
    pub sender: String,
    pub content: String,
    pub message_type: MessageType,
}

#[derive(Clone)]
pub enum MessageType {
    UserMessage,
    SystemMessage,
    ConnectionInfo,
    ErrorMessage,
}

/// Message manager handles message storage and retrieval
pub struct MessageManager {
    messages: VecDeque<ChatMessage>,
    max_messages: usize,
}

impl MessageManager {
    /// Create new message manager
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_messages),
            max_messages,
        }
    }

    /// Add a new message
    pub fn add_message(&mut self, sender: String, content: String, message_type: MessageType) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        
        let message = ChatMessage {
            timestamp,
            sender,
            content,
            message_type,
        };
        
        self.messages.push_back(message);
        
        // Keep only max_messages
        if self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    /// Get messages for display
    pub fn get_messages(&self) -> &VecDeque<ChatMessage> {
        &self.messages
    }

    /// Clear all messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

}
