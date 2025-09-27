use serde::{Deserialize, Serialize};
use std::fmt;

/// message types that can be sent between client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// client joins the chat with a username
    Join { username: String },
    /// client sends a chat message
    Chat { username: String, content: String },
    /// client leaves the chat
    Leave { username: String },
    /// server sends system message to clients
    System { content: String },
    /// server sends user list to client
    UserList { users: Vec<String> },
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Join { username } => write!(f, "*** {} joined the chat", username),
            Message::Chat { username, content } => write!(f, "{}: {}", username, content),
            Message::Leave { username } => write!(f, "*** {} left the chat", username),
            Message::System { content } => write!(f, "*** {}", content),
            Message::UserList { users } => {
                write!(f, "*** Online users: {}", users.join(", "))
            }
        }
    }
}
