use clap::{Parser, Subcommand};
use dialoguer::{Input, Password, Confirm, Select};
use colored::*;
use chrono::{Utc, Duration};
use std::path::PathBuf;

use crate::identity::Identity;
use crate::crypto::{KeyPair, Encryption};
use crate::file_manager::FileManager;
use crate::error::{IdentityError, Result};

#[derive(Parser)]
#[command(name = "identity-gen")]
#[command(about = "CRYSTALS-Dilithium Identity Generator for DPQ Chat")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new cryptographic identity
    Generate {
        /// Username for the identity
        #[arg(short, long)]
        username: Option<String>,
        
        /// Custom output path for the identity file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Expiration time in days (optional)
        #[arg(short, long)]
        expires_days: Option<i64>,
        
        /// Skip interactive prompts
        #[arg(long)]
        non_interactive: bool,
    },
    
    /// List existing identities
    List,
    
    /// Show identity information
    Info {
        /// Username to show info for
        username: String,
    },
    
    /// Verify identity file integrity
    Verify {
        /// Path to identity file
        file: PathBuf,
    },
    
    /// Delete an identity
    Delete {
        /// Username to delete
        username: String,
    },
}

pub struct CliHandler;

impl CliHandler {
    pub fn run(cli: Cli) -> Result<()> {
        match cli.command {
            Some(Commands::Generate { username, output, expires_days, non_interactive }) => {
                Self::generate_identity(username, output, expires_days, non_interactive)
            },
            Some(Commands::List) => Self::list_identities(),
            Some(Commands::Info { username }) => Self::show_identity_info(&username),
            Some(Commands::Verify { file }) => Self::verify_identity(&file),
            Some(Commands::Delete { username }) => Self::delete_identity(&username),
            None => Self::interactive_mode(),
        }
    }
    
    pub fn interactive_mode() -> Result<()> {
        println!("{}", "🔐 CRYSTALS-Dilithium Identity Generator".cyan().bold());
        println!("{}", "Post-Quantum Cryptographic Identity Management".dimmed());
        println!();
        
        let options = vec![
            "Generate new identity",
            "List existing identities", 
            "Show identity info",
            "Verify identity file",
            "Delete identity",
            "Exit",
        ];
        
        loop {
            let selection = Select::new()
                .with_prompt("What would you like to do?")
                .items(&options)
                .default(0)
                .interact()
                .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
            
            match selection {
                0 => Self::generate_identity(None, None, None, false)?,
                1 => Self::list_identities()?,
                2 => {
                    let username: String = Input::new()
                        .with_prompt("Username")
                        .interact_text()
                        .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
                    Self::show_identity_info(&username)?;
                },
                3 => {
                    let file_path: String = Input::new()
                        .with_prompt("Identity file path")
                        .interact_text()
                        .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
                    Self::verify_identity(&PathBuf::from(file_path))?;
                },
                4 => {
                    let username: String = Input::new()
                        .with_prompt("Username to delete")
                        .interact_text()
                        .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
                    Self::delete_identity(&username)?;
                },
                5 => {
                    println!("{}", "Goodbye! 👋".green());
                    break;
                },
                _ => unreachable!(),
            }
            
            println!();
        }
        
        Ok(())
    }
    
