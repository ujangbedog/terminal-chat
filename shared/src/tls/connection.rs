/// TLS connection handling for P2P networking
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};
use std::net::SocketAddr;
use std::sync::Arc;
use rustls::{ClientConfig, ServerConfig};
use tracing::{info, debug};

/// TLS connection wrapper
pub enum TlsConnection {
    /// Plain TCP connection (when TLS is disabled)
    Plain(TcpStream),
    /// TLS-secured connection
    Tls(TlsStream<TcpStream>),
}

impl TlsConnection {
    /// Create a new TLS client connection
    pub async fn connect_tls(
        addr: SocketAddr,
        client_config: Arc<ClientConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Connecting to {} with TLS", addr);
        
        let tcp_stream = TcpStream::connect(addr).await?;
        let connector = TlsConnector::from(client_config);
        
        // Use the IP address as the server name for P2P connections
        let server_name = rustls::ServerName::try_from(addr.ip().to_string().as_str())?;
        let tls_stream = connector.connect(server_name, tcp_stream).await?;
        
        info!("Established TLS connection to {}", addr);
        Ok(TlsConnection::Tls(TlsStream::Client(tls_stream)))
    }

    /// Create a new plain TCP connection
    pub async fn connect_plain(addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Connecting to {} with plain TCP", addr);
        
        let tcp_stream = TcpStream::connect(addr).await?;
        
        info!("Established plain TCP connection to {}", addr);
        Ok(TlsConnection::Plain(tcp_stream))
    }

    /// Get the peer address
    pub fn peer_addr(&self) -> Result<SocketAddr, std::io::Error> {
        match self {
            TlsConnection::Plain(stream) => stream.peer_addr(),
            TlsConnection::Tls(stream) => stream.get_ref().0.peer_addr(),
        }
    }

    /// Get the local address
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        match self {
            TlsConnection::Plain(stream) => stream.local_addr(),
            TlsConnection::Tls(stream) => stream.get_ref().0.local_addr(),
        }
    }
}

impl tokio::io::AsyncRead for TlsConnection {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TlsConnection::Plain(stream) => {
                std::pin::Pin::new(stream).poll_read(cx, buf)
            }
            TlsConnection::Tls(stream) => {
                std::pin::Pin::new(stream).poll_read(cx, buf)
            }
        }
    }
}

impl tokio::io::AsyncWrite for TlsConnection {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            TlsConnection::Plain(stream) => {
                std::pin::Pin::new(stream).poll_write(cx, buf)
            }
            TlsConnection::Tls(stream) => {
                std::pin::Pin::new(stream).poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            TlsConnection::Plain(stream) => {
                std::pin::Pin::new(stream).poll_flush(cx)
            }
            TlsConnection::Tls(stream) => {
                std::pin::Pin::new(stream).poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            TlsConnection::Plain(stream) => {
                std::pin::Pin::new(stream).poll_shutdown(cx)
            }
            TlsConnection::Tls(stream) => {
                std::pin::Pin::new(stream).poll_shutdown(cx)
            }
        }
    }
}

/// TLS listener wrapper
pub struct TlsListener {
    tcp_listener: TcpListener,
    tls_acceptor: Option<TlsAcceptor>,
}

impl TlsListener {
    /// Create a new TLS listener
    pub async fn bind_tls(
        addr: SocketAddr,
        server_config: Arc<ServerConfig>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let tcp_listener = TcpListener::bind(addr).await?;
        let tls_acceptor = TlsAcceptor::from(server_config);
        
        info!("TLS listener bound to {}", addr);
        Ok(TlsListener {
            tcp_listener,
            tls_acceptor: Some(tls_acceptor),
        })
    }

    /// Create a new plain TCP listener
    pub async fn bind_plain(addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let tcp_listener = TcpListener::bind(addr).await?;
        
        info!("Plain TCP listener bound to {}", addr);
        Ok(TlsListener {
            tcp_listener,
            tls_acceptor: None,
        })
    }

    /// Accept a new connection
    pub async fn accept(&self) -> Result<(TlsConnection, SocketAddr), Box<dyn std::error::Error + Send + Sync>> {
        let (tcp_stream, peer_addr) = self.tcp_listener.accept().await?;
        
        match &self.tls_acceptor {
            Some(acceptor) => {
                debug!("Accepting TLS connection from {}", peer_addr);
                let tls_stream = acceptor.accept(tcp_stream).await?;
                info!("Accepted TLS connection from {}", peer_addr);
                Ok((TlsConnection::Tls(TlsStream::Server(tls_stream)), peer_addr))
            }
            None => {
                debug!("Accepting plain TCP connection from {}", peer_addr);
                info!("Accepted plain TCP connection from {}", peer_addr);
                Ok((TlsConnection::Plain(tcp_stream), peer_addr))
            }
        }
    }

    /// Get the local address
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.tcp_listener.local_addr()
    }
}

/// Connection utilities
pub struct ConnectionUtils;

impl ConnectionUtils {
    /// Test if a TLS connection can be established to an address
    pub async fn test_tls_connection(
        addr: SocketAddr,
        client_config: Arc<ClientConfig>,
        timeout_secs: u64,
    ) -> bool {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        
        match tokio::time::timeout(timeout, TlsConnection::connect_tls(addr, client_config)).await {
            Ok(Ok(_)) => {
                debug!("TLS connection test to {} succeeded", addr);
                true
            }
            Ok(Err(e)) => {
                debug!("TLS connection test to {} failed: {}", addr, e);
                false
            }
            Err(_) => {
                debug!("TLS connection test to {} timed out", addr);
                false
            }
        }
    }

    /// Test if a plain TCP connection can be established to an address
    pub async fn test_plain_connection(addr: SocketAddr, timeout_secs: u64) -> bool {
        let timeout = std::time::Duration::from_secs(timeout_secs);
        
        match tokio::time::timeout(timeout, TlsConnection::connect_plain(addr)).await {
            Ok(Ok(_)) => {
                debug!("Plain TCP connection test to {} succeeded", addr);
                true
            }
            Ok(Err(e)) => {
                debug!("Plain TCP connection test to {} failed: {}", addr, e);
                false
            }
            Err(_) => {
                debug!("Plain TCP connection test to {} timed out", addr);
                false
            }
        }
    }
}
