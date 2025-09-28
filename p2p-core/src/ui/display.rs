//! Display management for chat UI

use std::io::{self, Write};
use std::collections::VecDeque;
use colored::*;
use crossterm::{
    terminal::{Clear, ClearType},
    cursor::{MoveTo, MoveToColumn},
    execute, queue,
    style::Print,
};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::{sleep, Duration};

use super::messages::{ChatMessage, MessageType};

/// Display manager handles all terminal drawing operations
pub struct DisplayManager {
    terminal_width: u16,
    terminal_height: u16,
}

impl DisplayManager {
    /// Create new display manager
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            terminal_width: width,
            terminal_height: height,
        }
    }

    /// Update terminal size
    pub fn update_size(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;
    }

    /// Get visible length of string (excluding ANSI escape codes)
    fn get_visible_length(&self, text: &str) -> usize {
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

    /// Draw beautiful header with connection info
    pub fn draw_header(&self, username: &str, connected_peers: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stdout = io::stdout();
        
        // Top border
        let border = "═".repeat(self.terminal_width as usize);
        queue!(stdout, MoveTo(0, 0), Print(format!("╔{}╗", border).bright_cyan()))?;
        
        // Title line
        let title = "💬 P2P Terminal Chat";
        let visible_title_len = self.get_visible_length(title);
        let padding = (self.terminal_width as usize).saturating_sub(visible_title_len + 4) / 2;
        let title_line = format!("║{}{title}{}║", 
            " ".repeat(padding), 
            " ".repeat(self.terminal_width as usize - padding - visible_title_len - 2)
        );
        queue!(stdout, MoveTo(0, 1), Print(title_line.bright_cyan().bold()))?;
        
        // User info line
        let connected_info = if connected_peers.is_empty() {
            "🔍 Searching for peers...".yellow()
        } else {
            format!("🔗 Connected: {}", connected_peers.join(", ")).green()
        };
        
        let user_info = format!("👤 {} | {}", username, connected_info);
        let visible_info_len = self.get_visible_length(&user_info);
        let info_padding = (self.terminal_width as usize).saturating_sub(visible_info_len + 4) / 2;
        let info_line = format!("║{}{user_info}{}║", 
            " ".repeat(info_padding),
            " ".repeat(self.terminal_width as usize - info_padding - visible_info_len - 2)
        );
        queue!(stdout, MoveTo(0, 2), Print(info_line))?;
        
        // Bottom border of header
        queue!(stdout, MoveTo(0, 3), Print(format!("╠{}╣", border).bright_cyan()))?;
        
        stdout.flush()?;
        Ok(())
    }

    /// Draw chat message area
    pub fn draw_chat_area(&self, chat_area_height: u16, messages: &VecDeque<ChatMessage>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stdout = io::stdout();
        
        // Clear chat area first
        for i in 4..(4 + chat_area_height) {
            queue!(stdout, MoveTo(0, i), Print("║".bright_cyan()))?;
            // Clear the entire line content
            queue!(stdout, MoveTo(2, i))?;
            let clear_width = (self.terminal_width as usize).saturating_sub(4);
            queue!(stdout, Print(" ".repeat(clear_width)))?;
            queue!(stdout, MoveToColumn(self.terminal_width - 1), Print("║".bright_cyan()))?;
        }
        
        // Display messages
        let start_line = 4;
        let available_lines = chat_area_height as usize;
        let messages_to_show = messages.iter()
            .rev()
            .take(available_lines)
            .collect::<Vec<_>>();
        
        for (i, message) in messages_to_show.iter().rev().enumerate() {
            if i >= available_lines {
                break;
            }
            
            let line = start_line + i as u16;
            self.draw_message(line, message)?;
        }
        
        stdout.flush()?;
        Ok(())
    }

    /// Draw a single message
    fn draw_message(&self, line: u16, message: &ChatMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stdout = io::stdout();
        let content_width = (self.terminal_width as usize).saturating_sub(4); // Account for borders
        
        let formatted_message = match message.message_type {
            MessageType::UserMessage => {
                format!("[{}] {}: {}", 
                    message.timestamp.dimmed(),
                    message.sender.bright_blue().bold(),
                    message.content.white()
                )
            }
            MessageType::SystemMessage => {
                format!("🔔 {}", message.content.bright_yellow())
            }
            MessageType::ConnectionInfo => {
                format!("🔗 {}", message.content.bright_cyan())
            }
            MessageType::ErrorMessage => {
                format!("❌ {}", message.content.bright_red())
            }
        };
        
        // Calculate visible length and truncate if needed
        let visible_len = self.get_visible_length(&formatted_message);
        let display_message = if visible_len > content_width {
            // For now, just truncate without considering ANSI codes properly
            format!("{}...", &formatted_message[..formatted_message.len().min(content_width.saturating_sub(3))])
        } else {
            format!("{}{}", formatted_message, " ".repeat(content_width.saturating_sub(visible_len)))
        };
        
        queue!(stdout, MoveTo(2, line), Print(display_message))?;
        Ok(())
    }

    /// Draw input area
    pub fn draw_input_area(&self, username: &str, chat_area_height: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stdout = io::stdout();
        let input_line = 4 + chat_area_height;
        
        // Input area border
        let border = "═".repeat(self.terminal_width as usize);
        queue!(stdout, MoveTo(0, input_line), Print(format!("╠{}╣", border).bright_cyan()))?;
        
        // Input prompt
        let prompt = format!("💬 {}@chat", username);
        queue!(stdout, MoveTo(0, input_line + 1), Print("║".bright_cyan()))?;
        queue!(stdout, MoveTo(2, input_line + 1), Print(format!("{} > ", prompt.bright_green().bold())))?;
        queue!(stdout, MoveToColumn(self.terminal_width - 1), Print("║".bright_cyan()))?;
        
        // Bottom border
        queue!(stdout, MoveTo(0, input_line + 2), Print(format!("╚{}╝", border).bright_cyan()))?;
        
        stdout.flush()?;
        Ok(())
    }

    /// Show connection progress
    pub async fn show_connection_progress(&self, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ ")
        );

        pb.set_message(message.to_string());

        for i in 0..=100 {
            pb.set_position(i);
            sleep(Duration::from_millis(30)).await;
        }

        pb.finish_with_message("✅ Connected!");
        Ok(())
    }

    /// Show welcome screen
    pub fn show_welcome(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
        
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{}", "║                    💬 P2P Terminal Chat                      ║".bright_cyan());
        println!("{}", "║                   Welcome to secure chat!                    ║".bright_cyan());
        println!("{}", "║                  🔒 Encrypted • 🌐 Peer-to-Peer              ║".bright_cyan());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_cyan());
        println!();
        
        Ok(())
    }
}
