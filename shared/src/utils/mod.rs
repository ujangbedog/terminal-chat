use crate::config;

/// validate username for P2P chat
pub fn is_valid_username(username: &str) -> bool {
    !username.is_empty() 
        && username.len() <= config::MAX_USERNAME_LENGTH
        && username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// validate message content for P2P chat
pub fn is_valid_message_content(content: &str) -> bool {
    !content.trim().is_empty() && content.len() <= config::MAX_MESSAGE_LENGTH
}
