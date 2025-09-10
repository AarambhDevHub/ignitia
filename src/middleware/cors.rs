//! # Cross-Origin Resource Sharing (CORS) Middleware
//!
//! This module provides comprehensive CORS middleware for the Ignitia web framework,
//! enabling secure cross-origin requests from web browsers. CORS is essential for
//! modern web applications that need to make requests from one domain to another.
//!
//! ## Features
//!
//! - **Flexible Origin Control**: Support for specific origins, multiple origins, and regex patterns
//! - **Method Restrictions**: Configure allowed HTTP methods with secure defaults
//! - **Header Management**: Control both request and response headers
//! - **Automatic Preflight Handling**: Built-in OPTIONS request handling
//! - **Credentials Support**: Secure credential handling with proper validation
//! - **Security First**: Secure defaults with comprehensive validation
//! - **Builder Pattern**: Fluent API for easy configuration
//!
//! ## CORS Fundamentals
//!
//! CORS (Cross-Origin Resource Sharing) is a W3C specification that allows servers
//! to specify who can access their resources and how. Modern browsers enforce the
//! same-origin policy, and CORS provides a way to relax these restrictions securely.
//!
//! ### Key CORS Headers
//! - `Access-Control-Allow-Origin`: Specifies which origins can access the resource
//! - `Access-Control-Allow-Methods`: Lists allowed HTTP methods
//! - `Access-Control-Allow-Headers`: Lists allowed request headers
//! - `Access-Control-Expose-Headers`: Headers that clients can access
//! - `Access-Control-Max-Age`: How long browsers can cache preflight results
//! - `Access-Control-Allow-Credentials`: Allows cookies/credentials in requests
//!
//! ### Preflight Requests
//! For certain requests, browsers send a preflight OPTIONS request first to check
//! if the actual request is allowed. This middleware handles these automatically.
//!
//! ## Quick Start
//!
//! ### Basic Usage (Development)
//! ```
//! use ignitia::{Router, middleware::CorsMiddleware};
//!
//! let router = Router::new()
//!     .middleware(CorsMiddleware::new()?)
//!     .get("/api/data", || async {
//!         Ok(ignitia::Response::json(&"API Data")?)
//!     });
//! ```
//!
//! ### Production Setup
//! ```
//! use ignitia::{Router, middleware::Cors};
//!
//! let cors = Cors::new()
//!     .allowed_origin("https://myapp.com")
//!     .allowed_methods(&[Method::GET, Method::POST])
//!     .allowed_headers(&["Content-Type", "Authorization"])
//!     .max_age(3600)
//!     .build()?;
//!
//! let router = Router::new()
//!     .middleware(cors)
//!     .get("/api/users", || async {
//!         Ok(ignitia::Response::json(&"Users")?)
//!     });
//! ```
//!
//! ## Configuration Examples
//!
//! ### Multiple Origins
//! ```
//! use ignitia::middleware::Cors;
//!
//! let cors = Cors::new()
//!     .allowed_origins(&[
//!         "https://app1.com",
//!         "https://app2.com",
//!         "https://admin.myapp.com"
//!     ])
//!     .build()?;
//! ```
//!
//! ### Regex Pattern Matching
//! ```
//! use ignitia::middleware::Cors;
//!
//! // Allow all subdomains of myapp.com
//! let cors = Cors::new()
//!     .allowed_origin_regex(r"^https://[a-zA-Z0-9-]+\.myapp\.com$")
//!     .build()?;
//! ```
//!
//! ### Development with Credentials
//! ```
//! use ignitia::middleware::Cors;
//!
//! let cors = Cors::new()
//!     .allowed_origin("http://localhost:3000")
//!     .allow_credentials()
//!     .allowed_headers(&["Content-Type", "Authorization", "X-Requested-With"])
//!     .build()?;
//! ```
//!
//! ### API Gateway Setup
//! ```
//! use ignitia::middleware::Cors;
//!
//! let cors = Cors::secure_api(&[
//!     "https://dashboard.myapp.com",
//!     "https://mobile.myapp.com"
//! ])
//! .max_age(86400) // 24 hours
//! .expose_headers(&["X-Total-Count", "X-Page-Count"])
//! .build()?;
//! ```
//!
//! ## Advanced Usage
//!
//! ### Custom Middleware for Complex Logic
//! ```
//! use ignitia::{Middleware, Request, Response, Result};
//! use async_trait::async_trait;
//! use http::header;
//!
//! pub struct DynamicCorsMiddleware {
//!     allowed_origins: HashMap<String, Vec<String>>,
//! }
//!
//! impl DynamicCorsMiddleware {
//!     pub fn new() -> Self {
//!         let mut origins = HashMap::new();
//!         origins.insert("api".to_string(), vec![
//!             "https://api.myapp.com".to_string()
//!         ]);
//!         origins.insert("admin".to_string(), vec![
//!             "https://admin.myapp.com".to_string()
//!         ]);
//!
//!         Self { allowed_origins: origins }
//!     }
//! }
//!
//! #[async_trait]
//! impl Middleware for DynamicCorsMiddleware {
//!     async fn after(&self, res: &mut Response) -> Result<()> {
//!         // Implement custom origin validation logic here
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Security Best Practices
//!
//! ### Production Security Checklist
//! - ✅ **Never use `*` for origins** in production with sensitive data
//! - ✅ **Specify exact origins** for production environments
//! - ✅ **Be cautious with credentials** - never combine `*` origins with credentials
//! - ✅ **Validate origins dynamically** for multi-tenant applications
//! - ✅ **Use HTTPS origins only** in production
//! - ✅ **Limit exposed headers** to only what's necessary
//! - ✅ **Set reasonable max-age** for preflight caching
//!
//! ### Secure Production Example
//! ```
//! use ignitia::middleware::Cors;
//!
//! let cors = Cors::new()
//!     .allowed_origins(&[
//!         "https://myapp.com",
//!         "https://www.myapp.com",
//!         "https://mobile.myapp.com"
//!     ])
//!     .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
//!     .allowed_headers(&["Content-Type", "Authorization"])
//!     .allow_credentials()
//!     .max_age(3600) // 1 hour
//!     .build()?;
//! ```
//!
//! ## Troubleshooting Common Issues
//!
//! ### 1. Preflight Request Failures
//! **Problem**: Browser shows CORS error on OPTIONS requests
//! ```
//! Access to XMLHttpRequest has been blocked by CORS policy:
//! Response to preflight request doesn't pass access control check
//! ```
//! **Solution**: Ensure your middleware handles preflight requests properly:
//! ```
//! let cors = Cors::new()
//!     .allowed_methods(&[Method::OPTIONS, Method::GET, Method::POST])
//!     .build()?;
//! ```
//!
//! ### 2. Credentials Not Working
//! **Problem**: Cookies/authorization headers not sent with requests
//! **Solution**: Configure both server and client correctly:
//! ```
//! // Server side - specific origin required
//! let cors = Cors::new()
//!     .allowed_origin("https://myapp.com") // Cannot be "*" with credentials
//!     .allow_credentials()
//!     .build()?;
//! ```
//! ```
//! // Client side
//! fetch('/api/data', {
//!     credentials: 'include' // Include cookies
//! });
//! ```
//!
//! ### 3. Multiple Origins Not Working
//! **Problem**: Only the first origin works
//! **Solution**: Use the array method, not comma-separated strings:
//! ```
//! // ✅ Correct
//! .allowed_origins(&["https://app1.com", "https://app2.com"])
//!
//! // ❌ Incorrect
//! .allowed_origin("https://app1.com,https://app2.com")
//! ```
//!
//! ### 4. Custom Headers Blocked
//! **Problem**: Custom headers are rejected
//! **Solution**: Add them to allowed headers:
//! ```
//! let cors = Cors::new()
//!     .allowed_headers(&[
//!         "Content-Type",
//!         "Authorization",
//!         "X-Custom-Header",
//!         "X-API-Key"
//!     ])
//!     .build()?;
//! ```
//!
//! ## Browser Compatibility
//!
//! CORS is supported by all modern browsers:
//! - **Chrome**: 4+ (full support)
//! - **Firefox**: 3.5+ (full support)
//! - **Safari**: 4+ (full support)
//! - **Edge**: All versions (full support)
//! - **Internet Explorer**: 8+ (limited support, use XDomainRequest)
//!
//! ## Performance Considerations
//!
//! ### Preflight Caching
//! Set appropriate `max_age` to reduce preflight requests:
//! ```
//! let cors = Cors::new()
//!     .max_age(86400) // Cache for 24 hours
//!     .build()?;
//! ```
//!
//! ### Wildcard vs Specific Origins
//! - Wildcard (`*`) has better performance but less security
//! - Specific origins require origin validation on each request
//! - Consider using regex patterns for subdomain matching

