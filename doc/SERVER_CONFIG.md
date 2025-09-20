# Ignitia Server Configuration Guide 🔥

A complete guide to configuring high-performance servers with Ignitia - from basic HTTP servers to advanced multi-protocol deployments with HTTPS, HTTP/2, WebSockets, and enterprise-grade optimizations.

## Table of Contents

- [Quick Start](#quick-start)
- [Basic Server Setup](#basic-server-setup)
- [ServerConfig Structure](#serverconfig-structure)
- [HTTP/2 Configuration](#http2-configuration)
- [TLS/HTTPS Configuration](#tlshttps-configuration)
- [WebSocket Configuration](#websocket-configuration)
- [Performance Optimization](#performance-optimization)
- [Security Configuration](#security-configuration)
- [Advanced Patterns](#advanced-patterns)
- [Environment-Specific Configs](#environment-specific-configs)
- [Monitoring and Observability](#monitoring-and-observability)
- [Production Examples](#production-examples)
- [Troubleshooting](#troubleshooting)

## Quick Start

Get your Ignitia server running in seconds:

```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Hello, Ignitia! 🔥")) })
        .get("/health", || async {
            Ok(Response::json(serde_json::json!({
                "status": "healthy",
                "server": "ignitia",
                "version": env!("CARGO_PKG_VERSION")
            }))?)
        });

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr).ignitia().await
}
```

For production HTTPS with HTTP/2:

```rust
use ignitia::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Secure Ignitia! 🔒🔥")) });

    let addr = "0.0.0.0:8443".parse()?;
    Server::new(router, addr)
        .enable_https("cert.pem", "key.pem")?
        .with_performance_config(PerformanceConfig::max_rps())
        .ignitia()
        .await
}
```

## Basic Server Setup

### Simple HTTP Server

The most basic Ignitia server with default optimizations:

```rust
use ignitia::{Router, Server, Response, Result};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Hello, Ignitia! 🔥")) })
        .get("/health", health_check)
        .get("/metrics", metrics_handler);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    println!("🚀 Starting server on http://{}", addr);
    Server::new(router, addr).ignitia().await
}

async fn health_check() -> Result<Response> {
    Response::json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "uptime": get_uptime(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

### Server with Custom Configuration

Apply custom settings for specific requirements:

```rust
use ignitia::{Router, Server, ServerConfig, Http2Config, Result};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Configured Ignitia Server 🔧")) });

    let config = ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            enable_prior_knowledge: false,
            max_concurrent_streams: Some(1000),
            initial_connection_window_size: Some(1024 * 1024), // 1MB
            keep_alive_interval: Some(Duration::from_secs(60)),
            adaptive_window: true,
            ..Default::default()
        },
        auto_protocol_detection: true,
        max_request_body_size: 16 * 1024 * 1024, // 16MB
        ..Default::default()
    };

    let addr = "127.0.0.1:8080".parse()?;

    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

## ServerConfig Structure

### Complete Configuration Options

The `ServerConfig` struct provides comprehensive control over server behavior:

```rust
use ignitia::{ServerConfig, Http2Config, TlsConfig};
use std::time::Duration;

let config = ServerConfig {
    /// Enable HTTP/1.1 protocol support
    /// Default: true
    /// Set to false for HTTP/2-only servers
    http1_enabled: true,

    /// HTTP/2 protocol configuration
    /// Contains all HTTP/2-specific settings
    http2: Http2Config {
        /// Enable HTTP/2 protocol
        /// Default: true
        enabled: true,

        /// Enable HTTP/2 prior knowledge (H2C)
        /// Allows clients to start HTTP/2 without negotiation
        /// Default: false
        /// Set to true for testing with curl --http2-prior-knowledge
        enable_prior_knowledge: false,

        /// Maximum number of concurrent HTTP/2 streams per connection
        /// Default: Some(1000)
        /// Higher values allow more concurrent requests but use more memory
        max_concurrent_streams: Some(1000),

        /// Initial connection-level flow control window size
        /// Default: Some(1024 * 1024) // 1MB
        /// Larger values improve throughput for high-bandwidth connections
        initial_connection_window_size: Some(1024 * 1024),

        /// Initial stream-level flow control window size
        /// Default: Some(64 * 1024) // 64KB
        /// Affects per-request flow control
        initial_stream_window_size: Some(64 * 1024),

        /// Maximum HTTP/2 frame size
        /// Default: Some(16 * 1024) // 16KB
        /// Range: 16KB - 16MB
        max_frame_size: Some(16 * 1024),

        /// HTTP/2 keep-alive ping interval
        /// Default: Some(Duration::from_secs(60))
        /// Sends pings to detect dead connections
        keep_alive_interval: Some(Duration::from_secs(60)),

        /// HTTP/2 keep-alive timeout
        /// Default: Some(Duration::from_secs(20))
        /// How long to wait for ping responses
        keep_alive_timeout: Some(Duration::from_secs(20)),

        /// Enable adaptive flow control window sizing
        /// Default: true
        /// Automatically adjusts window sizes based on usage patterns
        adaptive_window: true,

        /// Maximum size of header lists (HPACK compressed)
        /// Default: Some(16 * 1024) // 16KB
        /// Prevents header-based DoS attacks
        max_header_list_size: Some(16 * 1024),
    },

    /// Enable automatic protocol detection
    /// Default: true
    /// Automatically chooses HTTP/1.1 or HTTP/2 based on client support
    auto_protocol_detection: true,

    /// Redirect HTTP requests to HTTPS
    /// Default: false
    /// When true, all HTTP requests receive 301 redirects to HTTPS
    redirect_http_to_https: false,

    /// HTTPS port for redirects
    /// Default: None
    /// Used when redirect_http_to_https is true
    https_port: None,

    /// Maximum request body size in bytes
    /// Default: 2 * 1024 * 1024 // 2MB
    /// Requests larger than this are rejected with 413 Payload Too Large
    max_request_body_size: 2 * 1024 * 1024,

    /// TLS configuration (when tls feature is enabled)
    /// Default: None
    /// Contains certificate and encryption settings
    #[cfg(feature = "tls")]
    tls: None,
};
```

### Configuration Builder Pattern

Use the builder pattern for cleaner configuration:

```rust
use ignitia::{ServerConfig, Http2Config};
use std::time::Duration;

let config = ServerConfig::new()
    .with_max_request_body_size(50 * 1024 * 1024) // 50MB
    .with_http2(Http2Config::new()
        .with_max_concurrent_streams(2000)
        .with_keep_alive(Duration::from_secs(30), Duration::from_secs(10))
        .with_adaptive_window(true)
        .build())
    .with_auto_protocol_detection(true)
    .build();

// Apply to server
Server::new(router, addr)
    .with_server_config(config)
    .ignitia()
    .await
```

### Environment-Based Configuration

Load configuration from environment variables:

```rust
use ignitia::{ServerConfig, Http2Config};
use std::env;
use std::time::Duration;

fn load_config_from_env() -> ServerConfig {
    let max_body_size = env::var("MAX_REQUEST_BODY_SIZE")
        .unwrap_or_else(|_| "16777216".to_string()) // 16MB default
        .parse()
        .unwrap_or(16 * 1024 * 1024);

    let max_streams = env::var("HTTP2_MAX_STREAMS")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .ok();

    let keep_alive_interval = env::var("HTTP2_KEEP_ALIVE_INTERVAL")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .map(Duration::from_secs)
        .ok();

    ServerConfig {
        http1_enabled: env::var("HTTP1_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),

        http2: Http2Config {
            enabled: env::var("HTTP2_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),

            enable_prior_knowledge: env::var("HTTP2_PRIOR_KNOWLEDGE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            max_concurrent_streams: max_streams,
            keep_alive_interval,
            adaptive_window: env::var("HTTP2_ADAPTIVE_WINDOW")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            ..Default::default()
        },

        max_request_body_size: max_body_size,
        auto_protocol_detection: env::var("AUTO_PROTOCOL_DETECTION")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true),

        redirect_http_to_https: env::var("REDIRECT_TO_HTTPS")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false),

        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config_from_env();
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Environment-configured server")) });

    Server::new(router, "0.0.0.0:8080".parse()?)
        .with_server_config(config)
        .ignitia()
        .await
}
```

## HTTP/2 Configuration

### Basic HTTP/2 Setup

Enable HTTP/2 with sensible defaults:

```rust
use ignitia::{ServerConfig, Http2Config};

let config = ServerConfig {
    http1_enabled: true,    // Support both protocols
    http2: Http2Config {
        enabled: true,
        enable_prior_knowledge: false,  // Set true for H2C testing
        ..Default::default()
    },
    auto_protocol_detection: true,  // Automatic protocol selection
    ..Default::default()
};

Server::new(router, addr)
    .with_server_config(config)
    .ignitia()
    .await
```

### High-Performance HTTP/2 Configuration

Optimized for maximum throughput:

```rust
use ignitia::{Http2Config, ServerConfig};
use std::time::Duration;

let http2_config = Http2Config {
    enabled: true,

    /// Enable H2C for development/testing
    /// curl --http2-prior-knowledge http://localhost:8080/
    enable_prior_knowledge: false, // Set true for development

    /// High concurrency settings
    max_concurrent_streams: Some(5000),

    /// Large flow control windows for high-bandwidth scenarios
    initial_connection_window_size: Some(4 * 1024 * 1024), // 4MB
    initial_stream_window_size: Some(2 * 1024 * 1024),     // 2MB

    /// Larger frames for bulk data transfer
    max_frame_size: Some(64 * 1024),  // 64KB (max: 16MB)

    /// Aggressive keep-alive for faster dead connection detection
    keep_alive_interval: Some(Duration::from_secs(15)),
    keep_alive_timeout: Some(Duration::from_secs(5)),

    /// Enable adaptive window sizing for dynamic adjustment
    adaptive_window: true,

    /// Large header buffer for complex APIs
    max_header_list_size: Some(64 * 1024), // 64KB
};

let config = ServerConfig {
    http1_enabled: true,
    http2: http2_config,
    auto_protocol_detection: true,
    max_request_body_size: 100 * 1024 * 1024, // 100MB for large uploads
    ..Default::default()
};
```

### HTTP/2 Only Server

Force HTTP/2 for all connections:

```rust
let config = ServerConfig {
    http1_enabled: false,   // Disable HTTP/1.1
    http2: Http2Config {
        enabled: true,
        enable_prior_knowledge: true,  // Required for HTTP/2 only
        ..Default::default()
    },
    auto_protocol_detection: false,  // No fallback needed
    ..Default::default()
};

println!("🚀 Starting HTTP/2-only server");
println!("Test with: curl --http2-prior-knowledge http://localhost:8080/");

Server::new(router, addr)
    .with_server_config(config)
    .ignitia()
    .await
```

### HTTP/2 with Server Push (Future Feature)

Configuration ready for HTTP/2 server push when implemented:

```rust
let http2_config = Http2Config {
    enabled: true,

    /// Settings optimized for server push scenarios
    max_concurrent_streams: Some(10000), // Higher for push streams
    initial_connection_window_size: Some(8 * 1024 * 1024), // 8MB

    /// Larger frames for pushing resources
    max_frame_size: Some(32 * 1024), // 32KB

    /// Future: Server push configuration
    /// enable_push: true,
    /// max_push_streams: Some(1000),

    ..Default::default()
};
```

### Testing HTTP/2 Configuration

Comprehensive HTTP/2 testing setup:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async {
            Ok(Response::html(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>HTTP/2 Test Server</title>
                    <style>body { font-family: monospace; padding: 2rem; }</style>
                </head>
                <body>
                    <h1>🚀 HTTP/2 Test Server</h1>
                    <h2>Test Commands:</h2>
                    <pre>
# HTTP/2 over TLS (recommended)
curl -v --http2 https://localhost:8443/

# HTTP/2 prior knowledge (H2C)
curl -v --http2-prior-knowledge http://localhost:8080/

# Check protocol version
curl -I --http2 https://localhost:8443/

# Multiple concurrent streams
curl --http2 -H "Connection: keep-alive" \
     -w "%{http_version}" \
     https://localhost:8443/api/test
                    </pre>
                    <p>Protocol: <span id="protocol">Unknown</span></p>
                    <script>
                        // Detect HTTP version in browser
                        if (window.chrome && chrome.loadTimes) {
                            const info = chrome.loadTimes();
                            document.getElementById('protocol').textContent =
                                info.connectionInfo || 'HTTP/1.x';
                        } else {
                            document.getElementById('protocol').textContent = 'Check DevTools Network tab';
                        }
                    </script>
                </body>
                </html>
            "#))
        })
        .get("/api/test", || async {
            Response::json(serde_json::json!({
                "message": "HTTP/2 API endpoint",
                "timestamp": chrono::Utc::now(),
                "features": ["multiplexing", "server_push_ready", "flow_control"]
            }))
        });

    let config = ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            enable_prior_knowledge: true, // Allow H2C testing
            max_concurrent_streams: Some(100),
            ..Default::default()
        },
        auto_protocol_detection: true,
        ..Default::default()
    };

    let addr = "127.0.0.1:8080".parse()?;
    println!("🧪 HTTP/2 test server starting on http://{}", addr);
    println!("📖 Visit http://{} for test commands", addr);

    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

## TLS/HTTPS Configuration

### Basic HTTPS Setup

Get HTTPS running quickly:

```rust
use ignitia::{Server, TlsConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Secure Ignitia! 🔒")) });

    let addr = "127.0.0.1:8443".parse()?;

    Server::new(router, addr)
        .enable_https("server.crt", "server.key")?
        .ignitia()
        .await
}
```

### Production TLS Configuration

Enterprise-grade TLS setup with security best practices:

```rust
#[cfg(feature = "tls")]
use ignitia::{TlsConfig, TlsVersion};

#[cfg(feature = "tls")]
#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Production HTTPS Server 🏢🔒")) });

    let tls_config = TlsConfig::new("production.crt", "production.key")
        /// ALPN protocols for HTTP/2 negotiation
        .with_alpn_protocols(vec!["h2", "http/1.1"])

        /// Support only modern TLS versions
        .with_protocol_versions(&[TlsVersion::TlsV12, TlsVersion::TlsV13])

        /// Use strong cipher suites
        .with_cipher_suites(&[
            "TLS_AES_256_GCM_SHA384",         // TLS 1.3
            "TLS_CHACHA20_POLY1305_SHA256",   // TLS 1.3
            "TLS_AES_128_GCM_SHA256",         // TLS 1.3
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384", // TLS 1.2
        ])

        /// Enable client certificate verification for mutual TLS
        .enable_client_cert_verification()

        /// Set certificate chain file
        .with_certificate_chain("cert_chain.pem")

        /// Configure OCSP stapling
        .enable_ocsp_stapling();

    let server_config = ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            max_concurrent_streams: Some(2000),
            initial_connection_window_size: Some(2 * 1024 * 1024), // 2MB
            keep_alive_interval: Some(Duration::from_secs(60)),
            adaptive_window: true,
            ..Default::default()
        },
        auto_protocol_detection: true,
        max_request_body_size: 50 * 1024 * 1024, // 50MB
        ..Default::default()
    };

    let addr = "0.0.0.0:8443".parse()?;

    println!("🔒 Starting production HTTPS server on https://{}", addr);

    Server::new(router, addr)
        .with_tls(tls_config)?
        .with_server_config(server_config)
        .ignitia()
        .await
}
```

### Development TLS with Self-Signed Certificates

Quick HTTPS setup for development:

```rust
#[cfg(all(feature = "tls", feature = "self-signed"))]
#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Development HTTPS 🛠️🔒")) });

    let addr = "127.0.0.1:8443".parse()?;

    println!("⚠️  Using self-signed certificate for development");
    println!("🔒 HTTPS server: https://{}", addr);
    println!("⚠️  Browser will show security warnings - this is expected");

    Server::new(router, addr)
        .with_self_signed_cert("localhost")?  // ⚠️ Development only!
        .ignitia()
        .await
}
```

### HTTP to HTTPS Redirect

Automatic redirection from HTTP to HTTPS:

```rust
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Redirected to HTTPS! ↗️🔒")) })
        .get("/api/status", || async {
            Response::json(serde_json::json!({
                "status": "secure",
                "protocol": "https",
                "timestamp": chrono::Utc::now()
            }))
        });

    // HTTPS server on port 8443
    let https_server = {
        let router = router.clone();
        let addr = "0.0.0.0:8443".parse()?;
        Server::new(router, addr)
            .enable_https("server.crt", "server.key")?
            .ignitia()
    };

    // HTTP redirect server on port 8080
    let redirect_server = {
        let router = Router::new(); // Empty router - all requests redirected
        let addr = "0.0.0.0:8080".parse()?;
        Server::new(router, addr)
            .redirect_to_https(8443)  // Redirect to HTTPS port
            .ignitia()
    };

    println!("🔒 HTTPS server: https://localhost:8443");
    println!("↗️  HTTP redirect: http://localhost:8080 -> https://localhost:8443");

    // Run both servers concurrently
    try_join!(https_server, redirect_server)?;
    Ok(())
}
```

### TLS Certificate Management

Advanced certificate handling:

```rust
#[cfg(feature = "tls")]
use ignitia::TlsConfig;
use std::path::Path;

#[cfg(feature = "tls")]
async fn setup_tls_with_validation() -> Result<TlsConfig> {
    let cert_path = "certs/server.crt";
    let key_path = "certs/server.key";
    let ca_path = "certs/ca.crt";

    // Validate certificate files exist and are readable
    for path in &[cert_path, key_path, ca_path] {
        if !Path::new(path).exists() {
            return Err(format!("Certificate file not found: {}", path).into());
        }
    }

    // Load and validate certificate
    let cert_content = tokio::fs::read_to_string(cert_path).await?;
    let key_content = tokio::fs::read_to_string(key_path).await?;

    // Basic certificate validation
    if !cert_content.contains("-----BEGIN CERTIFICATE-----") {
        return Err("Invalid certificate format".into());
    }
    if !key_content.contains("-----BEGIN PRIVATE KEY-----") &&
       !key_content.contains("-----BEGIN RSA PRIVATE KEY-----") {
        return Err("Invalid private key format".into());
    }

    let tls_config = TlsConfig::new(cert_path, key_path)
        .with_ca_file(ca_path)
        .with_alpn_protocols(vec!["h2", "http/1.1"])
        .with_protocol_versions(&[TlsVersion::TlsV12, TlsVersion::TlsV13])
        .enable_client_cert_verification();

    println!("✅ TLS certificate validation passed");
    Ok(tls_config)
}

#[cfg(feature = "tls")]
#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Validated TLS Server ✅🔒")) });

    let tls_config = setup_tls_with_validation().await?;
    let addr = "0.0.0.0:8443".parse()?;

    Server::new(router, addr)
        .with_tls(tls_config)?
        .ignitia()
        .await
}
```

## WebSocket Configuration

### Basic WebSocket Server

Simple WebSocket echo server:

```rust
#[cfg(feature = "websocket")]
use ignitia::websocket::{websocket_handler, Message, WebSocketConnection};

#[cfg(feature = "websocket")]
#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async {
            Ok(Response::html(r#"
                <!DOCTYPE html>
                <html>
                <head><title>WebSocket Test</title></head>
                <body>
                    <h1>WebSocket Echo Test 📡</h1>
                    <div id="messages"></div>
                    <input type="text" id="messageInput" placeholder="Type message...">
                    <button onclick="sendMessage()">Send</button>

                    <script>
                        const ws = new WebSocket('ws://localhost:8080/ws');
                        const messages = document.getElementById('messages');

                        ws.onopen = () => {
                            messages.innerHTML += '<p>✅ Connected to WebSocket</p>';
                        };

                        ws.onmessage = (event) => {
                            messages.innerHTML += `<p>📨 ${event.data}</p>`;
                        };

                        ws.onclose = () => {
                            messages.innerHTML += '<p>❌ Connection closed</p>';
                        };

                        function sendMessage() {
                            const input = document.getElementById('messageInput');
                            if (input.value.trim()) {
                                ws.send(input.value);
                                messages.innerHTML += `<p>📤 Sent: ${input.value}</p>`;
                                input.value = '';
                            }
                        }

                        document.getElementById('messageInput')
                            .addEventListener('keypress', (e) => {
                                if (e.key === 'Enter') sendMessage();
                            });
                    </script>
                </body>
                </html>
            "#))
        })
        .websocket("/ws", websocket_handler(|mut ws: WebSocketConnection| async move {
            println!("🔌 WebSocket connection established");

            while let Some(message) = ws.recv().await {
                match message {
                    Message::Text(text) => {
                        println!("📨 Received: {}", text);
                        let response = format!("Echo: {}", text);
                        if let Err(e) = ws.send_text(response).await {
                            println!("❌ Failed to send message: {}", e);
                            break;
                        }
                    }
                    Message::Binary(data) => {
                        println!("📨 Received {} bytes of binary data", data.len());
                        if let Err(e) = ws.send_bytes(data).await {
                            println!("❌ Failed to send binary data: {}", e);
                            break;
                        }
                    }
                    Message::Ping(data) => {
                        println!("🏓 Received ping, sending pong");
                        if let Err(e) = ws.send_pong(data).await {
                            println!("❌ Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Message::Close(frame) => {
                        println!("🔌 Connection closed: {:?}", frame);
                        break;
                    }
                    _ => {}
                }
            }

            println!("🔌 WebSocket connection terminated");
            Ok(())
        }));

    let addr = "127.0.0.1:8080".parse()?;
    println!("📡 WebSocket server starting on ws://{}/ws", addr);
    println!("🌐 Web interface: http://{}", addr);

    Server::new(router, addr).ignitia().await
}
```

### Production WebSocket with HTTPS

Secure WebSocket server with authentication:

```rust
#[cfg(all(feature = "websocket", feature = "tls"))]
use ignitia::websocket::{websocket_handler, Message, WebSocketConnection};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[cfg(all(feature = "websocket", feature = "tls"))]
#[derive(Clone)]
struct WebSocketState {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    message_count: Arc<RwLock<u64>>,
}

#[cfg(all(feature = "websocket", feature = "tls"))]
impl WebSocketState {
    fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            message_count: Arc::new(RwLock::new(0)),
        }
    }

    fn add_connection(&self, id: String, ws: WebSocketConnection) {
        self.connections.write().unwrap().insert(id, ws);
    }

    fn remove_connection(&self, id: &str) {
        self.connections.write().unwrap().remove(id);
    }

    fn broadcast(&self, message: &str) -> usize {
        let connections = self.connections.read().unwrap();
        let mut sent_count = 0;
        let mut failed_connections = Vec::new();

        for (id, ws) in connections.iter() {
            match ws.send_text(message.to_string()) {
                Ok(_) => sent_count += 1,
                Err(_) => failed_connections.push(id.clone()),
            }
        }

        // Remove failed connections (would need async context in real impl)
        drop(connections);
        for id in failed_connections {
            self.remove_connection(&id);
        }

        sent_count
    }

    fn increment_message_count(&self) {
        let mut count = self.message_count.write().unwrap();
        *count += 1;
    }

    fn get_stats(&self) -> (usize, u64) {
        let connections = self.connections.read().unwrap().len();
        let messages = *self.message_count.read().unwrap();
        (connections, messages)
    }
}

#[cfg(all(feature = "websocket", feature = "tls"))]
#[tokio::main]
async fn main() -> Result<()> {
    let ws_state = WebSocketState::new();

    let router = Router::new()
        .get("/", || async {
            Ok(Response::html(include_str!("websocket_dashboard.html")))
        })
        .get("/stats", {
            let state = ws_state.clone();
            move || {
                let state = state.clone();
                async move {
                    let (connections, messages) = state.get_stats();
                    Response::json(serde_json::json!({
                        "active_connections": connections,
                        "total_messages": messages,
                        "timestamp": chrono::Utc::now()
                    }))
                }
            }
        })
        .websocket("/ws", {
            let state = ws_state.clone();
            websocket_handler(move |ws: WebSocketConnection| {
                let state = state.clone();
                async move {
                    let client_id = Uuid::new_v4().to_string();
                    println!("🔌 New WebSocket connection: {}", client_id);

                    // Add to active connections
                    state.add_connection(client_id.clone(), ws.clone());

                    // Send welcome message
                    let welcome = serde_json::json!({
                        "type": "welcome",
                        "client_id": client_id,
                        "message": "Connected to secure WebSocket server",
                        "timestamp": chrono::Utc::now()
                    });
                    ws.send_text(welcome.to_string()).await?;

                    // Handle messages
                    while let Some(message) = ws.recv().await {
                        match message {
                            Message::Text(text) => {
                                state.increment_message_count();
                                println!("📨 [{}] Received: {}", client_id, text);

                                // Parse as JSON for structured communication
                                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                                    match msg["type"].as_str() {
                                        Some("broadcast") => {
                                            let content = msg["content"].as_str().unwrap_or("");
                                            let broadcast_msg = serde_json::json!({
                                                "type": "broadcast",
                                                "from": client_id,
                                                "content": content,
                                                "timestamp": chrono::Utc::now()
                                            });
                                            let sent = state.broadcast(&broadcast_msg.to_string());
                                            println!("📡 Broadcast sent to {} connections", sent);
                                        }
                                        Some("ping") => {
                                            let pong = serde_json::json!({
                                                "type": "pong",
                                                "timestamp": chrono::Utc::now()
                                            });
                                            ws.send_text(pong.to_string()).await?;
                                        }
                                        _ => {
                                            // Echo unknown messages
                                            let echo = serde_json::json!({
                                                "type": "echo",
                                                "original": msg,
                                                "timestamp": chrono::Utc::now()
                                            });
                                            ws.send_text(echo.to_string()).await?;
                                        }
                                    }
                                }
                            }
                            Message::Binary(data) => {
                                println!("📦 [{}] Received {} bytes", client_id, data.len());
                                ws.send_bytes(data).await?;
                            }
                            Message::Close(frame) => {
                                println!("🔌 [{}] Connection closed: {:?}", client_id, frame);
                                break;
                            }
                            _ => {}
                        }
                    }

                    // Cleanup
                    state.remove_connection(&client_id);
                    println!("🧹 [{}] Connection cleaned up", client_id);

                    Ok(())
                }
            })
        });

    // Server configuration with TLS
    let config = ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            max_concurrent_streams: Some(1000),
            ..Default::default()
        },
        auto_protocol_detection: true,
        max_request_body_size: 10 * 1024 * 1024, // 10MB
        ..Default::default()
    };

    let addr = "0.0.0.0:8443".parse()?;

    println!("🔒📡 Secure WebSocket server starting on wss://{}/ws", addr);
    println!("🌐 Dashboard: https://{}", addr);
    println!("📊 Stats API: https://{}/stats", addr);

    Server::new(router, addr)
        .enable_https("server.crt", "server.key")?
        .with_server_config(config)
        .ignitia()
        .await
}
```

### WebSocket with HTTP/2 Support

WebSocket over HTTP/2 connections:

```rust
#[cfg(feature = "websocket")]
#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .websocket("/ws", websocket_handler(|ws| async move {
            println!("📡 WebSocket over HTTP/2 connection");
            // Handle WebSocket messages
            Ok(())
        }));

    let config = ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            enable_prior_knowledge: true,
            max_concurrent_streams: Some(2000),
            initial_connection_window_size: Some(4 * 1024 * 1024), // 4MB
            adaptive_window: true,
            ..Default::default()
        },
        auto_protocol_detection: true,
        ..Default::default()
    };

    let addr = "127.0.0.1:8080".parse()?;
    println!("📡 WebSocket + HTTP/2 server on ws://{}/ws", addr);

    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

## Performance Optimization

### Maximum Performance Configuration

Squeeze every bit of performance from your server:

```rust
use ignitia::{ServerConfig, Http2Config, PerformanceConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Maximum Performance! ⚡🔥")) })
        .get("/api/fast", || async {
            Response::json(serde_json::json!({
                "status": "blazing_fast",
                "rps_target": "65000+",
                "optimizations": ["http2", "zero_copy", "connection_pooling"]
            }))
        });

    // Maximum performance server configuration
    let server_config = ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,

            // Ultra-high concurrency
            max_concurrent_streams: Some(10000),

            // Large flow control windows for maximum throughput
            initial_connection_window_size: Some(16 * 1024 * 1024), // 16MB
            initial_stream_window_size: Some(4 * 1024 * 1024),      // 4MB

            // Maximum frame size for bulk transfers
            max_frame_size: Some(1024 * 1024), // 1MB (near maximum)

            // Aggressive keep-alive for fastest dead connection detection
            keep_alive_interval: Some(Duration::from_secs(10)),
            keep_alive_timeout: Some(Duration::from_secs(3)),

            // Enable all performance features
            adaptive_window: true,

            // Large header buffer for complex APIs
            max_header_list_size: Some(128 * 1024), // 128KB
        },

        auto_protocol_detection: true,

        // Large request bodies for file uploads
        max_request_body_size: 1024 * 1024 * 1024, // 1GB

        ..Default::default()
    };

    // Maximum performance socket configuration
    let perf_config = PerformanceConfig::max_rps()
        .with_backlog(32768)  // Maximum connection backlog
        .with_buffer_sizes(
            2 * 1024 * 1024,  // 2MB send buffer
            1024 * 1024       // 1MB receive buffer
        )
        .with_worker_threads(num_cpus::get() * 4)  // More threads for I/O
        .enable_zero_copy()   // Enable zero-copy optimizations
        .enable_cpu_affinity(); // Pin threads to CPU cores

    let addr = "0.0.0.0:8080".parse()?;

    println!("⚡ Maximum performance server starting on http://{}", addr);
    println!("🎯 Target: 65,000+ RPS");
    println!("💾 Max request body: 1GB");
    println!("🔧 Optimizations: All enabled");

    Server::new(router, addr)
        .with_server_config(server_config)
        .with_performance_config(perf_config)
        .ignitia()
        .await
}
```

### Memory-Optimized Configuration

Minimize memory usage while maintaining good performance:

```rust
let config = ServerConfig {
    http1_enabled: true,
    http2: Http2Config {
        enabled: true,

        // Conservative concurrency limits
        max_concurrent_streams: Some(100),

        // Smaller flow control windows
        initial_connection_window_size: Some(64 * 1024),    // 64KB
        initial_stream_window_size: Some(32 * 1024),       // 32KB

        // Smaller frame size to reduce buffering
        max_frame_size: Some(8 * 1024),  // 8KB

        // Longer intervals to reduce overhead
        keep_alive_interval: Some(Duration::from_secs(120)),
        keep_alive_timeout: Some(Duration::from_secs(30)),

        // Disable adaptive window to save memory
        adaptive_window: false,

        // Smaller header buffer
        max_header_list_size: Some(4 * 1024), // 4KB
    },

    // Smaller request body limit
    max_request_body_size: 1024 * 1024, // 1MB

    ..Default::default()
};

let perf_config = PerformanceConfig::memory_constrained()
    .with_backlog(1024)    // Smaller backlog
    .with_buffer_sizes(
        32 * 1024,  // 32KB send buffer
        16 * 1024   // 16KB receive buffer
    )
    .with_worker_threads(2); // Minimal thread pool

println!("💾 Memory-optimized server configuration loaded");
```

### Latency-Optimized Configuration

Minimize response time for real-time applications:

```rust
let config = ServerConfig {
    http1_enabled: true,
    http2: Http2Config {
        enabled: true,

        // Moderate concurrency for low latency
        max_concurrent_streams: Some(1000),

        // Balanced flow control
        initial_connection_window_size: Some(512 * 1024),  // 512KB
        initial_stream_window_size: Some(128 * 1024),     // 128KB

        // Smaller frames for faster transmission
        max_frame_size: Some(8 * 1024),  // 8KB

        // Very aggressive keep-alive for immediate dead connection detection
        keep_alive_interval: Some(Duration::from_secs(5)),
        keep_alive_timeout: Some(Duration::from_secs(1)),

        // Enable adaptive window for responsiveness
        adaptive_window: true,

        max_header_list_size: Some(16 * 1024), // 16KB
    },

    max_request_body_size: 10 * 1024 * 1024, // 10MB

    ..Default::default()
};

let perf_config = PerformanceConfig::low_latency()
    .with_tcp_nodelay(true)  // Disable Nagle's algorithm
    .with_keep_alive(Duration::from_secs(1)) // Very aggressive keep-alive
    .with_buffer_sizes(
        128 * 1024,  // 128KB send buffer
        64 * 1024    // 64KB receive buffer
    )
    .enable_zero_copy()      // Reduce copy operations
    .enable_cpu_affinity();  // Consistent performance

println!("🚀 Latency-optimized server configuration loaded");
println!("🎯 Target: Sub-millisecond response times");
```

## Security Configuration

### Production Security Configuration

Enterprise-grade security settings:

```rust
#[cfg(feature = "tls")]
use ignitia::{TlsConfig, TlsVersion, CipherSuite};

#[cfg(feature = "tls")]
fn create_secure_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,

            // Security-focused limits to prevent DoS attacks
            max_concurrent_streams: Some(1000),        // Prevent stream exhaustion
            initial_connection_window_size: Some(1024 * 1024), // 1MB max
            initial_stream_window_size: Some(64 * 1024),       // 64KB max
            max_frame_size: Some(16 * 1024),          // 16KB prevents large frame attacks
            max_header_list_size: Some(8 * 1024),     // 8KB prevents header bombs

            // Conservative keep-alive to reduce attack surface
            keep_alive_interval: Some(Duration::from_secs(60)),
            keep_alive_timeout: Some(Duration::from_secs(20)),

            adaptive_window: true,
        },

        auto_protocol_detection: true,

        // Strict request size limits
        max_request_body_size: 50 * 1024 * 1024, // 50MB maximum

        // Force HTTPS in production
        redirect_http_to_https: true,
        https_port: Some(443),

        ..Default::default()
    }
}

#[cfg(feature = "tls")]
fn create_secure_tls_config() -> Result<TlsConfig> {
    let tls_config = TlsConfig::new("production.crt", "production.key")
        // Only support modern TLS versions
        .with_protocol_versions(&[TlsVersion::TlsV12, TlsVersion::TlsV13])

        // Use only strong cipher suites
        .with_cipher_suites(&[
            // TLS 1.3 cipher suites (preferred)
            "TLS_AES_256_GCM_SHA384",
            "TLS_CHACHA20_POLY1305_SHA256",
            "TLS_AES_128_GCM_SHA256",

            // TLS 1.2 cipher suites (fallback)
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
            "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
        ])

        // Enable ALPN for HTTP/2
        .with_alpn_protocols(vec!["h2", "http/1.1"])

        // Security enhancements
        .enable_ocsp_stapling()           // Certificate revocation checking
        .enable_client_cert_verification() // Mutual TLS (optional)
        .with_session_timeout(Duration::from_secs(300)) // 5-minute sessions
        .disable_session_resumption()     // Force fresh handshakes
        .enable_perfect_forward_secrecy(); // PFS for all connections

    Ok(tls_config)
}

#[cfg(feature = "tls")]
#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Secure Production Server 🛡️")) })
        .get("/security", || async {
            Response::json(serde_json::json!({
                "tls_version": "1.3",
                "cipher_suite": "TLS_AES_256_GCM_SHA384",
                "perfect_forward_secrecy": true,
                "hsts_enabled": true,
                "security_headers": true
            }))
        });

    let server_config = create_secure_config();
    let tls_config = create_secure_tls_config()?;

    let addr = "0.0.0.0:8443".parse()?;

    println!("🛡️ Secure production server starting on https://{}", addr);
    println!("🔒 TLS 1.2/1.3 only, strong cipher suites");
    println!("🚫 HTTP disabled, HTTPS redirects enabled");

    Server::new(router, addr)
        .with_server_config(server_config)
        .with_tls(tls_config)?
        .ignitia()
        .await
}
```

### Security Middleware Integration

Add security middleware to your server configuration:

```rust
use ignitia::{
    Router, SecurityMiddleware, RateLimitingMiddleware,
    CorsMiddleware, LoggerMiddleware
};

fn create_secure_router() -> Router {
    Router::new()
        // Security middleware (first in chain)
        .middleware(SecurityMiddleware::strict()
            .enable_hsts(Duration::from_secs(31536000)) // 1 year HSTS
            .content_security_policy("default-src 'self'; script-src 'self' 'unsafe-inline'")
            .frame_options("DENY")
            .content_type_options("nosniff")
            .referrer_policy("strict-origin-when-cross-origin"))

        // Rate limiting
        .middleware(RateLimitingMiddleware::new()
            .requests_per_minute(1000)
            .burst_size(100)
            .per_ip()
            .with_whitelist(vec!["127.0.0.1", "::1"]))

        // CORS configuration
        .middleware(CorsMiddleware::secure_api(&[
            "https://myapp.com",
            "https://admin.myapp.com"
        ]).build()?)

        // Audit logging
        .middleware(LoggerMiddleware::security_audit()
            .log_request_headers()
            .log_user_agent()
            .log_client_ip())

        // Application routes
        .get("/", || async { Ok(Response::text("Secure API")) })
        .post("/api/auth", auth_handler)
        .get("/api/protected", protected_handler);

    router
}
```

## Advanced Patterns

### Multi-Protocol Server

Support multiple protocols on different ports:

```rust
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<()> {
    let base_router = Router::new()
        .get("/", || async { Ok(Response::text("Multi-Protocol Server 🌐")) })
        .get("/health", || async {
            Response::json(serde_json::json!({
                "status": "healthy",
                "protocols": ["http/1.1", "http/2", "websocket"],
                "ports": {
                    "http": 8080,
                    "https": 8443,
                    "websocket": 8082
                }
            }))
        });

    // HTTP/1.1 only server (legacy support)
    let http1_server = {
        let router = base_router.clone();
        let config = ServerConfig {
            http1_enabled: true,
            http2: Http2Config { enabled: false, ..Default::default() },
            auto_protocol_detection: false,
            ..Default::default()
        };
        let addr = "127.0.0.1:8080".parse()?;
        Server::new(router, addr).with_server_config(config).ignitia()
    };

    // HTTP/2 + HTTPS server (modern clients)
    let https_server = {
        let router = base_router.clone();
        let config = ServerConfig {
            http1_enabled: true,
            http2: Http2Config {
                enabled: true,
                max_concurrent_streams: Some(2000),
                ..Default::default()
            },
            auto_protocol_detection: true,
            ..Default::default()
        };
        let addr = "127.0.0.1:8443".parse()?;
        Server::new(router, addr)
            .with_server_config(config)
            .enable_https("server.crt", "server.key")?
            .ignitia()
    };

    // WebSocket server (real-time features)
    #[cfg(feature = "websocket")]
    let ws_server = {
        let ws_router = Router::new()
            .websocket("/ws", websocket_handler(|ws| async move {
                println!("📡 WebSocket connection on dedicated port");
                Ok(())
            }));
        let addr = "127.0.0.1:8082".parse()?;
        Server::new(ws_router, addr).ignitia()
    };

    println!("🌐 Multi-protocol server starting:");
    println!("   HTTP/1.1: http://127.0.0.1:8080");
    println!("   HTTPS/2:  https://127.0.0.1:8443");
    println!("   WebSocket: ws://127.0.0.1:8082/ws");

    // Run all servers concurrently
    #[cfg(feature = "websocket")]
    try_join!(http1_server, https_server, ws_server)?;

    #[cfg(not(feature = "websocket"))]
    try_join!(http1_server, https_server)?;

    Ok(())
}
```

### Load Balancer Configuration

Configure Ignitia behind a load balancer:

```rust
fn create_load_balanced_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,

            // Optimize for load balancer scenarios
            max_concurrent_streams: Some(500),  // Per backend instance
            keep_alive_interval: Some(Duration::from_secs(30)), // LB health checks
            keep_alive_timeout: Some(Duration::from_secs(10)),

            // Conservative flow control for predictable performance
            initial_connection_window_size: Some(1024 * 1024), // 1MB
            initial_stream_window_size: Some(128 * 1024),     // 128KB

            adaptive_window: true,
            ..Default::default()
        },

        auto_protocol_detection: true,

        // Handle X-Forwarded-* headers from load balancer
        trust_proxy_headers: true,

        // Reasonable limits per backend instance
        max_request_body_size: 32 * 1024 * 1024, // 32MB

        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new()
        .middleware(ProxyHeadersMiddleware::new()
            .trust_x_forwarded_for()
            .trust_x_forwarded_proto()
            .trust_x_forwarded_host())
        .get("/", || async { Ok(Response::text("Load Balanced Instance")) })
        .get("/health", || async {
            Response::json(serde_json::json!({
                "status": "healthy",
                "instance_id": std::env::var("INSTANCE_ID").unwrap_or("unknown".into()),
                "load_balanced": true
            }))
        });

    let config = create_load_balanced_config();
    let addr = "0.0.0.0:8080".parse()?; // Bind to all interfaces for LB

    println!("⚖️ Load-balanced server instance starting");
    println!("🌐 Listening on {}", addr);

    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

### Microservice Configuration

Optimized configuration for microservice deployments:

```rust
fn create_microservice_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,

            // Microservice-optimized settings
            max_concurrent_streams: Some(200),  // Conservative for small services
            initial_connection_window_size: Some(512 * 1024), // 512KB
            initial_stream_window_size: Some(64 * 1024),     // 64KB

            // Faster health check detection
            keep_alive_interval: Some(Duration::from_secs(15)),
            keep_alive_timeout: Some(Duration::from_secs(5)),

            adaptive_window: true,
            max_header_list_size: Some(8 * 1024), // 8KB for API headers
        },

        auto_protocol_detection: true,

        // Small request bodies for microservice APIs
        max_request_body_size: 5 * 1024 * 1024, // 5MB

        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service_name = env::var("SERVICE_NAME").unwrap_or_else(|_| "ignitia-service".to_string());
    let service_version = env::var("SERVICE_VERSION").unwrap_or_else(|_| "1.0.0".to_string());

    let router = Router::new()
        .middleware(LoggerMiddleware::microservice(&service_name))
        .middleware(RateLimitingMiddleware::per_minute(500)) // Conservative limit
        .get("/health", || async {
            Response::json(serde_json::json!({
                "status": "healthy",
                "service": env::var("SERVICE_NAME").unwrap_or_else(|_| "unknown".to_string()),
                "version": env::var("SERVICE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
                "timestamp": chrono::Utc::now()
            }))
        })
        .get("/metrics", prometheus_metrics_handler)
        .get("/ready", readiness_check_handler)
        .nest("/api/v1", create_api_routes());

    let config = create_microservice_config();
    let addr = "0.0.0.0:8080".parse()?;

    println!("🔬 Microservice '{}' v{} starting", service_name, service_version);
    println!("🌐 API: http://{}/api/v1", addr);
    println!("❤️ Health: http://{}/health", addr);

    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

## Environment-Specific Configs

### Production Configuration

Battle-tested configuration for production deployments:

```rust
pub fn production_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,

            // Production-optimized HTTP/2 settings
            max_concurrent_streams: Some(2000),
            initial_connection_window_size: Some(2 * 1024 * 1024), // 2MB
            initial_stream_window_size: Some(1024 * 1024),         // 1MB
            max_frame_size: Some(32 * 1024),                       // 32KB

            // Production keep-alive settings
            keep_alive_interval: Some(Duration::from_secs(60)),
            keep_alive_timeout: Some(Duration::from_secs(20)),

            adaptive_window: true,
            max_header_list_size: Some(32 * 1024), // 32KB for complex APIs
        },

        auto_protocol_detection: true,

        // Production security: Force HTTPS
        redirect_http_to_https: true,
        https_port: Some(443),

        // Production request limits
        max_request_body_size: 100 * 1024 * 1024, // 100MB for file uploads

        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load production configuration
    let config = production_config();

    let router = Router::new()
        .middleware(SecurityMiddleware::production())
        .middleware(LoggerMiddleware::json_structured())
        .middleware(CorsMiddleware::production_api())
        .get("/", || async { Ok(Response::text("Production Ignitia Server 🏭")) });

    let addr = "0.0.0.0:8443".parse()?;

    println!("🏭 Production server starting on https://{}", addr);

    Server::new(router, addr)
        .with_server_config(config)
        .enable_https("production.crt", "production.key")?
        .with_performance_config(PerformanceConfig::max_rps())
        .ignitia()
        .await
}
```

### Development Configuration

Developer-friendly configuration with debugging features:

```rust
pub fn development_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            enable_prior_knowledge: true,  // Enable H2C for curl testing
            max_concurrent_streams: Some(500),
            ..Default::default()
        },

        auto_protocol_detection: true,

        // Development: Allow HTTP for easier testing
        redirect_http_to_https: false,

        // Generous limits for development
        max_request_body_size: 500 * 1024 * 1024, // 500MB

        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Enable debug logging in development
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "ignitia=debug,tower_http=debug");
    }
    tracing_subscriber::init();

    let config = development_config();

    let router = Router::new()
        .middleware(LoggerMiddleware::development())
        .middleware(CorsMiddleware::permissive()) // Allow all origins in dev
        .get("/", || async { Ok(Response::text("Development Ignitia Server 🛠️")) })
        .get("/debug", || async {
            Response::json(serde_json::json!({
                "mode": "development",
                "debug_enabled": true,
                "cors": "permissive",
                "h2c_enabled": true,
                "features": ["hot_reload", "debug_logs", "permissive_cors"]
            }))
        });

    let addr = "127.0.0.1:8080".parse()?;

    println!("🛠️ Development server starting on http://{}", addr);
    println!("🧪 H2C testing: curl --http2-prior-knowledge http://{}/", addr);
    println!("🐛 Debug endpoint: http://{}/debug", addr);

    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

