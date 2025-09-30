//! Dilithium operations for handshake signing and verification

use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};

/// Dilithium keypair for signing operations
#[derive(Clone)]
pub struct DilithiumKeypair {
    pub public_key: dilithium2::PublicKey,
    pub secret_key: dilithium2::SecretKey,
}

impl std::fmt::Debug for DilithiumKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DilithiumKeypair")
            .field("public_key", &"<dilithium2::PublicKey>")
            .field("secret_key", &"<dilithium2::SecretKey>")
            .finish()
    }
}

impl DilithiumKeypair {
    /// Create keypair from raw bytes (loaded from identity)
    pub fn from_bytes(
        public_key_bytes: &[u8],
        secret_key_bytes: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let public_key = dilithium2::PublicKey::from_bytes(public_key_bytes)
            .map_err(|_| "Invalid Dilithium public key")?;
        let secret_key = dilithium2::SecretKey::from_bytes(secret_key_bytes)
            .map_err(|_| "Invalid Dilithium secret key")?;
        
        Ok(Self {
            public_key,
            secret_key,
        })
    }
    
    /// Sign data with private key
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        dilithium2::sign(data, &self.secret_key).as_bytes().to_vec()
    }
    
    /// Get public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        self.public_key.as_bytes()
    }
    
    /// Get secret key bytes
    pub fn secret_key_bytes(&self) -> &[u8] {
        self.secret_key.as_bytes()
    }
}

/// Dilithium signature verification utilities
pub struct DilithiumVerifier;

impl DilithiumVerifier {
    /// Verify signature with public key
    pub fn verify(
        message: &[u8],
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let public_key = dilithium2::PublicKey::from_bytes(public_key_bytes)
            .map_err(|_| "Invalid Dilithium public key for verification")?;
        
        let signed_message = dilithium2::SignedMessage::from_bytes(signature)
            .map_err(|_| "Invalid Dilithium signature format")?;
        
        match dilithium2::open(&signed_message, &public_key) {
            Ok(verified_message) => {
                // Verify that the message content matches
                Ok(verified_message == message)
            }
            Err(_) => Ok(false),
        }
    }
    
    /// Verify signature and extract message (for cases where message is embedded)
    pub fn verify_and_extract(
        signature: &[u8],
        public_key_bytes: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let public_key = dilithium2::PublicKey::from_bytes(public_key_bytes)
            .map_err(|_| "Invalid Dilithium public key for verification")?;
        
        let signed_message = dilithium2::SignedMessage::from_bytes(signature)
            .map_err(|_| "Invalid Dilithium signature format")?;
        
        dilithium2::open(&signed_message, &public_key)
            .map_err(|_| "Signature verification failed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dilithium_sign_verify() {
        // Generate a keypair for testing
        let (public_key, secret_key) = dilithium2::keypair();
        
        let keypair = DilithiumKeypair {
            public_key: public_key.clone(),
            secret_key,
        };
        
        let message = b"Hello, Dilithium!";
        let signature = keypair.sign(message);
        
        let is_valid = DilithiumVerifier::verify(
            message,
            &signature,
            public_key.as_bytes(),
        ).unwrap();
        
        assert!(is_valid);
    }
    
    #[test]
    fn test_keypair_from_bytes() {
        // Generate a keypair
        let (public_key, secret_key) = dilithium2::keypair();
        
        // Convert to bytes and back
        let keypair = DilithiumKeypair::from_bytes(
            public_key.as_bytes(),
            secret_key.as_bytes(),
        ).unwrap();
        
        // Test signing
        let message = b"Test message";
        let signature = keypair.sign(message);
        
        let is_valid = DilithiumVerifier::verify(
            message,
            &signature,
            keypair.public_key_bytes(),
        ).unwrap();
        
        assert!(is_valid);
    }
}
