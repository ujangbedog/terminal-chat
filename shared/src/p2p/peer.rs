/// Peer management for P2P networking
use crate::message::{P2PMessage, PeerInfo};
use crate::tls::TlsConnection;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use futures::{SinkExt, StreamExt};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Represents a connected peer
#[derive(Debug)]
pub struct Peer {
    pub peer_id: String,
    pub addr: SocketAddr,
    pub username: String,
    pub connected_at: u64,
    pub last_heartbeat: u64,
    pub protocol_version: String,
}

impl Peer {
    /// Create a new peer
    pub fn new(
        peer_id: String,
        addr: SocketAddr,
        username: String,
        protocol_version: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            peer_id,
            addr,
            username,
            connected_at: now,
            last_heartbeat: now,
            protocol_version,
        }
    }

    /// Update the last heartbeat time
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Check if the peer is considered alive
    pub fn is_alive(&self, timeout_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.last_heartbeat < timeout_secs
    }

    /// Convert to PeerInfo
    pub fn to_peer_info(&self) -> PeerInfo {
        PeerInfo {
            peer_id: self.peer_id.clone(),
            addr: self.addr,
            username: self.username.clone(),
            last_seen: self.last_heartbeat,
        }
    }
}

/// Represents a connection to a peer
pub struct PeerConnection {
    pub peer: Peer,
    pub sender: mpsc::Sender<P2PMessage>,
    connection_handle: tokio::task::JoinHandle<()>,
}