use crate::middleware::Middleware;
use crate::{Request, Response, Result};
use http::{header, Method, StatusCode};
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::debug;

/// CORS configuration builder following a fluent API pattern.
///
/// This builder provides a comprehensive way to configure Cross-Origin Resource Sharing
/// for your web application. It follows the builder pattern for easy configuration
/// and includes validation to ensure secure defaults.
///
/// # Examples
///
/// ## Basic Configuration
/// ```
/// use ignitia::middleware::Cors;
/// use http::Method;
///
/// let cors = Cors::new()
///     .allowed_origin("https://myapp.com")
///     .allowed_methods(&[Method::GET, Method::POST])
///     .build()?;
/// ```
///
/// ## Advanced Configuration
/// ```
/// use ignitia::middleware::Cors;
/// use http::Method;
///
/// let cors = Cors::new()
///     .allowed_origins(&["https://app1.com", "https://app2.com"])
///     .allowed_methods(&[Method::GET, Method::POST, Method::PUT, Method::DELETE])
///     .allowed_headers(&["Content-Type", "Authorization", "X-Custom-Header"])
///     .expose_headers(&["X-Total-Count"])
///     .allow_credentials()
///     .max_age(3600)
///     .build()?;
/// ```
#[derive(Clone)]
pub struct Cors {
    inner: Arc<Inner>,
}

