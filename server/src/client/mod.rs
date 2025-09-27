use std::net::SocketAddr;
use tokio::sync::mpsc;

/// type alias for message sender
pub type Tx = mpsc::UnboundedSender<String>;

/// type alias for message receiver
#[allow(dead_code)]
pub type Rx = mpsc::UnboundedReceiver<String>;

/// information about a connected client
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub username: Option<String>,
    pub sender: Tx,
    #[allow(dead_code)]
    pub addr: SocketAddr,
}

impl ClientInfo {
    /// create new client info
    pub fn new(sender: Tx, addr: SocketAddr) -> Self {
        Self {
            username: None,
            sender,
            addr,
        }
    }

    /// set username for this client
    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }
}
