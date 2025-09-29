//! Identity verification and password handling

use colored::*;
use dialoguer::{theme::ColorfulTheme, Select, Password};
use identity_gen::{list_identities, load_identity, Identity, Encryption};
use std::collections::HashMap;
use crate::auth::types::AuthenticatedUser;
use crate::auth::identity_manager::IdentityManager;

pub struct IdentityVerifier;

impl IdentityVerifier {
    /// Handle identity verification when identities exist
    pub async fn handle_identity_verification(
        identities: Vec<(String, std::path::PathBuf)>
    ) -> Result<AuthenticatedUser, Box<dyn std::error::Error>> {
        // Filter out expired identities and create a map
        let mut valid_identities = HashMap::new();
        let mut identity_options = Vec::new();
        
        for (username, _path) in identities {
            match load_identity(&username) {
                Ok(identity) => {
                    if identity.is_expired() {
                        println!("{} {} {}", 
                            "⚠️".bright_yellow(), 
                            format!("Identity '{}' has expired", username).bright_yellow(),
                            "(skipping)".dimmed()
                        );
                    } else {
                        identity_options.push(format!("👤 {} ({})", username, identity.short_fingerprint()));
                        valid_identities.insert(username.clone(), identity);
                    }
                }
                Err(_) => {
                    println!("{} {} {}", 
                        "❌".bright_red(), 
                        format!("Identity '{}' is corrupted", username).bright_red(),
                        "(skipping)".dimmed()
                    );
                }
            }
        }
        
        if valid_identities.is_empty() {
            println!();
            println!("{}", "❌ No valid identities found.".bright_red().bold());
            println!("{}", "All identities are either expired or corrupted.".bright_red());
            println!();
            
            return Self::handle_no_identities().await;
        }
        
        println!("{} {}", "Found".bright_white(), format!("{} valid identities", valid_identities.len()).bright_cyan().bold());
        println!("{}", "─".repeat(60).dimmed());
        println!();
        
        // Add option to create new identity
        identity_options.push("🆕 Create new identity".to_string());
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your identity")
            .default(0)
            .items(&identity_options)
            .interact()?;
        
        if selection == identity_options.len() - 1 {
            // Create new identity
            return IdentityManager::create_new_identity().await;
        }
        
        // Get selected username (extract from display string)
        let selected_display = &identity_options[selection];
        let username = selected_display
            .split_whitespace()
            .nth(1) // Get the username part
            .ok_or("Invalid selection")?
            .to_string();
        
        let identity = valid_identities.get(&username)
            .ok_or("Identity not found")?
            .clone();
        
        // Verify password
        Self::verify_identity_password(&username, &identity).await
    }
    
    /// Handle case when no identities exist
    async fn handle_no_identities() -> Result<AuthenticatedUser, Box<dyn std::error::Error>> {
        println!("{}", "🔍 No cryptographic identities found in your vault.".bright_yellow().bold());
        println!();
        
        println!("{}", "To use Terminal Chat, you need a secure identity.".bright_white());
        println!("{}", "This identity uses CRYSTALS-Dilithium post-quantum cryptography".dimmed());
        println!("{}", "to protect your communications against future quantum attacks.".dimmed());
        println!();
        
        let options = vec![
            "🆕 Create new identity now",
            "🚪 Exit application",
        ];
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&options)
            .interact()?;
        
        match selection {
            0 => {
                // Create new identity
                IdentityManager::create_new_identity().await
            }
            1 => {
                println!("{}", "👋 Goodbye! Come back when you're ready to create an identity.".bright_green());
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
    
    /// Verify identity password
    async fn verify_identity_password(
        username: &str,
        identity: &Identity,
    ) -> Result<AuthenticatedUser, Box<dyn std::error::Error>> {
        println!();
        println!("{}", format!("🔐 Verifying identity: {}", username).bright_cyan().bold());
        println!("{}", format!("Fingerprint: {}", identity.fingerprint).dimmed());
        println!();
        
        const MAX_ATTEMPTS: u8 = 3;
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            let password = Password::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Enter password for '{}' (attempt {}/{})", username, attempts, MAX_ATTEMPTS))
                .interact()?;
            
            // Try to decrypt the secret key to verify password
            match Self::verify_password(identity, &password) {
                Ok(true) => {
                    println!();
                    println!("{}", "✅ Authentication successful!".bright_green().bold());
                    println!("{}", format!("Welcome back, {}!", username).bright_green());
                    println!();
                    
                    // Wait a moment for user to see success message
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    
                    return Ok(AuthenticatedUser {
                        username: username.to_string(),
                        identity: identity.clone(),
                    });
                }
                Ok(false) => {
                    println!("{}", "❌ Invalid password".bright_red());
                    
                    if attempts >= MAX_ATTEMPTS {
                        println!();
                        println!("{}", "🚫 Maximum authentication attempts exceeded.".bright_red().bold());
                        println!("{}", "Access denied for security reasons.".bright_red());
                        return Err("Authentication failed".into());
                    }
                    
                    println!("{}", format!("Please try again. {} attempts remaining.", MAX_ATTEMPTS - attempts).bright_yellow());
                    println!();
                }
                Err(e) => {
                    return Err(format!("Authentication error: {}", e).into());
                }
            }
        }
    }
    
    /// Verify password by attempting to decrypt secret key
    fn verify_password(identity: &Identity, password: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let encrypted_secret_key = identity.get_secret_key_bytes()?;
        
        match Encryption::decrypt_secret_key(&encrypted_secret_key, password) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false), // Invalid password, not an error
        }
    }
    
    /// Check if any identities exist and handle verification
    pub async fn check_and_verify_identities() -> Result<AuthenticatedUser, Box<dyn std::error::Error>> {
        // Check if any identities exist
        let identities = list_identities()?;
        
        if identities.is_empty() {
            // No identities found - guide user to create one
            Self::handle_no_identities().await
        } else {
            // Identities exist - verify user
            Self::handle_identity_verification(identities).await
        }
    }
}