/// Internal CORS configuration data.
///
/// This struct holds the actual configuration values used by the CORS middleware.
/// It's wrapped in an Arc for efficient cloning and shared across requests.
#[derive(Clone)]
struct Inner {
    allowed_origins: AllowedOrigins,
    allowed_methods: HashSet<Method>,
    allowed_headers: HashSet<String>,
    exposed_headers: Vec<String>,
    max_age: Option<u64>,
    allow_credentials: bool,
    supports_credentials: bool,
    allowed_origins_all: bool,
}

/// Represents the different ways origins can be specified for CORS.
///
/// This enum allows for flexible origin matching:
/// - `Any`: Allows all origins (*)
/// - `Exact`: Matches specific origins exactly
/// - `Regex`: Uses regex patterns for complex matching scenarios
#[derive(Clone)]
enum AllowedOrigins {
    /// Allow all origins - equivalent to Access-Control-Allow-Origin: *
    Any,
    /// Allow specific origins only - matches exactly
    Exact(HashSet<String>),
    /// Allow origins matching a regex pattern - for subdomain matching
    Regex(Regex),
}

impl Cors {
    /// Creates a new CORS configuration with secure defaults.
    ///
    /// The default configuration includes:
    /// - **Origins**: All origins (*) - should be changed for production
    /// - **Methods**: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD
    /// - **Headers**: content-type, authorization, x-requested-with
    /// - **Max Age**: 24 hours (86400 seconds)
    /// - **Credentials**: Disabled
    ///
    /// # Security Note
    /// The default configuration allows all origins, which is suitable for
    /// development but should be restricted in production environments.
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let cors = Cors::new();
    /// // This creates a permissive configuration suitable for development
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

