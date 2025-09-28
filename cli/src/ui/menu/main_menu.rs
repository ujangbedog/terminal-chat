//! Main menu implementation with interactive keyboard navigation and dynamic layout

use super::MenuItem;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor,
    style::{Color, ResetColor, SetForegroundColor},
};
use std::io::{self, stdout, Write};

/// Action that can be performed in the menu
#[derive(Debug)]
enum MenuAction {
    MoveUp,
    MoveDown,
    Select,
    Exit,
    None,
}

/// Main menu with interactive keyboard navigation and dynamic layout
pub struct MainMenu {
    items: Vec<MenuItem>,
    selected_index: usize,
}

impl MainMenu {
    /// Creates a new main menu with default items
    pub fn new() -> Self {
        let items = vec![
            MenuItem::available(
                1,
                "Create P2P Chat",
                "Start a new peer-to-peer chat session"
            ),
            MenuItem::coming_soon(
                2,
                "Join Chat Room",
                "Join an existing chat room (Coming Soon)"
            ),
            MenuItem::coming_soon(
                3,
                "Settings",
                "Configure application settings (Coming Soon)"
            ),
            MenuItem::available(
                4,
                "Exit",
                "Exit the application"
            ),
        ];

        Self {
            items,
            selected_index: 0,
        }
    }

    /// Shows the menu and returns the selected item ID, or None if cancelled
    pub async fn show(&mut self) -> io::Result<Option<usize>> {
        self.setup_terminal()?;
        let result = self.run_menu_loop().await;
        self.cleanup_terminal()?;
        result
    }

    /// Sets up the terminal for menu display
    fn setup_terminal(&self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        Ok(())
    }

    /// Cleans up terminal state
    fn cleanup_terminal(&self) -> io::Result<()> {
        execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        disable_raw_mode()?;
        Ok(())
    }

    /// Main menu loop handling user input
    async fn run_menu_loop(&mut self) -> io::Result<Option<usize>> {
        loop {
            self.draw_menu()?;

            if let Event::Key(key_event) = event::read()? {
                match self.handle_key_event(key_event) {
                    MenuAction::MoveUp => self.move_up(),
                    MenuAction::MoveDown => self.move_down(),
                    MenuAction::Select => {
                        let selected_item = &self.items[self.selected_index];
                        if selected_item.available {
                            return Ok(Some(selected_item.id));
                        }
                    }
                    MenuAction::Exit => return Ok(None),
                    MenuAction::None => {}
                }
            }
        }
    }

    /// Draws the complete menu interface with dynamic layout
    fn draw_menu(&self) -> io::Result<()> {
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        // Get terminal size for dynamic layout
        let (term_width, term_height) = crossterm::terminal::size()?;
        let content_width = std::cmp::min(70, term_width.saturating_sub(4) as usize);
        let padding = ((term_width as usize).saturating_sub(content_width)) / 2;

        // Calculate vertical centering
        let total_content_lines = 8 + (self.items.len() * 2) + 2; // header + items + footer
        let vertical_padding = ((term_height as usize).saturating_sub(total_content_lines)) / 2;

        // Add vertical padding
        for _ in 0..vertical_padding {
            println!();
        }

        self.draw_header(content_width, padding)?;
        self.draw_instructions(content_width, padding)?;
        self.draw_menu_items(content_width, padding)?;
        self.draw_footer(content_width, padding)?;

        stdout().flush()?;
        Ok(())
    }

    /// Draws the menu header with dynamic width
    fn draw_header(&self, width: usize, padding: usize) -> io::Result<()> {
        let title = "🚀 Terminal Chat Client";
        let subtitle = "Main Menu";
        
        execute!(stdout(), SetForegroundColor(Color::Cyan))?;
        
        // Top border
        print!("{}", " ".repeat(padding));
        println!("╔{}╗", "═".repeat(width.saturating_sub(2)));
        
        // Title line
        let title_len = title.chars().count();
        let title_padding = (width.saturating_sub(2).saturating_sub(title_len)) / 2;
        print!("{}", " ".repeat(padding));
        println!("║{}{}{}║", 
            " ".repeat(title_padding),
            title,
            " ".repeat(width.saturating_sub(2).saturating_sub(title_padding).saturating_sub(title_len))
        );
        
        // Subtitle line
        let subtitle_padding = (width.saturating_sub(2).saturating_sub(subtitle.len())) / 2;
        print!("{}", " ".repeat(padding));
        println!("║{}{}{}║", 
            " ".repeat(subtitle_padding),
            subtitle,
            " ".repeat(width.saturating_sub(2).saturating_sub(subtitle_padding).saturating_sub(subtitle.len()))
        );
        
        // Bottom border
        print!("{}", " ".repeat(padding));
        println!("╚{}╝", "═".repeat(width.saturating_sub(2)));
        
        execute!(stdout(), ResetColor)?;
        println!();
        Ok(())
    }

