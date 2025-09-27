/// shared library for chat application
pub mod message;
pub mod config;
pub mod utils;

// re-export main types for convenience
pub use message::Message;
pub use config::*;
pub use utils::*;