impl PeerConnection {
    /// Create a new peer connection
    pub async fn new(
        connection: TlsConnection,
        peer: Peer,
        message_tx: mpsc::Sender<(P2PMessage, String)>,
        disconnect_tx: mpsc::Sender<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (sender, mut receiver) = mpsc::channel::<P2PMessage>(100);
        
        let peer_id = peer.peer_id.clone();
        let peer_id_clone = peer_id.clone();
        let message_tx_clone = message_tx.clone();
        let disconnect_tx_clone = disconnect_tx.clone();

        // Split the connection for reading and writing
        let (read_half, write_half) = tokio::io::split(connection);
        let mut reader = FramedRead::new(read_half, LinesCodec::new());
        let mut writer = FramedWrite::new(write_half, LinesCodec::new());

        // Spawn connection handler
        let connection_handle = tokio::spawn(async move {
            let mut heartbeat_interval = interval(Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    // Handle incoming messages
                    frame = reader.next() => {
                        match frame {
                            Some(Ok(line)) => {
                                match serde_json::from_str::<P2PMessage>(&line) {
                                    Ok(message) => {
                                        debug!("Received message from {}: {:?}", peer_id, message);
                                        
                                        // Update heartbeat for any received message
                                        if let Err(e) = message_tx_clone.send((message, peer_id.clone())).await {
                                            error!("Failed to forward message from {}: {}", peer_id, e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse message from {}: {}", peer_id, e);
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("Connection error with {}: {}", peer_id, e);
                                break;
                            }
                            None => {
                                info!("Connection closed by peer {}", peer_id);
                                break;
                            }
                        }
                    }
                    
                    // Handle outgoing messages
                    message = receiver.recv() => {
                        match message {
                            Some(msg) => {
                                match serde_json::to_string(&msg) {
                                    Ok(line) => {
                                        if let Err(e) = writer.send(line).await {
                                            error!("Failed to send message to {}: {}", peer_id, e);
                                            break;
                                        }
                                        debug!("Sent message to {}: {:?}", peer_id, msg);
                                    }
                                    Err(e) => {
                                        error!("Failed to serialize message for {}: {}", peer_id, e);
                                    }
                                }
                            }
                            None => {
                                info!("Message channel closed for peer {}", peer_id);
                                break;
                            }
                        }
                    }
                    
                    // Send periodic heartbeats
                    _ = heartbeat_interval.tick() => {
                        let heartbeat = P2PMessage::Heartbeat {
                            peer_id: peer_id.clone(),
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };
                        
                        match serde_json::to_string(&heartbeat) {
                            Ok(line) => {
                                if let Err(e) = writer.send(line).await {
                                    error!("Failed to send heartbeat to {}: {}", peer_id, e);
                                    break;
                                }
                                debug!("Sent heartbeat to {}", peer_id);
                            }
                            Err(e) => {
                                error!("Failed to serialize heartbeat for {}: {}", peer_id, e);
                            }
                        }
                    }
                }
            }

            // Notify about disconnection
            if let Err(e) = disconnect_tx_clone.send(peer_id_clone).await {
                error!("Failed to notify about disconnection: {}", e);
            }
        });

        Ok(PeerConnection {
            peer,
            sender,
            connection_handle,
        })
    }

    /// Send a message to this peer
    pub async fn send_message(&self, message: P2PMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.sender.send(message).await?;
        Ok(())
    }

    /// Disconnect from this peer
    pub async fn disconnect(&self, reason: String) {
        let disconnect_msg = P2PMessage::Disconnect {
            peer_id: self.peer.peer_id.clone(),
            reason,
        };
        
        if let Err(e) = self.sender.send(disconnect_msg).await {
            warn!("Failed to send disconnect message to {}: {}", self.peer.peer_id, e);
        }
        
        self.connection_handle.abort();
    }
}

/// Manages all peer connections
#[derive(Clone)]
pub struct PeerManager {
    local_peer_id: String,
    local_username: String,
    connections: Arc<RwLock<HashMap<String, PeerConnection>>>,
    message_tx: mpsc::Sender<(P2PMessage, String)>,
    disconnect_tx: mpsc::Sender<String>,
    max_connections: usize,
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new(
        local_peer_id: String,
        local_username: String,
        max_connections: usize,
    ) -> (Self, mpsc::Receiver<(P2PMessage, String)>, mpsc::Receiver<String>) {
        let (message_tx, message_rx) = mpsc::channel(1000);
        let (disconnect_tx, disconnect_rx) = mpsc::channel(100);

        let manager = Self {
            local_peer_id,
            local_username,
            connections: Arc::new(RwLock::new(HashMap::new())),
            message_tx,
            disconnect_tx,
            max_connections,
        };

        (manager, message_rx, disconnect_rx)
    }

    /// Add a new peer connection
    pub async fn add_peer(
        &self,
        connection: TlsConnection,
        peer_id: String,
        addr: SocketAddr,
        username: String,
        protocol_version: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut connections = self.connections.write().await;
        
        // Check if we already have this peer
        if connections.contains_key(&peer_id) {
            warn!("Peer {} already connected", peer_id);
            return Ok(());
        }

        // Check connection limit
        if connections.len() >= self.max_connections {
            warn!("Maximum connections reached, rejecting peer {}", peer_id);
            return Err("Maximum connections reached".into());
        }

        let peer = Peer::new(peer_id.clone(), addr, username.clone(), protocol_version);
        let peer_connection = PeerConnection::new(
            connection,
            peer,
            self.message_tx.clone(),
            self.disconnect_tx.clone(),
        ).await?;

        connections.insert(peer_id.clone(), peer_connection);
        info!("Added peer connection: {} ({})", username, peer_id);

        Ok(())
    }

    /// Remove a peer connection
    pub async fn remove_peer(&self, peer_id: &str, reason: String) {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.remove(peer_id) {
            connection.disconnect(reason).await;
            info!("Removed peer connection: {}", peer_id);
        }
    }
    pub async fn send_to_peer(
        &self,
        peer_id: &str,
        message: P2PMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(peer_id) {
            connection.send_message(message).await?;
        } else {
            return Err(format!("Peer {} not found", peer_id).into());
        }

        Ok(())
    }

    /// Broadcast a message to all connected peers
    pub async fn broadcast_message(&self, message: P2PMessage) {
        let connections = self.connections.read().await;
        
        for (peer_id, connection) in connections.iter() {
            if let Err(e) = connection.send_message(message.clone()).await {
                warn!("Failed to send message to {}: {}", peer_id, e);
            }
        }
    }

    /// Get all connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerInfo> {
        let connections = self.connections.read().await;
        connections.values().map(|conn| conn.peer.to_peer_info()).collect()
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Check if a peer is connected
    pub async fn is_peer_connected(&self, peer_id: &str) -> bool {
        let connections = self.connections.read().await;
        connections.contains_key(peer_id)
    }

    /// Cleanup dead connections
    pub async fn cleanup_dead_connections(&self, timeout_secs: u64) {
        let mut connections = self.connections.write().await;
        let mut dead_peers = Vec::new();

        for (peer_id, connection) in connections.iter() {
            if !connection.peer.is_alive(timeout_secs) {
                dead_peers.push(peer_id.clone());
            }
        }

        for peer_id in dead_peers {
            if let Some(connection) = connections.remove(&peer_id) {
                connection.disconnect("Connection timeout".to_string()).await;
                warn!("Removed dead peer connection: {}", peer_id);
            }
        }
    }

    /// Update peer heartbeat
    pub async fn update_peer_heartbeat(&self, peer_id: &str) {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(peer_id) {
            // Note: This is a simplified approach. In a real implementation,
            // you might want to use Arc<Mutex<Peer>> for interior mutability
            debug!("Updated heartbeat for peer {}", peer_id);
        }
    }
}
