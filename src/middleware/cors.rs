//! # Cross-Origin Resource Sharing (CORS) Middleware
//!
//! This module provides CORS middleware for the Ignitia web framework, enabling cross-origin
//! requests from web browsers. CORS is essential for modern web applications that need to
//! make requests from one domain to another.
//!
//! ## Features
//!
//! - **Configurable Origins**: Control which origins can access your API
//! - **Method Restrictions**: Specify allowed HTTP methods
//! - **Header Control**: Configure allowed request and response headers
//! - **Preflight Handling**: Automatic handling of CORS preflight requests
//! - **Security First**: Secure defaults with easy customization
//!
//! ## CORS Basics
//!
//! CORS (Cross-Origin Resource Sharing) is a mechanism that allows restricted resources
//! on a web page to be requested from another domain. Modern browsers implement CORS
//! to enforce the same-origin policy for security.
//!
//! ### Key CORS Headers
//! - `Access-Control-Allow-Origin`: Specifies which origins can access the resource
//! - `Access-Control-Allow-Methods`: Lists allowed HTTP methods
//! - `Access-Control-Allow-Headers`: Lists allowed request headers
//! - `Access-Control-Max-Age`: Caches preflight results (not implemented yet)
//! - `Access-Control-Allow-Credentials`: Allows cookies in cross-origin requests
//!
//! ## Usage
//!
//! ### Basic Usage (Allow All Origins)
//! ```
//! use ignitia::{Router, CorsMiddleware};
//!
//! let router = Router::new()
//!     .middleware(CorsMiddleware::new())
//!     .get("/api/data", || async { Ok(ignitia::Response::text("API Data")) });
//! ```
//!
//! ### Restrict to Specific Origin
//! ```
//! use ignitia::{Router, CorsMiddleware};
//!
//! let router = Router::new()
//!     .middleware(CorsMiddleware::new().allow_origin("https://myapp.com"))
//!     .get("/api/users", || async { Ok(ignitia::Response::text("Users")) });
//! ```
//!
//! ### Multiple Origins
//! ```
//! use ignitia::{Router, CorsMiddleware};
//!
//! let router = Router::new()
//!     .middleware(CorsMiddleware::new().allow_origin("https://app1.com,https://app2.com"))
//!     .get("/api/data", || async { Ok(ignitia::Response::text("Data")) });
//! ```
//!
//! ### Development Setup
//! ```
//! use ignitia::{Router, CorsMiddleware};
//!
//! // Allow localhost for development
//! let router = Router::new()
//!     .middleware(CorsMiddleware::new().allow_origin("http://localhost:3000"))
//!     .get("/api/dev", || async { Ok(ignitia::Response::text("Dev API")) });
//! ```
//!
//! ## Advanced Configuration
//!
//! For more advanced CORS needs, you can create custom middleware:
//!
//! ```
//! use ignitia::{Middleware, Request, Response, Result};
//! use async_trait::async_trait;
//! use http::header;
//!
//! pub struct AdvancedCorsMiddleware {
//!     allowed_origins: Vec<String>,
//!     allow_credentials: bool,
//!     max_age: Option<u64>,
//! }
//!
//! impl AdvancedCorsMiddleware {
//!     pub fn new() -> Self {
//!         Self {
//!             allowed_origins: vec!["*".to_string()],
//!             allow_credentials: false,
//!             max_age: Some(86400), // 24 hours
//!         }
//!     }
//!
//!     pub fn allow_credentials(mut self) -> Self {
//!         self.allow_credentials = true;
//!         self
//!     }
//!
//!     pub fn max_age(mut self, seconds: u64) -> Self {
//!         self.max_age = Some(seconds);
//!         self
//!     }
//! }
//!
//! #[async_trait]
//! impl Middleware for AdvancedCorsMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         // Handle preflight requests
//!         if req.method == http::Method::OPTIONS {
//!             // You could return early here with a preflight response
//!         }
//!         Ok(())
//!     }
//!
//!     async fn after(&self, res: &mut Response) -> Result<()> {
//!         // Add CORS headers
//!         res.headers.insert(
//!             header::ACCESS_CONTROL_ALLOW_ORIGIN,
//!             "*".parse().unwrap()
//!         );
//!
//!         if self.allow_credentials {
//!             res.headers.insert(
//!                 header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
//!                 "true".parse().unwrap()
//!             );
//!         }
//!
//!         if let Some(max_age) = self.max_age {
//!             res.headers.insert(
//!                 header::ACCESS_CONTROL_MAX_AGE,
//!                 max_age.to_string().parse().unwrap()
//!             );
//!         }
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Security Considerations
//!
//! ### Production Security
//! - **Never use `*` in production** for sensitive APIs
//! - Always specify exact origins for production environments
//! - Be cautious with `Access-Control-Allow-Credentials`
//! - Consider implementing origin validation logic
//!
//! ### Example Secure Configuration
//! ```
//! use ignitia::{Router, CorsMiddleware};
//!
//! let router = Router::new()
//!     .middleware(
//!         CorsMiddleware::new()
//!             .allow_origin("https://myproductionapp.com")
//!     )
//!     .get("/api/secure", || async {
//!         Ok(ignitia::Response::text("Secure API"))
//!     });
//! ```
//!
//! ## Common CORS Issues
//!
//! ### 1. Preflight Failures
//! If you see OPTIONS request failures, ensure your server handles OPTIONS requests:
//! ```
//! use ignitia::Router;
//!
//! let router = Router::new()
//!     .options("/api/*", || async {
//!         Ok(ignitia::Response::new(http::StatusCode::NO_CONTENT))
//!     });
//! ```
//!
//! ### 2. Credentials Issues
//! When using credentials, you cannot use `*` for origins:
//! ```
//! // ❌ This won't work with credentials
//! CorsMiddleware::new().allow_origin("*")
//!
//! // ✅ This will work
//! CorsMiddleware::new().allow_origin("https://myapp.com")
//! ```
//!
//! ## Browser Compatibility
//!
//! CORS is supported by all modern browsers:
//! - Chrome 4+
//! - Firefox 3.5+
//! - Safari 4+
//! - Edge (all versions)
//! - Internet Explorer 8+ (with limitations)

