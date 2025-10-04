# Ignitia Documentation

Welcome to the comprehensive documentation for **Ignitia** - a blazing fast, lightweight web framework for Rust that ignites your development journey! 🔥

Ignitia is designed to provide developers with a powerful, async HTTP server framework featuring modern web development capabilities including HTTP/2 support, WebSockets, middleware, file uploads, and robust security features.

## 📚 Documentation Index

### Getting Started
- **[📋 Installation Guide](INSTALLATION.md)** - Step-by-step installation instructions and feature flags
- **[🚀 Quick Start](QUICK_START.md)** - Get up and running with Ignitia in minutes
- **[📖 Examples](EXAMPLES.md)** - Practical code examples and use cases

### Core Concepts
- **[🛣️ Routing Guide](ROUTING_GUIDE.md)** - Complete routing system documentation including path parameters, nested routers, and route matching
- **[⚡ Middleware Guide](MIDDLEWARE_GUIDE.md)** - Built-in middleware and creating custom middleware
- **[🔧 Extractors](EXTRACTORS.md)** - JSON, Form, Query, Path, Headers, Cookies, and custom extractors
- **[📄 API Reference](API_REFERENCE.md)** - Comprehensive API reference for Ignitia's modules and components
- **[❌ Error Handling](ERROR_HANDLING.md)** - Error types, custom errors, and error responses

### Advanced Features
- **[🌐 WebSockets](WEB_SOCKETS.md)** - Real-time communication with WebSocket support
- **[📁 File Uploads](FILE_UPLOADS.md)** - Multipart form data and file handling
- **[🖥️ Server Configuration](SERVER_CONFIG.md)** - HTTP/2, TLS, and server optimization

### Production Deployment
- **[🔒 Security](SECURITY.md)** - Security middleware, HTTPS, and protection mechanisms

### Static Files and I/O
- **[📦 Static Files](STATIC_FILES.md)** - Handling of static file serving and caching
- **[📬 Responses](RESPONSES.md)** - Response structure, builders, and helpers
- **[📥 Requests](REQUESTS.md)** - Request structure, body extraction, and parameters

### Project Information
- **[📝 Migration Guide](MIGRATION.md)** - Upgrading between versions
- **[🤝 Contributing](CONTRIBUTING.md)** - How to contribute to Ignitia
- **[📋 Changelog](CHANGELOG.md)** - Version history and breaking changes


## 🎯 Quick Navigation

**New to Ignitia?** Start with the [Quick Start Guide](QUICK_START.md) to build your first application.

**Looking for specific features?**
- Building APIs? → [Extractors](EXTRACTORS.md) + [Error Handling](ERROR_HANDLING.md)
- Real-time apps? → [WebSockets](WEB_SOCKETS.md)
- File uploads? → [File Uploads](FILE_UPLOADS.md)
- Production deployment? → [Server Config](SERVER_CONFIG.md) + [Security](SECURITY.md)

**Need help?** Check out [Examples](EXAMPLES.md) for practical implementations.

## 🔥 What Makes Ignitia Special

- **🚀 Blazing Fast**: Built on Tokio with HTTP/2 support and optimized for performance
- **🛠️ Developer Friendly**: Intuitive API design with comprehensive error messages
- **📦 Modular**: Optional features mean you only include what you need
- **🔒 Secure by Default**: Built-in security middleware and best practices
- **🌟 Modern**: Async/await throughout, leveraging Rust's powerful type system

## 🏗️ Architecture Overview

Ignitia follows a layered, modular architecture built on Rust's async ecosystem. The framework is designed for performance, type safety, and developer productivity.

### High-Level Architecture

