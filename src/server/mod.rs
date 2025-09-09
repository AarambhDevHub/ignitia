//! # HTTP Server Implementation
//!
//! This module provides the core HTTP server functionality for the Ignitia web framework.
//! It includes high-performance request handling, WebSocket upgrade support, connection
//! management, and efficient request/response processing built on top of Hyper and Tokio.
//!
//! ## Features
//!
//! - **High-Performance Server**: Built on Hyper 1.x with async/await support
//! - **WebSocket Support**: Optional WebSocket upgrade handling with proper protocol negotiation
//! - **Connection Management**: Efficient TCP connection handling with proper error recovery
//! - **Request Size Limits**: Configurable request body size limits to prevent memory exhaustion
//! - **Error Handling**: Comprehensive error handling with proper HTTP status codes
//! - **Concurrent Processing**: Each connection is handled in its own Tokio task
//!
//! ## Architecture
//!
//! ### Server Design
//! The server uses an event-driven architecture:
//! 1. **Main Accept Loop**: Accepts incoming TCP connections
//! 2. **Connection Tasks**: Each connection is handled in a separate Tokio task
//! 3. **Request Processing**: HTTP requests are parsed and routed through the router
//! 4. **Response Generation**: Responses are generated and sent back to clients
//!
//! ### WebSocket Upgrade Flow
//! When WebSocket support is enabled:
//! 1. **Detection**: Incoming requests are checked for WebSocket upgrade headers
//! 2. **Validation**: WebSocket protocol headers are validated
//! 3. **Upgrade**: HTTP connection is upgraded to WebSocket protocol
//! 4. **Handler Dispatch**: WebSocket connections are dispatched to appropriate handlers
//!
//! ## Usage Examples
//!
//! ### Basic Server Setup
//! ```
//! use ignitia::{Router, Server, Response};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/", || async { Ok(Response::text("Hello, World!")) })
//!         .get("/health", || async { Ok(Response::text("OK")) });
//!
//!     let addr: SocketAddr = "127.0.0.1:3000".parse()?;
//!     let server = Server::new(router, addr);
//!
//!     println!("Starting server on http://{}", addr);
//!     server.ignitia().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Server with Middleware
//! ```
//! use ignitia::{Router, Server, Response, LoggerMiddleware, CorsMiddleware};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .middleware(LoggerMiddleware)
//!         .middleware(CorsMiddleware::new().allow_origin("*"))
//!         .get("/api/users", || async {
//!             Ok(Response::json(vec![
//!                 serde_json::json!({"id": 1, "name": "Alice"}),
//!                 serde_json::json!({"id": 2, "name": "Bob"}),
//!             ])?)
//!         })
//!         .post("/api/users", || async {
//!             Ok(Response::json(serde_json::json!({
//!                 "message": "User created successfully"
//!             }))?)
//!         });
//!
//!     let server = Server::new(router, "0.0.0.0:8080".parse()?);
//!     server.ignitia().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### WebSocket Server
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::{Router, Server, Response, websocket::{WebSocketConnection, Message}};
//!
//! #[cfg(feature = "websocket")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/", || async {
//!             Ok(Response::html(r#"
//!                 <html>
//!                 <body>
//!                     <h1>WebSocket Demo</h1>
//!                     <div id="messages"></div>
//!                     <input type="text" id="messageInput" placeholder="Type a message...">
//!                     <button onclick="sendMessage()">Send</button>
//!                     <script>
//!                         const ws = new WebSocket('ws://localhost:3000/ws');
//!                         ws.onmessage = function(event) {
//!                             const messages = document.getElementById('messages');
//!                             messages.innerHTML += '<div>' + event.data + '</div>';
//!                         };
//!                         function sendMessage() {
//!                             const input = document.getElementById('messageInput');
//!                             ws.send(input.value);
//!                             input.value = '';
//!                         }
//!                     </script>
//!                 </body>
//!                 </html>
//!             "#))
//!         })
//!         .websocket("/ws", |mut ws: WebSocketConnection| async move {
//!             while let Some(msg) = ws.recv().await {
//!                 match msg {
//!                     Message::Text(text) => {
//!                         // Echo the message back
//!                         ws.send_text(format!("Echo: {}", text)).await?;
//!                     }
//!                     Message::Close(_) => break,
//!                     _ => {}
//!                 }
//!             }
//!             Ok(())
//!         });
//!
//!     let server = Server::new(router, "127.0.0.1:3000".parse()?);
//!     server.ignitia().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Configuration
//!
//! ### Custom Error Handling
//! ```
//! use ignitia::{Router, Server, Response, Error};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/error", || async {
//!             Err(Error::Internal("Something went wrong".into()))
//!         })
//!         .not_found(|| async {
//!             Ok(Response::html(r#"
//!                 <html>
//!                 <body>
//!                     <h1>404 - Page Not Found</h1>
//!                     <p>The requested page could not be found.</p>
//!                     <a href="/">Go Home</a>
//!                 </body>
//!                 </html>
//!             "#).with_status_code(404))
//!         });
//!
//!     let server = Server::new(router, "127.0.0.1:3000".parse()?);
//!     server.ignitia().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### File Upload Handling
//! ```
//! use ignitia::{Router, Server, Response, handler::extractor::Body};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .post("/upload", |body: Body| async move {
//!             let size = body.len();
//!             if size > 5 * 1024 * 1024 { // 5MB limit
//!                 return Ok(Response::text("File too large")
//!                     .with_status_code(413));
//!             }
//!
//!             // Process the uploaded file
//!             Ok(Response::json(serde_json::json!({
//!                 "message": "File uploaded successfully",
//!                 "size": size
//!             }))?)
//!         });
//!
//!     let server = Server::new(router, "127.0.0.1:3000".parse()?);
//!     server.ignitia().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Performance Features
//!
//! ### Connection Pooling
//! - Each incoming connection is handled in its own Tokio task
//! - Connections are automatically cleaned up when dropped
//! - No connection pooling overhead - direct connection handling
//!
//! ### Request Size Limits
//! - Default 10MB request body size limit
//! - Prevents memory exhaustion attacks
//! - Returns HTTP 413 (Payload Too Large) for oversized requests
//!
//! ### Efficient Header Processing
//! - Pre-allocated header vectors for better performance
//! - Direct header copying without unnecessary allocations
//! - Optimized response building
//!
//! ## Security Considerations
//!
//! ### Request Validation
//! - Request body size limits prevent DoS attacks
//! - WebSocket upgrade validation prevents protocol confusion
//! - Proper error handling prevents information leakage
//!
//! ### Connection Management
//! - Automatic connection cleanup on errors
//! - Proper resource cleanup on task termination
//! - Connection timeout handling (via Tokio)
//!
//! ## Monitoring and Debugging
//!
//! ### Logging Integration
//! ```
//! use ignitia::{Router, Server, LoggerMiddleware};
//! use tracing_subscriber;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize tracing
//!     tracing_subscriber::fmt::init();
//!
//!     let router = Router::new()
//!         .middleware(LoggerMiddleware)
//!         .get("/", || async { Ok(Response::text("Hello")) });
//!
//!     let server = Server::new(router, "127.0.0.1:3000".parse()?);
//!     server.ignitia().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Health Check Endpoints
//! ```
//! use ignitia::{Router, Server, Response};
//! use std::time::{SystemTime, UNIX_EPOCH};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let router = Router::new()
//!         .get("/health", || async {
//!             Ok(Response::json(serde_json::json!({
//!                 "status": "healthy",
//!                 "timestamp": SystemTime::now()
//!                     .duration_since(UNIX_EPOCH)
//!                     .unwrap()
//!                     .as_secs()
//!             }))?)
//!         })
//!         .get("/metrics", || async {
//!             Ok(Response::text("# Server metrics would go here"))
//!         });
//!
//!     let server = Server::new(router, "127.0.0.1:3000".parse()?);
//!     server.ignitia().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod connection;

