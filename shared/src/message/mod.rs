use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;

/// P2P specific message types for peer-to-peer networking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    /// Peer discovery - announce presence to network
    PeerAnnounce {
        peer_id: String,
        listen_addr: SocketAddr,
        username: String,
    },
    /// Request peer list from another peer
    PeerListRequest {
        peer_id: String,
    },
    /// Response with known peers
    PeerListResponse {
        peers: Vec<PeerInfo>,
    },
    /// Chat message with routing information
    ChatMessage {
        message_id: String,
        sender_id: String,
        username: String,
        content: String,
        ttl: u8, // Time to live for message flooding
        seen_by: Vec<String>, // Peers that have already seen this message
    },
    /// Peer connection handshake
    Handshake {
        peer_id: String,
        username: String,
        protocol_version: String,
    },
    /// Heartbeat to maintain connection
    Heartbeat {
        peer_id: String,
        timestamp: u64,
    },
    /// Graceful disconnect notification
    Disconnect {
        peer_id: String,
        reason: String,
    },
}

/// Information about a peer in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub addr: SocketAddr,
    pub username: String,
    pub last_seen: u64,
}


impl fmt::Display for P2PMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            P2PMessage::PeerAnnounce { peer_id, listen_addr, username } => {
                write!(f, "*** Peer {} ({}) announced at {}", username, peer_id, listen_addr)
            }
            P2PMessage::PeerListRequest { peer_id } => {
                write!(f, "*** Peer list requested by {}", peer_id)
            }
            P2PMessage::PeerListResponse { peers } => {
                write!(f, "*** Peer list response with {} peers", peers.len())
            }
            P2PMessage::ChatMessage { username, content, .. } => {
                write!(f, "{}: {}", username, content)
            }
            P2PMessage::Handshake { peer_id, username, protocol_version } => {
                write!(f, "*** Handshake from {} ({}) using protocol {}", username, peer_id, protocol_version)
            }
            P2PMessage::Heartbeat { peer_id, .. } => {
                write!(f, "*** Heartbeat from {}", peer_id)
            }
            P2PMessage::Disconnect { peer_id, reason } => {
                write!(f, "*** Peer {} disconnected: {}", peer_id, reason)
            }
        }
    }
}
