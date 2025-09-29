//! Post-quantum key exchange using CRYSTALS-Kyber

use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{PublicKey, SharedSecret, Ciphertext};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

/// Kyber key exchange data for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KyberKeyExchange {
    /// Kyber public key for encapsulation
    pub public_key: Vec<u8>,
    /// Kyber ciphertext (encapsulated shared secret)
    pub ciphertext: Option<Vec<u8>>,
    /// Timestamp of key generation
    pub timestamp: u64,
    /// Key exchange role (initiator or responder)
    pub role: KeyExchangeRole,
}

/// Role in key exchange
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyExchangeRole {
    /// Initiator sends public key
    Initiator,
    /// Responder sends ciphertext
    Responder,
}

/// Kyber key exchange manager
pub struct KyberKeyExchangeManager {
    /// Our key pair (if we're the initiator)
    our_keypair: Option<(kyber768::PublicKey, kyber768::SecretKey)>,
    /// Derived shared secret
    shared_secret: Option<Vec<u8>>,
}

impl std::fmt::Debug for KyberKeyExchangeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KyberKeyExchangeManager")
            .field("has_keypair", &self.our_keypair.is_some())
            .field("has_shared_secret", &self.shared_secret.is_some())
            .finish()
    }
}

impl KyberKeyExchangeManager {
    /// Create a new Kyber key exchange manager
    pub fn new() -> Self {
        Self {
            our_keypair: None,
            shared_secret: None,
        }
    }
    
    /// Initiate key exchange (generate keypair and public key)
    pub fn initiate_key_exchange(&mut self) -> Result<KyberKeyExchange, Box<dyn std::error::Error>> {
        tracing::info!("Initiating Kyber key exchange");
        
        // Generate Kyber keypair
        let (public_key, secret_key) = kyber768::keypair();
        
        // Store our keypair
        self.our_keypair = Some((public_key.clone(), secret_key));
        
        // Create key exchange data
        let key_exchange = KyberKeyExchange {
            public_key: public_key.as_bytes().to_vec(),
            ciphertext: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            role: KeyExchangeRole::Initiator,
        };
        
        Ok(key_exchange)
    }
    
