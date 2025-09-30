//! Utilities for working with identities in cryptographic operations

use crate::crypto::dilithium_ops::DilithiumKeypair;
use identity_gen::{Identity, Encryption};

/// Load Dilithium keypair from decrypted identity data
pub fn load_dilithium_keypair_from_identity(
    public_key_bytes: &[u8],
    decrypted_secret_key_bytes: &[u8],
) -> Result<DilithiumKeypair, Box<dyn std::error::Error>> {
    DilithiumKeypair::from_bytes(public_key_bytes, decrypted_secret_key_bytes)
}

/// Create HandshakeManager with identity data
pub fn create_handshake_manager_with_identity(
    username: String,
    fingerprint: String,
    public_key_bytes: Vec<u8>,
    decrypted_secret_key_bytes: Vec<u8>,
) -> Result<crate::crypto::handshake::HandshakeManager, Box<dyn std::error::Error>> {
    // Create Dilithium keypair
    let dilithium_keypair = load_dilithium_keypair_from_identity(
        &public_key_bytes,
        &decrypted_secret_key_bytes,
    )?;
    
    // Create HandshakeManager with Dilithium support
    let manager = crate::crypto::handshake::HandshakeManager::new_with_dilithium(
        username,
        fingerprint,
        public_key_bytes,
        dilithium_keypair,
    );
    
    Ok(manager)
}

/// Create HandshakeManager from Identity and password
pub fn create_handshake_manager_from_identity(
    identity: &Identity,
    password: &str,
) -> Result<crate::crypto::handshake::HandshakeManager, Box<dyn std::error::Error>> {
    // Get public key bytes
    let public_key_bytes = identity.get_public_key_bytes()?;
    
    // Decrypt secret key
    let encrypted_secret_key = identity.get_secret_key_bytes()?;
    let decrypted_secret_key = Encryption::decrypt_secret_key(&encrypted_secret_key, password)?;
    
    // Create HandshakeManager with Dilithium support
    create_handshake_manager_with_identity(
        identity.username.clone(),
        identity.fingerprint.clone(),
        public_key_bytes,
        decrypted_secret_key,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pqcrypto_dilithium::dilithium2;
    
    #[test]
    fn test_load_dilithium_keypair() {
        // Generate test keypair
        let (public_key, secret_key) = dilithium2::keypair();
        
        let keypair = load_dilithium_keypair_from_identity(
            public_key.as_bytes(),
            secret_key.as_bytes(),
        ).unwrap();
        
        // Test that we can sign with the loaded keypair
        let message = b"test message";
        let signature = keypair.sign(message);
        assert!(!signature.is_empty());
    }
    
    #[test]
    fn test_create_handshake_manager_with_identity() {
        // Generate test keypair
        let (public_key, secret_key) = dilithium2::keypair();
        
        let manager = create_handshake_manager_with_identity(
            "test_user".to_string(),
            "test:fingerprint".to_string(),
            public_key.as_bytes().to_vec(),
            secret_key.as_bytes().to_vec(),
        ).unwrap();
        
        assert_eq!(manager.our_info().username, "test_user");
        assert_eq!(manager.our_info().fingerprint, "test:fingerprint");
    }
}
