//! CORS (Cross-Origin Resource Sharing) middleware for the Ignitia web framework.
//!
//! This module provides comprehensive CORS support with flexible configuration options,
//! allowing cross-origin requests while maintaining security. The implementation follows
//! the CORS specification and provides both strict and permissive configurations.
//!
//! # Overview
//!
//! CORS is a security feature implemented by browsers that restricts cross-origin HTTP requests.
//! This middleware handles CORS preflight requests and adds appropriate CORS headers to responses,
//! enabling controlled access from different origins.
//!
//! # Features
//!
//! - **Flexible Origin Control** - Allow specific origins, wildcards, or regex patterns
//! - **Method Filtering** - Specify allowed HTTP methods
//! - **Header Management** - Control allowed and exposed headers
//! - **Credentials Support** - Enable/disable credentials (cookies, authorization headers)
//! - **Preflight Caching** - Configure max-age for preflight requests
//! - **Builder Pattern** - Fluent API for configuration
//!
//! # Examples
//!
//! ## Permissive CORS (Development)
//!
//! ```
//! use ignitia::prelude::*;
//!
//! let router = Router::new()
//!     .middleware(CorsMiddleware::permissive())
//!     .get("/api/data", || async { "Data" });
//! ```
//!
//! ## Strict CORS (Production)
//!
//! ```
//! use ignitia::prelude::*;
//!
//! let cors = CorsMiddleware::new()
//!     .allowed_origins(&["https://example.com", "https://app.example.com"])
//!     .allowed_methods(&[Method::GET, Method::POST])
//!     .allowed_headers(&["content-type", "authorization"])
//!     .allow_credentials()
//!     .max_age(3600);
//!
//! let router = Router::new()
//!     .middleware(cors)
//!     .get("/api/data", || async { "Data" });
//! ```
//!
//! ## Regex-based Origin Matching
//!
//! ```
//! use ignitia::prelude::*;
//!
//! let cors = CorsMiddleware::new()
//!     .allowed_origin_regex(r"^https://.*\.example\.com$")
//!     .allowed_methods(&[Method::GET, Method::POST])
//!     .max_age(7200);
//!
//! let router = Router::new()
//!     .middleware(cors)
//!     .get("/api/data", || async { "Data" });
//! ```
//!
//! ## API with Credentials
//!
//! ```
//! use ignitia::prelude::*;
//!
//! let cors = CorsMiddleware::new()
//!     .allowed_origins(&["https://app.example.com"])
//!     .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
//!     .allowed_headers(&["content-type", "authorization", "x-api-key"])
//!     .expose_headers(&["x-request-id", "x-ratelimit-remaining"])
//!     .allow_credentials()
//!     .max_age(86400);
//!
//! let router = Router::new()
//!     .middleware(cors)
//!     .post("/api/users", || async { Response::json(json!({"status": "created"})) });
//! ```

use crate::middleware::{Middleware, Next};
use crate::{Request, Response, Result};
use http::{header, Method, StatusCode};
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::debug;

/// CORS configuration builder providing flexible cross-origin resource sharing setup.
///
/// This builder allows fine-grained control over CORS behavior, including origin validation,
/// method filtering, header management, and credential handling. Use the builder pattern
/// to construct a [`CorsMiddleware`] instance.
///
/// # Examples
///
/// ```
/// use ignitia::middleware::CorsMiddleware;
/// use http::Method;
///
/// let cors = CorsMiddleware::new()
///     .allowed_origins(&["https://example.com"])
///     .allowed_methods(&[Method::GET, Method::POST])
///     .allowed_headers(&["content-type"])
///     .max_age(3600);
/// ```
#[derive(Clone)]
pub struct Cors {
    inner: Arc<Inner>,
}