### Testing Configuration

Minimal configuration optimized for automated tests:

```rust
pub fn test_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: false,  // Disable HTTP/2 for simpler testing
            ..Default::default()
        },
        auto_protocol_detection: false,
        redirect_http_to_https: false,
        max_request_body_size: 10 * 1024 * 1024, // 10MB
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_startup() {
        let router = Router::new()
            .get("/test", || async { Ok(Response::text("Test response")) });

        let config = test_config();
        let addr = "127.0.0.1:0".parse().unwrap(); // Random port

        // Start server in background
        let server_handle = tokio::spawn(async move {
            Server::new(router, addr)
                .with_server_config(config)
                .ignitia()
                .await
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Test client request
        let client = reqwest::Client::new();
        let response = client
            .get("http://127.0.0.1:8080/test")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(response.text().await.unwrap(), "Test response");

        // Cleanup
        server_handle.abort();
    }
}
```

## Monitoring and Observability

### Metrics Collection Configuration

Integrate comprehensive monitoring:

```rust
use prometheus::{Counter, Histogram, Registry, Encoder, TextEncoder};
use std::sync::Arc;

#[derive(Clone)]
struct ServerMetrics {
    request_counter: Counter,
    request_duration: Histogram,
    registry: Arc<Registry>,
}

impl ServerMetrics {
    fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let request_counter = Counter::new("http_requests_total", "Total HTTP requests")
            .expect("Failed to create counter");

        let request_duration = Histogram::new("http_request_duration_seconds", "HTTP request duration")
            .expect("Failed to create histogram");

        registry.register(Box::new(request_counter.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();

        Self {
            request_counter,
            request_duration,
            registry,
        }
    }

    fn record_request(&self, duration: Duration) {
        self.request_counter.inc();
        self.request_duration.observe(duration.as_secs_f64());
    }

    fn export_metrics(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families).unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let metrics = ServerMetrics::new();

    let router = Router::new()
        .middleware(MetricsMiddleware::new(metrics.clone()))
        .get("/", || async { Ok(Response::text("Monitored Ignitia Server 📊")) })
        .get("/metrics", {
            let metrics = metrics.clone();
            move || {
                let metrics = metrics.clone();
                async move {
                    Ok(Response::text(metrics.export_metrics())
                        .with_header("Content-Type", "text/plain; version=0.0.4"))
                }
            }
        })
        .get("/health", || async {
            Response::json(serde_json::json!({
                "status": "healthy",
                "monitoring": "enabled",
                "metrics_endpoint": "/metrics"
            }))
        });

    let config = ServerConfig {
        http2: Http2Config {
            enabled: true,
            max_concurrent_streams: Some(1000),
            ..Default::default()
        },
        ..Default::default()
    };

    let addr = "0.0.0.0:8080".parse()?;

    println!("📊 Monitored server starting on http://{}", addr);
    println!("📈 Metrics: http://{}/metrics", addr);

    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

### Health Check Configuration

Comprehensive health checking for production:

```rust
#[derive(Clone)]
struct HealthChecker {
    start_time: Instant,
    dependencies: Vec<String>,
}

