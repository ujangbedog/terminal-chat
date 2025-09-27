/// configuration constants for P2P chat
pub mod constants {
    pub const MAX_MESSAGE_LENGTH: usize = 1024;
    pub const MAX_USERNAME_LENGTH: usize = 32;
}

// re-export for convenience
pub use constants::*;
