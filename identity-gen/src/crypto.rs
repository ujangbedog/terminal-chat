use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand::rngs::OsRng as StdOsRng;
use base64::{Engine as _, engine::general_purpose};

use crate::error::{IdentityError, Result};

pub struct KeyPair {
    pub public_key: dilithium2::PublicKey,
    pub secret_key: dilithium2::SecretKey,
}

impl KeyPair {
    pub fn generate() -> Result<Self> {
        let (public_key, secret_key) = dilithium2::keypair();
        
        Ok(KeyPair {
            public_key,
            secret_key,
        })
    }
    
    pub fn public_key_bytes(&self) -> &[u8] {
        self.public_key.as_bytes()
    }
    
    pub fn secret_key_bytes(&self) -> &[u8] {
        self.secret_key.as_bytes()
    }
    
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        dilithium2::sign(message, &self.secret_key).as_bytes().to_vec()
    }
    
    pub fn verify(_message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        if let Ok(pk) = dilithium2::PublicKey::from_bytes(public_key) {
            if let Ok(sig) = dilithium2::SignedMessage::from_bytes(signature) {
                return dilithium2::open(&sig, &pk).is_ok();
            }
        }
        false
    }
}

pub struct Encryption;

impl Encryption {
    pub fn encrypt_secret_key(secret_key: &[u8], password: &str) -> Result<Vec<u8>> {
        // Generate salt for password hashing
        let salt = SaltString::generate(&mut StdOsRng);
        
        // Hash password using Argon2
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| IdentityError::PasswordHash(e.to_string()))?;
        
        // Use the hash as encryption key (first 32 bytes)
        let binding = password_hash.hash.unwrap();
        let hash_bytes = binding.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
        
        // Generate nonce
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the secret key
        let ciphertext = cipher
            .encrypt(&nonce, secret_key)
            .map_err(|e| IdentityError::Encryption(e.to_string()))?;
        
        // Combine salt + nonce + ciphertext with base64 encoding for binary data
        let nonce_b64 = general_purpose::STANDARD.encode(&nonce);
        let ciphertext_b64 = general_purpose::STANDARD.encode(&ciphertext);
        
        let combined = format!("{}|{}|{}", salt.as_str(), nonce_b64, ciphertext_b64);
        Ok(combined.into_bytes())
    }
    
    pub fn decrypt_secret_key(encrypted_data: &[u8], password: &str) -> Result<Vec<u8>> {
        // Split the data: salt|nonce|ciphertext
        let data_str = std::str::from_utf8(encrypted_data)
            .map_err(|e| IdentityError::Decryption(format!("Invalid UTF-8: {}", e)))?;
        
        let parts: Vec<&str> = data_str.split('|').collect();
        if parts.len() != 3 {
            return Err(IdentityError::Decryption("Invalid encrypted data format".to_string()));
        }
        
        let salt_str = parts[0];
        let nonce_bytes = general_purpose::STANDARD
            .decode(parts[1])
            .map_err(|e| IdentityError::Decryption(format!("Invalid nonce base64: {}", e)))?;
        let ciphertext = general_purpose::STANDARD
            .decode(parts[2])
            .map_err(|e| IdentityError::Decryption(format!("Invalid ciphertext base64: {}", e)))?;
        
        // Recreate password hash
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| IdentityError::Decryption(format!("Invalid salt: {}", e)))?;
        
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| IdentityError::PasswordHash(e.to_string()))?;
        
        // Use the hash as decryption key
        let binding = password_hash.hash.unwrap();
        let hash_bytes = binding.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes[..32]);
        
        // Decrypt
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|e| IdentityError::Decryption(e.to_string()))?;
        
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert!(!keypair.public_key_bytes().is_empty());
        assert!(!keypair.secret_key_bytes().is_empty());
    }
    
    #[test]
    fn test_sign_verify() {
        let keypair = KeyPair::generate().unwrap();
        let message = b"Hello, World!";
        
        let signature = keypair.sign(message);
        let is_valid = KeyPair::verify(message, &signature, keypair.public_key_bytes());
        
        assert!(is_valid);
    }
    
    #[test]
    fn test_encryption_decryption() {
        let secret_data = b"super secret key data";
        let password = "strong_password_123";
        
        let encrypted = Encryption::encrypt_secret_key(secret_data, password).unwrap();
        let decrypted = Encryption::decrypt_secret_key(&encrypted, password).unwrap();
        
        assert_eq!(secret_data, decrypted.as_slice());
    }
}