    /// Allow requests from any origin.
    ///
    /// This sets the `Access-Control-Allow-Origin` header to `*`, which allows
    /// requests from any origin. This is convenient for development but should
    /// be avoided in production, especially when credentials are involved.
    ///
    /// # Security Warning
    /// Using `allow_any_origin()` with `allow_credentials()` is not supported
    /// and will cause validation to fail when calling `build()`.
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let cors = Cors::new()
    ///     .allow_any_origin()
    ///     .build()?;
    /// ```
    pub fn allow_any_origin(mut self) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allowed_origins = AllowedOrigins::Any;
        inner.allowed_origins_all = true;
        self
    }

    /// Allow requests from a specific origin.
    ///
    /// This method adds a single origin to the list of allowed origins.
    /// The origin must include the protocol (http:// or https://) and
    /// should not include trailing slashes.
    ///
    /// # Parameters
    /// - `origin`: The origin URL to allow (e.g., "https://myapp.com")
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let cors = Cors::new()
    ///     .allowed_origin("https://myapp.com")
    ///     .build()?;
    /// ```
    ///
    /// ## Multiple Calls
    /// You can call this method multiple times to allow multiple origins:
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let cors = Cors::new()
    ///     .allowed_origin("https://app1.com")
    ///     .allowed_origin("https://app2.com")
    ///     .build()?;
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
    /// This method sets the allowed origins to exactly match the provided list.
    /// Any previously configured origins will be replaced.
    ///
    /// # Parameters
    /// - `origins`: Array of origin URLs to allow
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let cors = Cors::new()
    ///     .allowed_origins(&[
    ///         "https://myapp.com",
    ///         "https://www.myapp.com",
    ///         "https://mobile.myapp.com"
    ///     ])
    ///     .build()?;
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
    /// This method is useful for allowing subdomains or complex origin patterns.
    /// The regex is compiled once during configuration and cached for performance.
    ///
    /// # Parameters
    /// - `pattern`: Regular expression pattern to match against origins
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// // Allow all subdomains of myapp.com
    /// let cors = Cors::new()
    ///     .allowed_origin_regex(r"^https://[a-zA-Z0-9-]+\.myapp\.com$")
    ///     .build()?;
    /// ```
    ///
    /// ## Common Patterns
    /// ```
    /// // Allow localhost with any port for development
    /// let dev_cors = Cors::new()
    ///     .allowed_origin_regex(r"^https?://localhost(:\d+)?$")
    ///     .build()?;
    ///
    /// // Allow staging and production environments
    /// let env_cors = Cors::new()
    ///     .allowed_origin_regex(r"^https://(staging\.|www\.)?myapp\.com$")
    ///     .build()?;
    /// ```
    ///
    /// # Panics
    /// Panics if the provided regex pattern is invalid.
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
    /// This configures which HTTP methods are allowed for cross-origin requests.
    /// The methods are used in both preflight responses and validation.
    ///
    /// # Parameters
    /// - `methods`: Array of HTTP methods to allow
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    /// use http::Method;
    ///
    /// // Read-only API
    /// let readonly_cors = Cors::new()
    ///     .allowed_methods(&[Method::GET, Method::HEAD, Method::OPTIONS])
    ///     .build()?;
    ///
    /// // Full REST API
    /// let rest_cors = Cors::new()
    ///     .allowed_methods(&[
    ///         Method::GET,
    ///         Method::POST,
    ///         Method::PUT,
    ///         Method::DELETE,
    ///         Method::PATCH,
    ///         Method::OPTIONS
    ///     ])
    ///     .build()?;
    /// ```
    ///
    /// # Note
    /// OPTIONS is typically included automatically for preflight handling,
    /// but it's good practice to include it explicitly.
    pub fn allowed_methods(mut self, methods: &[Method]) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allowed_methods = methods.iter().cloned().collect();
        self
    }

    /// Set the allowed request headers for CORS requests.
    ///
    /// This configures which headers can be used in the actual request.
    /// These headers are validated during preflight requests and must be
    /// explicitly allowed.
    ///
    /// # Parameters
    /// - `headers`: Array of header names to allow (case-insensitive)
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// // Basic headers for most APIs
    /// let basic_cors = Cors::new()
    ///     .allowed_headers(&["Content-Type", "Authorization"])
    ///     .build()?;
    ///
    /// // Extended headers for advanced APIs
    /// let advanced_cors = Cors::new()
    ///     .allowed_headers(&[
    ///         "Content-Type",
    ///         "Authorization",
    ///         "X-Requested-With",
    ///         "Accept",
    ///         "Origin",
    ///         "X-Api-Key",
    ///         "X-Custom-Header"
    ///     ])
    ///     .build()?;
    /// ```
    ///
    /// # Common Headers
    /// - `Content-Type`: Required for POST/PUT requests with JSON/form data
    /// - `Authorization`: Required for Bearer tokens and basic auth
    /// - `X-Requested-With`: Often required by AJAX libraries
    /// - `Accept`: For content negotiation
    pub fn allowed_headers(mut self, headers: &[&str]) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allowed_headers = headers.iter().map(|&s| s.to_lowercase()).collect();
        self
    }

    /// Set the headers that should be exposed to the client.
    ///
    /// By default, only simple response headers are accessible to JavaScript.
    /// This method allows you to expose additional headers that the client
    /// can read from the response.
    ///
    /// # Parameters
    /// - `headers`: Array of header names to expose
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// // Expose pagination headers
    /// let pagination_cors = Cors::new()
    ///     .expose_headers(&[
    ///         "X-Total-Count",
    ///         "X-Page-Count",
    ///         "X-Per-Page"
    ///     ])
    ///     .build()?;
    ///
    /// // Expose rate limiting headers
    /// let rate_limit_cors = Cors::new()
    ///     .expose_headers(&[
    ///         "X-RateLimit-Limit",
    ///         "X-RateLimit-Remaining",
    ///         "X-RateLimit-Reset"
    ///     ])
    ///     .build()?;
    /// ```
    ///
    /// # Note
    /// Simple response headers (like `Content-Type`, `Content-Length`) are
    /// always accessible and don't need to be explicitly exposed.
    pub fn expose_headers(mut self, headers: &[&str]) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.exposed_headers = headers.iter().map(|&s| s.to_string()).collect();
        self
    }

    /// Set the maximum age for preflight cache in seconds.
    ///
    /// This sets how long browsers can cache the preflight response,
    /// reducing the number of preflight requests for subsequent requests.
    ///
    /// # Parameters
    /// - `seconds`: Cache duration in seconds
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// // Cache for 1 hour
    /// let short_cache_cors = Cors::new()
    ///     .max_age(3600)
    ///     .build()?;
    ///
    /// // Cache for 24 hours (recommended for production)
    /// let long_cache_cors = Cors::new()
    ///     .max_age(86400)
    ///     .build()?;
    ///
    /// // Disable caching (useful for development)
    /// let no_cache_cors = Cors::new()
    ///     .max_age(0)
    ///     .build()?;
    /// ```
    ///
    /// # Recommendations
    /// - **Development**: 0 seconds (no caching)
    /// - **Staging**: 1 hour (3600 seconds)
    /// - **Production**: 24 hours (86400 seconds) or more
    pub fn max_age(mut self, seconds: u64) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.max_age = Some(seconds);
        self
    }

    /// Enable credentials support for CORS requests.
    ///
    /// This allows cookies, authorization headers, and TLS client certificates
    /// to be sent with cross-origin requests. When enabled, the wildcard (*)
    /// cannot be used for origins - specific origins must be configured.
    ///
    /// # Security Warning
    /// Enabling credentials increases security risk. Only enable if you need
    /// to send cookies or authorization headers with cross-origin requests,
    /// and always use specific origins, never wildcards.
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// // Secure credentials setup
    /// let credentials_cors = Cors::new()
    ///     .allowed_origin("https://myapp.com")  // Specific origin required
    ///     .allow_credentials()
    ///     .build()?;
    /// ```
    ///
    /// ## Client-side Usage
    /// ```
    /// // JavaScript fetch with credentials
    /// fetch('https://api.myapp.com/data', {
    ///     credentials: 'include'
    /// });
    ///
    /// // XMLHttpRequest with credentials
    /// xhr.withCredentials = true;
    /// ```
    pub fn allow_credentials(mut self) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allow_credentials = true;
        inner.supports_credentials = true;
        self
    }

    /// Disable credentials support (default behavior).
    ///
    /// This explicitly disables credentials support, which is the default
    /// and more secure option. Use this method if you want to be explicit
    /// about credentials being disabled.
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let no_credentials_cors = Cors::new()
    ///     .disable_credentials()
    ///     .build()?;
    /// ```
    pub fn disable_credentials(mut self) -> Self {
        let inner = Arc::make_mut(&mut self.inner);
        inner.allow_credentials = false;
        inner.supports_credentials = false;
        self
    }

    /// Build the CORS middleware with validation.
    ///
    /// This method validates the configuration and returns a `CorsMiddleware`
    /// instance that can be used with the router. Validation ensures that
    /// the configuration is secure and follows CORS specifications.
    ///
    /// # Returns
    /// - `Ok(CorsMiddleware)`: Successfully validated and built middleware
    /// - `Err(Error)`: Configuration validation failed
    ///
    /// # Validation Rules
    /// - Credentials cannot be combined with wildcard origins
    /// - At least one method must be allowed
    /// - Regex patterns must be valid
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// // This will succeed
    /// let cors = Cors::new()
    ///     .allowed_origin("https://myapp.com")
    ///     .build()?;
    ///
    /// // This will fail - credentials with wildcard origin
    /// let invalid_cors = Cors::new()
    ///     .allow_any_origin()
    ///     .allow_credentials()
    ///     .build(); // Returns Err
    /// ```
    pub fn build(self) -> Result<CorsMiddleware> {
        self.validate()?;
        Ok(CorsMiddleware { cors: self })
    }

    /// Validates the CORS configuration for security and correctness.
    ///
    /// This method performs comprehensive validation to ensure the CORS
    /// configuration follows security best practices and CORS specifications.
    ///
    /// # Validation Checks
    /// 1. **Credentials Security**: Ensures credentials are not combined with wildcard origins
    /// 2. **Method Coverage**: Verifies at least one method is allowed
    /// 3. **Pattern Validity**: Confirms regex patterns are valid
    ///
    /// # Errors
    /// Returns `Error::Internal` with descriptive messages for configuration issues.
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