    fn generate_identity(
        username: Option<String>,
        output_path: Option<PathBuf>,
        expires_days: Option<i64>,
        non_interactive: bool,
    ) -> Result<()> {
        println!("{}", "🔑 Generating new CRYSTALS-Dilithium identity...".cyan().bold());
        println!();
        
        // Get username
        let username = if let Some(name) = username {
            name
        } else if non_interactive {
            return Err(IdentityError::InvalidInput("Username required in non-interactive mode".to_string()));
        } else {
            Input::new()
                .with_prompt("Username (display name)")
                .validate_with(|input: &String| -> std::result::Result<(), &str> {
                    if input.trim().is_empty() {
                        Err("Username cannot be empty")
                    } else if input.len() > 50 {
                        Err("Username too long (max 50 characters)")
                    } else {
                        Ok(())
                    }
                })
                .interact_text()
                .map_err(|e| IdentityError::InvalidInput(e.to_string()))?
        };
        
        // Check if identity already exists
        if FileManager::identity_exists(&username)? {
            if non_interactive {
                return Err(IdentityError::InvalidInput(format!("Identity already exists: {}", username)));
            }
            
            let overwrite = Confirm::new()
                .with_prompt(format!("Identity '{}' already exists. Overwrite?", username))
                .default(false)
                .interact()
                .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
            
            if !overwrite {
                println!("{}", "Operation cancelled.".yellow());
                return Ok(());
            }
            
            // Delete existing identity
            FileManager::delete_identity(&username)?;
        }
        
        // Get password for private key encryption
        let password = if non_interactive {
            return Err(IdentityError::InvalidInput("Password required in non-interactive mode".to_string()));
        } else {
            Password::new()
                .with_prompt("Password to encrypt private key")
                .with_confirmation("Confirm password", "Passwords don't match")
                .validate_with(|input: &String| -> std::result::Result<(), &str> {
                    if input.len() < 8 {
                        Err("Password must be at least 8 characters")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .map_err(|e| IdentityError::InvalidInput(e.to_string()))?
        };
        
        // Calculate expiration date
        let expires_at = if let Some(days) = expires_days {
            Some(Utc::now() + Duration::days(days))
        } else if !non_interactive {
            let add_expiry = Confirm::new()
                .with_prompt("Add expiration date?")
                .default(false)
                .interact()
                .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
            
            if add_expiry {
                let days: i64 = Input::new()
                    .with_prompt("Expiration in days")
                    .default(365)
                    .interact()
                    .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
                Some(Utc::now() + Duration::days(days))
            } else {
                None
            }
        } else {
            None
        };
        
        // Generate key pair
        println!("{}", "⚡ Generating CRYSTALS-Dilithium key pair...".yellow());
        let keypair = KeyPair::generate()
            .map_err(|e| IdentityError::KeyGeneration(e.to_string()))?;
        
        // Encrypt private key
        println!("{}", "🔒 Encrypting private key...".yellow());
        let encrypted_secret_key = Encryption::encrypt_secret_key(
            keypair.secret_key_bytes(),
            &password
        )?;
        
        // Create identity
        let identity = Identity::new(
            username.clone(),
            "dilithium2".to_string(),
            keypair.public_key_bytes(),
            &encrypted_secret_key,
            expires_at,
        )?;
        
        // Save identity
        let file_path = FileManager::save_identity(&identity, output_path.as_deref())?;
        
        // Export public and private key files
        let identities_dir = FileManager::get_identities_dir()?;
        let pub_key_path = identities_dir.join(format!("{}.pub", username));
        let priv_key_path = identities_dir.join(format!("{}.key", username));
        
        // Save public key in PEM format
        use base64::{Engine as _, engine::general_purpose};
        let pub_key_b64 = general_purpose::STANDARD.encode(&keypair.public_key_bytes());
        let pub_key_pem = format!(
            "-----BEGIN DILITHIUM2 PUBLIC KEY-----\n{}\n-----END DILITHIUM2 PUBLIC KEY-----\n",
            pub_key_b64
        );
        std::fs::write(&pub_key_path, pub_key_pem)?;
        
        // Save encrypted private key (base64 encoded for readability)
        let priv_key_b64 = general_purpose::STANDARD.encode(&encrypted_secret_key);
        std::fs::write(&priv_key_path, priv_key_b64)?;
        
        // Set file permissions (read/write for owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            // Set permissions for public key file
            let mut pub_perms = std::fs::metadata(&pub_key_path)?.permissions();
            pub_perms.set_mode(0o644); // rw-r--r--
            std::fs::set_permissions(&pub_key_path, pub_perms)?;
            
            // Set permissions for private key file (more restrictive)
            let mut priv_perms = std::fs::metadata(&priv_key_path)?.permissions();
            priv_perms.set_mode(0o600); // rw-------
            std::fs::set_permissions(&priv_key_path, priv_perms)?;
        }
        
        println!("{}", "✓ Public key exported to:".green());
        println!("  {}", pub_key_path.display().to_string().cyan());
        println!("{}", "✓ Private key exported to:".green());
        println!("  {}", priv_key_path.display().to_string().cyan());
        
        // Display results
        println!();
        println!("{}", "✅ Identity generated successfully!".green().bold());
        println!();
        println!("{}: {}", "Username".bold(), identity.username.cyan());
        println!("{}: {}", "Algorithm".bold(), identity.algorithm.cyan());
        println!("{}: {}", "Fingerprint".bold(), identity.fingerprint.cyan());
        println!("{}: {}", "Short Fingerprint".bold(), identity.short_fingerprint().cyan());
        println!("{}: {}", "Created".bold(), identity.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string().cyan());
        
        if let Some(expires) = identity.expires_at {
            println!("{}: {}", "Expires".bold(), expires.format("%Y-%m-%d %H:%M:%S UTC").to_string().cyan());
        } else {
            println!("{}: {}", "Expires".bold(), "Never".cyan());
        }
        
        println!("{}: {}", "File".bold(), file_path.display().to_string().cyan());
        
        // Exit the program after successful generation
        std::process::exit(0);
    }
    
    fn list_identities() -> Result<()> {
        println!("{}", "📋 Existing Identities".cyan().bold());
        println!();
        
        let identities = FileManager::list_identities()?;
        
        if identities.is_empty() {
            println!("{}", "No identities found.".dimmed());
            println!("Use 'generate' command to create a new identity.");
            return Ok(());
        }
        
        for (username, path) in identities {
            match FileManager::load_identity(&path) {
                Ok(identity) => {
                    let status = if identity.is_expired() {
                        "EXPIRED".red()
                    } else {
                        "ACTIVE".green()
                    };
                    
                    println!("👤 {} [{}]", username.cyan().bold(), status);
                    println!("   Fingerprint: {}", identity.fingerprint.dimmed());
                    println!("   Created: {}", identity.created_at.format("%Y-%m-%d").to_string().dimmed());
                    if let Some(expires) = identity.expires_at {
                        println!("   Expires: {}", expires.format("%Y-%m-%d").to_string().dimmed());
                    }
                    println!();
                },
                Err(_) => {
                    println!("❌ {} [CORRUPTED]", username.red());
                    println!();
                }
            }
        }
        
        Ok(())
    }
    
    fn show_identity_info(username: &str) -> Result<()> {
        let identity_dir = FileManager::get_identity_dir()?;
        let filename = FileManager::get_identity_filename(username);
        let file_path = identity_dir.join(filename);
        
        let identity = FileManager::load_identity(&file_path)?;
        
        println!("{}", format!("🔍 Identity Information: {}", username).cyan().bold());
        println!();
        println!("{}: {}", "Username".bold(), identity.username.cyan());
        println!("{}: {}", "Algorithm".bold(), identity.algorithm.cyan());
        println!("{}: {}", "Fingerprint".bold(), identity.fingerprint.cyan());
        println!("{}: {}", "Short Fingerprint".bold(), identity.short_fingerprint().cyan());
        println!("{}: {}", "Created".bold(), identity.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string().cyan());
        
        if let Some(expires) = identity.expires_at {
            let status = if identity.is_expired() {
                "EXPIRED".red()
            } else {
                "ACTIVE".green()
            };
            println!("{}: {} [{}]", "Expires".bold(), expires.format("%Y-%m-%d %H:%M:%S UTC").to_string().cyan(), status);
        } else {
            println!("{}: {} [{}]", "Expires".bold(), "Never".cyan(), "ACTIVE".green());
        }
        
        println!("{}: {}", "File".bold(), file_path.display().to_string().cyan());
        
        Ok(())
    }
    
    fn verify_identity(file_path: &PathBuf) -> Result<()> {
        println!("{}", "🔍 Verifying identity file...".cyan().bold());
        
        let identity = FileManager::load_identity(file_path)?;
        
        // Verify public key fingerprint
        let public_key_bytes = identity.get_public_key_bytes()?;
        let calculated_fingerprint = Identity::generate_fingerprint(&public_key_bytes)?;
        
        if calculated_fingerprint == identity.fingerprint {
            println!("{} Identity file is valid", "✅".green());
            println!("   Username: {}", identity.username.cyan());
            println!("   Fingerprint: {}", identity.fingerprint.cyan());
            
            if identity.is_expired() {
                println!("{} Identity has expired", "⚠️".yellow());
            }
        } else {
            println!("{} Identity file is corrupted (fingerprint mismatch)", "❌".red());
            println!("   Expected: {}", identity.fingerprint.red());
            println!("   Calculated: {}", calculated_fingerprint.red());
        }
        
        Ok(())
    }
    
    fn delete_identity(username: &str) -> Result<()> {
        if !FileManager::identity_exists(username)? {
            return Err(IdentityError::InvalidInput(format!("Identity not found: {}", username)));
        }
        
        let confirm = Confirm::new()
            .with_prompt(format!("Are you sure you want to delete identity '{}'?", username))
            .default(false)
            .interact()
            .map_err(|e| IdentityError::InvalidInput(e.to_string()))?;
        
        if confirm {
            FileManager::delete_identity(username)?;
        } else {
            println!("{}", "Operation cancelled.".yellow());
        }
        
        Ok(())
    }
}
