/// P2P networking module for peer-to-peer chat
pub mod node;
pub mod peer;
pub mod discovery;
pub mod routing;

// Re-export main types for convenience
pub use node::{P2PNode, P2PNodeConfig};
pub use peer::{Peer, PeerConnection, PeerManager};
pub use discovery::{PeerDiscovery, DiscoveryMethod};
pub use routing::{MessageRouter, RoutingTable};

use crate::message::{P2PMessage, PeerInfo};
use std::net::SocketAddr;

/// P2P network events
#[derive(Debug, Clone)]
pub enum P2PEvent {
    /// A new peer connected
    PeerConnected {
        peer_id: String,
        addr: SocketAddr,
        username: String,
    },
    /// A peer disconnected
    PeerDisconnected {
        peer_id: String,
        reason: String,
    },
    /// Received a chat message
    MessageReceived {
        message: P2PMessage,
        from_peer: String,
    },
    /// Network topology changed
    TopologyChanged {
        connected_peers: Vec<PeerInfo>,
    },
    /// Discovery found new peers
    PeersDiscovered {
        peers: Vec<SocketAddr>,
    },
    /// Error occurred
    Error {
        error: String,
        peer_id: Option<String>,
    },
}

/// P2P network statistics
#[derive(Debug, Clone)]
pub struct P2PStats {
    pub connected_peers: usize,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub uptime_secs: u64,
    pub discovery_attempts: u64,
    pub successful_connections: u64,
    pub failed_connections: u64,
}

impl Default for P2PStats {
    fn default() -> Self {
        Self {
            connected_peers: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            uptime_secs: 0,
            discovery_attempts: 0,
            successful_connections: 0,
            failed_connections: 0,
        }
    }
}