/// CORS middleware implementation.
///
/// This struct implements the `Middleware` trait and handles CORS for all requests.
/// It automatically processes preflight requests and adds appropriate CORS headers
/// to all responses.
///
/// # Features
/// - Automatic preflight request handling
/// - Origin validation for all request types
/// - Method and header validation for preflight requests
/// - Secure default behaviors
/// - Comprehensive error handling
///
/// # Usage
/// The middleware is typically created using the `Cors` builder:
/// ```
/// use ignitia::middleware::Cors;
///
/// let cors_middleware = Cors::new()
///     .allowed_origin("https://myapp.com")
///     .build()?;
/// ```
pub struct CorsMiddleware {
    cors: Cors,
}

#[async_trait::async_trait]
impl Middleware for CorsMiddleware {
    /// Handle requests before they reach the handler.
    ///
    /// This method processes incoming requests to handle CORS preflight requests
    /// (OPTIONS) and validate origins for all requests.
    ///
    /// # Preflight Handling
    /// For OPTIONS requests with CORS headers, this method:
    /// 1. Validates the request origin
    /// 2. Checks the requested method is allowed
    /// 3. Validates requested headers
    /// 4. Returns appropriate preflight response or error
    ///
    /// # Parameters
    /// - `req`: Mutable reference to the incoming request
    ///
    /// # Returns
    /// - `Ok(())`: Request can proceed normally
    /// - `Err(PreflightResponse)`: Preflight response should be sent
    /// - `Err(Error)`: Request should be rejected
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Handle preflight OPTIONS requests
        if req.method == Method::OPTIONS {
            if self.is_preflight_request(req) {
                return self.handle_preflight(req).await;
            }
        }

