//! CLI module for professional command-line interface
//! 
//! Uses clap for argument parsing and provides a clean CLI experience

pub mod args;
pub mod commands;

pub use args::{Cli, Commands};
pub use commands::handle_command;
