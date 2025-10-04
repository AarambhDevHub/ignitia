# Changelog

All notable changes to the Ignitia web framework will be documented in this file.

## [Unreleased]

### Planned
- OpenAPI/Swagger documentation generation
- Built-in metrics and observability
- Rate limiting per-route configuration
- Request validation middleware
- Database connection pooling utilities
- Session management middleware
- HTTP/2 WebSocket support (RFC 8441) when ecosystem matures

## [0.2.4] - 2025-10-03

### Added
- **Comprehensive Documentation**: Full rustdoc coverage across all modules
  - Module-level documentation with examples and architecture notes
  - Detailed inline documentation for all public types and methods
  - Performance notes and optimization guidance throughout
  - Migration guides and usage examples
- **Enhanced IntoResponse Implementations**: Extended response type support
  - Boolean responses: `bool` → JSON `true`/`false`
  - Numeric types: All integers and floats → text/plain responses
  - Static string optimization: Zero-copy `&'static str` responses
  - Cow string support: Efficient borrowed/owned string handling
  - Tuple responses: `(StatusCode, T)` for custom status codes
  - HTML wrapper: `Html<T>` for typed HTML responses
  - Option/Result automatic handling with appropriate status codes
- **WebSocket Extractor Support**: Full parameter extraction for WebSocket handlers
  - **Universal WebSocket Handler**: Up to 7 extractors before `WebSocketConnection` parameter
  - All standard extractors work: `State<T>`, `Path<T>`, `Query<T>`, `Headers`, `Extension<T>`, `Cookies`
  - Type-safe parameter extraction with automatic error responses
  - Follows same pattern as HTTP handlers for consistency
  - Examples: `|State(s), Path(id), Query(q), ws| async move { ... }`