        Ok(())
    }

    /// Add CORS headers to responses.
    ///
    /// This method is called after the request handler has processed the request
    /// and generated a response. It adds the necessary CORS headers to allow
    /// the browser to access the response.
    ///
    /// # Headers Added
    /// - `Access-Control-Allow-Origin`: Based on configuration
    /// - `Access-Control-Expose-Headers`: If configured
    /// - `Access-Control-Allow-Credentials`: If credentials are enabled
    ///
    /// # Parameters
    /// - `res`: Mutable reference to the response
    ///
    /// # Returns
    /// Always returns `Ok(())` as header addition should not fail
    async fn after(&self, res: &mut Response) -> Result<()> {
        self.add_cors_headers(res);
        Ok(())
    }
}

impl CorsMiddleware {
    /// Determines if a request is a CORS preflight request.
    ///
    /// A request is considered a preflight request if:
    /// - Method is OPTIONS
    /// - Contains `Access-Control-Request-Method` header
    ///
    /// # Parameters
    /// - `req`: The request to check
    ///
    /// # Returns
    /// `true` if this is a preflight request, `false` otherwise
    fn is_preflight_request(&self, req: &Request) -> bool {
        req.header("access-control-request-method").is_some()
    }

    /// Handles CORS preflight requests.
    ///
    /// This method processes OPTIONS requests that are part of the CORS preflight
    /// mechanism. It validates the request and either approves or rejects it
    /// based on the CORS configuration.
    ///
    /// # Validation Steps
    /// 1. **Origin Validation**: Checks if the origin is allowed
    /// 2. **Method Validation**: Ensures the requested method is allowed
    /// 3. **Header Validation**: Validates all requested headers
    /// 4. **Response Generation**: Creates appropriate preflight response
    ///
    /// # Parameters
    /// - `req`: Mutable reference to the preflight request
    ///
    /// # Returns
    /// - `Ok(())`: Should never happen in current implementation
    /// - `Err(PreflightResponse)`: Successful preflight response
    /// - `Err(Error)`: Preflight request rejected
    ///
    /// # Errors
    /// - `Error::Forbidden`: Origin not allowed
    /// - `Error::MethodNotAllowed`: Requested method not allowed
    /// - `Error::BadRequest`: Requested headers not allowed
    async fn handle_preflight(&self, req: &mut Request) -> Result<()> {
        let origin = match req.header("origin") {
            Some(origin) => origin,
            None => return Ok(()), // No origin header, not a CORS request
        };

        // Validate origin
        if !self.is_origin_allowed(origin) {
            debug!("CORS preflight rejected: origin not allowed: {}", origin);
            return Err(crate::Error::Forbidden("Origin not allowed".to_string()));
        }

        // Validate request method
        if let Some(request_method) = req.header("access-control-request-method") {
            if let Ok(method) = Method::from_bytes(request_method.as_bytes()) {
                if !self.cors.inner.allowed_methods.contains(&method) {
                    debug!(
                        "CORS preflight rejected: method not allowed: {}",
                        request_method
                    );
                    return Err(crate::Error::MethodNotAllowed(request_method.to_string()));
                }
            }
        }

        // Validate request headers
        if let Some(request_headers) = req.header("access-control-request-headers") {
            for header in request_headers.split(',') {
                let header = header.trim().to_lowercase();
                if !self.cors.inner.allowed_headers.contains(&header) {
                    debug!("CORS preflight rejected: header not allowed: {}", header);
                    return Err(crate::Error::BadRequest(format!(
                        "Header not allowed: {}",
                        header
                    )));
                }
            }
        }

        // Create preflight response
        let mut response = Response::new(StatusCode::NO_CONTENT);
        self.add_preflight_headers(&mut response, origin);

        debug!("CORS preflight request handled successfully");
        Err(crate::Error::Custom(Box::new(PreflightResponse(response))))
    }

