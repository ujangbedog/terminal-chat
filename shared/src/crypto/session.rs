//! Session key management for ephemeral encryption

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use rand::RngCore;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Ephemeral session key for peer-to-peer communication
#[derive(Debug, Clone)]
pub struct SessionKey {
    /// AES-256-GCM key for message encryption
    key: [u8; 32],
    /// Creation timestamp
    created_at: u64,
    /// Peer fingerprint this session is with
    peer_fingerprint: String,
}

impl SessionKey {
    /// Generate a new random session key
    pub fn generate(peer_fingerprint: String) -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            key,
            created_at,
            peer_fingerprint,
        }
    }
    
    /// Create session key from shared secret (from key exchange)
    pub fn from_shared_secret(shared_secret: &[u8], peer_fingerprint: String) -> Self {
        use sha2::{Sha256, Digest};
        
        // Derive session key from shared secret using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        hasher.update(b"dpq-chat-session-key");
        let hash = hasher.finalize();
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..32]);
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            key,
            created_at,
            peer_fingerprint,
        }
    }
    
    /// Get the encryption key
    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }
    
    /// Get creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }
    
    /// Get peer fingerprint
    pub fn peer_fingerprint(&self) -> &str {
        &self.peer_fingerprint
    }
    
    /// Check if session key is expired (older than 1 hour)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.created_at > 3600 // 1 hour
    }
    
    /// Encrypt a message using this session key
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        // Prepend nonce to ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// Decrypt a message using this session key
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data: too short".into());
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
}

/// Manages session keys for multiple peers
#[derive(Debug)]
pub struct SessionManager {
    /// Active session keys indexed by peer fingerprint
    sessions: HashMap<String, SessionKey>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
    
    /// Add a new session key for a peer
    pub fn add_session(&mut self, peer_fingerprint: String, session_key: SessionKey) {
        tracing::info!("Adding session key for peer: {}", peer_fingerprint);
        self.sessions.insert(peer_fingerprint, session_key);
    }
    
    /// Get session key for a peer
    pub fn get_session(&self, peer_fingerprint: &str) -> Option<&SessionKey> {
        self.sessions.get(peer_fingerprint)
    }
    
    /// Remove session key for a peer (when they disconnect)
    pub fn remove_session(&mut self, peer_fingerprint: &str) -> Option<SessionKey> {
        tracing::info!("Removing session key for peer: {}", peer_fingerprint);
        self.sessions.remove(peer_fingerprint)
    }
    
    /// Clean up expired session keys
    pub fn cleanup_expired(&mut self) {
        let expired_peers: Vec<String> = self.sessions
            .iter()
            .filter(|(_, session)| session.is_expired())
            .map(|(peer, _)| peer.clone())
            .collect();
        
        for peer in expired_peers {
            tracing::info!("Removing expired session key for peer: {}", peer);
            self.sessions.remove(&peer);
        }
    }
    
    /// Get all active peer fingerprints
    pub fn active_peers(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }
    
    /// Check if we have an active session with a peer
    pub fn has_session(&self, peer_fingerprint: &str) -> bool {
        self.sessions.contains_key(peer_fingerprint)
    }
    
    /// Get number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session key exchange data for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKeyExchange {
    /// Ephemeral public key for ECDH
    pub ephemeral_public_key: Vec<u8>,
    /// Timestamp of key generation
    pub timestamp: u64,
    /// Signature of the exchange data (signed with identity key)
    pub signature: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_key_generation() {
        let session_key = SessionKey::generate("test_peer".to_string());
        assert_eq!(session_key.peer_fingerprint(), "test_peer");
        assert!(!session_key.is_expired());
    }
    
    #[test]
    fn test_session_key_encryption() {
        let session_key = SessionKey::generate("test_peer".to_string());
        let message = b"Hello, secure world!";
        
        let encrypted = session_key.encrypt(message).unwrap();
        let decrypted = session_key.decrypt(&encrypted).unwrap();
        
        assert_eq!(message, decrypted.as_slice());
    }
    
    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();
        let session_key = SessionKey::generate("peer1".to_string());
        
        manager.add_session("peer1".to_string(), session_key);
        assert!(manager.has_session("peer1"));
        assert_eq!(manager.session_count(), 1);
        
        let removed = manager.remove_session("peer1");
        assert!(removed.is_some());
        assert!(!manager.has_session("peer1"));
        assert_eq!(manager.session_count(), 0);
    }
}