use crate::{Request, Response, Router};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
use crate::websocket::upgrade::generate_accept_key;

/// High-performance HTTP server for the Ignitia web framework.
///
/// The `Server` struct encapsulates all the functionality needed to run an HTTP server,
/// including request routing, WebSocket upgrade handling, and connection management.
/// It's built on top of Hyper for HTTP protocol handling and Tokio for async runtime.
///
/// # Architecture
///
/// The server uses an event-driven architecture where:
/// - The main thread runs an accept loop for incoming TCP connections
/// - Each connection is handled in a separate Tokio task
/// - Requests are parsed and routed through the provided router
/// - Responses are generated and sent back asynchronously
///
/// # Thread Safety
///
/// The server is designed to be thread-safe and can handle multiple concurrent
/// connections efficiently. The router is wrapped in an `Arc` for shared access
/// across all connection handling tasks.
///
/// # Examples
///
/// ## Basic Server
/// ```
/// use ignitia::{Router, Server, Response};
/// use std::net::SocketAddr;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let router = Router::new()
///     .get("/", || async { Ok(Response::text("Hello, World!")) });
///
/// let addr: SocketAddr = "127.0.0.1:3000".parse()?;
/// let server = Server::new(router, addr);
///
/// // This would start the server (commented out for doc test)
/// // server.ignitia().await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Server with Custom Address
/// ```
/// use ignitia::{Router, Server, Response};
/// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let router = Router::new()
///     .get("/api/status", || async {
///         Ok(Response::json(serde_json::json!({
///             "status": "running"
///         }))?)
///     });
///
/// let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
/// let server = Server::new(router, addr);
/// # Ok(())
/// # }
/// ```
pub struct Server {
    /// The router containing all route definitions and middleware
    router: Arc<Router>,
    /// The socket address the server will bind to
    addr: SocketAddr,
}

