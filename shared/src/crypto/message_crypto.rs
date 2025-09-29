//! Message encryption and decryption using session keys

use serde::{Serialize, Deserialize};
use crate::crypto::session::SessionKey;

/// Encrypted message structure for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Sender's fingerprint (for identification)
    pub sender_fingerprint: String,
    /// Encrypted message content
    pub encrypted_content: Vec<u8>,
    /// Message timestamp
    pub timestamp: u64,
    /// Message type (text, file, system, etc.)
    pub message_type: MessageType,
    /// Message sequence number (for ordering)
    pub sequence: u64,
}

/// Types of messages that can be encrypted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Regular text message
    Text,
    /// File transfer
    File { filename: String, size: u64 },
    /// System message (join, leave, etc.)
    System,
    /// Typing indicator
    Typing,
    /// Message acknowledgment
    Ack { message_id: u64 },
}

/// Plain text message structure (before encryption)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainMessage {
    /// Message content
    pub content: String,
    /// Sender username
    pub sender: String,
    /// Message timestamp
    pub timestamp: u64,
    /// Message type
    pub message_type: MessageType,
}

/// Message encryption and decryption utilities
pub struct MessageCrypto;

impl MessageCrypto {
    /// Encrypt a plain message using session key
    pub fn encrypt_message(
        session_key: &SessionKey,
        message: &PlainMessage,
        sequence: u64,
    ) -> Result<EncryptedMessage, Box<dyn std::error::Error>> {
        // Serialize the plain message
        let message_bytes = serde_json::to_vec(message)?;
        
        // Encrypt using session key
        let encrypted_content = session_key.encrypt(&message_bytes)?;
        
        Ok(EncryptedMessage {
            sender_fingerprint: session_key.peer_fingerprint().to_string(),
            encrypted_content,
            timestamp: message.timestamp,
            message_type: message.message_type.clone(),
            sequence,
        })
    }
    
    /// Decrypt an encrypted message using session key
    pub fn decrypt_message(
        session_key: &SessionKey,
        encrypted_message: &EncryptedMessage,
    ) -> Result<PlainMessage, Box<dyn std::error::Error>> {
        // Decrypt the message content
        let decrypted_bytes = session_key.decrypt(&encrypted_message.encrypted_content)?;
        
        // Deserialize the plain message
        let plain_message: PlainMessage = serde_json::from_slice(&decrypted_bytes)?;
        
        Ok(plain_message)
    }
    
    /// Create a text message
    pub fn create_text_message(
        sender: String,
        content: String,
    ) -> PlainMessage {
        PlainMessage {
            content,
            sender,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message_type: MessageType::Text,
        }
    }
    
    /// Create a system message
    pub fn create_system_message(
        sender: String,
        content: String,
    ) -> PlainMessage {
        PlainMessage {
            content,
            sender,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message_type: MessageType::System,
        }
    }
    
    /// Create a typing indicator message
    pub fn create_typing_message(sender: String) -> PlainMessage {
        PlainMessage {
            content: "typing...".to_string(),
            sender,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message_type: MessageType::Typing,
        }
    }
    
    /// Verify message integrity (check timestamp, sequence, etc.)
    pub fn verify_message_integrity(
        encrypted_message: &EncryptedMessage,
        expected_sender: &str,
        max_age_seconds: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check timestamp (prevent replay attacks)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now - encrypted_message.timestamp > max_age_seconds {
            return Err("Message too old".into());
        }
        
        // Check sender fingerprint
        if encrypted_message.sender_fingerprint != expected_sender {
            return Err("Sender fingerprint mismatch".into());
        }
        
        Ok(())
    }
}

/// Message sequence manager to prevent replay attacks
#[derive(Debug)]
pub struct MessageSequenceManager {
    /// Last seen sequence numbers for each peer
    peer_sequences: std::collections::HashMap<String, u64>,
    /// Our outgoing sequence number
    our_sequence: u64,
}

impl MessageSequenceManager {
    /// Create a new sequence manager
    pub fn new() -> Self {
        Self {
            peer_sequences: std::collections::HashMap::new(),
            our_sequence: 0,
        }
    }
    
    /// Get next sequence number for outgoing message
    pub fn next_sequence(&mut self) -> u64 {
        self.our_sequence += 1;
        self.our_sequence
    }
    
    /// Validate incoming message sequence
    pub fn validate_sequence(
        &mut self,
        peer_fingerprint: &str,
        sequence: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.peer_sequences.get(peer_fingerprint) {
            Some(&last_sequence) => {
                if sequence <= last_sequence {
                    return Err("Duplicate or old message sequence".into());
                }
            }
            None => {
                // First message from this peer
            }
        }
        
        // Update last seen sequence
        self.peer_sequences.insert(peer_fingerprint.to_string(), sequence);
        Ok(())
    }
    
    /// Reset sequence for a peer (when they reconnect)
    pub fn reset_peer_sequence(&mut self, peer_fingerprint: &str) {
        self.peer_sequences.remove(peer_fingerprint);
    }
}

impl Default for MessageSequenceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::session::SessionKey;
    
    #[test]
    fn test_message_encryption_decryption() {
        let session_key = SessionKey::generate("test_peer".to_string());
        let plain_message = MessageCrypto::create_text_message(
            "alice".to_string(),
            "Hello, Bob!".to_string(),
        );
        
        let encrypted = MessageCrypto::encrypt_message(&session_key, &plain_message, 1).unwrap();
        let decrypted = MessageCrypto::decrypt_message(&session_key, &encrypted).unwrap();
        
        assert_eq!(plain_message.content, decrypted.content);
        assert_eq!(plain_message.sender, decrypted.sender);
    }
    
    #[test]
    fn test_sequence_manager() {
        let mut manager = MessageSequenceManager::new();
        
        assert_eq!(manager.next_sequence(), 1);
        assert_eq!(manager.next_sequence(), 2);
        
        // Validate incoming sequences
        assert!(manager.validate_sequence("peer1", 1).is_ok());
        assert!(manager.validate_sequence("peer1", 2).is_ok());
        assert!(manager.validate_sequence("peer1", 1).is_err()); // Duplicate
    }
}
