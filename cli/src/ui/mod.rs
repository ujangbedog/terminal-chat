//! UI module for DPQ chat client
//! 
//! Contains all user interface components including menus and display functions

pub mod menu;
pub mod interactive;

pub use menu::{MainMenu, MenuItem};
pub use interactive::InteractiveMenu;

use colored::*;

/// Display application header for P2P chat mode
pub fn display_header() {
    println!("{}", "=== DPQ Chat Client - P2P Mode ===".bright_cyan().bold());
}
