//! Shared constants and utility functions

use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::MoveTo,
};
use std::io::stdout;

/// Force cleanup terminal state and exit
pub fn force_cleanup_terminal(_message: &str) {
    // Clear terminal completely first (like /quit behavior)
    let _ = execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0)
    );
    
    // Force exit the program cleanly (no messages to keep terminal clean)
    std::process::exit(1);
}