    /// Checks if an origin is allowed based on the configuration.
    ///
    /// This method validates origins against the configured allowed origins,
    /// supporting exact matches, wildcard matching, and regex patterns.
    ///
    /// # Parameters
    /// - `origin`: The origin to validate
    ///
    /// # Returns
    /// `true` if the origin is allowed, `false` otherwise
    ///
    /// # Origin Matching Rules
    /// - `AllowedOrigins::Any`: Always returns true
    /// - `AllowedOrigins::Exact`: Exact string match
    /// - `AllowedOrigins::Regex`: Regex pattern match
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

    /// Adds standard CORS headers to responses.
    ///
    /// This method adds the basic CORS headers that allow browsers to
    /// access the response. It's called for all responses, not just
    /// preflight responses.
    ///
    /// # Headers Added
    /// - `Access-Control-Allow-Origin`: Based on configuration
    /// - `Access-Control-Expose-Headers`: If headers are configured to be exposed
    /// - `Access-Control-Allow-Credentials`: If credentials are enabled
    ///
    /// # Parameters
    /// - `response`: Mutable reference to the response
    fn add_cors_headers(&self, response: &mut Response) {
        let inner = &self.cors.inner;

        if inner.allowed_origins_all {
            response
                .headers
                .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
        }

        if !inner.exposed_headers.is_empty() {
            response.headers.insert(
                header::ACCESS_CONTROL_EXPOSE_HEADERS,
                inner.exposed_headers.join(", ").parse().unwrap(),
            );
        }

        if inner.supports_credentials {
            response.headers.insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                "true".parse().unwrap(),
            );
        }
    }

    /// Adds preflight-specific headers to OPTIONS responses.
    ///
    /// This method adds all the headers required for a successful preflight
    /// response, including allowed methods, headers, and cache control.
    ///
    /// # Headers Added
    /// - `Access-Control-Allow-Origin`: Specific origin or *
    /// - `Access-Control-Allow-Methods`: Configured allowed methods
    /// - `Access-Control-Allow-Headers`: Configured allowed headers
    /// - `Access-Control-Max-Age`: Cache duration if configured
    /// - `Access-Control-Allow-Credentials`: If credentials are enabled
    ///
    /// # Parameters
    /// - `response`: Mutable reference to the preflight response
    /// - `origin`: The origin making the preflight request
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
                    .collect::<Vec<String>>()
                    .join(", ")
                    .parse()
                    .unwrap(),
            );
        }

        // Set max age
        if let Some(max_age) = inner.max_age {
            response.headers.insert(
                header::ACCESS_CONTROL_MAX_AGE,
                max_age.to_string().parse().unwrap(),
            );
        }

        // Set allow credentials
        if inner.supports_credentials {
            response.headers.insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                "true".parse().unwrap(),
            );
        }
    }
}

