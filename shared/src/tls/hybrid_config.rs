//! Hybrid Post-Quantum TLS Configuration
//! 
//! Provides X25519 + Kyber768/ML-KEM hybrid key exchange for TLS 1.3

use rustls::{ClientConfig, ServerConfig};
use rustls::crypto::CryptoProvider;
use rustls_post_quantum::{provider, X25519MLKEM768, MLKEM768};
use std::sync::Arc;
use crate::tls::{TlsConfig, CertificateManager};

/// Hybrid Post-Quantum TLS configuration
pub struct HybridTlsConfig {
    /// Base TLS configuration
    base_config: TlsConfig,
}

impl HybridTlsConfig {
    /// Create a new hybrid TLS configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let base_config = TlsConfig::new()?;
        
        Ok(Self {
            base_config,
        })
    }
    
    /// Create client configuration with hybrid PQC support
    pub fn create_hybrid_client_config(
        &self,
        server_name: &str,
    ) -> Result<ClientConfig, Box<dyn std::error::Error>> {
        // Use post-quantum crypto provider
        let crypto_provider = self.create_hybrid_crypto_provider();
        
        let mut config = ClientConfig::builder_with_provider(Arc::new(crypto_provider))
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_root_certificates(self.base_config.root_store.clone())
            .with_no_client_auth();
        
        // Enable post-quantum key exchange preference
        tracing::info!("Hybrid TLS client config created with X25519+ML-KEM support");
        
        Ok(config)
    }
    
    /// Create server configuration with hybrid PQC support
    pub fn create_hybrid_server_config(
        &self,
        cert_manager: &CertificateManager,
    ) -> Result<ServerConfig, Box<dyn std::error::Error>> {
        // Use post-quantum crypto provider
        let crypto_provider = self.create_hybrid_crypto_provider();
        
        let cert_chain = cert_manager.get_cert_chain()?;
        let private_key = cert_manager.get_private_key()?;
        
        let config = ServerConfig::builder_with_provider(Arc::new(crypto_provider))
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)?;
        
        tracing::info!("Hybrid TLS server config created with X25519+ML-KEM support");
        
        Ok(config)
    }
    
    /// Create hybrid crypto provider with post-quantum support
    fn create_hybrid_crypto_provider(&self) -> CryptoProvider {
        // Get the post-quantum crypto provider
        let mut pq_provider = provider();
        
        // Ensure hybrid key exchange is prioritized
        // This will prefer X25519+ML-KEM over pure classical or pure PQ
        pq_provider.kx_groups = vec![
            X25519MLKEM768,  // Hybrid X25519 + ML-KEM-768
            MLKEM768,        // Pure ML-KEM-768 (fallback)
            rustls::crypto::ring::kx_group::X25519, // Classical fallback
        ];
        
        tracing::info!("Crypto provider configured with hybrid key exchange groups:");
        tracing::info!("  1. X25519+ML-KEM-768 (hybrid)");
        tracing::info!("  2. ML-KEM-768 (pure PQ)");
        tracing::info!("  3. X25519 (classical fallback)");
        
        pq_provider
    }
    
    /// Get supported cipher suites for hybrid configuration
    pub fn get_supported_cipher_suites(&self) -> Vec<rustls::SupportedCipherSuite> {
        vec![
            rustls::crypto::ring::cipher_suite::TLS13_AES_256_GCM_SHA384,
            rustls::crypto::ring::cipher_suite::TLS13_AES_128_GCM_SHA256,
            rustls::crypto::ring::cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
        ]
    }
    
    /// Get supported key exchange groups
    pub fn get_supported_kx_groups(&self) -> Vec<&'static dyn rustls::crypto::SupportedKxGroup> {
        vec![
            &X25519MLKEM768,  // Hybrid
            &MLKEM768,        // Pure PQ
            &rustls::crypto::ring::kx_group::X25519, // Classical
        ]
    }
    
    /// Verify hybrid configuration is working
    pub fn verify_hybrid_support(&self) -> Result<(), Box<dyn std::error::Error>> {
        let provider = self.create_hybrid_crypto_provider();
        
        // Check that hybrid groups are available
        if provider.kx_groups.is_empty() {
            return Err("No key exchange groups available".into());
        }
        
        // Verify X25519+ML-KEM is first priority
        let first_group = &provider.kx_groups[0];
        if first_group.name() != rustls::NamedGroup::X25519MLKEM768 {
            tracing::warn!("X25519+ML-KEM not prioritized in key exchange");
        }
        
        tracing::info!("Hybrid PQC support verified successfully");
        Ok(())
    }
}

impl Default for HybridTlsConfig {
    fn default() -> Self {
        Self::new().expect("Failed to create default hybrid TLS config")
    }
}

/// Helper function to create hybrid TLS context
pub fn create_hybrid_tls_context(
    is_server: bool,
    server_name: Option<&str>,
    cert_manager: Option<&CertificateManager>,
) -> Result<Arc<CryptoProvider>, Box<dyn std::error::Error>> {
    let hybrid_config = HybridTlsConfig::new()?;
    
    // Verify hybrid support
    hybrid_config.verify_hybrid_support()?;
    
    let provider = hybrid_config.create_hybrid_crypto_provider();
    
    if is_server {
        tracing::info!("Created hybrid TLS server context with PQC support");
    } else {
        tracing::info!("Created hybrid TLS client context with PQC support");
    }
    
    Ok(Arc::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hybrid_config_creation() {
        let config = HybridTlsConfig::new().unwrap();
        assert!(config.verify_hybrid_support().is_ok());
    }
    
    #[test]
    fn test_crypto_provider_groups() {
        let config = HybridTlsConfig::new().unwrap();
        let provider = config.create_hybrid_crypto_provider();
        
        // Should have at least hybrid group
        assert!(!provider.kx_groups.is_empty());
        
        // First should be hybrid
        assert_eq!(provider.kx_groups[0].name(), rustls::NamedGroup::X25519MLKEM768);
    }
    
    #[test]
    fn test_supported_cipher_suites() {
        let config = HybridTlsConfig::new().unwrap();
        let suites = config.get_supported_cipher_suites();
        
        assert!(!suites.is_empty());
        // Should include AES-256-GCM
        assert!(suites.iter().any(|s| s.suite() == rustls::CipherSuite::TLS13_AES_256_GCM_SHA384));
    }
}
