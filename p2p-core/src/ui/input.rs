//! Input handling for chat UI

use std::io::{self, Write};
use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
};

/// Input handler manages cursor positioning and input area clearing
pub struct InputHandler {
    username: String,
}

impl InputHandler {
    /// Create new input handler
    pub fn new(username: String) -> Self {
        Self { username }
    }

    /// Position cursor for input
    pub fn position_cursor_for_input(&self, chat_area_height: u16, _terminal_width: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input_line = 4 + chat_area_height + 1;
        let prompt_len = format!("💬 {}@chat > ", self.username).len();
        execute!(io::stdout(), MoveTo((prompt_len + 1) as u16, input_line))?;
        io::stdout().flush()?;
        Ok(())
    }
    
    /// Clear input area after sending message
    pub fn clear_input_area(&self, chat_area_height: u16, terminal_width: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input_line = 4 + chat_area_height + 1;
        let prompt_len = format!("💬 {}@chat > ", self.username).len();
        
        // Clear the input area (everything after the prompt)
        execute!(io::stdout(), MoveTo((prompt_len + 1) as u16, input_line))?;
        
        // Clear to end of line
        let clear_width = (terminal_width as usize).saturating_sub(prompt_len + 4);
        execute!(io::stdout(), Print(" ".repeat(clear_width)))?;
        
        // Position cursor back to start of input
        execute!(io::stdout(), MoveTo((prompt_len + 1) as u16, input_line))?;
        io::stdout().flush()?;
        Ok(())
    }

}
