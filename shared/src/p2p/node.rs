/// Main P2P node implementation
use crate::message::{P2PMessage, PeerInfo};
use crate::tls::{TlsContext, CertificateManager, TlsListener, TlsConnection};
use crate::p2p::{
    peer::PeerManager,
    discovery::{PeerDiscovery, DiscoveryMethod},
    routing::MessageRouter,
    P2PEvent, P2PStats,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Configuration for P2P node
#[derive(Debug, Clone)]
pub struct P2PNodeConfig {
    /// Local listening address
    pub listen_addr: SocketAddr,
    /// Username for this node
    pub username: String,
    /// Enable TLS
    pub enable_tls: bool,
    /// Maximum number of connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Discovery methods
    pub discovery_methods: Vec<DiscoveryMethod>,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<SocketAddr>,
}

impl Default for P2PNodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            username: "Anonymous".to_string(),
            enable_tls: true,
            max_connections: 50,
            connection_timeout_secs: 30,
            heartbeat_interval_secs: 30,
            discovery_methods: crate::p2p::discovery::default_discovery_methods(),
            bootstrap_peers: vec![],
        }
    }
}

/// Main P2P node
pub struct P2PNode {
    /// Node configuration
    config: P2PNodeConfig,
    /// Unique peer ID
    peer_id: String,
    /// TLS context
    tls_context: Option<TlsContext>,
    /// Peer manager
    peer_manager: PeerManager,
    /// Message router
    message_router: MessageRouter,
    /// Peer discovery
    peer_discovery: PeerDiscovery,
    /// Event sender
    event_tx: mpsc::Sender<P2PEvent>,
    /// Statistics
    stats: Arc<RwLock<P2PStats>>,
    /// Running flag
    running: Arc<RwLock<bool>>,
    /// Actual listening address
    actual_listen_addr: Arc<RwLock<Option<SocketAddr>>>,
    /// Message receiver
    message_rx: Option<mpsc::Receiver<(P2PMessage, String)>>,
    /// Disconnect receiver
    disconnect_rx: Option<mpsc::Receiver<String>>,
}

impl P2PNode {
    /// Create a new P2P node
    pub async fn new(
        config: P2PNodeConfig,
    ) -> Result<(Self, mpsc::Receiver<P2PEvent>), Box<dyn std::error::Error + Send + Sync>> {
        let peer_id = Uuid::new_v4().to_string();
        let (event_tx, event_rx) = mpsc::channel(1000);

        // Initialize TLS if enabled
        let tls_context = if config.enable_tls {
            let mut cert_manager = CertificateManager::new(peer_id.clone());
            cert_manager.generate_self_signed_cert().await?;
            Some(TlsContext::new(&cert_manager).await?)
        } else {
            None
        };

        // Create peer manager
        let (peer_manager, message_rx, disconnect_rx) = PeerManager::new(
            peer_id.clone(),
            config.username.clone(),
            config.max_connections,
        );

        // Create message router
        let message_router = MessageRouter::new(peer_id.clone(), config.username.clone());

        // Create peer discovery
        let peer_discovery = PeerDiscovery::new(
            peer_id.clone(),
            config.username.clone(),
            config.listen_addr,
            config.discovery_methods.clone(),
        );

        let node = Self {
            config,
            peer_id,
            tls_context,
            peer_manager,
            message_router,
            peer_discovery,
            event_tx,
            stats: Arc::new(RwLock::new(P2PStats::default())),
            running: Arc::new(RwLock::new(false)),
            actual_listen_addr: Arc::new(RwLock::new(None)),
            message_rx: Some(message_rx),
            disconnect_rx: Some(disconnect_rx),
        };

        Ok((node, event_rx))
    }

    /// Start the P2P node
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting P2P node {} with username: {}", self.peer_id, self.config.username);

        // Set running flag
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Start listening for incoming connections
        self.start_listener().await?;

        // Start peer discovery
        self.start_discovery().await?;

        // Start message processing
        if let (Some(message_rx), Some(disconnect_rx)) = (self.message_rx.take(), self.disconnect_rx.take()) {
            self.start_message_processing(message_rx, disconnect_rx).await;
        }

        // Start background tasks
        self.start_background_tasks().await;

        // Connect to bootstrap peers
        self.connect_to_bootstrap_peers().await;

