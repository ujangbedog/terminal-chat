/// Terminal styling constants and ANSI codes for P2P chat client

use crossterm::{
    execute,
    terminal::{LeaveAlternateScreen, Clear, ClearType},
    cursor::Show,
};
use std::io::{self, Write};
use std::process;

// ANSI color codes for terminal
pub const COLOR_RESET: &str = "\x1b[0m";
pub const COLOR_GREEN: &str = "\x1b[32m";
pub const COLOR_BLUE: &str = "\x1b[34m";
pub const COLOR_YELLOW: &str = "\x1b[33m";
pub const COLOR_CYAN: &str = "\x1b[36m";
pub const COLOR_BOLD: &str = "\x1b[1m";
pub const COLOR_DIM: &str = "\x1b[2m";
pub const COLOR_WHITE: &str = "\x1b[37m";
pub const COLOR_RED: &str = "\x1b[31m";

// Box drawing characters (unused but kept for future UI enhancements)
pub const BOX_HORIZONTAL: &str = "─";
pub const BOX_VERTICAL: &str = "│";
pub const BOX_TOP_LEFT: &str = "┌";
pub const BOX_TOP_RIGHT: &str = "┐";
pub const BOX_BOTTOM_LEFT: &str = "└";
pub const BOX_BOTTOM_RIGHT: &str = "┘";
pub const BOX_CROSS: &str = "┼";
pub const BOX_T_DOWN: &str = "┬";
pub const BOX_T_UP: &str = "┴";
pub const BOX_T_RIGHT: &str = "├";
pub const BOX_T_LEFT: &str = "┤";

/// Force cleanup terminal and exit the program
/// This function clears the terminal and exits with code 1
pub fn force_cleanup_terminal(message: &str) -> ! {
    // Try to clean up the terminal using crossterm
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        Clear(ClearType::All),
        Show
    );
    
    // Print the exit message with color
    println!("{}*** {} ***{}", COLOR_RED, message, COLOR_RESET);
    println!("{}Program terminated due to network disconnect{}", COLOR_YELLOW, COLOR_RESET);
    
    // Force exit the program
    process::exit(1);
}