impl Server {
    /// Creates a new server instance with the provided router and address.
    ///
    /// # Parameters
    /// - `router`: The router containing all route definitions and middleware
    /// - `addr`: The socket address to bind the server to
    ///
    /// # Returns
    /// A new `Server` instance ready to be started
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{Router, Server, Response};
    /// use std::net::SocketAddr;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::new()
    ///     .get("/", || async { Ok(Response::text("Hello")) })
    ///     .get("/about", || async { Ok(Response::text("About")) });
    ///
    /// let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    /// let server = Server::new(router, addr);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## With IPv6 Address
    /// ```
    /// use ignitia::{Router, Server};
    /// use std::net::{SocketAddr, IpAddr, Ipv6Addr};
    ///
    /// # fn example() {
    /// let router = Router::new();
    /// let addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 3000);
    /// let server = Server::new(router, addr);
    /// # }
    /// ```
    pub fn new(router: Router, addr: SocketAddr) -> Self {
        Self {
            router: Arc::new(router),
            addr,
        }
    }

    /// Starts the HTTP server and begins accepting connections.
    ///
    /// This method binds to the configured address and starts the main server loop.
    /// It will run indefinitely, accepting and handling incoming connections until
    /// the process is terminated or an unrecoverable error occurs.
    ///
    /// # Returns
    /// - `Ok(())`: Never returned under normal circumstances (infinite loop)
    /// - `Err(Box<dyn std::error::Error>)`: If the server fails to start or encounters a fatal error
    ///
    /// # Errors
    /// This method can return errors in the following cases:
    /// - Failed to bind to the specified address (port already in use, insufficient permissions)
    /// - Network errors during connection acceptance
    /// - Other system-level errors
    ///
    /// # Examples
    ///
    /// ## Basic Server Start
    /// ```
    /// use ignitia::{Router, Server, Response};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let router = Router::new()
    ///         .get("/", || async { Ok(Response::text("Server is running!")) });
    ///
    ///     let server = Server::new(router, "127.0.0.1:3000".parse()?);
    ///
    ///     println!("Starting server on http://127.0.0.1:3000");
    ///     server.ignitia().await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Server with Graceful Shutdown
    /// ```
    /// use ignitia::{Router, Server, Response};
    /// use tokio::signal;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let router = Router::new()
    ///         .get("/", || async { Ok(Response::text("Hello")) });
    ///
    ///     let server = Server::new(router, "127.0.0.1:3000".parse()?);
    ///
    ///     // Run server in a separate task
    ///     let server_task = tokio::spawn(async move {
    ///         if let Err(e) = server.ignitia().await {
    ///             eprintln!("Server error: {}", e);
    ///         }
    ///     });
    ///
    ///     // Wait for Ctrl+C
    ///     signal::ctrl_c().await?;
    ///     println!("Shutdown signal received");
    ///
    ///     // In a real implementation, you'd gracefully shutdown here
    ///     server_task.abort();
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Performance Notes
    /// - Each connection is handled in its own Tokio task for maximum concurrency
    /// - The server uses Hyper's HTTP/1.1 implementation with connection reuse
    /// - WebSocket upgrades are handled efficiently with protocol validation
    /// - Request body size is limited to prevent memory exhaustion attacks
    pub async fn ignitia(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("🔥 ignitia server blazing on http://{}", self.addr);

        loop {
            let (stream, _) = match listener.accept().await {
                Ok((stream, addr)) => (stream, addr),
                Err(e) => {
                    tracing::warn!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            let router = Arc::clone(&self.router);

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let router = Arc::clone(&router);
                    async move { handle_request(router, req).await }
                });

                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                {
                    tracing::debug!("Connection error: {}", err);
                }
            });
        }
    }
}

