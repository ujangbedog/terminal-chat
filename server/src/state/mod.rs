use crate::client::{ClientInfo, Tx};
use shared::{utils, Message};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::{error, info, warn};
use uuid::Uuid;

/// shared state between all connected clients
#[derive(Debug)]
pub struct SharedState {
    /// map of client ID to their message sender
    clients: HashMap<Uuid, ClientInfo>,
    /// map of usernames to client IDs for quick lookup
    usernames: HashMap<String, Uuid>,
}

impl SharedState {
    /// create new shared state
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            usernames: HashMap::new(),
        }
    }

    /// add a new client to the shared state
    pub fn add_client(&mut self, client_id: Uuid, sender: Tx, addr: SocketAddr) {
        let client_info = ClientInfo::new(sender, addr);
        self.clients.insert(client_id, client_info);
        info!("Client {} connected from {}", client_id, addr);
    }

    /// remove a client from the shared state
    pub fn remove_client(&mut self, client_id: &Uuid) -> Option<String> {
        if let Some(client_info) = self.clients.remove(client_id) {
            if let Some(username) = &client_info.username {
                self.usernames.remove(username);
                info!("Client {} ({}) disconnected", client_id, username);
                return Some(username.clone());
            }
            info!("Client {} disconnected", client_id);
        }
        None
    }

    /// set username for a client
    pub fn set_username(&mut self, client_id: &Uuid, username: String) -> Result<(), String> {
        // check if username is already taken
        if self.usernames.contains_key(&username) {
            return Err("Username already taken".to_string());
        }

        if let Some(client_info) = self.clients.get_mut(client_id) {
            client_info.set_username(username.clone());
            self.usernames.insert(username, *client_id);
            Ok(())
        } else {
            Err("Client not found".to_string())
        }
    }

    /// broadcast message to all clients except the sender
    pub async fn broadcast(&self, sender_id: Option<Uuid>, message: &Message) {
        let message_str = match utils::serialize_message(message) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to serialize message: {}", e);
                return;
            }
        };

        for (client_id, client_info) in &self.clients {
            // skip sender if specified
            if let Some(sender) = sender_id {
                if *client_id == sender {
                    continue;
                }
            }

            if let Err(e) = client_info.sender.send(message_str.clone()) {
                warn!("Failed to send message to client {}: {}", client_id, e);
            }
        }
    }

    /// get list of all connected usernames
    pub fn get_usernames(&self) -> Vec<String> {
        self.clients
            .values()
            .filter_map(|client| client.username.clone())
            .collect()
    }

    /// get client info by id
    pub fn get_client(&self, client_id: &Uuid) -> Option<&ClientInfo> {
        self.clients.get(client_id)
    }
}
