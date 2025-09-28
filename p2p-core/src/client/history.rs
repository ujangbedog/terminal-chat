/// Message history management for P2P chat client
use std::cell::RefCell;
use std::io::{self, Write};

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

    /// Refresh the entire chat display with current message history
    pub fn refresh_display(&self) {
        // Clear chat area and redraw messages
        let chat_start_line = 6; // After header "💬 CHAT MESSAGES"
        let chat_lines = 15;
        
        // Get the last messages that fit in the chat area
        let history = self.messages.borrow();
        let messages_to_show = if history.len() > chat_lines {
            &history[history.len() - chat_lines..]
        } else {
            &history[..]
        };
        
        // Clear and redraw chat area (simple design)
        for i in 0..chat_lines {
            print!("\x1b[{};1H", chat_start_line + i); // Move to line, column 1
            print!("\x1b[K"); // Clear line
            
            if i < messages_to_show.len() {
                print!("{}", messages_to_show[i]);
            }
        }
        
        io::stdout().flush().unwrap();
    }

    /// Get current message count
    #[allow(dead_code)]
    pub fn message_count(&self) -> usize {
        self.messages.borrow().len()
    }
}
