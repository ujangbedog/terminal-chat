/// Message routing and flooding for P2P networks
use crate::message::{P2PMessage, PeerInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, debug};
use uuid::Uuid;

/// Routing table for P2P network
#[derive(Debug, Clone)]
pub struct RoutingTable {
    /// Local peer ID
    local_peer_id: String,
    /// Connected peers
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// Message cache to prevent loops
    message_cache: Arc<RwLock<HashMap<String, u64>>>,
    /// Maximum cache size
    max_cache_size: usize,
    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

impl RoutingTable {
    /// Create a new routing table
    pub fn new(local_peer_id: String) -> Self {
        Self {
            local_peer_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size: 10000,
            cache_ttl_secs: 300, // 5 minutes
        }
    }

    /// Add a peer to the routing table
    pub async fn add_peer(&self, peer_info: PeerInfo) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_info.peer_id.clone(), peer_info.clone());
        info!("Added peer to routing table: {} ({})", peer_info.username, peer_info.peer_id);
    }

    /// Remove a peer from the routing table
    pub async fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write().await;
        if peers.remove(peer_id).is_some() {
            info!("Removed peer from routing table: {}", peer_id);
        }
    }

    /// Get all peers
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    /// Check if we have seen this message before
    pub async fn has_seen_message(&self, message_id: &str) -> bool {
        let cache = self.message_cache.read().await;
        cache.contains_key(message_id)
    }

    /// Mark a message as seen
    pub async fn mark_message_seen(&self, message_id: String) {
        let mut cache = self.message_cache.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        cache.insert(message_id, now);

        // Clean up old entries if cache is too large
        if cache.len() > self.max_cache_size {
            let cutoff_time = now - self.cache_ttl_secs;
            cache.retain(|_, &mut timestamp| timestamp > cutoff_time);
        }
    }

    /// Clean up old message cache entries
    pub async fn cleanup_message_cache(&self) {
        let mut cache = self.message_cache.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cutoff_time = now - self.cache_ttl_secs;

        let old_size = cache.len();
        cache.retain(|_, &mut timestamp| timestamp > cutoff_time);
        let new_size = cache.len();

        if old_size != new_size {
            debug!("Cleaned up message cache: {} -> {} entries", old_size, new_size);
        }
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }
}

/// Message router for handling P2P message propagation
#[derive(Clone)]
pub struct MessageRouter {
    routing_table: RoutingTable,
    local_peer_id: String,
    local_username: String,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(local_peer_id: String, local_username: String) -> Self {
        let routing_table = RoutingTable::new(local_peer_id.clone());
        
        Self {
            routing_table,
            local_peer_id,
            local_username,
        }
    }

    /// Get the routing table
    pub fn routing_table(&self) -> &RoutingTable {
        &self.routing_table
    }

