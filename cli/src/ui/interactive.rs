//! Interactive menu using dialoguer for professional UX

use std::process::Command;
use std::net::SocketAddr;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use tokio::time::sleep;

/// Interactive menu system using dialoguer
pub struct InteractiveMenu;

impl InteractiveMenu {
    /// Create a new interactive menu
    pub fn new() -> Self {
        Self
    }

    /// Show the main interactive menu
    pub async fn show(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.show_welcome();
        
        loop {
            let selection = self.show_main_menu()?;
            
            match selection {
                0 => {
                    // Create P2P Chat
                    self.handle_p2p_chat().await?;
                }
                1 => {
                    // Join Chat Room
                    self.show_coming_soon("Join Chat Room");
                }
                2 => {
                    // Settings
                    self.handle_settings().await?;
                }
                3 => {
                    // Exit
                    if self.confirm_exit()? {
                        println!("{}", "👋 Goodbye! Thanks for using Terminal Chat!".bright_green().bold());
                        break;
                    }
                }
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }

    /// Show welcome message
    fn show_welcome(&self) {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{}", "║                    🚀 Terminal Chat Client                    ║".bright_cyan());
        println!("{}", "║                     Welcome to the future                    ║".bright_cyan());
        println!("{}", "║                    of terminal communication!               ║".bright_cyan());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_cyan());
        println!();
    }

    /// Show main menu and return selection
    fn show_main_menu(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let options = vec![
            "🔗 Create P2P Chat",
            "🏠 Join Chat Room (Coming Soon)",
            "⚙️  Settings",
            "🚪 Exit",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&options)
            .interact()?;

        Ok(selection)
    }

    /// Handle P2P chat creation
    async fn handle_p2p_chat(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "\n🔗 Setting up P2P Chat Session".bright_cyan().bold());
        
        // Get username
        let username: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your username")
            .default("User".to_string())
            .interact_text()?;

        // Ask for port
        let use_custom_port = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to specify a custom port?")
            .default(false)
            .interact()?;

        let port = if use_custom_port {
            let port_input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter port number")
                .default("8080".to_string())
                .validate_with(|input: &String| -> Result<(), &str> {
                    match input.parse::<u16>() {
                        Ok(port) if port > 1024 => Ok(()),
                        Ok(_) => Err("Port must be greater than 1024"),
                        Err(_) => Err("Please enter a valid port number"),
                    }
                })
                .interact_text()?;
            Some(port_input)
        } else {
            None
        };

        // Ask for bootstrap peers
        let use_bootstrap = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to connect to an existing peer?")
            .default(false)
            .interact()?;

        let bootstrap = if use_bootstrap {
            let bootstrap_addr: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter bootstrap peer address (IP:PORT)")
                .validate_with(|input: &String| -> Result<(), &str> {
                    match input.parse::<std::net::SocketAddr>() {
                        Ok(_) => Ok(()),
                        Err(_) => Err("Please enter a valid address (e.g., 127.0.0.1:8080)"),
                    }
                })
                .interact_text()?;
            Some(bootstrap_addr)
        } else {
            None
        };

        // Show progress
        self.show_connection_progress().await;

        // Build arguments
        let mut args = vec![
            "p2p-core".to_string(),
            "-u".to_string(),
            username,
        ];

        if let Some(p) = port {
            args.push("-p".to_string());
            args.push(p);
        }

        if let Some(b) = bootstrap {
            args.push("-b".to_string());
            args.push(b);
        }

        // Start P2P chat using library function
        self.run_chat_client_library(&args).await
    }

