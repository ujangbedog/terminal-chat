/// Message history management for P2P chat client
use std::cell::RefCell;

/// Message history manager
pub struct MessageHistory {
    messages: RefCell<Vec<String>>,
    max_history: usize,
}

impl MessageHistory {
    /// Create new message history manager
    pub fn new(max_history: usize) -> Self {
        Self {
            messages: RefCell::new(Vec::new()),
            max_history,
        }
    }

    /// Add message to history
    pub fn add_message(&self, message: String) {
        let mut history = self.messages.borrow_mut();
        history.push(message);
        
        // Keep only the last max_history messages
        let len = history.len();
        if len > self.max_history {
            history.drain(0..len - self.max_history);
        }
    }


    /// Get current message count
    #[allow(dead_code)]
    pub fn message_count(&self) -> usize {
        self.messages.borrow().len()
    }
}
