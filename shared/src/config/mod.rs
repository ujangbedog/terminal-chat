/// Configuration constants for P2P chat
pub mod constants {
    // Message and username limits
    pub const MAX_MESSAGE_LENGTH: usize = 1024;
    pub const MAX_USERNAME_LENGTH: usize = 32;
    
    // Network configuration
    pub const DEFAULT_HOST_LOCALHOST: &str = "127.0.0.1";
    pub const DEFAULT_HOST_WILDCARD: &str = "0.0.0.0";
    pub const FIXED_PORT: u16 = 40000;
    pub const FALLBACK_PORT_START: u16 = 40001;
    pub const FALLBACK_PORT_END: u16 = 40010;
    
    // TLS configuration (always enabled)
    pub const TLS_ENABLED: bool = true;
    
    // Other network settings
    pub const MULTICAST_ADDR: &str = "224.0.0.1:9999";
    pub const CONNECTION_TIMEOUT: u64 = 30; // seconds
    pub const HEARTBEAT_INTERVAL: u64 = 60; // seconds
    pub const MAX_CONNECTIONS: usize = 50;
    
    // Logging
    pub const DEFAULT_LOG_LEVEL: &str = "error";
}

/// Host selection options for user interface
#[derive(Debug, Clone, PartialEq)]
pub enum HostOption {
    Localhost,      // 127.0.0.1
    LocalNetwork,   // 192.168.x.x (auto-detect)
    Wildcard,       // 0.0.0.0
}

impl HostOption {
    pub fn to_ip(&self) -> String {
        match self {
            HostOption::Localhost => constants::DEFAULT_HOST_LOCALHOST.to_string(),
            HostOption::LocalNetwork => {
                // Try to detect local network IP, fallback to localhost
                Self::get_local_network_ip().unwrap_or_else(|| constants::DEFAULT_HOST_LOCALHOST.to_string())
            }
            HostOption::Wildcard => constants::DEFAULT_HOST_WILDCARD.to_string(),
        }
    }
    
    pub fn display_name(&self) -> &str {
        match self {
            HostOption::Localhost => "Localhost (127.0.0.1) - Only local connections",
            HostOption::LocalNetwork => "Local Network (192.168.x.x) - LAN connections",
            HostOption::Wildcard => "All Interfaces (0.0.0.0) - External connections",
        }
    }
    
    /// Get the local network IP address (192.168.x.x)
    fn get_local_network_ip() -> Option<String> {
        use std::net::UdpSocket;
        
        // Try to connect to a dummy address to get our local IP
        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;
        let local_addr = socket.local_addr().ok()?;
        
        let ip = local_addr.ip().to_string();
        
        // Check if it's a private network IP
        if ip.starts_with("192.168.") || ip.starts_with("10.") || ip.starts_with("172.") {
            Some(ip)
        } else {
            None
        }
    }
}

/// Port management utilities
pub mod port_utils {
    use super::constants::*;
    use std::net::{TcpListener, SocketAddr};
    
    /// Find an available port starting from FIXED_PORT, then trying fallback range
    pub fn find_available_port(host: &str) -> Result<u16, Box<dyn std::error::Error>> {
        // Try fixed port first
        if is_port_available(host, FIXED_PORT) {
            return Ok(FIXED_PORT);
        }
        
        // Try fallback range
        for port in FALLBACK_PORT_START..=FALLBACK_PORT_END {
            if is_port_available(host, port) {
                return Ok(port);
            }
        }
        
        Err(format!("No available ports in range {}-{}", FIXED_PORT, FALLBACK_PORT_END).into())
    }
    
    /// Check if a port is available on the given host
    fn is_port_available(host: &str, port: u16) -> bool {
        let addr = format!("{}:{}", host, port);
        match addr.parse::<SocketAddr>() {
            Ok(socket_addr) => {
                TcpListener::bind(socket_addr).is_ok()
            }
            Err(_) => false,
        }
    }
}

// re-export for convenience
pub use constants::*;
pub use port_utils::*;
