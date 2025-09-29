//! Authentication system module
//! 
//! Provides identity verification and management for DPQ Chat

pub mod types;
pub mod system;
pub mod identity_manager;
pub mod verification;

// Re-export main types and functions
pub use types::AuthenticatedUser;
pub use system::AuthSystem;
