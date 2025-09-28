/// Certificate management for TLS connections
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ECDSA_P256_SHA256};
use rustls::{ClientConfig, ServerConfig, Certificate as RustlsCertificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::Cursor;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{info, warn};

/// TLS Certificate wrapper
#[derive(Debug, Clone)]
pub struct TlsCertificate {
    pub cert_pem: String,
    pub key_pem: String,
    pub fingerprint: String,
}

/// Certificate manager for generating and managing TLS certificates
pub struct CertificateManager {
    certificate: Option<TlsCertificate>,
    peer_id: String,
}

impl CertificateManager {
    /// Create a new certificate manager
    pub fn new(peer_id: String) -> Self {
        Self {
            certificate: None,
            peer_id,
        }
    }

    /// Generate a self-signed certificate for this peer
    pub async fn generate_self_signed_cert(&mut self) -> Result<&TlsCertificate, Box<dyn std::error::Error + Send + Sync>> {
        info!("Generating self-signed certificate for peer: {}", self.peer_id);

        let mut params = CertificateParams::new(vec![
            format!("peer-{}", self.peer_id),
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ]);

        // Set certificate validity (rcgen uses time crate, not std::time)
        let now = time::OffsetDateTime::now_utc();
        params.not_before = now - time::Duration::seconds(60);
        params.not_after = now + time::Duration::days(365); // 1 year

        // Set distinguished name
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, format!("Chat-Client-Peer-{}", self.peer_id));
        distinguished_name.push(DnType::OrganizationName, "Terminal Chat Network");
        distinguished_name.push(DnType::CountryName, "ID");
        params.distinguished_name = distinguished_name;

        // Generate key pair
        let key_pair = KeyPair::generate(&PKCS_ECDSA_P256_SHA256)?;
        params.key_pair = Some(key_pair);

        // Generate certificate
        let cert = Certificate::from_params(params)?;
        let cert_pem = cert.serialize_pem()?;
        let key_pem = cert.serialize_private_key_pem();

        // Calculate fingerprint
        let fingerprint = self.calculate_fingerprint(&cert_pem)?;

        let tls_cert = TlsCertificate {
            cert_pem,
            key_pem,
            fingerprint,
        };

        info!("Generated certificate with fingerprint: {}", tls_cert.fingerprint);
        self.certificate = Some(tls_cert);
        
        Ok(self.certificate.as_ref().unwrap())
    }

    /// Get the current certificate
    pub fn get_certificate(&self) -> Option<&TlsCertificate> {
        self.certificate.as_ref()
    }

    /// Create a client TLS configuration
    pub async fn create_client_config(&self) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(P2PVerifier::new()))
            .with_no_client_auth();

        Ok(config)
    }

    /// Create a server TLS configuration
    pub async fn create_server_config(&self) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        let cert = self.certificate.as_ref()
            .ok_or("No certificate available. Call generate_self_signed_cert first.")?;

        let cert_chain = self.parse_certificates(&cert.cert_pem)?;
        let private_key = self.parse_private_key(&cert.key_pem)?;

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(P2PVerifier::new()))
            .with_single_cert(cert_chain, private_key)?;

        Ok(config)
    }

    /// Parse PEM certificates
    fn parse_certificates(&self, pem: &str) -> Result<Vec<RustlsCertificate>, Box<dyn std::error::Error + Send + Sync>> {
        let mut cursor = Cursor::new(pem.as_bytes());
        let certs = certs(&mut cursor)?
            .into_iter()
            .map(RustlsCertificate)
            .collect();
        Ok(certs)
    }

    /// Parse PEM private key
    fn parse_private_key(&self, pem: &str) -> Result<PrivateKey, Box<dyn std::error::Error + Send + Sync>> {
        let mut cursor = Cursor::new(pem.as_bytes());
        let keys = pkcs8_private_keys(&mut cursor)?;
        
        if keys.is_empty() {
            return Err("No private key found".into());
        }
        
        Ok(PrivateKey(keys[0].clone()))
    }

    /// Calculate certificate fingerprint
    fn calculate_fingerprint(&self, cert_pem: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        cert_pem.hash(&mut hasher);
        let hash = hasher.finish();
        
        Ok(format!("{:016x}", hash))
    }
}

/// Custom certificate verifier for P2P connections
struct P2PVerifier;

impl P2PVerifier {
    fn new() -> Self {
        Self
    }
}

impl rustls::client::ServerCertVerifier for P2PVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &RustlsCertificate,
        _intermediates: &[RustlsCertificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        // For P2P, we accept any certificate (trust on first use model)
        // In production, you might want to implement proper certificate validation
        warn!("P2P: Accepting server certificate without validation");
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

impl rustls::server::ClientCertVerifier for P2PVerifier {
    fn offer_client_auth(&self) -> bool {
        false // Optional client authentication
    }

    fn client_auth_mandatory(&self) -> bool {
        false
    }

    fn client_auth_root_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        _end_entity: &RustlsCertificate,
        _intermediates: &[RustlsCertificate],
        _now: SystemTime,
    ) -> Result<rustls::server::ClientCertVerified, rustls::Error> {
        // Accept any client certificate
        warn!("P2P: Accepting client certificate without validation");
        Ok(rustls::server::ClientCertVerified::assertion())
    }
}
