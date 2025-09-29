//! Main authentication system coordinator

use colored::*;
use crate::auth::types::AuthenticatedUser;
use crate::auth::verification::IdentityVerifier;

/// Authentication system
pub struct AuthSystem;

impl AuthSystem {
    /// Main authentication flow - checks for keys and verifies user
    pub async fn authenticate() -> Result<AuthenticatedUser, Box<dyn std::error::Error>> {
        // Clear screen for clean presentation
        print!("\x1B[2J\x1B[1;1H");
        
        // Show authentication header
        Self::show_auth_header();
        
        // Delegate to identity verifier
        IdentityVerifier::check_and_verify_identities().await
    }
    
    /// Show authentication header
    fn show_auth_header() {
        println!("{}", "╔══════════════════════════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{}", "║                           🔐 IDENTITY VERIFICATION                           ║".bright_cyan().bold());
        println!("{}", "║                        Post-Quantum Cryptographic Security                   ║".bright_cyan());
        println!("{}", "╚══════════════════════════════════════════════════════════════════════════════╝".bright_cyan());
        println!();
    }
}
