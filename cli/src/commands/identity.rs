//! Identity management command handlers

use colored::*;
use identity_gen;

/// Handle identity generation command
pub async fn handle_generate_key(_username: Option<String>, _expires_days: Option<i64>) -> Result<(), Box<dyn std::error::Error>> {
    show_identity_generation_welcome();
    
    // Use the identity-gen library directly
    match identity_gen::run_identity_generator().await {
        Ok(_) => {
            // Don't show success message, just exit cleanly
            // The identity-gen library will show the identity details
        }
        Err(e) => {
            show_generation_error(&e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// Handle list identities command
pub async fn handle_list_identities() -> Result<(), Box<dyn std::error::Error>> {
    // Clear screen for better presentation
    print!("\x1B[2J\x1B[1;1H");
    
    // Clean header
    println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_blue());
    println!("{}", "║                           CRYPTOGRAPHIC IDENTITIES                           ║".bright_blue().bold());
    println!("{}", "║                            Your Digital Identity Vault                       ║".bright_blue());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_blue());
    println!();
    
    match identity_gen::list_identities() {
        Ok(identities) => {
            if identities.is_empty() {
                show_no_identities_found();
            } else {
                show_identities_list(&identities).await?;
            }
        }
        Err(e) => {
            println!("{} {}", "❌ Failed to list identities:".bright_red().bold(), e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// Show welcome screen for identity generation
fn show_identity_generation_welcome() {
    // Clear screen for better presentation
    print!("\x1B[2J\x1B[1;1H");
    
    // Clean header
    println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║                    CRYSTALS-Dilithium Identity Generator                     ║".bright_cyan().bold());
    println!("{}", "║                         Post-Quantum Cryptographic Security                  ║".bright_cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
    
    println!("{}", "Starting identity generation process...".bright_cyan().bold());
    println!("{}", "─".repeat(80).dimmed());
    println!();
}

/// Show error message if generation fails
fn show_generation_error(error: &identity_gen::IdentityError) {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_red());
    println!("{}", "║                              GENERATION FAILED                              ║".bright_red().bold());
    println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_red());
    println!();
    
    println!("{} {}", "Error:".bright_red().bold(), error.to_string().bright_white());
    println!();
    
    // Troubleshooting tips
    println!("{}", "Troubleshooting Tips:".bright_yellow().bold());
    println!("  • Check if ~/.dpq-chat/ directory is writable");
    println!("  • Ensure you have sufficient disk space");
    println!("  • Try running with different username");
    println!("  • Check if identity already exists");
    println!();
    
    println!("{}", "Try again or contact support if the problem persists.".dimmed());
}

/// Show message when no identities are found
fn show_no_identities_found() {
    println!("{}", "No identities found in your vault.".bright_yellow().bold());
    println!();
    
    println!("{}", "Getting Started:".bright_cyan().bold());
    println!("  • Create your first identity: {}", "cargo run -- generate-key".cyan());
    println!("  • Specify username: {}", "cargo run -- generate-key -u <username>".cyan());
    println!("  • Set expiration: {}", "cargo run -- generate-key -u <username> -e 365".cyan());
    println!();
    
    println!("{}", "Your identity will be securely stored and ready for P2P communication!".dimmed());
}

/// Show list of identities with clean formatting
async fn show_identities_list(identities: &[(String, std::path::PathBuf)]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} {}", "Found".bright_white(), format!("{} identities", identities.len()).bright_cyan().bold());
    println!("{}", "─".repeat(80).dimmed());
    println!();
    
    for (i, (username, path)) in identities.iter().enumerate() {
        match identity_gen::load_identity(username) {
            Ok(identity) => {
                let status = if identity.is_expired() {
                    "EXPIRED".bright_red().bold()
                } else {
                    "ACTIVE".bright_green().bold()
                };
                
                println!("{}. {} [{}]", i + 1, username.bright_cyan().bold(), status);
                
                // Show details with clean formatting
                println!("   Fingerprint: {}", identity.fingerprint.bright_white());
                println!("   Algorithm: {}", identity.algorithm.bright_white());
                println!("   Created: {}", identity.created_at.format("%Y-%m-%d %H:%M UTC").to_string().bright_white());
                
                if let Some(expires) = identity.expires_at {
                    let expires_str = expires.format("%Y-%m-%d %H:%M UTC").to_string();
                    if identity.is_expired() {
                        println!("   Expired: {}", expires_str.bright_red());
                    } else {
                        println!("   Expires: {}", expires_str.bright_white());
                    }
                } else {
                    println!("   Expires: {}", "Never".bright_green());
                }
                
                println!("   File: {}", path.display().to_string().dimmed());
                
                if i < identities.len() - 1 {
                    println!();
                }
                println!();
            }
            Err(_) => {
                println!("{}. {} [CORRUPTED]", i + 1, username.bright_red().bold());
                println!("   File: {}", path.display().to_string().bright_red());
                println!();
            }
        }
    }
    
    // Show usage tips
    println!("{}", "─".repeat(80).dimmed());
    println!("{}", "Quick Actions:".bright_yellow().bold());
    println!("  • Generate new identity: {}", "cargo run -- generate-key".cyan());
    println!("  • Start P2P chat: {}", "cargo run -- p2p -u <username> -p <port>".cyan());
    
    Ok(())
}
