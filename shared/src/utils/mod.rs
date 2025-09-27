use crate::message::Message;
use crate::config;

/// serialize a message to JSON string
pub fn serialize_message(msg: &Message) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}

/// deserialize a message from JSON string
pub fn deserialize_message(data: &str) -> Result<Message, serde_json::Error> {
    serde_json::from_str(data)
}

/// validate username
pub fn is_valid_username(username: &str) -> bool {
    !username.is_empty() 
        && username.len() <= config::MAX_USERNAME_LENGTH
        && username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// validate message content
pub fn is_valid_message_content(content: &str) -> bool {
    !content.trim().is_empty() && content.len() <= config::MAX_MESSAGE_LENGTH
}
