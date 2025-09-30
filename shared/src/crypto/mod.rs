//! Cryptographic utilities for DPQ Chat
//! 
//! Provides session key management, handshake protocols, and message encryption

pub mod session;
pub mod handshake;
pub mod message_crypto;
pub mod kyber_kex;
pub mod dilithium_ops;
pub mod identity_utils;

pub use session::{SessionKey, SessionManager};
pub use handshake::{HandshakeManager, HandshakeData, PeerInfo};
pub use message_crypto::{MessageCrypto, EncryptedMessage, MessageType, PlainMessage};
pub use kyber_kex::{KyberKeyExchangeManager, KyberKeyExchange};
pub use dilithium_ops::{DilithiumKeypair, DilithiumVerifier};
pub use identity_utils::{
    load_dilithium_keypair_from_identity, 
    create_handshake_manager_with_identity,
    create_handshake_manager_from_identity
};