/// Internal CORS configuration state.
#[derive(Clone)]
struct Inner {
    /// Configured allowed origins
    allowed_origins: AllowedOrigins,
    /// Set of allowed HTTP methods
    allowed_methods: HashSet<Method>,
    /// Set of allowed request headers (lowercase)
    allowed_headers: HashSet<String>,
    /// List of headers to expose to the client
    exposed_headers: Vec<String>,
    /// Cache duration for preflight requests in seconds
    max_age: Option<u64>,
    /// Whether credentials (cookies, auth headers) are allowed
    allow_credentials: bool,
    /// Internal flag tracking credential support
    supports_credentials: bool,
    /// Whether all origins are allowed (wildcard)
    allowed_origins_all: bool,
}

/// Enumeration of allowed origin configurations.
#[derive(Clone)]
enum AllowedOrigins {
    /// Allow any origin (wildcard)
    Any,
    /// Allow specific origins from a set
    Exact(HashSet<String>),
    /// Allow origins matching a regex pattern
    Regex(Regex),
}

impl Cors {
    /// Create a new CORS configuration builder with sensible defaults.
    ///
    /// Default configuration:
    /// - **Origins**: Any origin (`*`)
    /// - **Methods**: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD
    /// - **Headers**: content-type, authorization, x-requested-with
    /// - **Max Age**: 86400 seconds (24 hours)
    /// - **Credentials**: Disabled
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origins(&["https://example.com"]);
    /// ```
    pub fn new() -> Self {
        let mut methods = HashSet::new();
        methods.extend(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
            Method::HEAD,
        ]);

        let mut headers = HashSet::new();
        headers.extend(vec![
            "content-type".to_string(),
            "authorization".to_string(),
            "x-requested-with".to_string(),
        ]);