    /// Draws navigation instructions
    fn draw_instructions(&self, width: usize, padding: usize) -> io::Result<()> {
        let instruction = "Use ↑/↓ arrows to navigate, Enter to select, Esc/Ctrl+C to exit";
        let inst_padding = (width.saturating_sub(instruction.len())) / 2;
        
        execute!(stdout(), SetForegroundColor(Color::Yellow))?;
        print!("{}", " ".repeat(padding + inst_padding));
        println!("{}", instruction);
        execute!(stdout(), ResetColor)?;
        println!();
        Ok(())
    }

    /// Draws all menu items with proper spacing
    fn draw_menu_items(&self, width: usize, padding: usize) -> io::Result<()> {
        for (index, item) in self.items.iter().enumerate() {
            self.draw_menu_item(index, item, width, padding)?;
        }
        Ok(())
    }

    /// Draws a single menu item with proper alignment
    fn draw_menu_item(&self, index: usize, item: &MenuItem, width: usize, padding: usize) -> io::Result<()> {
        let is_selected = index == self.selected_index;
        let prefix = if is_selected { "► " } else { "  " };
        
        // Set color based on item state
        if is_selected {
            execute!(stdout(), SetForegroundColor(Color::Green))?;
        } else if !item.available {
            execute!(stdout(), SetForegroundColor(Color::DarkGrey))?;
        } else {
            execute!(stdout(), SetForegroundColor(Color::White))?;
        }

        let status_indicator = if !item.available { " [Coming Soon]" } else { "" };
        let menu_text = format!("{}{}. {}{}", prefix, item.id, item.title, status_indicator);
        
        // Center the menu item
        let item_padding = (width.saturating_sub(menu_text.len())) / 2;
        print!("{}", " ".repeat(padding + item_padding));
        println!("{}", menu_text);
        
        // Show description for selected item
        if is_selected {
            execute!(stdout(), SetForegroundColor(Color::Cyan))?;
            let desc_padding = (width.saturating_sub(item.description.len() + 4)) / 2;
            print!("{}", " ".repeat(padding + desc_padding));
            println!("    {}", item.description);
        }
        
        execute!(stdout(), ResetColor)?;
        println!();
        Ok(())
    }

    /// Draws the menu footer
    fn draw_footer(&self, width: usize, padding: usize) -> io::Result<()> {
        println!();
        execute!(stdout(), SetForegroundColor(Color::DarkGrey))?;
        
        // Dynamic separator line
        print!("{}", " ".repeat(padding));
        println!("{}", "━".repeat(width));
        
        let footer_text = "Terminal Chat v0.1.0 - Built with Rust 🦀";
        let footer_padding = (width.saturating_sub(footer_text.chars().count())) / 2;
        print!("{}", " ".repeat(padding + footer_padding));
        println!("{}", footer_text);
        
        execute!(stdout(), ResetColor)?;
        Ok(())
    }

    /// Handles keyboard input and returns appropriate action
    fn handle_key_event(&self, key_event: KeyEvent) -> MenuAction {
        match key_event {
            KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                ..
            } => MenuAction::MoveUp,
            KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                ..
            } => MenuAction::MoveDown,
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => MenuAction::Select,
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => MenuAction::Exit,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => MenuAction::Exit,
            _ => MenuAction::None,
        }
    }

    /// Moves selection up (with wrapping)
    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.items.len() - 1;
        }
    }

    /// Moves selection down (with wrapping)
    fn move_down(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}
