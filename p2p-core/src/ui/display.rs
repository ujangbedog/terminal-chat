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

    /// Get visible length of string (excluding ANSI escape codes, accounting for emoji width)
    fn get_visible_length(&self, text: &str) -> usize {
        let mut visible_len = 0;
        let mut in_escape = false;
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Start of ANSI escape sequence
                in_escape = true;
            } else if in_escape {
                // Skip all characters until we find the end of escape sequence
                if ch.is_ascii_alphabetic() || ch == 'm' || ch == 'K' || ch == 'J' {
                    in_escape = false;
                }
                // Continue skipping characters in escape sequence
            } else {
                // Count visible character width
                match ch {
                    // Common emoji characters that take 2 display columns
                    '💬' | '🔔' | '🔗' | '❌' | '👤' | '🔍' | '🚀' | '💡' | '👥' | '📜' | '👋' | '🔌' => {
                        visible_len += 2;
                    }
                    // Regular ASCII characters
                    _ if ch.is_ascii() => {
                        visible_len += 1;
                    }
                    // Other Unicode characters (assume 1 column for most)
                    _ => {
                        visible_len += 1;
                    }
                }
            }
        }
        visible_len
    }

    /// Safely truncate string while preserving ANSI escape codes
    fn safe_truncate(&self, text: &str, max_width: usize) -> String {
        let visible_len = self.get_visible_length(text);
        
        if visible_len <= max_width {
            return text.to_string();
        }
        
        let mut result = String::new();
        let mut visible_count = 0;
        let mut in_escape = false;
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Start of ANSI escape sequence - always include
                result.push(ch);
                in_escape = true;
            } else if in_escape {
                // Include all escape sequence characters
                result.push(ch);
                if ch.is_ascii_alphabetic() || ch == 'm' || ch == 'K' || ch == 'J' {
                    in_escape = false;
                }
            } else {
                // Regular character - count towards visible limit
                if visible_count >= max_width.saturating_sub(3) {
                    result.push_str("...");
                    break;
                }
                result.push(ch);
                visible_count += 1;
            }
        }
        
        result
    }

    /// Draw beautiful header with connection info
    pub fn draw_header(&self, username: &str, listen_port: Option<u16>, connected_peers: &[String]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stdout = io::stdout();
        
        // Top border - fix width calculation
        let border_width = (self.terminal_width as usize).saturating_sub(2);
        let border = "═".repeat(border_width);
        queue!(stdout, MoveTo(0, 0), Print(format!("╔{}╗", border).bright_cyan()))?;
        
        // Title line
        let title = "💬 P2P Terminal Chat";
        let visible_title_len = self.get_visible_length(title);
        let content_width = (self.terminal_width as usize).saturating_sub(4); // Account for borders
        let padding = content_width.saturating_sub(visible_title_len) / 2;
        let title_line = format!("║ {}{title}{} ║", 
            " ".repeat(padding),
            " ".repeat(content_width - padding - visible_title_len)
        );
        queue!(stdout, MoveTo(0, 1), Print(title_line))?;
        
        // User info line: username | listening | peer status
        let listen_info = if let Some(port) = listen_port {
            format!("🔊 Listening: {}", port)
        } else {
            "🔊 Not listening".to_string()
        };
        
        let peer_status = if connected_peers.is_empty() {
            "⏳ Waiting for peers...".to_string()
        } else {
            format!("🔗 Connected: {}", connected_peers.join(", "))
        };
        
        let user_info = format!("👤 {} | {} | {}", username, listen_info, peer_status);
        let visible_info_len = self.get_visible_length(&user_info);
        let info_padding = content_width.saturating_sub(visible_info_len) / 2;
        let info_line = format!("║ {}{user_info}{} ║", 
            " ".repeat(info_padding),
            " ".repeat(content_width - info_padding - visible_info_len)
        );
        queue!(stdout, MoveTo(0, 2), Print(info_line))?;
        
        queue!(stdout, MoveTo(0, 3), Print(format!("╠{}╣", border).bright_cyan()))?;
        
        stdout.flush()?;
        Ok(())
    }
    
    /// Get user color based on username hash
    fn get_user_color(&self, username: &str) -> colored::Color {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        username.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Use hash to select from a set of nice colors
        let colors = [
            colored::Color::BrightBlue,
            colored::Color::BrightGreen, 
            colored::Color::BrightMagenta,
            colored::Color::BrightCyan,
            colored::Color::Yellow,
            colored::Color::BrightRed,
        ];
        
        colors[(hash as usize) % colors.len()]
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
                let user_color = self.get_user_color(&message.sender);
                format!("[{}] {}: {}", 
                    message.timestamp.dimmed(),
                    message.sender.color(user_color).bold(),
                    message.content.white()
                )
            }
            MessageType::SystemMessage => {
                format!("🔔 {}", message.content.bright_yellow())
            }
            MessageType::ConnectionInfo => {
                format!("🔗 {}", message.content.bright_green())
            }
            MessageType::ErrorMessage => {
                format!("❌ {}", message.content.bright_red())
            }
        };
        
        // Safely truncate message if needed and pad to full width
        let truncated_message = self.safe_truncate(&formatted_message, content_width);
        let visible_len = self.get_visible_length(&truncated_message);
        let display_message = format!("{}{}", 
            truncated_message, 
            " ".repeat(content_width.saturating_sub(visible_len))
        );
        
        queue!(stdout, MoveTo(2, line), Print(display_message))?;
        Ok(())
    }

    /// Draw input area
    pub fn draw_input_area(&self, username: &str, chat_area_height: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut stdout = io::stdout();
        let input_line = 4 + chat_area_height;
        
        // Input area border - fix width calculation
        let border_width = (self.terminal_width as usize).saturating_sub(2);
        let border = "═".repeat(border_width);
        queue!(stdout, MoveTo(0, input_line), Print(format!("╠{}╣", border).bright_cyan()))?;
        
        // Input prompt line
        let prompt = format!("💬 {}@chat > ", username);
        let prompt_visible_len = self.get_visible_length(&prompt);
        let content_width = (self.terminal_width as usize).saturating_sub(4); // Account for borders
        let padding = content_width.saturating_sub(prompt_visible_len);
        
        queue!(stdout, MoveTo(0, input_line + 1), Print("║".bright_cyan()))?;
        queue!(stdout, MoveTo(2, input_line + 1), Print(format!("{}{}", prompt.bright_green().bold(), " ".repeat(padding))))?;
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
