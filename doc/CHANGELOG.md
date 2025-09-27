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
