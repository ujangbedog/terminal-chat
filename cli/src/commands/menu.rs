//! Menu command handlers

use colored::*;
use crate::ui::InteractiveMenu;
use crate::auth::AuthSystem;

/// Handle menu command (interactive mode)
pub async fn handle_menu_command() -> Result<(), Box<dyn std::error::Error>> {
    // Interactive menu mode with authentication
    println!("{}", "🎯 Starting Terminal Chat...".bright_green().bold());
    
    // First authenticate the user
    let authenticated_user = AuthSystem::authenticate().await?;
    
    // Then show the interactive menu with authenticated user
    let mut menu = InteractiveMenu::new_with_user(authenticated_user);
    menu.show().await
}
