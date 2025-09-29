use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};

use crate::error::{IdentityError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub username: String,
    pub algorithm: String,
    pub public_key: String,      // Base64 encoded for JSON readability
    pub secret_key: String,      // Base64 encoded (encrypted)
    pub fingerprint: String,     // Hex format like "d1:34:fe:77:ab:99"
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Identity {
    pub fn new(
        username: String,
        algorithm: String,
        public_key_bytes: &[u8],
        encrypted_secret_key: &[u8],
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let public_key = general_purpose::STANDARD.encode(public_key_bytes); // Store as base64 string
        let secret_key = general_purpose::STANDARD.encode(encrypted_secret_key);
        let fingerprint = Self::generate_fingerprint(public_key_bytes)?;
        
        Ok(Identity {
            username,
            algorithm,
            public_key,
            secret_key,
            fingerprint,
            created_at: Utc::now(),
            expires_at,
        })
    }
    
    pub fn generate_fingerprint(public_key_bytes: &[u8]) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(public_key_bytes);
        let hash = hasher.finalize();
        
        // Take first 6 bytes and format as colon-separated hex
        let fingerprint_bytes = &hash[..6];
        let fingerprint = fingerprint_bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(":");
            
        Ok(fingerprint)
    }
    
    pub fn get_public_key_bytes(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.public_key)
            .map_err(IdentityError::Base64)
    }
    
    pub fn get_secret_key_bytes(&self) -> Result<Vec<u8>> {
        general_purpose::STANDARD
            .decode(&self.secret_key)
            .map_err(IdentityError::Base64)
    }
    
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(IdentityError::Json)
    }
    
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(IdentityError::Json)
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    pub fn short_fingerprint(&self) -> String {
        // Return first 2 segments for easy verification
        self.fingerprint
            .split(':')
            .take(2)
            .collect::<Vec<_>>()
            .join(":")
    }
}
