//! Identity creation and management

use colored::*;
use dialoguer::{theme::ColorfulTheme, Select, Input, Password};
use identity_gen::{Identity, KeyPair, Encryption};
use crate::auth::types::AuthenticatedUser;

pub struct IdentityManager;

impl IdentityManager {
    /// Create a new identity interactively
    pub async fn create_new_identity() -> Result<AuthenticatedUser, Box<dyn std::error::Error>> {
        println!();
        println!("{}", "🆕 Creating New Identity".bright_green().bold());
        println!("{}", "─".repeat(60).dimmed());
        println!();
        
        // Get username
        let username: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a username")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Username cannot be empty")
                } else if input.len() > 32 {
                    Err("Username must be 32 characters or less")
                } else if input.contains(' ') {
                    Err("Username cannot contain spaces")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;
        
        // Check if username already exists
        if identity_gen::identity_exists(&username)? {
            return Err(format!("Identity '{}' already exists. Please choose a different username.", username).into());
        }
        
        // Get password
        let password = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Create a secure password")
            .with_confirmation("Confirm password", "Passwords don't match")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.len() < 8 {
                    Err("Password must be at least 8 characters long")
                } else {
                    Ok(())
                }
            })
            .interact()?;
        
        // Get expiration (optional)
        let expire_options = vec![
            "Never expires",
            "1 year (365 days)",
            "2 years (730 days)",
            "Custom duration",
        ];
        
        let expire_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Identity expiration")
            .default(1)
            .items(&expire_options)
            .interact()?;
        
        let expires_days = match expire_selection {
            0 => None,
            1 => Some(365),
            2 => Some(730),
            3 => {
                let days: i64 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter number of days")
                    .validate_with(|input: &i64| -> Result<(), &str> {
                        if *input <= 0 {
                            Err("Days must be positive")
                        } else if *input > 3650 {
                            Err("Maximum 10 years (3650 days)")
                        } else {
                            Ok(())
                        }
                    })
                    .interact_text()?;
                Some(days)
            }
            _ => unreachable!(),
        };
        
        // Show creation progress
        println!();
        println!("{}", "🔐 Generating cryptographic keys...".bright_cyan());
        println!("{}", "This may take a moment...".dimmed());
        
        // Generate identity using the identity-gen library
        let identity = Self::generate_identity_with_password(&username, &password, expires_days).await?;
        
        println!();
        println!("{}", "✅ Identity created successfully!".bright_green().bold());
        println!();
        
        // Show identity details
        Self::show_identity_details(&identity);
        
        // Wait for user acknowledgment
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Press Enter to continue to DPQ Chat")
            .allow_empty(true)
            .interact_text()?;
        
        Ok(AuthenticatedUser {
            username: identity.username.clone(),
            identity,
        })
    }
    
    /// Generate identity with password (custom implementation)
    async fn generate_identity_with_password(
        username: &str,
        password: &str,
        expires_days: Option<i64>,
    ) -> Result<Identity, Box<dyn std::error::Error>> {
        use chrono::{Utc, Duration};
        
        // Calculate expiration date
        let expires_at = expires_days.map(|days| {
            Utc::now() + Duration::days(days)
        });
        
        // Generate key pair
        let keypair = KeyPair::generate()
            .map_err(|e| format!("Key generation failed: {}", e))?;
        
        // Encrypt private key with user's password
        let encrypted_secret_key = Encryption::encrypt_secret_key(
            keypair.secret_key_bytes(),
            password
        )?;
        
        // Create identity
        let identity = Identity::new(
            username.to_string(),
            "dilithium2".to_string(),
            keypair.public_key_bytes(),
            &encrypted_secret_key,
            expires_at,
        )?;
        
        // Save identity
        identity_gen::FileManager::save_identity(&identity, None)?;
        
        // Also save public and private key files (like CLI identity-gen does)
        Self::save_key_files(&identity, &keypair, &encrypted_secret_key).await?;
        
        Ok(identity)
    }
    
    /// Save public and private key files (like CLI identity-gen does)
    async fn save_key_files(
        identity: &Identity,
        keypair: &KeyPair,
        encrypted_secret_key: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        use base64::{Engine as _, engine::general_purpose};
        use std::fs;
        
        // Get identities directory
        let identities_dir = identity_gen::FileManager::get_identities_dir()?;
        let username = &identity.username;
        
        // Create file paths
        let pub_key_path = identities_dir.join(format!("{}.pub", username));
        let priv_key_path = identities_dir.join(format!("{}.key", username));
        
        // Save public key in PEM format
        let pub_key_b64 = general_purpose::STANDARD.encode(keypair.public_key_bytes());
        let pub_key_pem = format!(
            "-----BEGIN DILITHIUM2 PUBLIC KEY-----\n{}\n-----END DILITHIUM2 PUBLIC KEY-----\n",
            pub_key_b64
        );
        fs::write(&pub_key_path, pub_key_pem)?;
        
        // Save encrypted private key (base64 encoded for readability)
        let priv_key_b64 = general_purpose::STANDARD.encode(encrypted_secret_key);
        fs::write(&priv_key_path, priv_key_b64)?;
        
        // Set file permissions (read/write for owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            // Set permissions for public key file
            let mut pub_perms = fs::metadata(&pub_key_path)?.permissions();
            pub_perms.set_mode(0o644); // rw-r--r--
            fs::set_permissions(&pub_key_path, pub_perms)?;
            
            // Set permissions for private key file (more restrictive)
            let mut priv_perms = fs::metadata(&priv_key_path)?.permissions();
            priv_perms.set_mode(0o600); // rw-------
            fs::set_permissions(&priv_key_path, priv_perms)?;
        }
        
        println!("{}", "✓ Public key exported to:".bright_green());
        println!("  {}", pub_key_path.display().to_string().bright_cyan());
        println!("{}", "✓ Private key exported to:".bright_green());
        println!("  {}", priv_key_path.display().to_string().bright_cyan());
        
        Ok(())
    }
    
    /// Show identity details after creation
    fn show_identity_details(identity: &Identity) {
        println!("{}", "📋 Identity Details".bright_yellow().bold());
        println!("{}", "─".repeat(50).dimmed());
        println!("👤 Username: {}", identity.username.bright_white());
        println!("🔐 Algorithm: {}", identity.algorithm.bright_white());
        println!("🔑 Fingerprint: {}", identity.fingerprint.bright_white());
        println!("📅 Created: {}", identity.created_at.format("%Y-%m-%d %H:%M UTC").to_string().bright_white());
        
        if let Some(expires) = identity.expires_at {
            println!("⏰ Expires: {}", expires.format("%Y-%m-%d %H:%M UTC").to_string().bright_white());
        } else {
            println!("⏰ Expires: {}", "Never".bright_green());
        }
        
        println!("{}", "─".repeat(50).dimmed());
        println!();
        
        println!("{}", "🔒 Your identity is secured with post-quantum cryptography!".bright_magenta().bold());
        println!("{}", "Keep your password safe - it's required to access your identity.".bright_yellow());
        println!();
    }
}