impl HealthChecker {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            dependencies: vec![
                "database".to_string(),
                "redis".to_string(),
                "external_api".to_string(),
            ],
        }
    }

    async fn check_health(&self) -> HealthStatus {
        let mut checks = Vec::new();

        // Check each dependency
        for dep in &self.dependencies {
            let status = match dep.as_str() {
                "database" => self.check_database().await,
                "redis" => self.check_redis().await,
                "external_api" => self.check_external_api().await,
                _ => DependencyStatus::Unknown,
            };
            checks.push(DependencyCheck {
                name: dep.clone(),
                status,
                response_time: Duration::from_millis(10), // Mock response time
            });
        }

        let overall_healthy = checks.iter().all(|check| {
            matches!(check.status, DependencyStatus::Healthy)
        });

        HealthStatus {
            healthy: overall_healthy,
            uptime: self.start_time.elapsed(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            dependencies: checks,
            timestamp: chrono::Utc::now(),
        }
    }

    async fn check_database(&self) -> DependencyStatus {
        // Mock database check
        DependencyStatus::Healthy
    }

    async fn check_redis(&self) -> DependencyStatus {
        // Mock Redis check
        DependencyStatus::Healthy
    }

    async fn check_external_api(&self) -> DependencyStatus {
        // Mock external API check
        DependencyStatus::Healthy
    }
}