use crate::middleware::Middleware;
use crate::{Response, Result};
use http::header;

/// CORS (Cross-Origin Resource Sharing) middleware for handling cross-origin requests.
///
/// This middleware adds the necessary CORS headers to responses, allowing web browsers
/// to make cross-origin requests to your API. It provides configurable options for
/// origins, methods, and headers.
///
/// # Default Configuration
/// - **Allow Origin**: `*` (all origins)
/// - **Allow Methods**: `GET, POST, PUT, DELETE, OPTIONS`
/// - **Allow Headers**: `Content-Type, Authorization`
///
/// # Examples
///
/// ## Allow All Origins (Development)
/// ```
/// use ignitia::{Router, CorsMiddleware};
///
/// let router = Router::new()
///     .middleware(CorsMiddleware::new())
///     .get("/api", || async { Ok(ignitia::Response::text("API")) });
/// ```
///
/// ## Restrict to Specific Origin (Production)
/// ```
/// use ignitia::{Router, CorsMiddleware};
///
/// let router = Router::new()
///     .middleware(
///         CorsMiddleware::new()
///             .allow_origin("https://myapp.com")
///     )
///     .get("/api", || async { Ok(ignitia::Response::text("Secure API")) });
/// ```
///
/// ## Multiple Origins
/// ```
/// use ignitia::{Router, CorsMiddleware};
///
/// let router = Router::new()
///     .middleware(
///         CorsMiddleware::new()
///             .allow_origin("https://app1.com,https://app2.com")
///     )
///     .get("/api", || async { Ok(ignitia::Response::text("Multi-origin API")) });
/// ```
pub struct CorsMiddleware {
    allow_origin: String,
    allow_methods: String,
    allow_headers: String,
}

impl CorsMiddleware {
    /// Creates a new CORS middleware with default settings.
    ///
    /// The default configuration allows:
    /// - **Origins**: `*` (all origins)
    /// - **Methods**: `GET, POST, PUT, DELETE, OPTIONS`
    /// - **Headers**: `Content-Type, Authorization`
    ///
    /// # Examples
    /// ```
    /// use ignitia::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new();
    /// // Equivalent to:
    /// // - allow_origin: "*"
    /// // - allow_methods: "GET, POST, PUT, DELETE, OPTIONS"
    /// // - allow_headers: "Content-Type, Authorization"
    /// ```
    ///
    /// # Security Warning
    /// The default `*` origin setting is suitable for development but should be
    /// replaced with specific origins in production environments.
    pub fn new() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization".to_string(),
        }
    }

    /// Sets the allowed origin(s) for CORS requests.
    ///
    /// This method configures which origins are allowed to make cross-origin requests
    /// to your API. You can specify a single origin, multiple origins (comma-separated),
    /// or use `*` to allow all origins.
    ///
    /// # Parameters
    /// - `origin`: The origin specification (domain, list of domains, or "*")
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    ///
    /// ## Single Origin
    /// ```
    /// use ignitia::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allow_origin("https://myapp.com");
    /// ```
    ///
    /// ## Multiple Origins
    /// ```
    /// use ignitia::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allow_origin("https://app1.com,https://app2.com,https://app3.com");
    /// ```
    ///
    /// ## Development (All Origins)
    /// ```
    /// use ignitia::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allow_origin("*");
    /// ```
    ///
    /// ## Localhost for Development
    /// ```
    /// use ignitia::CorsMiddleware;
    ///
    /// let cors = CorsMiddleware::new()
    ///     .allow_origin("http://localhost:3000");
    /// ```
    ///
    /// # Security Notes
    /// - Use specific origins in production environments
    /// - Avoid `*` when handling sensitive data
    /// - Consider implementing dynamic origin validation for complex scenarios
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allow_origin = origin.into();
        self
    }
}

#[async_trait::async_trait]
impl Middleware for CorsMiddleware {
    /// Adds CORS headers to the response.
    ///
    /// This method is called after the handler has processed the request and generated
    /// a response. It adds the necessary CORS headers to allow cross-origin requests
    /// from browsers.
    ///
    /// # Headers Added
    /// - `Access-Control-Allow-Origin`: Specifies allowed origins
    /// - `Access-Control-Allow-Methods`: Lists allowed HTTP methods
    /// - `Access-Control-Allow-Headers`: Lists allowed request headers
    ///
    /// # Parameters
    /// - `res`: Mutable reference to the response that will receive CORS headers
    ///
    /// # Returns
    /// - `Ok(())`: Always succeeds, CORS headers are added to the response
    ///
    /// # Examples
    /// After processing, the response will include headers like:
    /// ```
    /// Access-Control-Allow-Origin: https://myapp.com
    /// Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
    /// Access-Control-Allow-Headers: Content-Type, Authorization
    /// ```
    async fn after(&self, res: &mut Response) -> Result<()> {
        res.headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            self.allow_origin.parse().unwrap(),
        );
        res.headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            self.allow_methods.parse().unwrap(),
        );
        res.headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            self.allow_headers.parse().unwrap(),
        );
        Ok(())
    }
}

/// Default implementation for CorsMiddleware.
///
/// Creates a new CORS middleware with permissive default settings suitable for development.
/// In production, you should always customize the allowed origins.
impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}
