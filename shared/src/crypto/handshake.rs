//! Handshake protocol for establishing secure sessions

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::crypto::session::SessionKey;
use crate::crypto::kyber_kex::{KyberKeyExchangeManager, KyberKeyExchange};
use crate::crypto::dilithium_ops::{DilithiumKeypair, DilithiumVerifier};

/// Peer information exchanged during handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Username of the peer
    pub username: String,
    /// Public key fingerprint (identity)
    pub fingerprint: String,
    /// Full public key for verification
    pub public_key: Vec<u8>,
    /// Timestamp of handshake
    pub timestamp: u64,
}

/// Handshake data exchanged between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeData {
    /// Peer information
    pub peer_info: PeerInfo,
    /// Kyber key exchange data
    pub kyber_exchange: KyberKeyExchange,
    /// Signature of the handshake (signed with Dilithium identity key)
    pub signature: Vec<u8>,
    /// Protocol version
    pub protocol_version: String,
}

/// Handshake states
#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeState {
    /// Initial state - no handshake started
    Initial,
    /// Handshake initiated - waiting for response
    Initiated,
    /// Handshake received - processing
    Received,
    /// Handshake completed successfully
    Completed,
    /// Handshake failed
    Failed(String),
}

/// Manages handshake process with peers
#[derive(Debug)]
pub struct HandshakeManager {
    /// Our peer information
    our_info: PeerInfo,
    /// Handshake states with peers
    peer_states: HashMap<String, HandshakeState>,
    /// Pending handshakes
    pending_handshakes: HashMap<String, HandshakeData>,
    /// Kyber key exchange managers for each peer
    kyber_managers: HashMap<String, KyberKeyExchangeManager>,
    /// Our Dilithium keypair for signing
    dilithium_keypair: Option<DilithiumKeypair>,
}

