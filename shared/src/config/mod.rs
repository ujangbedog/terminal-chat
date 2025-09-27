/// configuration constants
pub mod constants {
    pub const DEFAULT_SERVER_PORT: u16 = 8080;
    pub const DEFAULT_CLIENT_PORT: u16 = 8081;
    pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1";
    pub const MAX_MESSAGE_LENGTH: usize = 1024;
    pub const MAX_USERNAME_LENGTH: usize = 32;
}

// re-export for backward compatibility
pub use constants::*;