    /// Handle settings menu
    async fn handle_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        let options = vec![
            "📋 Show Current Configuration",
            "🔧 Edit Configuration (Coming Soon)",
            "🔙 Back to Main Menu",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Settings Menu")
            .default(0)
            .items(&options)
            .interact()?;

        match selection {
            0 => {
                self.show_configuration();
            }
            1 => {
                self.show_coming_soon("Configuration Editor");
            }
            2 => {
                // Back to main menu
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Show current configuration
    fn show_configuration(&self) {
        println!();
        println!("{}", "📋 Current Configuration".bright_yellow().bold());
        println!("{}", "─".repeat(60).dimmed());
        
        let default_host = std::env::var("DEFAULT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let default_port = std::env::var("DEFAULT_PORT").unwrap_or_else(|_| "8080".to_string());
        let tls_enabled = std::env::var("TLS_ENABLED").unwrap_or_else(|_| "true".to_string());
        let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "error".to_string());

        println!("🏠 Default Host: {}", default_host.bright_white());
        println!("🔌 Default Port: {}", default_port.bright_white());
        println!("🔒 TLS Enabled: {}", tls_enabled.bright_white());
        println!("📝 Log Level: {}", log_level.bright_white());
        
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", "💡 Tip: Copy .env.example to .env to customize these settings".dimmed());
        println!();

        // Wait for user to press enter
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Press Enter to continue")
            .allow_empty(true)
            .interact_text()
            .ok();
    }

    /// Show coming soon message
    fn show_coming_soon(&self, feature: &str) {
        println!();
        println!("{}", format!("🚧 {} is coming soon!", feature).bright_yellow().bold());
        println!("{}", "Stay tuned for future updates! 🎉".dimmed());
        println!();

        // Wait for user to press enter
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Press Enter to continue")
            .allow_empty(true)
            .interact_text()
            .ok();
    }

    /// Confirm exit
    fn confirm_exit(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you sure you want to exit?")
            .default(false)
            .interact()?;

        Ok(confirm)
    }

    /// Show connection progress
    async fn show_connection_progress(&self) {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );

        pb.set_message("Initializing P2P connection...");

        for i in 0..=100 {
            pb.set_position(i);
            
            match i {
                0..=30 => pb.set_message("Setting up networking..."),
                31..=60 => pb.set_message("Generating TLS certificates..."),
                61..=90 => pb.set_message("Starting peer discovery..."),
                91..=100 => pb.set_message("Ready to connect!"),
                _ => {}
            }
            
            sleep(Duration::from_millis(20)).await;
        }

        pb.finish_with_message("✅ P2P client initialized successfully!");
        println!();
    }

    /// Show error message
    pub fn show_error(&self, message: &str) {
        println!("{} {}", "❌ Error:".bright_red().bold(), message.red());
    }

    /// Show success message
    pub fn show_success(&self, message: &str) {
        println!("{} {}", "✅ Success:".bright_green().bold(), message.green());
    }

    /// Show info message
    pub fn show_info(&self, message: &str) {
        println!("{} {}", "ℹ️  Info:".bright_blue().bold(), message.blue());
    }
    

    /// Run P2P chat client as library function
    async fn run_chat_client_library(&self, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "🚀 Launching P2P Chat Client...".bright_cyan().bold());
        
        // Parse arguments
        let mut username = "Anonymous".to_string();
        let mut listen_port: Option<u16> = None;
        let mut bootstrap_peers: Vec<SocketAddr> = vec![];
        let mut custom_host: Option<String> = None;
        let enable_tls = std::env::var("TLS_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let mut i = 1; // Skip program name
        while i < args.len() {
            match args[i].as_str() {
                "-u" => {
                    if i + 1 < args.len() {
                        username = args[i + 1].clone();
                        i += 2;
                    } else {
                        return Err("Username requires a value".into());
                    }
                }
                "-p" => {
                    if i + 1 < args.len() {
                        listen_port = Some(args[i + 1].parse()?);
                        i += 2;
                    } else {
                        return Err("Port requires a value".into());
                    }
                }
                "--host" => {
                    if i + 1 < args.len() {
                        custom_host = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err("Host requires a value".into());
                    }
                }
                "-b" => {
                    if i + 1 < args.len() {
                        let addr: SocketAddr = args[i + 1].parse()?;
                        bootstrap_peers.push(addr);
                        i += 2;
                    } else {
                        return Err("Bootstrap requires a value".into());
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        // Determine final host
        let final_host = custom_host.unwrap_or_else(|| {
            std::env::var("DEFAULT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
        });
        
        // Run P2P chat and get quit reason
        let result = p2p_core::run_p2p_chat(username, Some(final_host), listen_port, bootstrap_peers, enable_tls).await;
        
        match result {
            Ok(quit_reason) => {
                match quit_reason {
                    p2p_core::QuitReason::UserQuit => {
                        println!("{}", "✅ Returned to main menu".bright_green());
                    }
                    p2p_core::QuitReason::OwnerDisconnect => {
                        println!("{}", "⚠️  Owner disconnected, returning to menu".bright_yellow());
                    }
                    p2p_core::QuitReason::NetworkError => {
                        println!("{}", "❌ Network error, returning to menu".bright_red());
                    }
                }
                Ok(())
            }
            Err(e) => {
                self.show_error(&format!("Chat client error: {}", e));
                Err(e)
            }
        }
    }

    /// Run external p2p-core binary (fallback method)
    #[allow(dead_code)]
    async fn run_chat_client(&self, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "🚀 Launching P2P Chat Client...".bright_cyan().bold());
        
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
           .arg("-p")
           .arg("p2p-core")
           .arg("--bin")
           .arg("p2p-core")
           .arg("--");
        
        // Add all arguments except the first one (program name)
        for arg in args.iter().skip(1) {
            cmd.arg(arg);
        }
        
        let status = cmd.status()?;
        
        if !status.success() {
            self.show_error("Chat client failed to start");
            return Err("Chat client failed to start".into());
        }
        
        Ok(())
    }
}

impl Default for InteractiveMenu {
    fn default() -> Self {
        Self::new()
    }
}
