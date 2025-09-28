/// Display utilities for P2P chat client terminal interface
use crate::client::constants::*;
use crossterm::terminal;
use std::io::{self, Write};

/// Display utilities for terminal interface
pub struct DisplayManager;

impl DisplayManager {
    /// Get terminal width, fallback to 80 if detection fails
    pub fn get_terminal_width() -> usize {
        if let Ok((width, _)) = terminal::size() {
            width as usize
        } else {
            80 // fallback width
        }
    }
    
    /// Get visible length of any string (excluding ANSI color codes)
    pub fn get_string_visible_length(text: &str) -> usize {
        let mut visible_len = 0;
        let mut in_escape = false;
        
        for ch in text.chars() {
            if ch == '\x1b' {
                in_escape = true;
            } else if in_escape && ch == 'm' {
                in_escape = false;
            } else if !in_escape {
                visible_len += 1;
            }
        }
        visible_len
    }

    /// Draw the simple chat area header
    pub fn draw_chat_area() {
        // Simple chat header
        println!("{}💬 CHAT MESSAGES{}", COLOR_BOLD, COLOR_RESET);
        
        // Reserve space for messages (will be filled dynamically)
        for _ in 0..15 { // 15 lines for chat messages
            println!(); // Empty lines for messages
        }
    }

    /// Draw the input area with simple separator
    pub fn draw_input_area(prompt: &str) {
        // Simple separator line
        let width = Self::get_terminal_width();
        for _ in 0..width {
            print!("-");
        }
        println!();
        
        // Simple input line
        print!("{}", prompt);
        io::stdout().flush().unwrap();
    }

    /// Clear input area and redraw prompt
    pub fn clear_input_area(prompt: &str) {
        // Move to input line and clear it completely
        print!("\x1b[22;1H"); // Move to input line (line 22: header + 15 chat lines + separator line)
        print!("\x1b[K"); // Clear entire line
        
        // Redraw the clean input line
        print!("{}", prompt);
        
        // Position cursor right after prompt
        let prompt_visible_len = Self::get_string_visible_length(prompt);
        print!("\x1b[22;{}H", prompt_visible_len + 1); // Move to correct position (right after prompt)
        io::stdout().flush().unwrap();
    }
}