    /// Process an incoming P2P message and determine routing action
    pub async fn process_message(
        &self,
        message: P2PMessage,
        from_peer_id: String,
    ) -> RoutingAction {
        match message {
            P2PMessage::ChatMessage {
                message_id,
                sender_id,
                username,
                content,
                ttl,
                mut seen_by,
            } => {
                // Check if we've seen this message before
                if self.routing_table.has_seen_message(&message_id).await {
                    debug!("Ignoring duplicate message: {}", message_id);
                    return RoutingAction::Drop;
                }

                // Check if TTL is expired
                if ttl == 0 {
                    debug!("Dropping message with expired TTL: {}", message_id);
                    return RoutingAction::Drop;
                }

                // Check if we're in the seen_by list
                if seen_by.contains(&self.local_peer_id) {
                    debug!("Ignoring message already seen by us: {}", message_id);
                    return RoutingAction::Drop;
                }

                // Mark message as seen
                self.routing_table.mark_message_seen(message_id.clone()).await;

                // Add ourselves to seen_by list
                seen_by.push(self.local_peer_id.clone());

                // Create modified message for forwarding
                let forward_message = P2PMessage::ChatMessage {
                    message_id: message_id.clone(),
                    sender_id: sender_id.clone(),
                    username: username.clone(),
                    content: content.clone(),
                    ttl: ttl - 1,
                    seen_by: seen_by.clone(),
                };

                // Determine which peers to forward to
                let peers = self.routing_table.get_peers().await;
                let forward_to: Vec<String> = peers
                    .iter()
                    .filter(|peer| {
                        peer.peer_id != from_peer_id && 
                        peer.peer_id != sender_id &&
                        !seen_by.contains(&peer.peer_id)
                    })
                    .map(|peer| peer.peer_id.clone())
                    .collect();

                RoutingAction::ForwardAndDeliver {
                    original_message: P2PMessage::ChatMessage {
                        message_id,
                        sender_id,
                        username,
                        content,
                        ttl,
                        seen_by,
                    },
                    forward_message,
                    forward_to,
                }
            }

            P2PMessage::PeerAnnounce { peer_id, listen_addr, username } => {
                // Update routing table
                let peer_info = PeerInfo {
                    peer_id: peer_id.clone(),
                    addr: listen_addr,
                    username: username.clone(),
                    last_seen: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                
                self.routing_table.add_peer(peer_info).await;
                
                RoutingAction::Deliver {
                    message: P2PMessage::PeerAnnounce { peer_id, listen_addr, username },
                }
            }

            P2PMessage::PeerListRequest { peer_id } => {
                // Respond with our peer list
                let peers = self.routing_table.get_peers().await;
                let response = P2PMessage::PeerListResponse { peers };
                
                RoutingAction::Respond {
                    to_peer: peer_id,
                    message: response,
                }
            }

            P2PMessage::PeerListResponse { peers } => {
                // Update routing table with received peers
                for peer in &peers {
                    self.routing_table.add_peer(peer.clone()).await;
                }
                
                RoutingAction::Deliver {
                    message: P2PMessage::PeerListResponse { peers },
                }
            }

            P2PMessage::Handshake { peer_id, username, protocol_version } => {
                RoutingAction::Deliver {
                    message: P2PMessage::Handshake { peer_id, username, protocol_version },
                }
            }

            P2PMessage::Heartbeat { peer_id, timestamp: _ } => {
                // Update peer's last seen time
                debug!("Received heartbeat from {}", peer_id);
                RoutingAction::UpdateHeartbeat { peer_id }
            }

            P2PMessage::Disconnect { peer_id, reason } => {
                // Remove peer from routing table
                self.routing_table.remove_peer(&peer_id).await;
                
                RoutingAction::Deliver {
                    message: P2PMessage::Disconnect { peer_id, reason },
                }
            }
        }
    }

    /// Create a new chat message for broadcasting
    pub fn create_chat_message(&self, content: String) -> P2PMessage {
        let message_id = Uuid::new_v4().to_string();
        
        P2PMessage::ChatMessage {
            message_id,
            sender_id: self.local_peer_id.clone(),
            username: self.local_username.clone(),
            content,
            ttl: 7, // Default TTL
            seen_by: vec![self.local_peer_id.clone()],
        }
    }

    /// Create a peer announcement message
    pub fn create_peer_announce(&self, listen_addr: std::net::SocketAddr) -> P2PMessage {
        P2PMessage::PeerAnnounce {
            peer_id: self.local_peer_id.clone(),
            listen_addr,
            username: self.local_username.clone(),
        }
    }

    /// Create a handshake message
    pub fn create_handshake(&self) -> P2PMessage {
        P2PMessage::Handshake {
            peer_id: self.local_peer_id.clone(),
            username: self.local_username.clone(),
            protocol_version: "1.0".to_string(),
        }
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            connected_peers: self.routing_table.peer_count().await,
            cached_messages: {
                let cache = self.routing_table.message_cache.read().await;
                cache.len()
            },
        }
    }
}

/// Actions to take after processing a message
#[derive(Debug)]
pub enum RoutingAction {
    /// Drop the message (duplicate, expired TTL, etc.)
    Drop,
    /// Deliver the message to the local application
    Deliver {
        message: P2PMessage,
    },
    /// Forward the message to other peers and deliver locally
    ForwardAndDeliver {
        original_message: P2PMessage,
        forward_message: P2PMessage,
        forward_to: Vec<String>,
    },
    /// Respond to a specific peer
    Respond {
        to_peer: String,
        message: P2PMessage,
    },
    /// Update heartbeat for a peer
    UpdateHeartbeat {
        peer_id: String,
    },
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub cached_messages: usize,
}
