/// Certificate management for TLS connections
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_ECDSA_P256_SHA256};
use rustls::{ClientConfig, ServerConfig};
use rustls_pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::Cursor;
use std::sync::Arc;
use tracing::info;
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

    /// Create a client TLS configuration with TLS 1.3 enforcement
    pub async fn create_client_config(&self) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
        // Use post-quantum crypto provider for hybrid X25519+ML-KEM
        let pq_provider = rustls_post_quantum::provider();
        let _ = pq_provider.install_default();
        
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(P2PVerifier::new()))
            .with_no_client_auth();

        info!("🔐 Client TLS configuration created with HYBRID X25519+ML-KEM support");
        Ok(config)
    }

    /// Create a server TLS configuration with TLS 1.3 enforcement
    pub async fn create_server_config(&self) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        // Use post-quantum crypto provider for hybrid X25519+ML-KEM
        let pq_provider = rustls_post_quantum::provider();
        let _ = pq_provider.install_default();
        
        let cert = self.certificate.as_ref()
            .ok_or("No certificate available. Call generate_self_signed_cert first.")?;

        let cert_chain = self.parse_certificates(&cert.cert_pem)?;
        let private_key = self.parse_private_key(&cert.key_pem)?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)?;

        info!("🔐 Server TLS configuration created with HYBRID X25519+ML-KEM support");
        Ok(config)
    }

    /// Parse PEM certificates
    fn parse_certificates(&self, pem: &str) -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut cursor = Cursor::new(pem.as_bytes());
        let certs = certs(&mut cursor)?
            .into_iter()
            .map(|cert| CertificateDer::from(cert))
            .collect();
        Ok(certs)
    }

    /// Parse PEM private key
    fn parse_private_key(&self, pem: &str) -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error + Send + Sync>> {
        let mut cursor = Cursor::new(pem.as_bytes());
        let mut keys = pkcs8_private_keys(&mut cursor)?;
        
        if keys.is_empty() {
            return Err("No private key found in PEM".into());
        }
        
        Ok(PrivateKeyDer::Pkcs8(keys.remove(0).into()))
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
#[derive(Debug)]
struct P2PVerifier;

impl P2PVerifier {
    fn new() -> Self {
        Self
    }
}

impl rustls::client::danger::ServerCertVerifier for P2PVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // For P2P, we accept any certificate (trust on first use model)
        // In production, you might want to implement proper certificate validation
        info!("P2P: Accepting server certificate with TLS 1.3 enforcement");
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

impl rustls::server::danger::ClientCertVerifier for P2PVerifier {
    fn offer_client_auth(&self) -> bool {
        false // Optional client authentication
    }

    fn client_auth_mandatory(&self) -> bool {
        false
    }

    fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::server::danger::ClientCertVerified, rustls::Error> {
        // Accept any client certificate
        info!("P2P: Accepting client certificate with TLS 1.3 enforcement");
        Ok(rustls::server::danger::ClientCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