    /// Respond to key exchange (encapsulate shared secret)
    pub fn respond_to_key_exchange(
        &mut self,
        initiator_data: &KyberKeyExchange,
    ) -> Result<(KyberKeyExchange, Vec<u8>), Box<dyn std::error::Error>> {
        tracing::info!("Responding to Kyber key exchange");
        
        if initiator_data.role != KeyExchangeRole::Initiator {
            return Err("Invalid key exchange role".into());
        }
        
        if initiator_data.ciphertext.is_some() {
            return Err("Initiator should not have ciphertext".into());
        }
        
        // Reconstruct initiator's public key
        let initiator_public_key = kyber768::PublicKey::from_bytes(&initiator_data.public_key)
            .map_err(|e| format!("Invalid Kyber public key: {:?}", e))?;
        
        // Encapsulate shared secret
        let (shared_secret, ciphertext) = kyber768::encapsulate(&initiator_public_key);
        
        // Derive final shared secret using KDF
        let derived_secret = Self::derive_shared_secret(&shared_secret, "kyber-session")?;
        
        // Store shared secret
        self.shared_secret = Some(derived_secret.clone());
        
        // Create response
        let response = KyberKeyExchange {
            public_key: Vec::new(), // Not needed in response
            ciphertext: Some(ciphertext.as_bytes().to_vec()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            role: KeyExchangeRole::Responder,
        };
        
        Ok((response, derived_secret))
    }
    
    /// Complete key exchange (decapsulate shared secret)
    pub fn complete_key_exchange(
        &mut self,
        responder_data: &KyberKeyExchange,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        tracing::info!("Completing Kyber key exchange");
        
        if responder_data.role != KeyExchangeRole::Responder {
            return Err("Invalid key exchange role".into());
        }
        
        let ciphertext_bytes = responder_data.ciphertext
            .as_ref()
            .ok_or("Responder must have ciphertext")?;
        
        // Get our secret key
        let (_, secret_key) = self.our_keypair
            .as_ref()
            .ok_or("No keypair available for decapsulation")?;
        
        // Reconstruct ciphertext
        let ciphertext = kyber768::Ciphertext::from_bytes(ciphertext_bytes)
            .map_err(|e| format!("Invalid Kyber ciphertext: {:?}", e))?;
        
        // Decapsulate shared secret
        let shared_secret = kyber768::decapsulate(&ciphertext, secret_key);
        
        // Derive final shared secret using KDF
        let derived_secret = Self::derive_shared_secret(&shared_secret, "kyber-session")?;
        
        // Store shared secret
        self.shared_secret = Some(derived_secret.clone());
        
        Ok(derived_secret)
    }
    
    /// Get the derived shared secret
    pub fn get_shared_secret(&self) -> Option<&[u8]> {
        self.shared_secret.as_deref()
    }
    
    /// Clear sensitive data
    pub fn clear(&mut self) {
        self.our_keypair = None;
        if let Some(ref mut secret) = self.shared_secret {
            // Zero out the secret
            for byte in secret.iter_mut() {
                *byte = 0;
            }
        }
        self.shared_secret = None;
    }
    
    /// Derive shared secret using HKDF-like construction
    fn derive_shared_secret(
        kyber_shared_secret: &kyber768::SharedSecret,
        context: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Use SHA-256 to derive the final shared secret
        let mut hasher = Sha256::new();
        hasher.update(kyber_shared_secret.as_bytes());
        hasher.update(context.as_bytes());
        hasher.update(b"terminal-chat-kyber-kdf");
        
        let hash = hasher.finalize();
        Ok(hash.to_vec())
    }
    
    /// Verify key exchange integrity
    pub fn verify_key_exchange(
        data: &KyberKeyExchange,
        max_age_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now - data.timestamp > max_age_seconds {
            return Err("Key exchange data too old".into());
        }
        
        // Validate based on role
        match data.role {
            KeyExchangeRole::Initiator => {
                if data.public_key.is_empty() {
                    return Err("Initiator must have public key".into());
                }
                if data.ciphertext.is_some() {
                    return Err("Initiator should not have ciphertext".into());
                }
                
                // Validate public key length
                if data.public_key.len() != kyber768::public_key_bytes() {
                    return Err("Invalid Kyber public key length".into());
                }
            }
            KeyExchangeRole::Responder => {
                if data.ciphertext.is_none() {
                    return Err("Responder must have ciphertext".into());
                }
                
                // Validate ciphertext length
                if let Some(ref ct) = data.ciphertext {
                    if ct.len() != kyber768::ciphertext_bytes() {
                        return Err("Invalid Kyber ciphertext length".into());
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for KyberKeyExchangeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for KyberKeyExchangeManager {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Kyber key exchange result
#[derive(Debug)]
pub struct KeyExchangeResult {
    /// Derived shared secret
    pub shared_secret: Vec<u8>,
    /// Key exchange completion timestamp
    pub completed_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kyber_key_exchange_flow() {
        let mut alice = KyberKeyExchangeManager::new();
        let mut bob = KyberKeyExchangeManager::new();
        
        // Alice initiates
        let alice_init = alice.initiate_key_exchange().unwrap();
        assert_eq!(alice_init.role, KeyExchangeRole::Initiator);
        assert!(!alice_init.public_key.is_empty());
        assert!(alice_init.ciphertext.is_none());
        
        // Bob responds
        let (bob_response, bob_secret) = bob.respond_to_key_exchange(&alice_init).unwrap();
        assert_eq!(bob_response.role, KeyExchangeRole::Responder);
        assert!(bob_response.ciphertext.is_some());
        
        // Alice completes
        let alice_secret = alice.complete_key_exchange(&bob_response).unwrap();
        
        // Secrets should match
        assert_eq!(alice_secret, bob_secret);
        assert_eq!(alice.get_shared_secret().unwrap(), bob.get_shared_secret().unwrap());
    }
    
    #[test]
    fn test_key_exchange_verification() {
        let mut manager = KyberKeyExchangeManager::new();
        let data = manager.initiate_key_exchange().unwrap();
        
        // Should pass verification
        assert!(KyberKeyExchangeManager::verify_key_exchange(&data, 300).is_ok());
        
        // Should fail with old timestamp
        let mut old_data = data.clone();
        old_data.timestamp = 0;
        assert!(KyberKeyExchangeManager::verify_key_exchange(&old_data, 300).is_err());
    }
    
    #[test]
    fn test_shared_secret_derivation() {
        // Create a dummy shared secret
        let dummy_secret = kyber768::SharedSecret::from_bytes(&[42u8; kyber768::shared_secret_bytes()]).unwrap();
        
        let secret1 = KyberKeyExchangeManager::derive_shared_secret(&dummy_secret, "test-context").unwrap();
        let secret2 = KyberKeyExchangeManager::derive_shared_secret(&dummy_secret, "test-context").unwrap();
        let secret3 = KyberKeyExchangeManager::derive_shared_secret(&dummy_secret, "different-context").unwrap();
        
        // Same context should produce same result
        assert_eq!(secret1, secret2);
        
        // Different context should produce different result
        assert_ne!(secret1, secret3);
    }
}
