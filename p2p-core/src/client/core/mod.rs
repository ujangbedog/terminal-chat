//! Core client functionality
//! 
//! Contains the main P2P chat client implementation and core logic.

pub mod client;
pub mod event_handler;
pub mod command_handler;

pub use client::P2PChatClient;
pub use event_handler::EventHandler;
pub use command_handler::CommandHandler;
