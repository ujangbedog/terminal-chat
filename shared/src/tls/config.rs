/// TLS configuration management
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// TLS configuration for P2P connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS for connections
    pub enabled: bool,
    /// Path to certificate file (optional, will generate if not provided)
    pub cert_path: Option<PathBuf>,
    /// Path to private key file (optional, will generate if not provided)
    pub key_path: Option<PathBuf>,
    /// Verify peer certificates (for production use)
    pub verify_peer_certs: bool,
    /// Minimum TLS version to accept
    pub min_tls_version: TlsVersion,
    /// Maximum TLS version to use
    pub max_tls_version: TlsVersion,
    /// Cipher suites to allow (empty means use defaults)
    pub allowed_cipher_suites: Vec<String>,
}

/// Supported TLS versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsVersion {
    #[serde(rename = "1.2")]
    V1_2,
    #[serde(rename = "1.3")]
    V1_3,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert_path: None,
            key_path: None,
            verify_peer_certs: false, // For P2P, we use trust-on-first-use
            min_tls_version: TlsVersion::V1_3, // Enforce TLS 1.3 minimum
            max_tls_version: TlsVersion::V1_3, // Only allow TLS 1.3
            allowed_cipher_suites: vec![],
        }
    }
}

impl TlsConfig {
    /// Create a new TLS configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable TLS with TLS 1.3 enforcement
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            min_tls_version: TlsVersion::V1_3,
            max_tls_version: TlsVersion::V1_3,
            ..Default::default()
        }
    }

    /// Disable TLS (use plain TCP)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Set certificate and key paths
    pub fn with_cert_and_key(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.cert_path = Some(cert_path);
        self.key_path = Some(key_path);
        self
    }

    /// Enable peer certificate verification
    pub fn with_peer_verification(mut self, verify: bool) -> Self {
        self.verify_peer_certs = verify;
        self
    }

    /// Set TLS version range (enforces TLS 1.3 only)
    pub fn with_tls_versions(mut self, _min: TlsVersion, _max: TlsVersion) -> Self {
        // Force TLS 1.3 only regardless of input
        self.min_tls_version = TlsVersion::V1_3;
        self.max_tls_version = TlsVersion::V1_3;
        self
    }

    /// Create a TLS 1.3 only configuration (recommended)
    pub fn tls13_only() -> Self {
        Self {
            enabled: true,
            cert_path: None,
            key_path: None,
            verify_peer_certs: false,
            min_tls_version: TlsVersion::V1_3,
            max_tls_version: TlsVersion::V1_3,
            allowed_cipher_suites: vec![],
        }
    }

    /// Check if configuration enforces TLS 1.3
    pub fn is_tls13_only(&self) -> bool {
        matches!(
            (&self.min_tls_version, &self.max_tls_version),
            (TlsVersion::V1_3, TlsVersion::V1_3)
        )
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        // Enforce TLS 1.3 only
        match (&self.min_tls_version, &self.max_tls_version) {
            (TlsVersion::V1_2, _) => return Err("TLS 1.2 is not allowed. Only TLS 1.3 is supported.".to_string()),
            (_, TlsVersion::V1_2) => return Err("TLS 1.2 is not allowed. Only TLS 1.3 is supported.".to_string()),
            (TlsVersion::V1_3, TlsVersion::V1_3) => {}, // Valid
        }

        // If cert_path is provided, key_path must also be provided
        match (&self.cert_path, &self.key_path) {
            (Some(_), None) => return Err("Certificate path provided but no key path".to_string()),
            (None, Some(_)) => return Err("Key path provided but no certificate path".to_string()),
            _ => {}
        }

        Ok(())
    }
}

/// P2P network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Local listening address
    pub listen_addr: SocketAddr,
    /// TLS configuration
    pub tls: TlsConfig,
    /// Bootstrap peers to connect to initially
    pub bootstrap_peers: Vec<SocketAddr>,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Message TTL for flooding
    pub message_ttl: u8,
    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:0".parse().unwrap(), // Random port
            tls: TlsConfig::tls13_only(), // Enforce TLS 1.3 only
            bootstrap_peers: vec![],
            max_connections: 50,
            connection_timeout_secs: 30,
            heartbeat_interval_secs: 30,
            message_ttl: 7,
            max_message_size: 1024 * 1024, // 1MB
        }
    }
}

impl P2PConfig {
    /// Create a new P2P configuration
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            ..Default::default()
        }
    }

    /// Add bootstrap peers
    pub fn with_bootstrap_peers(mut self, peers: Vec<SocketAddr>) -> Self {
        self.bootstrap_peers = peers;
        self
    }

    /// Set TLS configuration
    pub fn with_tls(mut self, tls_config: TlsConfig) -> Self {
        self.tls = tls_config;
        self
    }

    /// Set maximum connections
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        self.tls.validate()?;

        // Ensure TLS 1.3 is enforced
        if self.tls.enabled && !self.tls.is_tls13_only() {
            return Err("P2P configuration must enforce TLS 1.3 only for security".to_string());
        }

        if self.max_connections == 0 {
            return Err("Maximum connections must be greater than 0".to_string());
        }

        if self.connection_timeout_secs == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }

        if self.message_ttl == 0 {
            return Err("Message TTL must be greater than 0".to_string());
        }

        Ok(())
    }
}
