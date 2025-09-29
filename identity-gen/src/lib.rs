pub mod error;
pub mod identity;
pub mod crypto;
pub mod file_manager;
pub mod cli;

use chrono::{Utc, Duration};

// Re-export main types and functions for easy use
pub use error::{IdentityError, Result};
pub use identity::Identity;
pub use crypto::{KeyPair, Encryption};
pub use file_manager::FileManager;
pub use cli::{CliHandler, Commands};

/// Main entry point for identity generation functionality
/// This function provides the same interface as the CLI but can be called programmatically
pub async fn run_identity_generator() -> Result<()> {
    CliHandler::interactive_mode()
}

/// Generate a new identity with the given parameters
pub async fn generate_identity(
    username: Option<String>,
    expires_days: Option<i64>,
    non_interactive: bool,
) -> Result<Identity> {
    if non_interactive && username.is_none() {
        return Err(IdentityError::InvalidInput("Username required in non-interactive mode".to_string()));
    }
    
    // For non-interactive mode, we need a default password
    // In a real implementation, this should be passed as a parameter
    let password = if non_interactive {
        "default_password_123" // This should be configurable
    } else {
        return Err(IdentityError::InvalidInput("Interactive mode not supported in library mode".to_string()));
    };
    
    let username = username.unwrap_or_else(|| "default_user".to_string());
    
    // Calculate expiration date
    let expires_at = expires_days.map(|days| {
        Utc::now() + Duration::days(days)
    });
    
    // Generate key pair
    let keypair = KeyPair::generate()
        .map_err(|e| IdentityError::KeyGeneration(e.to_string()))?;
    
    // Encrypt private key
    let encrypted_secret_key = Encryption::encrypt_secret_key(
        keypair.secret_key_bytes(),
        password
    )?;
    
    // Create identity
    let identity = Identity::new(
        username,
        "dilithium2".to_string(),
        keypair.public_key_bytes(),
        &encrypted_secret_key,
        expires_at,
    )?;
    
    // Save identity
    FileManager::save_identity(&identity, None)?;
    
    Ok(identity)
}

/// List all existing identities
pub fn list_identities() -> Result<Vec<(String, std::path::PathBuf)>> {
    FileManager::list_identities()
}

/// Load an identity by username
pub fn load_identity(username: &str) -> Result<Identity> {
    let identity_dir = FileManager::get_identity_dir()?;
    let filename = FileManager::get_identity_filename(username);
    let file_path = identity_dir.join(filename);
    FileManager::load_identity(&file_path)
}

/// Check if an identity exists
pub fn identity_exists(username: &str) -> Result<bool> {
    FileManager::identity_exists(username)
}

/// Delete an identity
pub fn delete_identity(username: &str) -> Result<()> {
    FileManager::delete_identity(username)
}

/// Verify an identity file
pub fn verify_identity_file(file_path: &std::path::Path) -> Result<bool> {
    let identity = FileManager::load_identity(file_path)?;
    
    // Verify public key fingerprint
    let public_key_bytes = identity.get_public_key_bytes()?;
    let calculated_fingerprint = Identity::generate_fingerprint(&public_key_bytes)?;
    
    Ok(calculated_fingerprint == identity.fingerprint)
}
