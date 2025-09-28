//! CLI module for Terminal Chat
//! 
//! Provides professional command-line interface, interactive menus,
//! and user interaction components using modern CLI libraries.

pub mod args;
pub mod commands;
pub mod ui;

pub use args::{Cli, Commands};
pub use commands::handle_command;
pub use ui::InteractiveMenu;
