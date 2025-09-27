/// shared library for chat application
pub mod message;
pub mod config;
pub mod utils;
pub mod tls;
pub mod p2p;

// re-export main types for convenience
pub use message::{P2PMessage, PeerInfo};
pub use config::*;
pub use utils::*;
pub use tls::{TlsContext, TlsConfig, CertificateManager};
pub use p2p::{P2PNode, P2PEvent, P2PStats, P2PNodeConfig};