        Self {
            inner: Arc::new(Inner {
                allowed_origins: AllowedOrigins::Any,
                allowed_methods: methods,
                allowed_headers: headers,
                exposed_headers: Vec::new(),
                max_age: Some(86400), // 24 hours
                allow_credentials: false,
                supports_credentials: false,
                allowed_origins_all: true,
            }),
        }
    }

    /// Allow requests from any origin (sets `Access-Control-Allow-Origin: *`).
    ///
    /// This is the most permissive configuration and should generally only be used
    /// for public APIs or development. Cannot be combined with credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allow_any_origin()
    ///     .allowed_methods(&[Method::GET, Method::POST]);
    /// ```
    pub fn allow_any_origin(mut self) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allowed_origins = AllowedOrigins::Any;
        inner.allowed_origins_all = true;
        self
    }

    /// Allow requests from a specific origin.
    ///
    /// Can be called multiple times to allow multiple origins. The origin should include
    /// the protocol and port (e.g., `https://example.com:8080`).
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin URL to allow
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origin("https://example.com")
    ///     .allowed_origin("https://api.example.com");
    /// ```
    pub fn allowed_origin(mut self, origin: &str) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        if let AllowedOrigins::Exact(ref mut origins) = inner.allowed_origins {
            origins.insert(origin.to_string());
        } else {
            let mut origins = HashSet::new();
            origins.insert(origin.to_string());
            inner.allowed_origins = AllowedOrigins::Exact(origins);
            inner.allowed_origins_all = false;
        }
        self
    }

    /// Allow requests from multiple specific origins.
    ///
    /// Replaces any previously configured origins with the provided list.
    ///
    /// # Arguments
    ///
    /// * `origins` - Slice of origin URLs to allow
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origins(&[
    ///         "https://example.com",
    ///         "https://app.example.com",
    ///         "https://admin.example.com",
    ///     ]);
    /// ```
    pub fn allowed_origins(mut self, origins: &[&str]) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        let origin_set: HashSet<String> = origins.iter().map(|&s| s.to_string()).collect();
        inner.allowed_origins = AllowedOrigins::Exact(origin_set);
        inner.allowed_origins_all = false;
        self
    }

    /// Allow origins matching a regular expression pattern.
    ///
    /// Useful for allowing subdomains or dynamic origin patterns. The pattern should
    /// match the full origin URL including protocol.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Regular expression pattern to match origins
    ///
    /// # Panics
    ///
    /// Panics if the regex pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// // Allow all subdomains of example.com
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origin_regex(r"^https://.*\.example\.com$");
    ///
    /// // Allow localhost with any port
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origin_regex(r"^https://localhost:\d+$");
    /// ```
    pub fn allowed_origin_regex(mut self, pattern: &str) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allowed_origins = AllowedOrigins::Regex(
            Regex::new(pattern).expect("Invalid regex pattern for CORS origins"),
        );
        inner.allowed_origins_all = false;
        self
    }

    /// Set the allowed HTTP methods for CORS requests.
    ///
    /// Replaces any previously configured methods. These methods will be returned
    /// in the `Access-Control-Allow-Methods` header for preflight requests.
    ///
    /// # Arguments
    ///
    /// * `methods` - Slice of HTTP methods to allow
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    /// use http::Method;
    ///
    /// // Read-only API
    /// let cors = CorsMiddleware::new()
    ///     .allowed_methods(&[Method::GET, Method::HEAD, Method::OPTIONS]);
    ///
    /// // Full CRUD API
    /// let cors = CorsMiddleware::new()
    ///     .allowed_methods(&[
    ///         Method::GET,
    ///         Method::POST,
    ///         Method::PUT,
    ///         Method::DELETE,
    ///         Method::PATCH,
    ///     ]);
    /// ```
    pub fn allowed_methods(mut self, methods: &[Method]) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allowed_methods = methods.iter().cloned().collect();
        self
    }

    /// Set the allowed request headers for CORS requests.
    ///
    /// Replaces any previously configured headers. Header names are automatically
    /// converted to lowercase for case-insensitive matching.
    ///
    /// # Arguments
    ///
    /// * `headers` - Slice of header names to allow
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allowed_headers(&[
    ///         "content-type",
    ///         "authorization",
    ///         "x-api-key",
    ///         "x-request-id",
    ///     ]);
    /// ```
    pub fn allowed_headers(mut self, headers: &[&str]) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allowed_headers = headers.iter().map(|&s| s.to_lowercase()).collect();
        self
    }

    /// Set headers that should be exposed to the client.
    ///
    /// These headers will be included in the `Access-Control-Expose-Headers` response header,
    /// making them accessible to JavaScript on the client side.
    ///
    /// # Arguments
    ///
    /// * `headers` - Slice of header names to expose
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .expose_headers(&[
    ///         "x-request-id",
    ///         "x-ratelimit-remaining",
    ///         "x-ratelimit-reset",
    ///     ]);
    /// ```
    pub fn expose_headers(mut self, headers: &[&str]) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.exposed_headers = headers.iter().map(|&s| s.to_string()).collect();
        self
    }

    /// Set the max-age for preflight request caching.
    ///
    /// Determines how long (in seconds) browsers can cache the preflight response.
    /// Longer durations reduce preflight requests but may delay configuration changes.
    ///
    /// # Arguments
    ///
    /// * `seconds` - Cache duration in seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// // Cache for 1 hour
    /// let cors = CorsMiddleware::new().max_age(3600);
    ///
    /// // Cache for 24 hours
    /// let cors = CorsMiddleware::new().max_age(86400);
    ///
    /// // Minimal caching for development
    /// let cors = CorsMiddleware::new().max_age(60);
    /// ```
    pub fn max_age(mut self, seconds: u64) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.max_age = Some(seconds);
        self
    }

    /// Enable credentials support (cookies, authorization headers, TLS certificates).
    ///
    /// When enabled, the `Access-Control-Allow-Credentials: true` header is sent.
    /// Cannot be used with wildcard origins - specific origins must be configured.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origins(&["https://app.example.com"])
    ///     .allow_credentials();
    /// ```
    pub fn allow_credentials(mut self) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allow_credentials = true;
        inner.supports_credentials = true;
        self
    }

    /// Disable credentials support.
    ///
    /// This is the default behavior. Use this to explicitly disable credentials
    /// if they were previously enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allow_credentials()  // Enable
    ///     .disable_credentials();  // Then disable
    /// ```
    pub fn disable_credentials(mut self) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allow_credentials = false;
        inner.supports_credentials = false;
        self
    }

    /// Build and validate the CORS middleware.
    ///
    /// Validates the configuration, ensuring no conflicting settings (e.g., credentials
    /// with wildcard origins). Returns an error if validation fails.
    ///
    /// # Returns
    ///
    /// Returns `Ok(CorsMiddleware)` if configuration is valid, or `Err` if validation fails.
    ///
    /// # Errors
    ///
    /// - Returns an error if credentials are enabled with wildcard origins
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origins(&["https://example.com"])
    ///     .allow_credentials()
    ///     .build()
    ///     .expect("Invalid CORS configuration");
    /// ```
    pub fn build(self) -> Result<CorsMiddleware> {
        self.validate()?;
        Ok(CorsMiddleware { cors: self })
    }

    /// Validate the CORS configuration.
    ///
    /// Checks for configuration conflicts and security issues.
    ///
    /// # Validation Rules
    ///
    /// - Credentials cannot be used with wildcard origins
    /// - Credentials cannot be used with `*` in the allowed origins set
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if valid, or `Err` with a descriptive error message.
    fn validate(&self) -> Result<()> {
        let inner = &self.inner;

        // Check if credentials are allowed with wildcard origin
        if inner.supports_credentials {
            match inner.allowed_origins {
                AllowedOrigins::Any => {
                    return Err(crate::Error::Internal(
                        "Cannot use wildcard origin with credentials".into(),
                    ));
                }
                AllowedOrigins::Exact(ref origins) if origins.contains("*") => {
                    return Err(crate::Error::Internal(
                        "Cannot use wildcard origin with credentials".into(),
                    ));
                }
                _ => {}
            }
        }

        debug!("CORS configuration validated successfully");
        Ok(())
    }
}

