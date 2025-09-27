use shared::utils;
use std::io::{self, Write};

/// get username from user input
pub fn get_username() -> Result<String, Box<dyn std::error::Error>> {
    print!("Enter your username: ");
    io::stdout().flush()?;
    
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();
    
    if !utils::is_valid_username(&username) {
        return Err("Invalid username. Use only alphanumeric characters, underscore, or dash (max 32 chars)".into());
    }

    Ok(username)
}

/// display welcome message
pub fn display_welcome(username: &str) {
    println!("Connected as: {}", username);
    println!("Type your messages and press Enter. Type '/quit' or '/exit' to disconnect.");
    println!("{}", "=".repeat(60));
}

/// display client header
pub fn display_header() {
    println!("=== Simple Chat Client ===");
}