#[derive(Serialize)]
struct HealthStatus {
    healthy: bool,
    uptime: Duration,
    version: String,
    dependencies: Vec<DependencyCheck>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct DependencyCheck {
    name: String,
    status: DependencyStatus,
    response_time: Duration,
}

#[derive(Serialize)]
enum DependencyStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let health_checker = HealthChecker::new();

    let router = Router::new()
        .get("/", || async { Ok(Response::text("Health-Monitored Server 🏥")) })
        .get("/health", {
            let checker = health_checker.clone();
            move || {
                let checker = checker.clone();
                async move {
                    let health = checker.check_health().await;
                    let status_code = if health.healthy {
                        StatusCode::OK
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE
                    };

                    Ok(Response::json(health)?
                        .with_status(status_code)
                        .with_header("Cache-Control", "no-cache"))
                }
            }
        })
        .get("/health/ready", {
            let checker = health_checker.clone();
            move || {
                let checker = checker.clone();
                async move {
                    // Readiness check (can serve traffic)
                    let health = checker.check_health().await;
                    if health.healthy {
                        Ok(Response::json(serde_json::json!({"ready": true}))?)
                    } else {
                        Ok(Response::json(serde_json::json!({"ready": false}))?
                            .with_status(StatusCode::SERVICE_UNAVAILABLE))
                    }
                }
            }
        })
        .get("/health/live", || async {
            // Liveness check (process is alive)
            Response::json(serde_json::json!({"alive": true}))
        });