- **WebSocket Protocol Documentation**: Comprehensive protocol support notes
  - HTTP/1.1 WebSocket fully supported (RFC 6455) over ws:// and wss://
  - Secure WebSocket (wss://) works over HTTPS/TLS with HTTP/1.1 upgrade
  - HTTP/2 WebSocket (RFC 8441) noted as future enhancement
  - Clear documentation of protocol limitations and capabilities

### Changed
- **Breaking**: Removed Base Router mode - Now exclusively uses Radix tree routing
  - All routing now uses high-performance Radix tree implementation
  - Removed `RouterMode` enum and Base router code paths
  - Simplified router architecture with single optimized path
  - ~20% performance improvement from eliminating mode switching overhead
- **Breaking**: Route parameter syntax standardized
  - Path parameters: `:param` → `{param}` (more intuitive, matches industry standards)
  - Wildcard parameters: `*path` → `{*path}` (explicit wildcard syntax)
  - Examples: `/users/:id` → `/users/{id}`, `/files/*path` → `/files/{*path}`
  - Better readability and consistency with OpenAPI/Swagger conventions
- **Breaking**: Body handling optimized - Removed double `Arc<Bytes>` wrapping
  - Changed `Request.body: Arc<Bytes>` → `Request.body: Bytes`
  - Changed `Response.body: Arc<Bytes>` → `Response.body: Bytes`
  - Eliminated redundant reference counting (40% reduction in allocations)
  - `Bytes` already provides efficient cloning via internal Arc
- **Breaking**: `Json<T>` unified as single type for extraction and response
  - Moved from separate extractor/response types to shared implementation
  - Single `Json<T>` type implements both `FromRequest` and `IntoResponse`
  - Consistent behavior across input and output operations
  - Follows Axum's proven design pattern
- **Breaking**: WebSocket handler signature requirements
  - WebSocket handlers must return types implementing `IntoResponse`
  - Examples: `Response`, `String`, `Json<T>`, `(StatusCode, T)`, `Result<Response>`
  - Empty tuple `()` no longer valid return type
  - Clearer error messages for invalid return types
- **Middleware system completely revamped**
  - The old middleware trait with separate before and after methods was replaced by a modern composable async middleware model with a single handle(req, next) method. This aligns with industry best practices, improves performance, and simplifies middleware authoring.

### Performance Optimizations
- **Response Body Handling**: 30-40% reduction in allocations per request
  - Removed double Arc wrapping (`Arc<Bytes>` → `Bytes`)
  - Zero-copy static string/byte responses with `Bytes::from_static()`
  - Optimized ResponseBody enum (removed redundant Shared variant)
- **WebSocket Performance**: Zero-copy message passing where possible
  - Efficient WebSocket stream handling with tokio-tungstenite
  - Optimized extractor chain for WebSocket upgrades
  - Minimal overhead for HTTP/1.1 upgrade handshake
  - Batch message processing for high-throughput scenarios
- **Handler Optimization**: Cross-crate optimization with inline hints
  - Added `#[inline]` attributes to all hot-path functions
  - Optimized `FromRequest` trait to use borrowed `&Request`
  - Reduced clone operations in extractor implementations
  - Better compiler optimization opportunities
- **Router Performance**: Radix-only architecture improvements
  - Eliminated routing mode switching overhead
  - Optimized path matching with single code path
  - Pre-allocated header maps with optimal capacity
  - Efficient parameter extraction with minimal allocations

### Enhanced
- **WebSocket Handler System**: Production-ready WebSocket support
  - Automatic HTTP/1.1 upgrade handshake (RFC 6455)
  - Full extractor support matching HTTP handler patterns
  - Secure WebSocket (wss://) over TLS/HTTPS
  - Graceful connection lifecycle management
  - Per-message and batch processing handlers
  - Comprehensive error handling and logging
- **Response Building**: Improved ergonomics and performance
  - All response builders use zero-copy where possible
  - Consistent error handling across all response types
  - Better integration with `IntoResponse` trait
  - Optimized JSON serialization error messages
- **Handler System**: Extended parameter support
  - HTTP handlers: Up to 16 extractors (matches Axum)
  - WebSocket handlers: Up to 7 extractors plus WebSocket connection
  - Generated macro implementations for all combinations
  - Consistent with established Rust web framework patterns
- **Type System**: Better type inference and error messages
  - Unified `Json<T>` type reduces confusion
  - Clearer compile-time errors for invalid handlers
  - Improved IDE hints and autocomplete
  - Better documentation generation

### Benchmarks (wrk -c100 -d30s)
- **Throughput**: 51,574 req/sec (competitive with Axum)
- **Latency**: 1.90ms avg, 10.60ms max (94% improvement)
- **Transfer**: 7.97 MB/sec
- **Consistency**: 70.54% requests within one standard deviation

### Migration Guide

#### Router Mode Removal

**Old Code:**
```rust
use ignitia::router::RouterMode;

let router = Router::new()
    .with_mode(RouterMode::Radix);  // ❌ No longer needed
```

**New Code:**
```rust
let router = Router::new();  // ✅ Always uses Radix
```

#### Route Parameter Syntax

**Old Code:**
```rust
// Path parameters with colon prefix
router.get("/users/:id", handler);
router.get("/posts/:user_id/:post_id", handler);

// Wildcard with asterisk
router.get("/files/*path", handler);
```

**New Code:**
```rust
// Path parameters with curly braces (OpenAPI/Swagger style)
router.get("/users/{id}", handler);
router.get("/posts/{user_id}/{post_id}", handler);

// Wildcard with explicit syntax
router.get("/files/{*path}", handler);
```

#### Body Type Changes

**Old Code:**
```rust
// Request body
let body: Arc<Bytes> = req.body.clone();

// Response body
response.body = Arc::new(Bytes::from(data));
```

**New Code:**
```rust
// Request body
let body: Bytes = req.body.clone();  // ✅ Still cheap (just refcount)

// Response body
response.body = Bytes::from(data);  // ✅ No Arc needed
```

#### Json Extractor/Response

**Old Code:**
```rust
// Separate types for extraction and response
async fn handler(JsonExtractor(data): JsonExtractor<Data>) -> JsonResponse<Output> {
    JsonResponse(output)
}
```

**New Code:**
```rust
use ignitia::Json;  // ✅ One type for both

async fn handler(Json(data): Json<Data>) -> Json<Output> {
    Json(output)
}
```

#### WebSocket Handler Returns

**Old Code:**
```rust
async fn websocket_handler(mut ws: WebSocketConnection) -> Result<()> {
    while let Some(msg) = ws.recv().await {
        // Handle messages...
    }
    Ok(())  // ❌ Empty tuple doesn't implement IntoResponse
}
```

**New Code:**
```rust
async fn websocket_handler(mut ws: WebSocketConnection) -> Response {
    while let Some(msg) = ws.recv().await {
        // Handle messages...
    }
    Response::ok()  // ✅ Returns proper response
}

// Or use other IntoResponse types:
async fn websocket_handler(mut ws: WebSocketConnection) -> &'static str {
    // ...
    "Connection closed"  // ✅ String implements IntoResponse
}
```

#### WebSocket with Extractors

**New Feature:**
```rust
use ignitia::prelude::*;

// Before: Manual state passing
router.websocket_fn("/ws", move |ws| {
    let state = state_clone.clone();
    async move {
        handle_websocket(ws, state).await
    }
});

// After: Use extractors like HTTP handlers
router.websocket("/ws", |
    State(state): State<AppState>,
    mut ws: WebSocketConnection
| async move {
    // State automatically extracted!
    ws.send_text("Connected!").await.ok();
    Response::ok()
});

// Multiple extractors (up to 7)
router.websocket("/ws/{room_id}", |
    Path(room_id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
    Extension(db): Extension<Arc<Database>>,
    mut ws: WebSocketConnection
| async move {
    // All extractors work just like HTTP handlers!
    ws.send_text(format!("Room: {}", room_id)).await.ok();
    Response::ok()
});
```

### Fixed
- Middleware latency regression (30ms → 1.9ms)
- Memory leak in Arc-wrapped body types
- Type inference issues with Json extractor/response
- Handler trait bound conflicts with multiple extractors
- Route matching edge cases in complex nested routers
- WebSocket extractor extraction failures causing silent disconnects
- WebSocket handler return type validation at compile time

### Internal Changes
- Refactored middleware chain building for better performance
- Optimized radix tree traversal with inline hints
- Improved memory layout for Request and Response types
- Enhanced documentation generation with detailed examples
- Added comprehensive performance benchmarks
- WebSocket handler wrapper with PhantomData for type safety
- Unified WebSocket and HTTP extractor implementation

### Protocol Support
- **HTTP/1.1**: ✅ Fully supported for all request types
- **HTTP/2**: ✅ Supported for regular HTTP requests
- **HTTP/1.1 WebSocket (RFC 6455)**: ✅ Fully supported (ws:// and wss://)
- **HTTP/2 WebSocket (RFC 8441)**: ⏳ Not yet supported (ecosystem limitations)
  - Requires Hyper/h2 crate enhancements to expose `:protocol` pseudo-header
  - Limited browser support as of 2025
  - WebSocket connections automatically use HTTP/1.1 even on HTTP/2 servers
  - Planned for future release when Rust ecosystem support matures

### Acknowledgments
Design patterns inspired by established Rust web frameworks:
- Axum: Middleware patterns, unified Json type, extractor system, WebSocket handler ergonomics
- Actix-web: Performance optimizations, response handling
- Rocket: Response type ergonomics
- RFC 6455: WebSocket Protocol specification
- RFC 8441: WebSocket over HTTP/2 (future enhancement)

---

## [0.2.3] - 2025-09-28

### Added
- Scoped middleware for nested routers: Middleware added to nested routers now applies only to their specific routes, preventing global propagation.
- New middleware wrapping mechanism during router nesting to ensure isolation.
- Enhanced route compilation to preserve middleware scoping in both Radix and Base modes.

### Changed
- Updated nesting logic in `Router::nest` to wrap handlers with nested middleware instead of merging globally.
- Refactored `compile_inner` and added helper functions like `wrap_handler_with_middleware` and `wrap_tree_handlers` for scoped application.

## [0.2.2] - 2025-09-27

### Added
- **Router Merge Functionality**: New powerful router composition feature
  - `Router::merge()`: Combine multiple routers into a single router
  - Cross-mode merging: Seamlessly merge Base and Radix mode routers
  - State and middleware preservation during merge operations
  - Plugin-style architecture support with conditional merging
- **Extended HTTP Method Support**: Complete HTTP method coverage
  - `Router::any()`: Handle all HTTP methods with a single handler
  - `Router::connect()`: Support for HTTP CONNECT method (proxy/tunneling)
  - `Router::trace()`: Support for HTTP TRACE method (debugging/diagnostics)
  - Enhanced method matching with improved performance
- **Extensions System Optimization**: Major performance improvements
  - **Breaking**: Extensions map changed from `Box<dyn Any>` to `Arc<dyn Any + Send + Sync>`
  - `pub map: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>`: Zero-copy extension sharing
  - Reduced memory allocations and improved concurrent access
  - Better cache locality and reduced contention under high load

### Enhanced
- **Router Performance**: Significant improvements in route resolution and composition
  - Optimized merge operations with intelligent route deduplication
  - Enhanced cross-mode compatibility for maximum flexibility
  - Improved memory usage patterns in complex routing scenarios
- **Extension Management**: Better performance and thread safety
  - Atomic reference counting for extension values
  - Reduced lock contention in multi-threaded scenarios
  - Memory-efficient sharing across request contexts
- **HTTP Method Handling**: Complete method coverage with optimizations
  - Unified method dispatch system for better performance
  - Improved method parsing and validation
  - Enhanced debugging and error reporting for unsupported methods

### Performance
- **Extension Access**: 40% faster extension retrieval with Arc-based sharing
- **Router Merging**: Optimized merge operations with minimal overhead
- **Memory Usage**: Reduced memory footprint for extension storage
- **Method Dispatch**: Faster HTTP method resolution and routing

### Fixed
- Extension cleanup race conditions in high-concurrency scenarios
- Router merge conflicts with overlapping route patterns
- Method-specific middleware application edge cases
- Extension type safety issues in concurrent access patterns

### Changed
- **Breaking**: Extension storage now uses `Arc<dyn Any + Send + Sync>` instead of `Box<dyn Any>`
- **Breaking**: Extension map structure updated for better concurrent performance
- Router merge behavior now preserves route precedence more predictably
- Method handling unified across all HTTP methods for consistency

### Migration Notes
- **Extensions**: Update extension insertion/extraction code to work with Arc-based values
- **Custom Extensions**: Ensure all extension types implement `Send + Sync` for Arc compatibility
- **Router Merging**: Review route precedence in merged routers to ensure expected behavior

## [0.2.1] - 2024-09-20

### Added
- **Radix Tree Router**: New high-performance routing engine with O(log n) lookup time
  - `router/radix.rs`: Compressed trie implementation for ultra-fast route matching
  - `RouterMode` enum: Choose between Radix (default) and Base routing modes
  - Up to 3x performance improvement over regex-based routing
  - Memory-efficient shared prefix storage
- **Advanced Server Performance**: New performance optimization modules
  - `server/performance.rs`: Socket-level optimizations and benchmarking tools
  - `server/pool.rs`: Connection pooling and resource management
  - `PerformanceConfig::max_rps()`: Configuration for 65K+ RPS throughput
  - Zero-copy optimizations and CPU affinity settings
- **Enhanced Request Processing**: Improved request handling capabilities
  - Better parameter extraction with type safety
  - Optimized body parsing for large payloads
  - Streaming request support for file uploads
- **Response Optimization**: Advanced response generation features
  - Improved compression algorithms and content negotiation
  - Better caching header management
  - Streaming response support for large datasets

### Enhanced
- **Router Module (`router/mod.rs`)**: Complete refactoring for better performance
  - Radix tree integration as default routing engine
  - Backward compatibility with existing route definitions
  - Improved error handling and debugging information
- **Server Module (`server/mod.rs`)**: Major performance and stability improvements
  - Enhanced connection management with pooling
  - Better protocol detection and negotiation
  - Improved error handling and graceful shutdown
- **Request Module (`request/`)**: Enhanced request processing capabilities
  - Better multipart form handling
  - Improved parameter extraction performance
  - Enhanced security validation
- **Response Module (`response/`)**: Advanced response building and optimization
  - Better content type detection
  - Improved compression handling
  - Enhanced security headers management

### Performance
- **Route Matching**: 3x faster route resolution with Radix tree implementation
- **Memory Usage**: 50% reduction in memory footprint for typical applications
- **Request Throughput**: Optimized for 65,000+ requests per second
- **Connection Handling**: Improved connection pooling reduces resource overhead

### Fixed
- Route parameter extraction edge cases with special characters
- Memory leaks in connection pooling under high load
- HTTP/2 flow control issues with large payloads
- WebSocket connection cleanup race conditions
- CORS handling with complex preflight scenarios

### Changed
- **Router**: Default routing mode changed to Radix for better performance
- **Server**: Enhanced performance configuration options
- **Request/Response**: Improved API consistency and error handling
- Internal optimizations for better async performance

## [0.2.0] - 2024-09-14

### Added
- **WebSocket Support**: Complete WebSocket implementation with connection management, message handling, and batch processing
- **Advanced Middleware Stack**:
  - `CompressionMiddleware`: Gzip and Brotli compression with configurable levels
  - `RateLimitingMiddleware`: Token bucket algorithm with per-IP and custom key extraction
  - `SecurityMiddleware`: HSTS, CSP, and security headers
  - `RequestIdMiddleware`: Request tracking with UUID and NanoID support
- **File Upload Support**: Multipart form data handling with streaming and temp file management
- **Enhanced Error Handling**: Custom error types, structured error responses, and middleware-based error handling
- **Server Configuration**: HTTP/2 configuration, TLS support, and protocol detection
- **Request Extractors**: `State<T>`, `Extension<T>`, `Form<T>` extractors for better ergonomics
- **Cookie Management**: Full cookie jar implementation with SameSite support

### Enhanced
- **Router Performance**: Compiled router with path optimization and faster matching
- **HTTP/2 Support**: Full HTTP/2 implementation with ALPN negotiation
- **TLS Integration**: rustls-based TLS with self-signed certificate generation
- **Memory Management**: Optimized request/response handling with reduced allocations
- **Concurrent Handling**: Improved async processing with better error propagation

### Fixed
- WebSocket upgrade handling race conditions
- Path parameter extraction with special characters
- Memory leaks in long-running connections
- HTTP/2 stream multiplexing issues
- CORS preflight request handling

### Changed
- **Breaking**: Middleware trait now uses separate `before` and `after` methods
- **Breaking**: Error handling moved to dedicated error types and responses
- Router API now uses builder pattern consistently
- Response building simplified with `ResponseBuilder`

## [0.1.2] - 2024-08-15

### Added
- Basic middleware support with `LoggerMiddleware`
- Path parameter extraction with `:param` syntax
- Query parameter parsing and extraction
- JSON request/response handling
- Basic error handling and custom error types

### Fixed
- Route matching precedence issues
- JSON deserialization error messages
- HTTP method parsing edge cases

## [0.1.1] - 2024-08-05

### Added
- Wildcard route matching with `*param` syntax
- Nested router support for modular applications
- Extensions system for request-scoped data
- Basic HTTP status code utilities

### Fixed
- Route compilation performance issues
- Memory usage in request parsing
- Header handling for non-ASCII values

## [0.1.0] - 2024-08-01

### Added
- Initial release of Ignitia web framework
- Core HTTP server based on Hyper
- Basic routing system with method-specific handlers
- Request and Response abstractions
- Async/await support throughout
- Builder pattern for server and router configuration
- Development-focused error messages and debugging
- MIT license and initial documentation

### Features
- HTTP/1.1 support with keep-alive
- Route handlers with automatic extraction
- Minimalist API inspired by modern web frameworks
- Zero-config development server
- Fast compilation times
- Memory-safe request handling

***

## Version Numbering

Ignitia follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

## Migration Guides

For breaking changes and migration instructions between major versions, see our [Migration Guide](MIGRATION.md).

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and contribution guidelines.

## Security

Security vulnerabilities should be reported privately. See our [Security Policy](SECURITY.md) for details.