```
                          ┌─────────────────────────────────────┐
                          │           Application               │
                          │        (Your Business Logic)       │
                          └─────────────────┬───────────────────┘
                                           │
    ┌──────────────────────────────────────▼──────────────────────────────────────┐
    │                                 Ignitia Framework                           │
    │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  ┌─────────────────┐   │
    │  │   Handlers  │  │  Extractors  │  │ Middleware  │  │     Router      │   │
    │  │ (Business)  │  │ (Type-Safe   │  │ (Pipeline)  │  │ (Smart Route    │   │
    │  │   Logic     │  │  Request     │  │ Processing  │  │   Matching)     │   │
    │  │             │  │ Parsing)     │  │             │  │                 │   │
    │  └─────────────┘  └──────────────┘  └─────────────┘  └─────────────────┘   │
    └──────────────────────────────────────┬──────────────────────────────────────┘
                                           │
    ┌──────────────────────────────────────▼──────────────────────────────────────┐
    │                             Core Infrastructure                              │
    │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  ┌─────────────────┐   │
    │  │   Server    │  │   Protocol   │  │   Request/  │  │   WebSocket     │   │
    │  │ (HTTP/1&2,  │  │  Detection   │  │  Response   │  │    Support      │   │
    │  │  TLS, H2C)  │  │  (ALPN/Auto) │  │   Engine    │  │                 │   │
    │  └─────────────┘  └──────────────┘  └─────────────┘  └─────────────────┘   │
    └──────────────────────────────────────┬──────────────────────────────────────┘
                                           │
    ┌──────────────────────────────────────▼──────────────────────────────────────┐
    │                          Tokio Runtime & Hyper                              │
    │               (Async I/O, Connection Management, HTTP Protocol)             │
    └─────────────────────────────────────────────────────────────────────────────┘
```

### Core Components Deep Dive

#### 🚀 **Server Layer**
The server layer handles connection management and protocol negotiation:

- **Multi-Protocol Support**: HTTP/1.1, HTTP/2, and HTTP/2 Cleartext (H2C)
- **TLS Integration**: Optional TLS with ALPN protocol negotiation
- **Connection Management**: Efficient connection pooling and lifecycle management
- **Protocol Detection**: Automatic protocol detection and upgrade handling

```
// Example server configuration
let server = Server::new(router, "127.0.0.1:3000".parse().unwrap())
    .with_config(ServerConfig {
        http2: Http2Config {
            enabled: true,
            max_concurrent_streams: Some(1000),
            ..Default::default()
        },
        ..Default::default()
    });
```

#### 🛣️ **Router System**
Advanced routing with performance optimizations:

- **Pre-compiled Routes**: Routes are compiled to optimized regex patterns at startup
- **Smart Matching**: Routes sorted by specificity for faster matching
- **Nested Routing**: Support for modular route organization
- **Parameter Extraction**: Type-safe path and wildcard parameter extraction

```
// Router architecture
Router::new()
    .get("/users/{id}", user_handler)           // Path parameters
    .post("/users", create_user_handler)       // HTTP methods
    .nest("/api/v1", api_routes)               // Nested routing
    .websocket("/ws", websocket_handler)       // WebSocket endpoints
```

#### ⚙️ **Middleware Pipeline**
Composable middleware system with before/after hooks:

- **Layered Processing**: Request flows through middleware chain
- **Built-in Middleware**: CORS, auth, logging, rate limiting, compression
- **Custom Middleware**: Easy to implement custom processing logic
- **Error Handling**: Centralized error processing with custom handlers

```
// Middleware execution flow
Request → Middleware₁.before() → Middleware₂.before() → Handler →
        ← Middleware₂.after()  ← Middleware₁.after()  ← Response
```

#### 🔧 **Type-Safe Extractors**
Automatic request data extraction with compile-time guarantees:

- **JSON/Form Data**: Automatic deserialization with serde
- **Path Parameters**: Type-safe path parameter extraction
- **Query Strings**: Structured query parameter parsing
- **Headers & Cookies**: Easy access to request metadata
- **Custom Extractors**: Implement `FromRequest` for custom types

```
// Extractor examples
async fn handler(
    Path(user_id): Path<u32>,           // Path parameter
    Query(params): Query<SearchQuery>,   // Query parameters
    Json(data): Json<CreateUser>,        // JSON body
    State(db): State<Database>,          // Shared state
) -> Result<Response> { /* ... */ }
```