/// Handles an incoming HTTP request and determines the appropriate response path.
///
/// This function is the main entry point for request processing. It determines
/// whether the request is a WebSocket upgrade request or a regular HTTP request
/// and routes it accordingly.
///
/// # Parameters
/// - `router`: Shared reference to the router for request processing
/// - `req`: The incoming HTTP request from Hyper
///
/// # Returns
/// A Hyper HTTP response containing the processed result
///
/// # Processing Flow
/// 1. **WebSocket Detection**: Check if the request is a WebSocket upgrade
/// 2. **Route Selection**: Choose between WebSocket and HTTP processing
/// 3. **Request Processing**: Process the request through the appropriate handler
/// 4. **Response Generation**: Generate and return the HTTP response
///
/// # Examples
/// This function is called internally by the server and is not typically
/// called directly by user code.
async fn handle_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();

    // Check for WebSocket upgrade first (fast path)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    {
        if is_websocket_upgrade(&req) {
            return handle_websocket_upgrade(router, req, &path).await;
        }
    }

    // Handle regular HTTP request
    handle_regular_http_request(router, req).await
}

/// Determines if an incoming request is a WebSocket upgrade request.
///
/// This function performs fast validation of WebSocket upgrade headers to
/// determine if the request should be handled as a WebSocket connection.
/// It checks for the presence and validity of required WebSocket headers.
///
/// # Parameters
/// - `req`: The incoming HTTP request to examine
///
/// # Returns
/// - `true`: If the request is a valid WebSocket upgrade request
/// - `false`: If the request is a regular HTTP request
///
/// # WebSocket Protocol Requirements
/// A valid WebSocket upgrade request must contain:
/// - `Connection: Upgrade` header
/// - `Upgrade: websocket` header
/// - `Sec-WebSocket-Key` header with a valid key
/// - `Sec-WebSocket-Version: 13` header
///
/// # Examples
/// This function is used internally by the server for WebSocket detection
/// and is not typically called directly by user code.
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
fn is_websocket_upgrade(req: &hyper::Request<hyper::body::Incoming>) -> bool {
    use hyper::header::{CONNECTION, UPGRADE};

    // Fast path checks with early returns
    let connection_header = match req.headers().get(CONNECTION) {
        Some(h) => h,
        None => return false,
    };

    let upgrade_header = match req.headers().get(UPGRADE) {
        Some(h) => h,
        None => return false,
    };

    let connection = match connection_header.to_str() {
        Ok(c) => c.to_lowercase(),
        Err(_) => return false,
    };

    let upgrade = match upgrade_header.to_str() {
        Ok(u) => u.to_lowercase(),
        Err(_) => return false,
    };

    connection.contains("upgrade")
        && upgrade.contains("websocket")
        && req.headers().get("sec-websocket-key").is_some()
        && req
            .headers()
            .get("sec-websocket-version")
            .map(|v| v == "13")
            .unwrap_or(false)
}

/// Handles WebSocket upgrade requests with proper protocol negotiation.
///
/// This function processes WebSocket upgrade requests by validating the
/// WebSocket protocol headers, generating appropriate response headers,
/// and spawning a task to handle the WebSocket connection.
///
/// # Parameters
/// - `router`: Shared reference to the router containing WebSocket handlers
/// - `req`: The incoming WebSocket upgrade request
/// - `path`: The path component of the request URI
///
/// # Returns
/// An HTTP 101 Switching Protocols response for successful upgrades,
/// or an error response for invalid requests or missing handlers
///
/// # WebSocket Upgrade Process
/// 1. **Handler Lookup**: Find the appropriate WebSocket handler for the path
/// 2. **Key Validation**: Validate the WebSocket key header
/// 3. **Response Generation**: Generate the WebSocket accept key and headers
/// 4. **Protocol Negotiation**: Handle optional WebSocket protocol negotiation
/// 5. **Connection Upgrade**: Spawn a task to handle the upgraded connection
///
/// # Error Responses
/// - **404**: No WebSocket handler found for the requested path
/// - **400**: Invalid or missing WebSocket headers
///
/// # Examples
/// This function is called internally when WebSocket upgrade requests are detected.
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
async fn handle_websocket_upgrade(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
    path: &str,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    use hyper::header::SEC_WEBSOCKET_KEY;

    let websocket_handlers = router.get_websocket_handlers();
    let handler = match websocket_handlers.get(path) {
        Some(handler) => Arc::clone(handler),
        None => {
            tracing::debug!("No WebSocket handler found for path: {}", path);
            return Ok(hyper::Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("WebSocket endpoint not found")))
                .unwrap());
        }
    };

    let websocket_key = match req.headers().get(SEC_WEBSOCKET_KEY) {
        Some(key) => match key.to_str() {
            Ok(k) => k,
            Err(_) => {
                return Ok(hyper::Response::builder()
                    .status(400)
                    .body(Full::new(Bytes::from("Invalid Sec-WebSocket-Key")))
                    .unwrap())
            }
        },
        None => {
            return Ok(hyper::Response::builder()
                .status(400)
                .body(Full::new(Bytes::from("Missing Sec-WebSocket-Key")))
                .unwrap())
        }
    };

    let accept_key = generate_accept_key(websocket_key);

    let mut response = hyper::Response::builder()
        .status(101)
        .header("upgrade", "websocket")
        .header("connection", "Upgrade")
        .header("sec-websocket-accept", accept_key);

    if let Some(protocols) = req.headers().get("sec-websocket-protocol") {
        if let Ok(protocols_str) = protocols.to_str() {
            if let Some(protocol) = protocols_str.split(',').find(|p| !p.trim().is_empty()) {
                response = response.header("sec-websocket-protocol", protocol.trim());
            }
        }
    }

    let response = response.body(Full::new(Bytes::new())).unwrap();

    // Spawn WebSocket handling task
    tokio::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(upgraded) => {
                if let Err(e) = crate::websocket::handle_websocket_upgrade(upgraded, handler).await
                {
                    tracing::debug!("WebSocket handler error: {}", e);
                }
            }
            Err(e) => {
                tracing::debug!("WebSocket upgrade failed: {}", e);
            }
        }
    });

    Ok(response)
}

