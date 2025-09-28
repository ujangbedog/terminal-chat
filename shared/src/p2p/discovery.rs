/// Peer discovery mechanisms for P2P networking
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::time::{interval, timeout};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};

/// Discovery methods for finding peers
#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    /// Multicast discovery on local network
    Multicast {
        multicast_addr: SocketAddr,
        interface: Option<std::net::Ipv4Addr>,
    },
    /// Bootstrap from known peers
    Bootstrap {
        peers: Vec<SocketAddr>,
    },
    /// Manual peer addition
    Manual,
}

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    /// Announce presence on the network
    Announce {
        peer_id: String,
        listen_addr: SocketAddr,
        username: String,
        protocol_version: String,
        timestamp: u64,
    },
    /// Request peer list
    PeerRequest {
        peer_id: String,
        timestamp: u64,
    },
    /// Response with peer list
    PeerResponse {
        peer_id: String,
        peers: Vec<DiscoveredPeer>,
        timestamp: u64,
    },
}

/// Information about a discovered peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub addr: SocketAddr,
    pub username: String,
    pub last_seen: u64,
    pub protocol_version: String,
}

/// Peer discovery service
pub struct PeerDiscovery {
    peer_id: String,
    username: String,
    listen_addr: SocketAddr,
    discovery_methods: Vec<DiscoveryMethod>,
    discovered_peers: std::collections::HashMap<String, DiscoveredPeer>,
    protocol_version: String,
    running: std::sync::Arc<tokio::sync::RwLock<bool>>,
}

