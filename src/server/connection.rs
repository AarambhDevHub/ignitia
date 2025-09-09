//! # TCP Connection Management
//!
//! This module provides utilities for managing TCP connections within the Ignitia web framework.
//! It includes connection state management, address resolution, and connection lifecycle handling
//! for both HTTP and WebSocket connections.
//!
//! ## Features
//!
//! - **Connection Wrapping**: Safe wrapper around Tokio TCP streams
//! - **Address Management**: Easy access to local and peer addresses
//! - **Resource Management**: Automatic cleanup and resource management
//! - **Stream Access**: Direct access to underlying TCP stream when needed
//! - **Error Handling**: Proper error handling for network operations
//!
//! ## Usage Examples
//!
//! ### Basic Connection Handling
//! ```
//! use ignitia::server::connection::Connection;
//! use tokio::net::TcpStream;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // This would typically be done by the server automatically
//! let stream = TcpStream::connect("127.0.0.1:80").await?;
//! let connection = Connection::new(stream);
//!
//! // Get connection information
//! let peer_addr = connection.peer_addr()?;
//! let local_addr = connection.local_addr()?;
//!
//! println!("Connection from {} to {}", peer_addr, local_addr);
//! # Ok(())
//! # }
//! ```
//!
//! ### Connection Monitoring
//! ```
//! use ignitia::server::connection::Connection;
//! use tokio::net::TcpStream;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let stream = TcpStream::connect("example.com:80").await?;
//! let connection = Connection::new(stream);
//!
//! // Monitor connection details
//! match connection.peer_addr() {
//!     Ok(addr) => {
//!         if addr.ip().is_ipv4() {
//!             println!("IPv4 connection from: {}", addr);
//!         } else {
//!             println!("IPv6 connection from: {}", addr);
//!         }
//!     }
//!     Err(e) => {
//!         eprintln!("Failed to get peer address: {}", e);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::io;
use tokio::net::TcpStream;

/// A wrapper around a TCP stream that provides additional connection management functionality.
///
/// The `Connection` struct encapsulates a Tokio `TcpStream` and provides methods for
/// accessing connection metadata, managing the connection lifecycle, and obtaining
/// address information for both the local and remote endpoints.
///
/// # Design Philosophy
///
/// This struct follows the principle of providing a clean, safe interface around
/// the underlying TCP stream while maintaining high performance and low overhead.
/// It allows for future extensibility of connection management features without
/// breaking existing code.
///
/// # Thread Safety
///
/// The `Connection` struct is not inherently thread-safe, as it wraps a `TcpStream`
/// which is designed to be used from a single task. However, it can be safely
/// moved between tasks and used in async contexts.
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::server::connection::Connection;
/// use tokio::net::TcpStream;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a connection (usually done by the server)
/// let stream = TcpStream::connect("httpbin.org:80").await?;
/// let connection = Connection::new(stream);
///
/// // Access connection information
/// let peer = connection.peer_addr()?;
/// let local = connection.local_addr()?;
///
/// println!("Connected to {} from {}", peer, local);
/// # Ok(())
/// # }
/// ```
///
/// ## Connection Information Logging
/// ```
/// use ignitia::server::connection::Connection;
/// use tokio::net::TcpStream;
/// use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let stream = TcpStream::connect("example.com:443").await?;
/// let connection = Connection::new(stream);
///
/// if let Ok(peer_addr) = connection.peer_addr() {
///     match peer_addr.ip() {
///         IpAddr::V4(ipv4) => {
///             println!("IPv4 connection: {}:{}", ipv4, peer_addr.port());
///             if ipv4.is_private() {
///                 println!("Connection from private network");
///             }
///         }
///         IpAddr::V6(ipv6) => {
///             println!("IPv6 connection: [{}]:{}", ipv6, peer_addr.port());
///             if ipv6.is_loopback() {
///                 println!("Local IPv6 connection");
///             }
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct Connection {
    /// The underlying TCP stream
    stream: TcpStream,
}