/// CORS middleware that handles cross-origin requests.
///
/// This middleware intercepts requests, handles CORS preflight (OPTIONS) requests,
/// and adds appropriate CORS headers to responses. Created via [`Cors::build()`]
/// or convenience methods.
///
/// # Examples
///
/// ```
/// use ignitia::prelude::*;
///
/// let cors = CorsMiddleware::permissive();
///
/// let router = Router::new()
///     .middleware(cors)
///     .get("/api/data", || async { "Data" });
/// ```
#[derive(Clone)]
pub struct CorsMiddleware {
    cors: Cors,
}

impl CorsMiddleware {
    /// Create a new CORS middleware builder.
    ///
    /// Alias for [`Cors::new()`] for convenience.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allowed_origins(&["https://example.com"]);
    /// ```
    pub fn new() -> Cors {
        Cors::new()
    }

    /// Create a permissive CORS configuration suitable for development.
    ///
    /// This configuration:
    /// - Allows any origin
    /// - Allows all common HTTP methods
    /// - Allows common headers
    /// - Sets max-age to 1 hour
    /// - Does not allow credentials
    ///
    /// **Warning**: Do not use in production for sensitive APIs.
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::prelude::*;
    ///
    /// let router = Router::new()
    ///     .middleware(CorsMiddleware::permissive())
    ///     .get("/api/data", || async { "Data" });
    /// ```
    pub fn permissive() -> Self {
        Self {
            cors: Cors::new().allow_any_origin().max_age(3600),
        }
    }