impl PeerDiscovery {
    /// Create a new peer discovery service
    pub fn new(
        peer_id: String,
        username: String,
        listen_addr: SocketAddr,
        discovery_methods: Vec<DiscoveryMethod>,
    ) -> Self {
        Self {
            peer_id,
            username,
            listen_addr,
            discovery_methods,
            discovered_peers: std::collections::HashMap::new(),
            protocol_version: "1.0".to_string(),
            running: std::sync::Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Start the discovery service
    pub async fn start(&mut self) -> Result<tokio::sync::mpsc::Receiver<DiscoveredPeer>, Box<dyn std::error::Error + Send + Sync>> {
        // Set running flag
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        for method in &self.discovery_methods {
            match method {
                DiscoveryMethod::Multicast { multicast_addr, interface } => {
                    self.start_multicast_discovery(*multicast_addr, *interface, tx.clone()).await?;
                }
                DiscoveryMethod::Bootstrap { peers } => {
                    self.start_bootstrap_discovery(peers.clone(), tx.clone()).await?;
                }
                DiscoveryMethod::Manual => {
                    info!("Manual discovery method enabled");
                }
            }
        }

        Ok(rx)
    }
    
    /// Stop the discovery service
    pub async fn stop(&mut self) {
        info!("Stopping peer discovery");
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Start multicast discovery
    async fn start_multicast_discovery(
        &self,
        multicast_addr: SocketAddr,
        _interface: Option<std::net::Ipv4Addr>,
        tx: tokio::sync::mpsc::Sender<DiscoveredPeer>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting multicast discovery on {}", multicast_addr);

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.join_multicast_v4(
            multicast_addr.ip().to_string().parse()?,
            std::net::Ipv4Addr::UNSPECIFIED,
        )?;

        let peer_id = self.peer_id.clone();
        let username = self.username.clone();
        let listen_addr = self.listen_addr;
        let protocol_version = self.protocol_version.clone();
        let running = self.running.clone();

        // Spawn announcement task
        let announce_socket = UdpSocket::bind("0.0.0.0:0").await?;
        announce_socket.join_multicast_v4(
            multicast_addr.ip().to_string().parse()?,
            std::net::Ipv4Addr::UNSPECIFIED,
        )?;
        let peer_id_announce = peer_id.clone();
        let running_announce = running.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            while *running_announce.read().await {
                interval.tick().await;
                
                let announce_msg = DiscoveryMessage::Announce {
                    peer_id: peer_id_announce.clone(),
                    listen_addr,
                    username: username.clone(),
                    protocol_version: protocol_version.clone(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                if let Ok(data) = serde_json::to_vec(&announce_msg) {
                    if let Err(e) = announce_socket.send_to(&data, multicast_addr).await {
                        warn!("Failed to send multicast announcement: {}", e);
                    } else {
                        debug!("Sent multicast announcement");
                    }
                }
            }
        });

        // Spawn listener task
        let listen_socket = socket;
        let tx_clone = tx.clone();
        let running_listen = running.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while *running_listen.read().await {
                match listen_socket.recv_from(&mut buf).await {
                    Ok((len, from_addr)) => {
                        if let Ok(msg) = serde_json::from_slice::<DiscoveryMessage>(&buf[..len]) {
                            match msg {
                                DiscoveryMessage::Announce {
                                    peer_id: remote_peer_id,
                                    listen_addr: remote_listen_addr,
                                    username: remote_username,
                                    protocol_version: remote_protocol_version,
                                    timestamp,
                                } => {
                                    if remote_peer_id != peer_id {
                                        let discovered_peer = DiscoveredPeer {
                                            peer_id: remote_peer_id,
                                            addr: remote_listen_addr,
                                            username: remote_username,
                                            last_seen: timestamp,
                                            protocol_version: remote_protocol_version,
                                        };

                                        debug!("Discovered peer via multicast: {:?}", discovered_peer);
                                        if let Err(e) = tx_clone.send(discovered_peer).await {
                                            warn!("Failed to send discovered peer: {}", e);
                                        }
                                    }
                                }
                                _ => {
                                    debug!("Received other discovery message from {}", from_addr);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to receive multicast message: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Start bootstrap discovery
    async fn start_bootstrap_discovery(
        &self,
        bootstrap_peers: Vec<SocketAddr>,
        tx: tokio::sync::mpsc::Sender<DiscoveredPeer>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting bootstrap discovery with {} peers", bootstrap_peers.len());

        let peer_id = self.peer_id.clone();
        let username = self.username.clone();
        let protocol_version = self.protocol_version.clone();

        for bootstrap_addr in bootstrap_peers {
            let tx_clone = tx.clone();
            let peer_id_clone = peer_id.clone();
            let username_clone = username.clone();
            let protocol_version_clone = protocol_version.clone();

            tokio::spawn(async move {
                // Try to connect to bootstrap peer and request peer list
                match Self::query_bootstrap_peer(bootstrap_addr, peer_id_clone, username_clone, protocol_version_clone).await {
                    Ok(peers) => {
                        for peer in peers {
                            if let Err(e) = tx_clone.send(peer).await {
                                warn!("Failed to send bootstrap discovered peer: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to query bootstrap peer {}: {}", bootstrap_addr, e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Query a bootstrap peer for its peer list
    async fn query_bootstrap_peer(
        addr: SocketAddr,
        peer_id: String,
        _username: String,
        _protocol_version: String,
    ) -> Result<Vec<DiscoveredPeer>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Querying bootstrap peer: {}", addr);

        // For now, we'll implement a simple UDP-based query
        // In a real implementation, you might want to use the actual P2P protocol
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        let request = DiscoveryMessage::PeerRequest {
            peer_id,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let data = serde_json::to_vec(&request)?;
        socket.send_to(&data, addr).await?;

        // Wait for response with timeout
        let mut buf = [0u8; 4096];
        match timeout(Duration::from_secs(5), socket.recv_from(&mut buf)).await {
            Ok(Ok((len, _))) => {
                if let Ok(DiscoveryMessage::PeerResponse { peers, .. }) = 
                    serde_json::from_slice::<DiscoveryMessage>(&buf[..len]) {
                    debug!("Received {} peers from bootstrap peer {}", peers.len(), addr);
                    Ok(peers)
                } else {
                    Ok(vec![])
                }
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => {
                warn!("Timeout querying bootstrap peer {}", addr);
                Ok(vec![])
            }
        }
    }

    /// Add a peer manually
    pub fn add_manual_peer(&mut self, peer: DiscoveredPeer) {
        info!("Manually adding peer: {} at {}", peer.username, peer.addr);
        self.discovered_peers.insert(peer.peer_id.clone(), peer);
    }

    /// Get all discovered peers
    pub fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers.values().cloned().collect()
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &str) {
        if self.discovered_peers.remove(peer_id).is_some() {
            info!("Removed peer: {}", peer_id);
        }
    }

    /// Clean up old peers
    pub fn cleanup_old_peers(&mut self, max_age_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let old_peers: Vec<String> = self
            .discovered_peers
            .iter()
            .filter(|(_, peer)| now - peer.last_seen > max_age_secs)
            .map(|(id, _)| id.clone())
            .collect();

        for peer_id in old_peers {
            self.discovered_peers.remove(&peer_id);
            debug!("Removed old peer: {}", peer_id);
        }
    }
}

/// Default multicast address for P2P discovery
pub const DEFAULT_MULTICAST_ADDR: &str = "239.255.42.99:8899";

/// Create default discovery methods
pub fn default_discovery_methods() -> Vec<DiscoveryMethod> {
    vec![
        DiscoveryMethod::Multicast {
            multicast_addr: DEFAULT_MULTICAST_ADDR.parse().unwrap(),
            interface: None,
        },
        DiscoveryMethod::Manual,
    ]
}