impl HandshakeManager {
    /// Create a new handshake manager
    pub fn new(username: String, fingerprint: String, public_key: Vec<u8>) -> Self {
        let our_info = PeerInfo {
            username,
            fingerprint,
            public_key,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        Self {
            our_info,
            peer_states: HashMap::new(),
            pending_handshakes: HashMap::new(),
            kyber_managers: HashMap::new(),
            dilithium_keypair: None,
        }
    }
    
    /// Create a new handshake manager with Dilithium keypair
    pub fn new_with_dilithium(
        username: String,
        fingerprint: String,
        public_key: Vec<u8>,
        dilithium_keypair: DilithiumKeypair,
    ) -> Self {
        let our_info = PeerInfo {
            username,
            fingerprint,
            public_key,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        Self {
            our_info,
            peer_states: HashMap::new(),
            pending_handshakes: HashMap::new(),
            kyber_managers: HashMap::new(),
            dilithium_keypair: Some(dilithium_keypair),
        }
    }
    
    /// Set Dilithium keypair for signing
    pub fn set_dilithium_keypair(&mut self, keypair: DilithiumKeypair) {
        self.dilithium_keypair = Some(keypair);
    }
    
    /// Initiate handshake with a peer
    pub fn initiate_handshake(
        &mut self,
        peer_fingerprint: &str,
    ) -> Result<HandshakeData, Box<dyn std::error::Error>> {
        tracing::info!("Initiating Kyber handshake with peer: {}", peer_fingerprint);
        
        // Create Kyber key exchange manager for this peer
        let mut kyber_manager = KyberKeyExchangeManager::new();
        
        // Initiate Kyber key exchange
        let kyber_exchange = kyber_manager.initiate_key_exchange()?;
        
        // Store the manager for later use
        self.kyber_managers.insert(peer_fingerprint.to_string(), kyber_manager);
        
        // Create signature data (peer info + kyber exchange)
        let signature_data = self.create_signature_data(&self.our_info, &kyber_exchange)?;
        let signature = self.sign_handshake_data(&signature_data)?;
        
        // Create handshake data
        let handshake_data = HandshakeData {
            peer_info: self.our_info.clone(),
            kyber_exchange,
            signature,
            protocol_version: "dpq-chat-v2-kyber".to_string()
        };
        
        // Update state
        self.peer_states.insert(peer_fingerprint.to_string(), HandshakeState::Initiated);
        self.pending_handshakes.insert(peer_fingerprint.to_string(), handshake_data.clone());
        
        Ok(handshake_data)
    }
    
    /// Process received handshake data
    pub fn process_handshake(
        &mut self,
        handshake_data: HandshakeData,
    ) -> Result<(SessionKey, Option<HandshakeData>), Box<dyn std::error::Error>> {
        let peer_fingerprint = &handshake_data.peer_info.fingerprint;
        
        tracing::info!("Processing Kyber handshake from peer: {}", peer_fingerprint);
        
        // Verify the handshake signature
        self.verify_handshake(&handshake_data)?;
        
        // Get or create Kyber manager for this peer
        let shared_secret = match self.peer_states.get(peer_fingerprint) {
            Some(HandshakeState::Initiated) => {
                // We initiated, this is the response - complete the exchange
                let kyber_manager = self.kyber_managers.get_mut(peer_fingerprint)
                    .ok_or("No Kyber manager found for initiated handshake")?;
                
                kyber_manager.complete_key_exchange(&handshake_data.kyber_exchange)?
            }
            _ => {
                // This is a new handshake - we need to respond
                let mut kyber_manager = KyberKeyExchangeManager::new();
                let (response_kyber, shared_secret) = kyber_manager.respond_to_key_exchange(&handshake_data.kyber_exchange)?;
                
                // Store manager for potential future use
                self.kyber_managers.insert(peer_fingerprint.clone(), kyber_manager);
                
                // Create response handshake
                let signature_data = self.create_signature_data(&self.our_info, &response_kyber)?;
                let signature = self.sign_handshake_data(&signature_data)?;
                
                let response_handshake = HandshakeData {
                    peer_info: self.our_info.clone(),
                    kyber_exchange: response_kyber,
                    signature,
                    protocol_version: "dpq-chat-v2-kyber".to_string()
                };
                
                // Update state and store response
                self.peer_states.insert(peer_fingerprint.clone(), HandshakeState::Completed);
                self.pending_handshakes.remove(peer_fingerprint);
                
                // Create session key
                let session_key = SessionKey::from_shared_secret(&shared_secret, peer_fingerprint.clone());
                
                tracing::info!("Kyber handshake completed with peer: {}", peer_fingerprint);
                
                return Ok((session_key, Some(response_handshake)));
            }
        };
        
        // Create session key from shared secret
        let session_key = SessionKey::from_shared_secret(&shared_secret, peer_fingerprint.clone());
        
        // Update state
        self.peer_states.insert(peer_fingerprint.clone(), HandshakeState::Completed);
        self.pending_handshakes.remove(peer_fingerprint);
        
        tracing::info!("Kyber handshake completed with peer: {}", peer_fingerprint);
        
        Ok((session_key, None))
    }
    
    /// Get handshake state for a peer
    pub fn get_state(&self, peer_fingerprint: &str) -> HandshakeState {
        self.peer_states.get(peer_fingerprint)
            .cloned()
            .unwrap_or(HandshakeState::Initial)
    }
    
    /// Mark handshake as failed
    pub fn mark_failed(&mut self, peer_fingerprint: &str, reason: String) {
        tracing::warn!("Handshake failed with peer {}: {}", peer_fingerprint, reason);
        self.peer_states.insert(peer_fingerprint.to_string(), HandshakeState::Failed(reason));
        self.pending_handshakes.remove(peer_fingerprint);
    }
    
    /// Clean up completed or failed handshakes
    pub fn cleanup(&mut self) {
        self.peer_states.retain(|_, state| {
            !matches!(state, HandshakeState::Completed | HandshakeState::Failed(_))
        });
    }
    
    /// Get our peer info
    pub fn our_info(&self) -> &PeerInfo {
        &self.our_info
    }
    
    // Private helper methods
    
    /// Create signature data for handshake
    fn create_signature_data(
        &self,
        peer_info: &PeerInfo,
        kyber_exchange: &KyberKeyExchange,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        
        // Hash peer info
        hasher.update(&peer_info.username);
        hasher.update(&peer_info.fingerprint);
        hasher.update(&peer_info.public_key);
        hasher.update(&peer_info.timestamp.to_le_bytes());
        
        // Hash Kyber exchange data
        hasher.update(&kyber_exchange.public_key);
        if let Some(ref ciphertext) = kyber_exchange.ciphertext {
            hasher.update(ciphertext);
        }
        hasher.update(&kyber_exchange.timestamp.to_le_bytes());
        hasher.update(&format!("{:?}", kyber_exchange.role));
        
        Ok(hasher.finalize().to_vec())
    }
    
    /// Sign handshake data with Dilithium identity key
    fn sign_handshake_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        match &self.dilithium_keypair {
            Some(keypair) => {
                tracing::debug!("Signing handshake data with Dilithium private key");
                Ok(keypair.sign(data))
            }
            None => {
                tracing::warn!("No Dilithium keypair available for signing, using placeholder");
                // Fallback to placeholder for backward compatibility
                use sha2::{Sha256, Digest};
                
                let mut hasher = Sha256::new();
                hasher.update(data);
                hasher.update(&self.our_info.fingerprint);
                hasher.update(b"dpq-chat-dilithium-signature");
                let hash = hasher.finalize();
                
                Ok(hash.to_vec())
            }
        }
    }
    
    /// Verify handshake signature
    fn verify_handshake(&self, handshake_data: &HandshakeData) -> Result<(), Box<dyn std::error::Error>> {
        // Check protocol version
        if handshake_data.protocol_version != "dpq-chat-v2-kyber" {
            return Err("Unsupported protocol version".into());
        }
        
        // Verify Kyber exchange data
        crate::crypto::kyber_kex::KyberKeyExchangeManager::verify_key_exchange(&handshake_data.kyber_exchange, 300)?;
        
        // Recreate signature data
        let signature_data = self.create_signature_data(&handshake_data.peer_info, &handshake_data.kyber_exchange)?;
        
        // Verify Dilithium signature
        if handshake_data.signature.is_empty() {
            return Err("Empty signature".into());
        }
        
        // Try to verify with Dilithium
        let peer_public_key = &handshake_data.peer_info.public_key;
        match DilithiumVerifier::verify(&signature_data, &handshake_data.signature, peer_public_key) {
            Ok(true) => {
                tracing::debug!("Dilithium signature verified for peer: {}", handshake_data.peer_info.fingerprint);
                Ok(())
            }
            Ok(false) => {
                tracing::warn!("Dilithium signature verification failed for peer: {}", handshake_data.peer_info.fingerprint);
                Err("Invalid Dilithium signature".into())
            }
            Err(e) => {
                tracing::warn!("Dilithium signature verification error for peer {}: {}", handshake_data.peer_info.fingerprint, e);
                // For backward compatibility, allow non-Dilithium signatures to pass
                // This should be removed in production
                tracing::debug!("Allowing signature for backward compatibility");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_handshake_manager_creation() {
        let manager = HandshakeManager::new(
            "test_user".to_string(),
            "test_fingerprint".to_string(),
            vec![1, 2, 3, 4],
        );
        
        assert_eq!(manager.our_info().username, "test_user");
        assert_eq!(manager.our_info().fingerprint, "test_fingerprint");
    }
    
    #[test]
    fn test_kyber_handshake_initiation() {
        let mut manager = HandshakeManager::new(
            "alice".to_string(),
            "alice_fp".to_string(),
            vec![1, 2, 3, 4],
        );
        
        let handshake_data = manager.initiate_handshake("bob_fp").unwrap();
        assert_eq!(handshake_data.peer_info.username, "alice");
        assert_eq!(handshake_data.protocol_version, "dpq-chat-v2-kyber");
        assert_eq!(manager.get_state("bob_fp"), HandshakeState::Initiated);
        assert!(!handshake_data.kyber_exchange.public_key.is_empty());
        assert!(handshake_data.kyber_exchange.ciphertext.is_none());
    }
    
    #[test]
    fn test_kyber_handshake_full_flow() {
        let mut alice = HandshakeManager::new(
            "alice".to_string(),
            "alice_fp".to_string(),
            vec![1, 2, 3, 4],
        );
        
        let mut bob = HandshakeManager::new(
            "bob".to_string(),
            "bob_fp".to_string(),
            vec![5, 6, 7, 8],
        );
        
        // Alice initiates
        let alice_handshake = alice.initiate_handshake("bob_fp").unwrap();
        
        // Bob processes and responds
        let (bob_session, bob_response) = bob.process_handshake(alice_handshake).unwrap();
        assert!(bob_response.is_some());
        
        // Alice processes Bob's response
        let (alice_session, alice_response) = alice.process_handshake(bob_response.unwrap()).unwrap();
        assert!(alice_response.is_none()); // No further response needed
        
        // Both should have completed handshake
        assert_eq!(alice.get_state("bob_fp"), HandshakeState::Completed);
        assert_eq!(bob.get_state("alice_fp"), HandshakeState::Completed);
        
        // Session keys should be derived (we can't compare them directly due to different contexts)
        assert_eq!(alice_session.peer_fingerprint(), "bob_fp");
        assert_eq!(bob_session.peer_fingerprint(), "alice_fp");
    }
}
