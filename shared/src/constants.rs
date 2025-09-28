//! Shared constants and utility functions

use crossterm::{
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
    cursor::Show,
};
use std::io::stdout;

/// Force cleanup terminal state and exit
pub fn force_cleanup_terminal(message: &str) {
    // Try to cleanup terminal state
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen, Show);
    
    eprintln!("\n{}", message);
    std::process::exit(1);
}
