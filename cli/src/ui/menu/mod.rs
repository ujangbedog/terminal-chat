//! Main menu module for the terminal chat application
//! 
//! This module provides an interactive menu system with keyboard navigation
//! following Rust best practices for modular architecture.

pub mod main_menu;
pub mod menu_item;

pub use main_menu::MainMenu;
pub use menu_item::MenuItem;
