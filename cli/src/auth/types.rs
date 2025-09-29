//! Authentication types and data structures

use identity_gen::Identity;

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
    pub identity: Identity,
}
