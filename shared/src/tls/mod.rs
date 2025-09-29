/// TLS module for secure peer-to-peer connections
/// 
/// This module enforces TLS 1.3 for all connections to ensure maximum security.
/// TLS 1.2 and earlier versions are not supported.
pub mod cert;
pub mod config;
pub mod connection;
// pub mod hybrid_config;  // Disabled temporarily due to API compatibility

// Re-export main types for convenience
pub use cert::{CertificateManager, TlsCertificate};
pub use config::TlsConfig;
pub use connection::{TlsConnection, TlsListener};
// pub use hybrid_config::{HybridTlsConfig, create_hybrid_tls_context};

use std::sync::Arc;
use rustls::{ClientConfig, ServerConfig};

/// TLS context that holds both client and server configurations
#[derive(Clone)]
pub struct TlsContext {
    pub client_config: Arc<ClientConfig>,
    pub server_config: Arc<ServerConfig>,
}

impl TlsContext {
    /// Create a new TLS context with the given certificate manager
    pub async fn new(cert_manager: &CertificateManager) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client_config = cert_manager.create_client_config().await?;
        let server_config = cert_manager.create_server_config().await?;
        
        Ok(TlsContext {
            client_config: Arc::new(client_config),
            server_config: Arc::new(server_config),
        })
    }
    
    // /// Create a new hybrid TLS context with post-quantum support
    // pub async fn new_hybrid(cert_manager: &CertificateManager) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
    //     // TODO: Implement hybrid TLS with rustls 0.23 API
    //     // For now, use regular TLS
    //     Self::new(cert_manager).await
    // }
}
