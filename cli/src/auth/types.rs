//! Authentication types and data structures

use identity_gen::Identity;

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub username: String,
    pub identity: Identity,
}

impl AuthenticatedUser {
    /// Create a HandshakeManager with Dilithium support for this user
    /// Requires the user's password to decrypt the private key
    pub fn create_handshake_manager(
        &self,
        password: &str,
    ) -> Result<shared::crypto::HandshakeManager, Box<dyn std::error::Error>> {
        shared::crypto::create_handshake_manager_from_identity(&self.identity, password)
    }
    
    /// Get user's public key bytes
    pub fn get_public_key_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(self.identity.get_public_key_bytes()?)
    }
    
    /// Get user's fingerprint
    pub fn get_fingerprint(&self) -> &str {
        &self.identity.fingerprint
    }
}
