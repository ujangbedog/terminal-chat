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

    /// Get visible length of prompt (accounting for emoji width)
    fn get_visible_prompt_length(&self, prompt: &str) -> usize {
        let mut visible_len = 0;
        for ch in prompt.chars() {
            match ch {
                // Emoji characters typically take 2 display columns
                '💬' => visible_len += 2,
                // Regular ASCII characters
                _ if ch.is_ascii() => visible_len += 1,
                // Other Unicode characters (assume 1 column for simplicity)
                _ => visible_len += 1,
            }
        }
        visible_len
    }

    /// Position cursor for input
    pub fn position_cursor_for_input(&self, chat_area_height: u16, _terminal_width: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input_line = 4 + chat_area_height + 1;
        let prompt = format!("💬 {}@chat > ", self.username);
        // Calculate visible length properly (excluding emoji and ANSI codes)
        let prompt_visible_len = self.get_visible_prompt_length(&prompt);
        execute!(io::stdout(), MoveTo((2 + prompt_visible_len) as u16, input_line))?;
        io::stdout().flush()?;
        Ok(())
    }
    
    /// Clear input area after sending message
    pub fn clear_input_area(&self, chat_area_height: u16, terminal_width: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let input_line = 4 + chat_area_height + 1;
        let prompt = format!("💬 {}@chat > ", self.username);
        let prompt_visible_len = self.get_visible_prompt_length(&prompt);
        
        // Clear the input area (everything after the prompt)
        let cursor_pos = 2 + prompt_visible_len;
        execute!(io::stdout(), MoveTo(cursor_pos as u16, input_line))?;
        
        // Clear to end of line
        let clear_width = (terminal_width as usize).saturating_sub(cursor_pos + 2);
        execute!(io::stdout(), Print(" ".repeat(clear_width)))?;
        
        // Position cursor back to start of input
        execute!(io::stdout(), MoveTo(cursor_pos as u16, input_line))?;
        io::stdout().flush()?;
        Ok(())
    }

}
