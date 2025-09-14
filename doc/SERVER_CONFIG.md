# Server Configuration Guide

This guide covers all server configuration options available in Ignitia, from basic HTTP servers to advanced HTTPS, HTTP/2, and WebSocket configurations.

## Table of Contents

- [Basic Server Setup](#basic-server-setup)
- [Server Configuration](#server-configuration)
- [HTTP/2 Configuration](#http2-configuration)
- [TLS/HTTPS Configuration](#tlshttps-configuration)
- [WebSocket Configuration](#websocket-configuration)
- [Protocol Detection](#protocol-detection)
- [Performance Optimization](#performance-optimization)
- [Security Settings](#security-settings)
- [Advanced Configuration](#advanced-configuration)
- [Environment-Specific Configs](#environment-specific-configs)

## Basic Server Setup

### Simple HTTP Server

```rust
use ignitia::{Router, Server, Response};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Hello, Ignitia! 🔥")) })
        .get("/health", || async { Ok(Response::json("OK")?) });

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    Server::new(router, addr)
        .ignitia()
        .await
}
```

### Server with Custom Configuration

```rust
use ignitia::{Router, Server, ServerConfig, Response};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Configured Server")) });

    let config = ServerConfig {
        http1_enabled: true,
        auto_protocol_detection: true,
        redirect_http_to_https: false,
        https_port: None,
        ..Default::default()
    };

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;

    Server::new(router, addr)
        .with_config(config)
        .ignitia()
        .await
}
```

## Server Configuration

### ServerConfig Structure

The `ServerConfig` struct provides comprehensive server configuration options:

```rust
use ignitia::{ServerConfig, Http2Config};
use std::time::Duration;

let config = ServerConfig {
    // Enable HTTP/1.1 support
    http1_enabled: true,

    // HTTP/2 configuration
    http2: Http2Config {
        enabled: true,
        enable_prior_knowledge: false,
        max_concurrent_streams: Some(1000),
        initial_connection_window_size: Some(1024 * 1024), // 1MB
        initial_stream_window_size: Some(64 * 1024),       // 64KB
        max_frame_size: Some(16 * 1024),                   // 16KB
        keep_alive_interval: Some(Duration::from_secs(60)),
        keep_alive_timeout: Some(Duration::from_secs(20)),
        adaptive_window: true,
        max_header_list_size: Some(16 * 1024),             // 16KB
    },

    // Protocol detection
    auto_protocol_detection: true,

    // HTTPS redirect
    redirect_http_to_https: false,
    https_port: None,

    // TLS configuration (when tls feature is enabled)
    #[cfg(feature = "tls")]
    tls: None,
};
```

### Configuration Methods

```rust
use ignitia::{Server, ServerConfig};

let server = Server::new(router, addr)
    .with_config(config)           // Apply custom configuration
    .redirect_to_https(443);       // Enable HTTPS redirect
```

## HTTP/2 Configuration

### Basic HTTP/2 Setup

```rust
use ignitia::{ServerConfig, Http2Config};
use std::time::Duration;

let http2_config = Http2Config {
    enabled: true,
    enable_prior_knowledge: false,  // Set to true for H2C support
    ..Default::default()
};

let config = ServerConfig {
    http1_enabled: true,    // Support both HTTP/1.1 and HTTP/2
    http2: http2_config,
    auto_protocol_detection: true,
    ..Default::default()
};
```

### Advanced HTTP/2 Configuration

```rust
use ignitia::{Http2Config};
use std::time::Duration;

let http2_config = Http2Config {
    enabled: true,

    // Enable H2C (HTTP/2 Cleartext) for curl --http2-prior-knowledge
    enable_prior_knowledge: true,

    // Connection limits
    max_concurrent_streams: Some(2000),

    // Window sizes for flow control
    initial_connection_window_size: Some(2 * 1024 * 1024), // 2MB
    initial_stream_window_size: Some(1024 * 1024),         // 1MB

    // Frame settings
    max_frame_size: Some(32 * 1024),                       // 32KB
    max_header_list_size: Some(32 * 1024),                 // 32KB

    // Keep-alive settings
    keep_alive_interval: Some(Duration::from_secs(30)),
    keep_alive_timeout: Some(Duration::from_secs(10)),

    // Enable adaptive window sizing
    adaptive_window: true,
};
```

### HTTP/2 Only Server

```rust
let config = ServerConfig {
    http1_enabled: false,   // Disable HTTP/1.1
    http2: Http2Config {
        enabled: true,
        enable_prior_knowledge: true,  // Required for HTTP/2 only
        ..Default::default()
    },
    auto_protocol_detection: false,
    ..Default::default()
};
```

## TLS/HTTPS Configuration

### Basic HTTPS Setup

```rust
use ignitia::{Server, TlsConfig};

let tls_config = TlsConfig::new("cert.pem", "key.pem");

let server = Server::new(router, addr)
    .with_tls(tls_config)?
    .ignitia()
    .await;
```

### Advanced TLS Configuration

```rust
use ignitia::{TlsConfig, TlsVersion};

let tls_config = TlsConfig::new("server.crt", "server.key")
    .with_alpn_protocols(vec!["h2", "http/1.1"])
    .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
    .enable_client_cert_verification();

let server = Server::new(router, addr)
    .with_tls(tls_config)?
    .ignitia()
    .await;
```

### Self-Signed Certificate (Development)

```rust
use ignitia::Server;

// Generate and use self-signed certificate
let server = Server::new(router, addr)
    .with_self_signed_cert("localhost")?
    .ignitia()
    .await;
```

### HTTP to HTTPS Redirect

```rust
use ignitia::{Server, ServerConfig};

let config = ServerConfig::default()
    .redirect_to_https(443);

let server = Server::new(router, addr)
    .with_config(config)
    .ignitia()
    .await;
```

### Dual HTTP/HTTPS Servers

```rust
use ignitia::{Router, Server, TlsConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Secure Server")) });

    // HTTP server (redirects to HTTPS)
    let http_router = router.clone();
    let http_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    tokio::spawn(async move {
        Server::new(http_router, http_addr)
            .redirect_to_https(8443)
            .ignitia()
            .await
    });

    // HTTPS server
    let tls_config = TlsConfig::new("cert.pem", "key.pem");
    let https_addr: SocketAddr = "127.0.0.1:8443".parse()?;

    Server::new(router, https_addr)
        .with_tls(tls_config)?
        .ignitia()
        .await
}
```

## WebSocket Configuration

### Basic WebSocket Server

```rust
use ignitia::{Router, websocket_handler, Message, Response};

let router = Router::new()
    .get("/", || async { Ok(Response::html(r#"
        <!DOCTYPE html>
        <html>
        <body>
            <script>
                const ws = new WebSocket('ws://localhost:8080/ws');
                ws.onmessage = (event) => console.log('Received:', event.data);
                ws.send('Hello Server!');
            </script>
        </body>
        </html>
    "#)) })
    .websocket_fn("/ws", |mut websocket| async move {
        while let Some(message) = websocket.recv().await {
            match message {
                Message::Text(text) => {
                    println!("Received: {}", text);
                    websocket.send_text(format!("Echo: {}", text)).await?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok(())
    });
```

### Advanced WebSocket with HTTP/2

```rust
use ignitia::{Router, ServerConfig, Http2Config, websocket_handler};

let router = Router::new()
    .websocket_fn("/chat", |websocket| async move {
        // Handle WebSocket connection
        Ok(())
    });

let config = ServerConfig {
    http1_enabled: true,
    http2: Http2Config {
        enabled: true,
        ..Default::default()
    },
    auto_protocol_detection: true,
    ..Default::default()
};

Server::new(router, addr)
    .with_config(config)
    .ignitia()
    .await;
```

## Protocol Detection

### Automatic Protocol Detection

```rust
let config = ServerConfig {
    http1_enabled: true,
    http2: Http2Config {
        enabled: true,
        ..Default::default()
    },
    // Enables automatic HTTP/1.1 vs HTTP/2 detection
    auto_protocol_detection: true,
    ..Default::default()
};
```

### TLS ALPN Protocol Negotiation

```rust
use ignitia::TlsConfig;

let tls_config = TlsConfig::new("cert.pem", "key.pem")
    .with_alpn_protocols(vec![
        "h2",        // HTTP/2
        "http/1.1",  // HTTP/1.1
        "http/1.0"   // HTTP/1.0
    ]);
```

## Performance Optimization

### High-Performance Configuration

```rust
use ignitia::{ServerConfig, Http2Config};
use std::time::Duration;

let config = ServerConfig {
    http1_enabled: true,
    http2: Http2Config {
        enabled: true,

        // Optimize for high concurrency
        max_concurrent_streams: Some(5000),

        // Larger windows for better throughput
        initial_connection_window_size: Some(4 * 1024 * 1024), // 4MB
        initial_stream_window_size: Some(2 * 1024 * 1024),     // 2MB

        // Larger frames for bulk data
        max_frame_size: Some(64 * 1024),  // 64KB

        // Aggressive keep-alive
        keep_alive_interval: Some(Duration::from_secs(15)),
        keep_alive_timeout: Some(Duration::from_secs(5)),

        // Enable adaptive windowing
        adaptive_window: true,

        // Larger header buffer
        max_header_list_size: Some(64 * 1024), // 64KB
    },
    auto_protocol_detection: true,
    ..Default::default()
};
```

### Memory-Optimized Configuration

```rust
let config = ServerConfig {
    http2: Http2Config {
        enabled: true,

        // Conservative limits
        max_concurrent_streams: Some(100),
        initial_connection_window_size: Some(64 * 1024),    // 64KB
        initial_stream_window_size: Some(32 * 1024),       // 32KB
        max_frame_size: Some(8 * 1024),                    // 8KB
        max_header_list_size: Some(8 * 1024),              // 8KB

        // Longer intervals to reduce overhead
        keep_alive_interval: Some(Duration::from_secs(120)),
        keep_alive_timeout: Some(Duration::from_secs(30)),

        adaptive_window: false,
    },
    ..Default::default()
};
```

## Security Settings

### Secure Production Configuration

```rust
use ignitia::{ServerConfig, TlsConfig, TlsVersion};

let tls_config = TlsConfig::new("cert.pem", "key.pem")
    .with_alpn_protocols(vec!["h2", "http/1.1"])
    .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
    .enable_client_cert_verification();

let config = ServerConfig {
    http1_enabled: true,
    http2: Http2Config {
        enabled: true,
        // Reasonable limits to prevent DoS
        max_concurrent_streams: Some(1000),
        max_frame_size: Some(16 * 1024),
        max_header_list_size: Some(8 * 1024),
        ..Default::default()
    },
    auto_protocol_detection: true,
    redirect_http_to_https: true,
    https_port: Some(443),
    tls: Some(tls_config),
};
```

### Development Configuration

```rust
let config = ServerConfig {
    http1_enabled: true,
    http2: Http2Config {
        enabled: true,
        enable_prior_knowledge: true,  // Allow H2C for testing
        ..Default::default()
    },
    auto_protocol_detection: true,
    redirect_http_to_https: false,    // Keep HTTP for development
    ..Default::default()
};
```

## Advanced Configuration

### Custom Timeouts and Limits

```rust
use std::time::Duration;

let config = ServerConfig {
    http2: Http2Config {
        enabled: true,

        // Connection management
        keep_alive_interval: Some(Duration::from_secs(30)),
        keep_alive_timeout: Some(Duration::from_secs(10)),

        // Resource limits
        max_concurrent_streams: Some(1000),
        initial_connection_window_size: Some(1024 * 1024),
        initial_stream_window_size: Some(64 * 1024),
        max_frame_size: Some(16 * 1024),
        max_header_list_size: Some(16 * 1024),

        adaptive_window: true,
    },
    ..Default::default()
};
```

### Multi-Port Server

```rust
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new()
        .get("/", || async { Ok(Response::text("Multi-port server")) });

    // API server on port 8080
    let api_server = {
        let router = router.clone();
        let addr = "127.0.0.1:8080".parse()?;
        Server::new(router, addr).ignitia()
    };

    // Admin server on port 8081
    let admin_server = {
        let router = router.clone();
        let addr = "127.0.0.1:8081".parse()?;
        Server::new(router, addr).ignitia()
    };

    // WebSocket server on port 8082
    let ws_server = {
        let router = Router::new()
            .websocket_fn("/ws", |websocket| async move { Ok(()) });
        let addr = "127.0.0.1:8082".parse()?;
        Server::new(router, addr).ignitia()
    };

    // Run all servers concurrently
    try_join!(api_server, admin_server, ws_server)?;
    Ok(())
}
```

## Environment-Specific Configs

### Production Configuration

```rust
pub fn production_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            max_concurrent_streams: Some(2000),
            initial_connection_window_size: Some(2 * 1024 * 1024),
            initial_stream_window_size: Some(1024 * 1024),
            keep_alive_interval: Some(Duration::from_secs(60)),
            keep_alive_timeout: Some(Duration::from_secs(20)),
            adaptive_window: true,
            ..Default::default()
        },
        auto_protocol_detection: true,
        redirect_http_to_https: true,
        https_port: Some(443),
        ..Default::default()
    }
}
```

### Development Configuration

```rust
pub fn development_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            enable_prior_knowledge: true,  // Enable H2C for testing
            ..Default::default()
        },
        auto_protocol_detection: true,
        redirect_http_to_https: false,
        ..Default::default()
    }
}
```

### Testing Configuration

```rust
pub fn test_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: false,  // Simpler for tests
            ..Default::default()
        },
        auto_protocol_detection: false,
        ..Default::default()
    }
}
```

## Configuration Examples

### API Server with Rate Limiting

```rust
use ignitia::{Router, Server, ServerConfig, RateLimitingMiddleware};

let router = Router::new()
    .middleware(RateLimitingMiddleware::per_minute(1000))
    .get("/api/v1/users", || async { Ok(Response::json("users")?) });

let config = ServerConfig {
    http2: Http2Config {
        enabled: true,
        max_concurrent_streams: Some(1000),
        ..Default::default()
    },
    ..Default::default()
};

Server::new(router, "127.0.0.1:8080".parse()?)
    .with_config(config)
    .ignitia()
    .await;
```

### Microservice Configuration

```rust
pub fn microservice_config() -> ServerConfig {
    ServerConfig {
        http1_enabled: true,
        http2: Http2Config {
            enabled: true,
            max_concurrent_streams: Some(500),
            initial_connection_window_size: Some(512 * 1024), // 512KB
            initial_stream_window_size: Some(256 * 1024),    // 256KB
            keep_alive_interval: Some(Duration::from_secs(30)),
            adaptive_window: true,
            ..Default::default()
        },
        auto_protocol_detection: true,
        ..Default::default()
    }
}
```

This comprehensive guide covers all aspects of server configuration in Ignitia. For more specific use cases or advanced configurations, refer to the API documentation or check the examples in the Ignitia repository.