    let addr = "0.0.0.0:8080".parse()?;

    println!("🏥 Health-monitored server starting on http://{}", addr);
    println!("❤️ Health: http://{}/health", addr);
    println!("✅ Ready: http://{}/health/ready", addr);
    println!("💓 Live: http://{}/health/live", addr);

    Server::new(router, addr).ignitia().await
}
```

## Troubleshooting

### Common Configuration Issues

Debug common server configuration problems:

```rust
use tracing::{info, warn, error};

fn validate_server_config(config: &ServerConfig) -> Result<(), String> {
    let mut issues = Vec::new();

    // HTTP/2 validation
    if config.http2.enabled {
        if let Some(streams) = config.http2.max_concurrent_streams {
            if streams > 10000 {
                issues.push("HTTP/2 max_concurrent_streams is very high (>10000)".to_string());
            }
        }

        if let Some(window_size) = config.http2.initial_connection_window_size {
            if window_size > 16 * 1024 * 1024 {
                issues.push("HTTP/2 connection window size is very large (>16MB)".to_string());
            }
        }

        if let Some(frame_size) = config.http2.max_frame_size {
            if frame_size < 16 * 1024 || frame_size > 16 * 1024 * 1024 {
                issues.push("HTTP/2 frame size is outside recommended range (16KB-16MB)".to_string());
            }
        }
    }

    // Protocol detection validation
    if !config.http1_enabled && !config.http2.enabled {
        issues.push("Both HTTP/1.1 and HTTP/2 are disabled".to_string());
    }

    if config.auto_protocol_detection && (!config.http1_enabled || !config.http2.enabled) {
        issues.push("Auto protocol detection enabled but not all protocols are available".to_string());
    }

    // Request size validation
    if config.max_request_body_size > 1024 * 1024 * 1024 {
        issues.push("Request body size limit is very high (>1GB)".to_string());
    }

    // HTTPS redirect validation
    if config.redirect_http_to_https && config.https_port.is_none() {
        issues.push("HTTPS redirect enabled but no HTTPS port specified".to_string());
    }

    if !issues.is_empty() {
        for issue in &issues {
            warn!("⚠️ Configuration issue: {}", issue);
        }
        return Err(format!("Found {} configuration issues", issues.len()));
    }

    info!("✅ Server configuration validated successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();

    let config = ServerConfig {
        http2: Http2Config {
            enabled: true,
            max_concurrent_streams: Some(15000), // Will trigger warning
            ..Default::default()
        },
        redirect_http_to_https: true,
        https_port: None, // Will trigger warning
        ..Default::default()
    };

    // Validate configuration before starting server
    if let Err(e) = validate_server_config(&config) {
        error!("❌ Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    let router = Router::new()
        .get("/", || async { Ok(Response::text("Validated Configuration Server ✅")) });

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

### Debug Configuration Helper

Helper functions for debugging server configuration:

```rust
fn print_server_config_debug(config: &ServerConfig) {
    println!("🔧 Server Configuration Debug Info");
    println!("=====================================");

    println!("Protocol Support:");
    println!("  HTTP/1.1 enabled: {}", config.http1_enabled);
    println!("  HTTP/2 enabled: {}", config.http2.enabled);
    println!("  Auto detection: {}", config.auto_protocol_detection);

    if config.http2.enabled {
        println!("\nHTTP/2 Settings:");
        println!("  Prior knowledge: {}", config.http2.enable_prior_knowledge);
        println!("  Max streams: {:?}", config.http2.max_concurrent_streams);
        println!("  Connection window: {:?} bytes", config.http2.initial_connection_window_size);
        println!("  Stream window: {:?} bytes", config.http2.initial_stream_window_size);
        println!("  Max frame size: {:?} bytes", config.http2.max_frame_size);
        println!("  Keep-alive interval: {:?}", config.http2.keep_alive_interval);
        println!("  Keep-alive timeout: {:?}", config.http2.keep_alive_timeout);
        println!("  Adaptive window: {}", config.http2.adaptive_window);
        println!("  Max header size: {:?} bytes", config.http2.max_header_list_size);
    }

    println!("\nSecurity Settings:");
    println!("  HTTPS redirect: {}", config.redirect_http_to_https);
    println!("  HTTPS port: {:?}", config.https_port);

    println!("\nLimits:");
    println!("  Max request body: {} bytes ({:.1} MB)",
             config.max_request_body_size,
             config.max_request_body_size as f64 / (1024.0 * 1024.0));

    println!("=====================================\n");
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig::default();
    print_server_config_debug(&config);

    let router = Router::new()
        .get("/", || async { Ok(Response::text("Debug Configuration Server 🐛")) });

    let addr = "127.0.0.1:8080".parse()?;
    Server::new(router, addr)
        .with_server_config(config)
        .ignitia()
        .await
}
```

---

**This comprehensive server configuration guide covers all aspects of setting up and optimizing Ignitia servers. For additional examples and advanced configurations, refer to the [Ignitia Examples Repository](https://github.com/AarambhDevHub/ignitia/tree/main/examples) and [API Documentation](https://docs.rs/ignitia).**

**🔥 Ready to configure your high-performance Ignitia server? Start building blazing-fast applications today!**