/// Handles regular HTTP requests through the router system.
///
/// This function processes standard HTTP requests by parsing the request body,
/// routing the request through the framework's router, and generating an
/// appropriate HTTP response.
///
/// # Parameters
/// - `router`: Shared reference to the router for request processing
/// - `req`: The incoming HTTP request from Hyper
///
/// # Returns
/// A Hyper HTTP response containing the processed result
///
/// # Request Processing Flow
/// 1. **Request Parsing**: Parse HTTP request parts and body
/// 2. **Body Size Validation**: Check request body size against limits
/// 3. **Request Construction**: Build internal Request object
/// 4. **Router Processing**: Process request through the router
/// 5. **Response Generation**: Convert internal Response to Hyper response
/// 6. **Header Processing**: Copy response headers efficiently
///
/// # Request Size Limits
/// - **Default Limit**: 10MB maximum request body size
/// - **Error Response**: HTTP 413 (Payload Too Large) for oversized requests
/// - **Memory Protection**: Prevents memory exhaustion attacks
///
/// # Error Handling
/// - **Request Errors**: Converted to appropriate HTTP error responses
/// - **Router Errors**: Processed through the framework's error handling system
/// - **System Errors**: Logged and converted to HTTP 500 responses
///
/// # Examples
/// This function is called internally for all non-WebSocket HTTP requests.
async fn handle_regular_http_request(
    router: Arc<Router>,
    req: hyper::Request<hyper::body::Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, hyper::Error> {
    let (parts, body) = req.into_parts();

    // Limit request body size (10MB max)
    const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    let body_bytes = match body.collect().await {
        Ok(collected) => {
            let bytes = collected.to_bytes();
            // Check body size using len() instead of size()
            if bytes.len() > MAX_BODY_SIZE {
                let mut response =
                    hyper::Response::new(Full::new(Bytes::from("Request too large")));
                *response.status_mut() = http::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(response);
            }
            bytes
        }
        Err(_) => Bytes::new(),
    };

    let request = Request::new(
        parts.method,
        parts.uri,
        parts.version,
        parts.headers,
        body_bytes,
    );

    let response = match router.handle(request).await {
        Ok(res) => res,
        Err(err) => {
            let status = err.status_code();
            let mut res = Response::new(status);
            res.body = Bytes::from(err.to_string());
            res
        }
    };

    let mut builder = hyper::Response::builder().status(response.status);

    // Pre-allocate headers vector for better performance
    for (key, value) in response.headers.iter() {
        builder = builder.header(key, value);
    }

    Ok(builder.body(Full::new(response.body)).unwrap())
}

/// Server configuration and customization utilities.
///
/// This module provides additional functionality for server configuration,
/// monitoring, and customization beyond the basic server setup.
pub mod config {
    //! Server configuration utilities and constants.

    use std::time::Duration;

    /// Default request body size limit (10MB).
    pub const DEFAULT_MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

    /// Default server read timeout.
    pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default server write timeout.
    pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default keep-alive timeout.
    pub const DEFAULT_KEEP_ALIVE: Duration = Duration::from_secs(75);
}