#### 🔌 **Extension System**
Type-safe shared state management:

- **Dependency Injection**: Store and retrieve shared resources
- **Type Safety**: Compile-time guarantees for state access
- **Arc-wrapped**: Efficient shared ownership with `Arc<T>`
- **Request-scoped**: Per-request data storage

#### 🌐 **WebSocket Integration**
Full-featured WebSocket support with optimizations:

- **Protocol Upgrade**: Automatic WebSocket handshake handling
- **Message Types**: Text, Binary, Ping/Pong, Close frames
- **Batch Processing**: Efficient batch message handling
- **Custom Handlers**: Flexible handler patterns for different use cases

### Performance Optimizations

#### 📊 **Routing Performance**
- **Compiled Regex**: Routes pre-compiled to optimized regex patterns
- **Smart Sorting**: Routes ordered by specificity for faster matching
- **Early Rejection**: Quick path length and segment count checks
- **Cache-friendly**: Memory layout optimized for CPU cache performance

#### 🚀 **Connection Handling**
- **HTTP/2 Multiplexing**: Concurrent request handling over single connection
- **Connection Pooling**: Efficient resource reuse
- **Protocol Negotiation**: ALPN-based protocol selection
- **Keep-Alive**: Configurable connection persistence

#### 💾 **Memory Management**
- **Zero-Copy**: Minimal data copying with `Bytes` and `Arc`
- **Streaming**: Large request/response body streaming
- **Bounded Channels**: Prevent memory exhaustion in WebSocket handling
- **Efficient Parsing**: Optimized HTTP header and body parsing

### Security Architecture

#### 🛡️ **Built-in Security**
- **HTTPS/TLS**: Full TLS 1.2/1.3 support with modern cipher suites
- **Security Headers**: Automatic security header injection
- **CORS**: Flexible cross-origin resource sharing
- **Rate Limiting**: Token bucket algorithm for DDoS protection

#### 🔒 **Input Validation**
- **Type Safety**: Compile-time request validation
- **Size Limits**: Configurable request/field size limits
- **Content Validation**: MIME type and encoding validation
- **Sanitization**: Built-in XSS and injection protection

### Extension Points

#### 🔧 **Custom Middleware**
```
#[async_trait::async_trait]
impl Middleware for CustomMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> { /* ... */ }
    async fn after(&self, req: &Request, res: &mut Response) -> Result<()> { /* ... */ }
}
```

#### 📊 **Custom Extractors**
```
#[async_trait::async_trait]
impl FromRequest for CustomExtractor {
    fn from_request(req: &Request) -> Result<Self> { /* ... */ }
}
```

#### ❌ **Custom Error Handling**
```
impl CustomError for MyError {
    fn status_code(&self) -> StatusCode { /* ... */ }
    fn error_type(&self) -> &'static str { /* ... */ }
}
```

### Development Workflow

```
1. Define Routes     → 2. Add Middleware  → 3. Implement Handlers
       ↓                      ↓                    ↓
4. Configure Server  → 5. Add Extensions  → 6. Handle Errors
       ↓                      ↓                    ↓
7. Test & Debug     → 8. Deploy          → 9. Monitor
```

This architecture ensures that Ignitia applications are:
- **Fast**: Optimized routing and connection handling
- **Safe**: Type-safe extractors and error handling
- **Scalable**: Async architecture with efficient resource usage
- **Maintainable**: Clear separation of concerns and modular design
- **Extensible**: Rich plugin system for custom functionality

## 🚦 Status

- **Current Version**: 0.2.4
- **Stability**: Beta (API may change)
- **Minimum Rust Version**: 1.70+

## 📞 Support & Community

- 📚 **Documentation**: You're reading it!
- 🐛 **Issues**: Report bugs and request features
- 💬 **Discussions**: Ask questions and share projects
- 🤝 **Contributing**: See [Contributing Guide](CONTRIBUTING.md)

---

*Ready to ignite your next Rust web project? Start with the [Quick Start Guide](QUICK_START.md)!* 🔥