impl Connection {
    /// Creates a new `Connection` wrapper around the provided TCP stream.
    ///
    /// This constructor takes ownership of the TCP stream and wraps it in a
    /// `Connection` instance that provides additional functionality and a
    /// cleaner API for connection management.
    ///
    /// # Parameters
    /// - `stream`: The `TcpStream` to wrap
    ///
    /// # Returns
    /// A new `Connection` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("example.com:80").await?;
    /// let connection = Connection::new(stream);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Server Context Usage
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::{TcpListener, TcpStream};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let listener = TcpListener::bind("127.0.0.1:3000").await?;
    ///
    /// loop {
    ///     let (stream, _addr) = listener.accept().await?;
    ///     let connection = Connection::new(stream);
    ///
    ///     // Handle the connection
    ///     tokio::spawn(async move {
    ///         // Connection handling logic would go here
    ///         if let Ok(peer) = connection.peer_addr() {
    ///             println!("New connection from: {}", peer);
    ///         }
    ///     });
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Returns a reference to the underlying TCP stream.
    ///
    /// This method provides read-only access to the wrapped `TcpStream` without
    /// transferring ownership. This is useful when you need to inspect stream
    /// properties or perform operations that require a stream reference.
    ///
    /// # Returns
    /// A reference to the underlying `TcpStream`
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("example.com:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// // Access the stream reference
    /// let stream_ref = connection.stream();
    ///
    /// // You can now use stream_ref for operations that need a reference
    /// // For example, checking if the stream is readable
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Advanced Stream Operations
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    /// use tokio::io::{AsyncReadExt, AsyncWriteExt};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("httpbin.org:80").await?;
    /// let mut connection = Connection::new(stream);
    ///
    /// // Get a reference to perform operations
    /// let stream_ref = connection.stream();
    ///
    /// // Note: For actual I/O operations, you'd typically use into_stream()
    /// // to get ownership of the stream
    /// # Ok(())
    /// # }
    /// ```
    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Consumes the connection and returns the underlying TCP stream.
    ///
    /// This method transfers ownership of the wrapped `TcpStream` back to the caller,
    /// consuming the `Connection` wrapper in the process. This is useful when you
    /// need direct access to the stream for I/O operations or when integrating
    /// with APIs that require ownership of the stream.
    ///
    /// # Returns
    /// The underlying `TcpStream`
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    /// use tokio::io::{AsyncReadExt, AsyncWriteExt};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("httpbin.org:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// // Extract the stream for direct I/O operations
    /// let mut stream = connection.into_stream();
    ///
    /// // Now you can perform I/O operations directly
    /// stream.write_all(b"GET / HTTP/1.1\r\nHost: httpbin.org\r\n\r\n").await?;
    ///
    /// let mut response = Vec::new();
    /// stream.read_to_end(&mut response).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Integration with WebSocket Libraries
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("echo.websocket.org:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// // Extract stream for WebSocket upgrade
    /// let stream = connection.into_stream();
    ///
    /// // Use stream with WebSocket libraries
    /// // let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_stream(self) -> TcpStream {
        self.stream
    }

    /// Returns the remote peer address of the TCP connection.
    ///
    /// This method retrieves the socket address of the remote peer that this
    /// connection is connected to. This is useful for logging, security checks,
    /// rate limiting, and other connection management tasks.
    ///
    /// # Returns
    /// - `Ok(SocketAddr)`: The remote peer's socket address
    /// - `Err(io::Error)`: If the peer address cannot be determined
    ///
    /// # Errors
    /// This method can return an error if:
    /// - The connection has been closed
    /// - The underlying socket is in an invalid state
    /// - System-level networking errors occur
    ///
    /// # Examples
    ///
    /// ## Basic Peer Address Retrieval
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("httpbin.org:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// match connection.peer_addr() {
    ///     Ok(addr) => println!("Connected to: {}", addr),
    ///     Err(e) => eprintln!("Failed to get peer address: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Security and Rate Limiting
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("example.com:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// // Example rate limiting based on IP address
    /// if let Ok(peer_addr) = connection.peer_addr() {
    ///     let ip = peer_addr.ip();
    ///
    ///     // Check if this IP is allowed
    ///     if ip.is_loopback() {
    ///         println!("Local connection allowed: {}", ip);
    ///     } else if ip.to_string().starts_with("192.168.") {
    ///         println!("Private network connection: {}", ip);
    ///     } else {
    ///         println!("External connection from: {}", ip);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Geographic Information (Conceptual)
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    /// use std::net::IpAddr;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("google.com:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// if let Ok(peer_addr) = connection.peer_addr() {
    ///     match peer_addr.ip() {
    ///         IpAddr::V4(ipv4) => {
    ///             if ipv4.is_private() {
    ///                 println!("Private IPv4: {}", ipv4);
    ///             } else if ipv4.is_multicast() {
    ///                 println!("Multicast IPv4: {}", ipv4);
    ///             } else {
    ///                 println!("Public IPv4: {}", ipv4);
    ///                 // In a real application, you might do GeoIP lookup here
    ///             }
    ///         }
    ///         IpAddr::V6(ipv6) => {
    ///             if ipv6.is_loopback() {
    ///                 println!("Loopback IPv6: {}", ipv6);
    ///             } else {
    ///                 println!("IPv6 address: {}", ipv6);
    ///             }
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn peer_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.stream.peer_addr()
    }

    /// Returns the local socket address of the TCP connection.
    ///
    /// This method retrieves the local socket address that this connection is
    /// bound to. This is useful for determining which local interface and port
    /// the connection is using, which can be important for logging, debugging,
    /// and network configuration validation.
    ///
    /// # Returns
    /// - `Ok(SocketAddr)`: The local socket address
    /// - `Err(io::Error)`: If the local address cannot be determined
    ///
    /// # Errors
    /// This method can return an error if:
    /// - The connection has been closed
    /// - The underlying socket is in an invalid state
    /// - System-level networking errors occur
    ///
    /// # Examples
    ///
    /// ## Basic Local Address Retrieval
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("httpbin.org:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// match connection.local_addr() {
    ///     Ok(addr) => println!("Local address: {}", addr),
    ///     Err(e) => eprintln!("Failed to get local address: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Server Configuration Validation
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::{TcpListener, TcpStream};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Server setup
    /// let listener = TcpListener::bind("127.0.0.1:0").await?; // Bind to any available port
    /// let server_addr = listener.local_addr()?;
    ///
    /// // Client connection
    /// let stream = TcpStream::connect(server_addr).await?;
    /// let connection = Connection::new(stream);
    ///
    /// // Verify the connection is using the expected local configuration
    /// if let Ok(local_addr) = connection.local_addr() {
    ///     println!("Connected via local address: {}", local_addr);
    ///     println!("Local port: {}", local_addr.port());
    ///
    ///     if local_addr.ip().is_loopback() {
    ///         println!("Using loopback interface");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Multi-Interface Server Debugging
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("example.com:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// // Debug which local interface is being used
    /// if let (Ok(local_addr), Ok(peer_addr)) = (connection.local_addr(), connection.peer_addr()) {
    ///     println!("Connection: {} -> {}", local_addr, peer_addr);
    ///
    ///     match local_addr.ip() {
    ///         IpAddr::V4(ipv4) => {
    ///             if ipv4 == Ipv4Addr::LOCALHOST {
    ///                 println!("Using localhost interface");
    ///             } else if ipv4.is_private() {
    ///                 println!("Using private network interface: {}", ipv4);
    ///             } else {
    ///                 println!("Using public interface: {}", ipv4);
    ///             }
    ///         }
    ///         IpAddr::V6(ipv6) => {
    ///             println!("Using IPv6 interface: {}", ipv6);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Load Balancer Detection
    /// ```
    /// use ignitia::server::connection::Connection;
    /// use tokio::net::TcpStream;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = TcpStream::connect("example.com:80").await?;
    /// let connection = Connection::new(stream);
    ///
    /// // In a real server environment, you might want to detect
    /// // if connections are coming through a load balancer
    /// if let Ok(local_addr) = connection.local_addr() {
    ///     let port = local_addr.port();
    ///
    ///     if port == 80 || port == 443 {
    ///         println!("Direct HTTP/HTTPS connection on port {}", port);
    ///     } else {
    ///         println!("Connection on non-standard port: {}", port);
    ///         // Might indicate load balancer or proxy
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.stream.local_addr()
    }
}

/// Connection management utilities and helpers.
///
/// This module provides additional functionality for connection management,
/// monitoring, and debugging that extends beyond the basic `Connection` struct.
pub mod utils {
    //! Utility functions for connection management and analysis.

    use std::net::{IpAddr, SocketAddr};

    /// Determines if a socket address represents a local connection.
    ///
    /// # Parameters
    /// - `addr`: The socket address to check
    ///
    /// # Returns
    /// `true` if the address is a loopback address, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// use ignitia::server::connection::utils::is_local_connection;
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    ///
    /// let local_addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    /// let remote_addr: SocketAddr = "192.168.1.1:3000".parse().unwrap();
    ///
    /// assert!(is_local_connection(&local_addr));
    /// assert!(!is_local_connection(&remote_addr));
    /// ```
    pub fn is_local_connection(addr: &SocketAddr) -> bool {
        addr.ip().is_loopback()
    }

    /// Determines if a socket address represents a private network connection.
    ///
    /// # Parameters
    /// - `addr`: The socket address to check
    ///
    /// # Returns
    /// `true` if the address is in a private network range, `false` otherwise
    pub fn is_private_connection(addr: &SocketAddr) -> bool {
        match addr.ip() {
            IpAddr::V4(ipv4) => ipv4.is_private(),
            IpAddr::V6(ipv6) => {
                // IPv6 private ranges (simplified)
                ipv6.is_loopback() || ipv6.segments()[0] == 0xfc00 || ipv6.segments()[0] == 0xfd00
            }
        }
    }

    /// Extracts the IP address from a socket address.
    ///
    /// # Parameters
    /// - `addr`: The socket address
    ///
    /// # Returns
    /// The IP address component of the socket address
    pub fn extract_ip(addr: &SocketAddr) -> IpAddr {
        addr.ip()
    }

    /// Extracts the port from a socket address.
    ///
    /// # Parameters
    /// - `addr`: The socket address
    ///
    /// # Returns
    /// The port component of the socket address
    pub fn extract_port(addr: &SocketAddr) -> u16 {
        addr.port()
    }
}

/// Connection statistics and monitoring functionality.
///
/// This module provides structures and functions for tracking connection
/// metrics, performance data, and debugging information.
pub mod stats {
    //! Connection statistics and monitoring utilities.

    use std::time::{Duration, Instant};

    /// Statistics for a connection session.
    ///
    /// This struct tracks various metrics about a connection's lifecycle
    /// and can be used for monitoring, debugging, and performance analysis.
    #[derive(Debug, Clone)]
    pub struct ConnectionStats {
        /// When the connection was established
        pub connected_at: Instant,
        /// Total bytes received on this connection
        pub bytes_received: u64,
        /// Total bytes sent on this connection
        pub bytes_sent: u64,
        /// Number of requests processed on this connection
        pub requests_processed: u32,
        /// Whether the connection is still active
        pub is_active: bool,
    }

    impl ConnectionStats {
        /// Creates a new connection stats instance.
        pub fn new() -> Self {
            Self {
                connected_at: Instant::now(),
                bytes_received: 0,
                bytes_sent: 0,
                requests_processed: 0,
                is_active: true,
            }
        }

        /// Returns the duration since the connection was established.
        pub fn connection_duration(&self) -> Duration {
            self.connected_at.elapsed()
        }

        /// Records bytes received on the connection.
        pub fn record_bytes_received(&mut self, bytes: u64) {
            self.bytes_received += bytes;
        }

        /// Records bytes sent on the connection.
        pub fn record_bytes_sent(&mut self, bytes: u64) {
            self.bytes_sent += bytes;
        }

        /// Records a completed request.
        pub fn record_request(&mut self) {
            self.requests_processed += 1;
        }

        /// Marks the connection as closed.
        pub fn mark_closed(&mut self) {
            self.is_active = false;
        }
    }

    impl Default for ConnectionStats {
        fn default() -> Self {
            Self::new()
        }
    }
}