/// Custom error type for successful preflight responses.
///
/// This error type is used internally to short-circuit request processing
/// when a preflight request is successfully handled. It contains the
/// preflight response that should be sent to the client.
#[derive(Debug)]
struct PreflightResponse(Response);

impl crate::error::CustomError for PreflightResponse {
    /// Returns the HTTP status code for the preflight response.
    ///
    /// Preflight responses typically use 204 No Content status.
    fn status_code(&self) -> http::StatusCode {
        self.0.status
    }

    /// Returns the error type identifier.
    ///
    /// This is used for error categorization and logging.
    fn error_type(&self) -> &'static str {
        "cors_preflight"
    }
}

impl std::fmt::Display for PreflightResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CORS preflight response")
    }
}

impl Default for Cors {
    /// Creates a default CORS configuration.
    ///
    /// This is equivalent to calling `Cors::new()` and provides the same
    /// default configuration suitable for development environments.
    fn default() -> Self {
        Self::new()
    }
}

// Convenience methods for common CORS configurations
impl Cors {
    /// Creates a permissive CORS configuration for development.
    ///
    /// **WARNING**: This configuration should only be used in development
    /// environments. It allows all origins, methods, and headers.
    ///
    /// # Configuration
    /// - **Origins**: All (*)
    /// - **Methods**: All common HTTP methods
    /// - **Headers**: All (using wildcard)
    /// - **Credentials**: Disabled (cannot be used with wildcard origins)
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let dev_cors = Cors::permissive();
    /// // Only use this for development!
    /// ```
    pub fn permissive() -> Self {
        Self::new()
            .allow_any_origin()
            .allowed_methods(&[
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
                Method::HEAD,
            ])
            .allowed_headers(&["*"])
    }

    /// Creates a default CORS configuration for APIs.
    ///
    /// This configuration is suitable for public APIs that don't require
    /// credentials. It allows common methods and headers while being
    /// more restrictive than the permissive configuration.
    ///
    /// # Configuration
    /// - **Origins**: All (*)
    /// - **Methods**: GET, POST, PUT, DELETE, OPTIONS
    /// - **Headers**: Content-Type, Authorization, X-Requested-With
    /// - **Credentials**: Disabled
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let api_cors = Cors::default_api();
    /// ```
    pub fn default_api() -> Self {
        Self::new()
            .allowed_origin("*")
            .allowed_methods(&[
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allowed_headers(&["Content-Type", "Authorization", "X-Requested-With"])
    }

    /// Creates a secure CORS configuration for authenticated APIs.
    ///
    /// This configuration enables credentials support and requires specific
    /// origins to be provided. It's suitable for production APIs that need
    /// to send cookies or authorization headers.
    ///
    /// # Parameters
    /// - `allowed_origins`: Array of specific origins to allow
    ///
    /// # Configuration
    /// - **Origins**: Specific origins provided
    /// - **Methods**: GET, POST, PUT, DELETE, OPTIONS
    /// - **Headers**: Content-Type, Authorization, X-Requested-With
    /// - **Credentials**: Enabled
    ///
    /// # Examples
    /// ```
    /// use ignitia::middleware::Cors;
    ///
    /// let secure_cors = Cors::secure_api(&[
    ///     "https://myapp.com",
    ///     "https://admin.myapp.com"
    /// ]);
    /// ```
    pub fn secure_api(allowed_origins: &[&str]) -> Self {
        let mut cors = Self::new()
            .allowed_methods(&[
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allowed_headers(&["Content-Type", "Authorization", "X-Requested-With"]);

        if allowed_origins.is_empty() {
            cors = cors.allow_any_origin();
        } else {
            cors = cors.allowed_origins(allowed_origins);
        }

        cors
    }
}
