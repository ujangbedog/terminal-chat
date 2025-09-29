use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    
    #[error("Encryption failed: {0}")]
    Encryption(String),
    
    #[error("Decryption failed: {0}")]
    Decryption(String),
    
    #[error("File I/O error: {0}")]
    FileIo(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Base64 encoding/decoding error: {0}")]
    Base64(#[from] base64::DecodeError),
    
    #[error("Password hashing error: {0}")]
    PasswordHash(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, IdentityError>;