        info!("P2P node started successfully");
        Ok(())
    }

    /// Stop the P2P node
    pub async fn stop(&mut self) {
        info!("Stopping P2P node {}", self.peer_id);

        // Set running flag to false
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Send disconnect messages to all peers
        let disconnect_msg = P2PMessage::Disconnect {
            peer_id: self.peer_id.clone(),
            reason: "Node shutting down".to_string(),
        };
        
        self.peer_manager.broadcast_message(disconnect_msg).await;

        info!("P2P node stopped");
    }

    /// Send a chat message to the network
    pub async fn send_chat_message(&self, content: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = self.message_router.create_chat_message(content);
        self.peer_manager.broadcast_message(message).await;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_messages_sent += 1;
        }

        Ok(())
    }

    /// Get current network statistics
    pub async fn get_stats(&self) -> P2PStats {
        let stats = self.stats.read().await;
        let mut current_stats = stats.clone();
        current_stats.connected_peers = self.peer_manager.connection_count().await;
        current_stats
    }

    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerInfo> {
        self.peer_manager.get_connected_peers().await
    }

    /// Start listening for incoming connections
    async fn start_listener(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = if let Some(tls_context) = &self.tls_context {
            TlsListener::bind_tls(self.config.listen_addr, tls_context.server_config.clone()).await?
        } else {
            TlsListener::bind_plain(self.config.listen_addr).await?
        };

        let actual_addr = listener.local_addr()?;
        info!("Listening for connections on {}", actual_addr);
        
        // Store the actual listening address
        {
            let mut addr_lock = self.actual_listen_addr.write().await;
            *addr_lock = Some(actual_addr);
        }

        let peer_manager = self.peer_manager.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((connection, peer_addr)) => {
                        info!("Accepted connection from {}", peer_addr);
                        
                        // Handle the connection in a separate task
                        let peer_manager_clone = peer_manager.clone();
                        let event_tx_clone = event_tx.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_incoming_connection(
                                connection,
                                peer_addr,
                                peer_manager_clone,
                                event_tx_clone,
                            ).await {
                                error!("Failed to handle incoming connection from {}: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle an incoming connection
    async fn handle_incoming_connection(
        connection: TlsConnection,
        peer_addr: SocketAddr,
        peer_manager: PeerManager,
        event_tx: mpsc::Sender<P2PEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For now, we'll create a temporary peer ID
        // In a real implementation, you'd perform a handshake to get the actual peer ID
        let temp_peer_id = Uuid::new_v4().to_string();
        let temp_username = format!("Peer@{}", peer_addr);

        peer_manager.add_peer(
            connection,
            temp_peer_id.clone(),
            peer_addr,
            temp_username.clone(),
            "1.0".to_string(),
        ).await?;

        // Send peer connected event
        let event = P2PEvent::PeerConnected {
            peer_id: temp_peer_id,
            addr: peer_addr,
            username: temp_username,
        };

        if let Err(e) = event_tx.send(event).await {
            warn!("Failed to send peer connected event: {}", e);
        }

        Ok(())
    }

    /// Start peer discovery
    async fn start_discovery(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut discovery_rx = self.peer_discovery.start().await?;
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().await {
                match discovery_rx.recv().await {
                    Some(discovered_peer) => {
                        debug!("Discovered peer: {:?}", discovered_peer);
                        
                        let event = P2PEvent::PeersDiscovered {
                            peers: vec![discovered_peer.addr],
                        };

                        if let Err(e) = event_tx.send(event).await {
                            warn!("Failed to send peers discovered event: {}", e);
                        }
                    }
                    None => {
                        debug!("Discovery channel closed");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Start message processing
    async fn start_message_processing(
        &self,
        mut message_rx: mpsc::Receiver<(P2PMessage, String)>,
        mut disconnect_rx: mpsc::Receiver<String>,
    ) {
        let message_router = self.message_router.clone();
        let peer_manager = self.peer_manager.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().await {
                tokio::select! {
                    // Handle incoming messages
                    message = message_rx.recv() => {
                        if let Some((p2p_message, from_peer)) = message {
                            match message_router.process_message(p2p_message.clone(), from_peer.clone()).await {
                                crate::p2p::routing::RoutingAction::Drop => {
                                    debug!("Dropped message from {}", from_peer);
                                }
                                crate::p2p::routing::RoutingAction::Deliver { message } => {
                                    let event = P2PEvent::MessageReceived {
                                        message,
                                        from_peer,
                                    };
                                    if let Err(e) = event_tx.send(event).await {
                                        warn!("Failed to send message received event: {}", e);
                                    }
                                }
                                crate::p2p::routing::RoutingAction::ForwardAndDeliver { original_message, forward_message, forward_to } => {
                                    // Deliver locally
                                    let event = P2PEvent::MessageReceived {
                                        message: original_message,
                                        from_peer: from_peer.clone(),
                                    };
                                    if let Err(e) = event_tx.send(event).await {
                                        warn!("Failed to send message received event: {}", e);
                                    }

                                    // Forward to other peers
                                    for peer_id in forward_to {
                                        if let Err(e) = peer_manager.send_to_peer(&peer_id, forward_message.clone()).await {
                                            debug!("Failed to forward message to {}: {}", peer_id, e);
                                        }
                                    }
                                }
                                crate::p2p::routing::RoutingAction::Respond { to_peer, message } => {
                                    if let Err(e) = peer_manager.send_to_peer(&to_peer, message).await {
                                        debug!("Failed to send response to {}: {}", to_peer, e);
                                    }
                                }
                                crate::p2p::routing::RoutingAction::UpdateHeartbeat { peer_id } => {
                                    peer_manager.update_peer_heartbeat(&peer_id).await;
                                }
                            }
                        }
                    }

                    // Handle peer disconnections
                    disconnected_peer = disconnect_rx.recv() => {
                        if let Some(peer_id) = disconnected_peer {
                            peer_manager.remove_peer(&peer_id, "Connection lost".to_string()).await;
                            
                            let event = P2PEvent::PeerDisconnected {
                                peer_id,
                                reason: "Connection lost".to_string(),
                            };
                            if let Err(e) = event_tx.send(event).await {
                                warn!("Failed to send peer disconnected event: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Start background tasks
    async fn start_background_tasks(&self) {
        let peer_manager = self.peer_manager.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();

        // Cleanup task
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(60));
            
            while *running.read().await {
                cleanup_interval.tick().await;
                
                // Cleanup dead connections
                peer_manager.cleanup_dead_connections(120).await; // 2 minutes timeout
                
                debug!("Performed cleanup tasks");
            }
        });

        // Statistics update task
        let stats_clone = stats.clone();
        let running_clone = self.running.clone();
        
        tokio::spawn(async move {
            let mut stats_interval = interval(Duration::from_secs(10));
            let start_time = SystemTime::now();
            
            while *running_clone.read().await {
                stats_interval.tick().await;
                
                let mut stats = stats_clone.write().await;
                stats.uptime_secs = start_time.elapsed().unwrap_or_default().as_secs();
            }
        });
    }

    /// Connect to bootstrap peers
    async fn connect_to_bootstrap_peers(&self) {
        for bootstrap_addr in &self.config.bootstrap_peers {
            let peer_manager = self.peer_manager.clone();
            let tls_context = self.tls_context.clone();
            let bootstrap_addr = *bootstrap_addr;
            let event_tx = self.event_tx.clone();

            tokio::spawn(async move {
                match Self::connect_to_peer(bootstrap_addr, tls_context, peer_manager, event_tx).await {
                    Ok(_) => {
                        info!("Successfully connected to bootstrap peer: {}", bootstrap_addr);
                    }
                    Err(e) => {
                        warn!("Failed to connect to bootstrap peer {}: {}", bootstrap_addr, e);
                    }
                }
            });
        }
    }

    /// Connect to a specific peer
    async fn connect_to_peer(
        addr: SocketAddr,
        tls_context: Option<TlsContext>,
        peer_manager: PeerManager,
        event_tx: mpsc::Sender<P2PEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = if let Some(tls_context) = tls_context {
            TlsConnection::connect_tls(addr, tls_context.client_config).await?
        } else {
            TlsConnection::connect_plain(addr).await?
        };

        // For now, create a temporary peer ID
        // In a real implementation, you'd perform a handshake
        let temp_peer_id = Uuid::new_v4().to_string();
        let temp_username = format!("Peer@{}", addr);

        peer_manager.add_peer(
            connection,
            temp_peer_id.clone(),
            addr,
            temp_username.clone(),
            "1.0".to_string(),
        ).await?;

        // Send peer connected event
        let event = P2PEvent::PeerConnected {
            peer_id: temp_peer_id,
            addr,
            username: temp_username,
        };

        if let Err(e) = event_tx.send(event).await {
            warn!("Failed to send peer connected event: {}", e);
        }

        Ok(())
    }

    /// Get the local peer ID
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Get the local username
    pub fn username(&self) -> &str {
        &self.config.username
    }

    /// Get the listening address
    pub async fn listen_addr(&self) -> SocketAddr {
        let addr_lock = self.actual_listen_addr.read().await;
        addr_lock.unwrap_or(self.config.listen_addr)
    }
}