    /// Create a strict CORS configuration for production.
    ///
    /// Requires explicit configuration of allowed origins. Provides sensible
    /// defaults for other settings with security in mind.
    ///
    /// # Arguments
    ///
    /// * `origins` - Specific origins to allow
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::middleware::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::strict(&[
    ///     "https://example.com",
    ///     "https://app.example.com",
    /// ]);
    /// ```
    pub fn strict(origins: &[&str]) -> Self {
        Self {
            cors: Cors::new()
                .allowed_origins(origins)
                .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .max_age(3600),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for CorsMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // Handle preflight OPTIONS requests
        if req.method == Method::OPTIONS {
            if self.is_preflight_request(&req) {
                return self.handle_preflight(&req);
            }
        }

        // Process regular request
        let mut res = next.run(req).await;
        self.add_cors_headers(&mut res);
        res
    }
}

impl CorsMiddleware {
    /// Check if a request is a CORS preflight request.
    ///
    /// A request is considered a preflight if it's an OPTIONS request with
    /// the `Access-Control-Request-Method` header.
    fn is_preflight_request(&self, req: &Request) -> bool {
        req.header("access-control-request-method").is_some()
    }

    /// Handle a CORS preflight OPTIONS request.
    ///
    /// Validates the origin, method, and headers, then returns an appropriate
    /// preflight response with CORS headers.
    fn handle_preflight(&self, req: &Request) -> Response {
        let origin = match req.header("origin") {
            Some(origin) => origin,
            None => {
                // No origin header - not a CORS request
                return Response::new(StatusCode::BAD_REQUEST);
            }
        };

        // Validate origin
        if !self.is_origin_allowed(origin) {
            debug!("CORS preflight rejected: origin not allowed: {}", origin);
            return Response::new(StatusCode::FORBIDDEN);
        }

        // Validate request method
        if let Some(request_method) = req.header("access-control-request-method") {
            if let Ok(method) = Method::from_bytes(request_method.as_bytes()) {
                if !self.cors.inner.allowed_methods.contains(&method) {
                    debug!(
                        "CORS preflight rejected: method not allowed: {}",
                        request_method
                    );
                    return Response::new(StatusCode::METHOD_NOT_ALLOWED);
                }
            }
        }

        // Validate request headers
        if let Some(request_headers) = req.header("access-control-request-headers") {
            for header in request_headers.split(',') {
                let header = header.trim().to_lowercase();
                if !self.cors.inner.allowed_headers.contains(&header) {
                    debug!("CORS preflight rejected: header not allowed: {}", header);
                    return Response::new(StatusCode::BAD_REQUEST);
                }
            }
        }

        // Create preflight response
        let mut response = Response::new(StatusCode::NO_CONTENT);
        self.add_preflight_headers(&mut response, origin);

        debug!("CORS preflight request handled successfully");
        response
    }

    /// Check if an origin is allowed based on configuration.
    fn is_origin_allowed(&self, origin: &str) -> bool {
        if origin.is_empty() {
            return false;
        }

        match &self.cors.inner.allowed_origins {
            AllowedOrigins::Any => true,
            AllowedOrigins::Exact(origins) => origins.contains(origin),
            AllowedOrigins::Regex(regex) => regex.is_match(origin),
        }
    }

    /// Add CORS headers to a regular response.
    fn add_cors_headers(&self, response: &mut Response) {
        let inner = &self.cors.inner;

        // Add Allow-Origin header
        if inner.allowed_origins_all {
            response
                .headers
                .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
        }

        // Add exposed headers
        if !inner.exposed_headers.is_empty() {
            response.headers.insert(
                header::ACCESS_CONTROL_EXPOSE_HEADERS,
                inner.exposed_headers.join(", ").parse().unwrap(),
            );
        }

        // Add credentials header if enabled
        if inner.supports_credentials {
            response.headers.insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                "true".parse().unwrap(),
            );
        }
    }

    /// Add CORS headers to a preflight response.
    fn add_preflight_headers(&self, response: &mut Response, origin: &str) {
        let inner = &self.cors.inner;

        // Set allowed origin
        response.headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            if inner.allowed_origins_all {
                "*".parse().unwrap()
            } else {
                origin.parse().unwrap()
            },
        );

        // Set allowed methods
        let methods: Vec<String> = inner
            .allowed_methods
            .iter()
            .map(|m| m.as_str().to_string())
            .collect();
        response.headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            methods.join(", ").parse().unwrap(),
        );

        // Set allowed headers
        if !inner.allowed_headers.is_empty() {
            response.headers.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                inner
                    .allowed_headers
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
                    .parse()
                    .unwrap(),
            );
        }

        // Set max-age if configured
        if let Some(max_age) = inner.max_age {
            response.headers.insert(
                header::ACCESS_CONTROL_MAX_AGE,
                max_age.to_string().parse().unwrap(),
            );
        }

        // Set credentials if enabled
        if inner.supports_credentials {
            response.headers.insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                "true".parse().unwrap(),
            );
        }
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::permissive()
    }
}
