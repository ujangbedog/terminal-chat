use std::path::{Path, PathBuf};
use std::fs;
use colored::*;

use crate::identity::Identity;
use crate::error::{IdentityError, Result};

pub struct FileManager;

impl FileManager {
    /// Get the default identity directory
    pub fn get_identity_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| IdentityError::FileIo(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
            ))?;
        
        let identity_dir = home_dir.join(".terminal-chat").join("identities");
        
        // Create directory if it doesn't exist
        if !identity_dir.exists() {
            fs::create_dir_all(&identity_dir)?;
            println!("{} Created identity directory: {}", 
                "✓".green().bold(), 
                identity_dir.display().to_string().cyan()
            );
        }
        
        Ok(identity_dir)
    }
    
    /// Get the identities directory (alias for get_identity_dir for consistency)
    pub fn get_identities_dir() -> Result<PathBuf> {
        Self::get_identity_dir()
    }
    
    /// Generate filename for identity
    pub fn get_identity_filename(username: &str) -> String {
        format!("{}.identity.json", username.to_lowercase())
    }
    
    /// Save identity to file
    pub fn save_identity(identity: &Identity, custom_path: Option<&Path>) -> Result<PathBuf> {
        let file_path = if let Some(path) = custom_path {
            path.to_path_buf()
        } else {
            let identity_dir = Self::get_identity_dir()?;
            let filename = Self::get_identity_filename(&identity.username);
            identity_dir.join(filename)
        };
        
        // Check if file already exists
        if file_path.exists() {
            return Err(IdentityError::InvalidInput(
                format!("Identity file already exists: {}", file_path.display())
            ));
        }
        
        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write identity to file
        let json_content = identity.to_json()?;
        fs::write(&file_path, json_content)?;
        
        // Set file permissions (read/write for owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&file_path)?.permissions();
            perms.set_mode(0o600); // rw-------
            fs::set_permissions(&file_path, perms)?;
        }
        
        println!("{} Identity saved to: {}", 
            "✓".green().bold(), 
            file_path.display().to_string().cyan()
        );
        
        Ok(file_path)
    }
    
    /// Load identity from file
    pub fn load_identity(file_path: &Path) -> Result<Identity> {
        if !file_path.exists() {
            return Err(IdentityError::FileIo(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Identity file not found")
            ));
        }
        
        let json_content = fs::read_to_string(file_path)?;
        Identity::from_json(&json_content)
    }
    
    /// List all identity files in the default directory
    pub fn list_identities() -> Result<Vec<(String, PathBuf)>> {
        let identity_dir = Self::get_identity_dir()?;
        let mut identities = Vec::new();
        
        if identity_dir.exists() {
            for entry in fs::read_dir(identity_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                        if filename.ends_with(".identity") {
                            let username = filename.replace(".identity", "");
                            identities.push((username, path));
                        }
                    }
                }
            }
        }
        
        identities.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(identities)
    }
    
    /// Delete identity file and associated key files
    pub fn delete_identity(username: &str) -> Result<()> {
        let identity_dir = Self::get_identity_dir()?;
        let filename = Self::get_identity_filename(username);
        let file_path = identity_dir.join(filename);
        
        if file_path.exists() {
            // Delete main identity file
            fs::remove_file(&file_path)?;
            
            // Delete associated key files if they exist
            let pub_key_path = identity_dir.join(format!("{}.pub", username));
            let priv_key_path = identity_dir.join(format!("{}.key", username));
            
            if pub_key_path.exists() {
                fs::remove_file(&pub_key_path)?;
                println!("{} Public key file deleted: {}", 
                    "✓".green().bold(), 
                    pub_key_path.display().to_string().cyan()
                );
            }
            
            if priv_key_path.exists() {
                fs::remove_file(&priv_key_path)?;
                println!("{} Private key file deleted: {}", 
                    "✓".green().bold(), 
                    priv_key_path.display().to_string().cyan()
                );
            }
            
            println!("{} Identity deleted: {}", 
                "✓".green().bold(), 
                username.cyan()
            );
        } else {
            return Err(IdentityError::InvalidInput(
                format!("Identity not found: {}", username)
            ));
        }
        
        Ok(())
    }
    
    /// Check if identity exists
    pub fn identity_exists(username: &str) -> Result<bool> {
        let identity_dir = Self::get_identity_dir()?;
        let filename = Self::get_identity_filename(username);
        let file_path = identity_dir.join(filename);
        
        Ok(file_path.exists())
    }
}
