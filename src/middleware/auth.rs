//! # Token-Based Authentication Middleware
//!
//! This module provides a flexible authentication middleware for the Ignitia web framework.
//! It supports Bearer token authentication with configurable path protection, allowing
//! you to secure specific routes or entire sections of your API.
//!
//! ## Features
//!
//! - **Bearer Token Authentication**: Standard HTTP Bearer token authentication
//! - **Path-Based Protection**: Protect specific paths or path prefixes
//! - **Flexible Configuration**: Easy to configure and customize
//! - **Performance Optimized**: Efficient path matching with minimal overhead
//! - **Security First**: Secure by default with clear error messages
//!
//! ## Authentication Flow
//!
//! 1. **Request Arrives**: Middleware checks if the request path requires authentication
//! 2. **Token Extraction**: Extracts Bearer token from the Authorization header
//! 3. **Token Validation**: Validates the token against the configured secret
//! 4. **Access Control**: Allows or denies the request based on validation result
//!
//! ## Usage
//!
//! ### Basic Authentication
//! ```
//! use ignitia::{Router, AuthMiddleware};
//!
//! let router = Router::new()
//!     .middleware(
//!         AuthMiddleware::new("my-secret-token")
//!             .protect_path("/api")
//!     )
//!     .get("/", || async { Ok(ignitia::Response::text("Public")) })
//!     .get("/api/users", || async { Ok(ignitia::Response::text("Protected")) });
//! ```
//!
//! ### Multiple Protected Paths
//! ```
//! use ignitia::{Router, AuthMiddleware};
//!
//! let router = Router::new()
//!     .middleware(
//!         AuthMiddleware::new("secret-token")
//!             .protect_path("/api/admin")
//!             .protect_path("/api/users")
//!             .protect_path("/api/orders")
//!     )
//!     .get("/", || async { Ok(ignitia::Response::text("Public home")) })
//!     .get("/about", || async { Ok(ignitia::Response::text("Public about")) })
//!     .get("/api/admin", || async { Ok(ignitia::Response::text("Admin only")) })
//!     .get("/api/users", || async { Ok(ignitia::Response::text("Users")) });
//! ```
//!
//! ### Batch Path Protection
//! ```
//! use ignitia::{Router, AuthMiddleware};
//!
//! let protected_paths = vec!["/api/admin", "/api/users", "/api/orders"];
//!
//! let router = Router::new()
//!     .middleware(
//!         AuthMiddleware::new("secret-token")
//!             .protect_paths(protected_paths)
//!     )
//!     .get("/api/admin/dashboard", || async { Ok(ignitia::Response::text("Dashboard")) });
//! ```
//!
//! ### Environment-Based Configuration
//! ```
//! use ignitia::{Router, AuthMiddleware};
//! use std::env;
//!
//! let token = env::var("API_TOKEN").expect("API_TOKEN must be set");
//!
//! let router = Router::new()
//!     .middleware(
//!         AuthMiddleware::new(token)
//!             .protect_path("/api")
//!     )
//!     .get("/api/secure", || async { Ok(ignitia::Response::text("Secure data")) });
//! ```
//!
//! ## Client Usage
//!
//! Clients must include the Bearer token in the Authorization header:
//!
//! ### cURL Example
//! ```
//! # Successful request
//! curl -H "Authorization: Bearer my-secret-token" http://localhost:8080/api/users
//!
//! # Failed request (no token)
//! curl http://localhost:8080/api/users
//!
//! # Failed request (wrong token)
//! curl -H "Authorization: Bearer wrong-token" http://localhost:8080/api/users
//! ```
//!
//! ### JavaScript Fetch API
//! ```
//! // Successful request
//! fetch('/api/users', {
//!     headers: {
//!         'Authorization': 'Bearer my-secret-token'
//!     }
//! });
//!
//! // With POST request
//! fetch('/api/users', {
//!     method: 'POST',
//!     headers: {
//!         'Authorization': 'Bearer my-secret-token',
//!         'Content-Type': 'application/json'
//!     },
//!     body: JSON.stringify({ name: 'John Doe' })
//! });
//! ```
//!
//! ## Path Matching Behavior
//!
//! The middleware uses prefix matching for protected paths:
//!
//! ```
//! // This configuration:
//! AuthMiddleware::new("token").protect_path("/api")
//!
//! // Will protect these paths:
//! // ✅ /api
//! // ✅ /api/users
//! // ✅ /api/users/123
//! // ✅ /api/admin/dashboard
//!
//! // But NOT these paths:
//! // ❌ /
//! // ❌ /public
//! // ❌ /about
//! // ❌ /apikey (doesn't start with /api/)
//! ```
//!
//! ## Error Responses
//!
//! The middleware returns appropriate HTTP status codes:
//!
//! ### 401 Unauthorized - Missing Token
//! ```
//! Request: GET /api/users
//! Headers: (no Authorization header)
//!
//! Response: 401 Unauthorized
//! {
//!   "error": "Unauthorized",
//!   "message": "Unauthorized",
//!   "status": 401
//! }
//! ```
//!
//! ### 401 Unauthorized - Invalid Token Format
//! ```
//! Request: GET /api/users
//! Headers: Authorization: Basic dGVzdA==
//!
//! Response: 401 Unauthorized
//! {
//!   "error": "Unauthorized",
//!   "message": "Unauthorized",
//!   "status": 401
//! }
//! ```
//!
//! ### 401 Unauthorized - Wrong Token
//! ```
//! Request: GET /api/users
//! Headers: Authorization: Bearer wrong-token
//!
//! Response: 401 Unauthorized
//! {
//!   "error": "Unauthorized",
//!   "message": "Unauthorized",
//!   "status": 401
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ### Custom Token Validation
//! ```
//! use ignitia::{Middleware, Request, Response, Result, Error};
//! use async_trait::async_trait;
//!
//! pub struct JwtAuthMiddleware {
//!     jwt_secret: String,
//!     protected_paths: Vec<String>,
//! }
//!
//! impl JwtAuthMiddleware {
//!     pub fn new(jwt_secret: String) -> Self {
//!         Self {
//!             jwt_secret,
//!             protected_paths: Vec::new(),
//!         }
//!     }
//!
//!     pub fn protect_path(mut self, path: String) -> Self {
//!         self.protected_paths.push(path);
//!         self
//!     }
//!
//!     fn should_authenticate(&self, req: &Request) -> bool {
//!         let path = req.uri.path();
//!         self.protected_paths
//!             .iter()
//!             .any(|protected| path == protected || path.starts_with(&format!("{}/", protected)))
//!     }
//!
//!     fn validate_jwt(&self, token: &str) -> bool {
//!         // JWT validation logic here
//!         // This is a simplified example
//!         token.len() > 10 && token.contains('.')
//!     }
//! }
//!
//! #[async_trait]
//! impl Middleware for JwtAuthMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         if !self.should_authenticate(req) {
//!             return Ok(());
//!         }
//!
//!         let auth_header = req.header("Authorization")
//!             .ok_or(Error::Unauthorized)?;
//!
//!         if !auth_header.starts_with("Bearer ") {
//!             return Err(Error::Unauthorized);
//!         }
//!
//!         let token = &auth_header[7..];
//!         if !self.validate_jwt(token) {
//!             return Err(Error::Unauthorized);
//!         }
//!
//!         // Store user info in request extensions
//!         req.insert_extension(String::from("user_id_from_jwt"));
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ### Role-Based Access Control
//! ```
//! use ignitia::{Middleware, Request, Result, Error};
//! use async_trait::async_trait;
//! use std::collections::HashMap;
//!
//! pub struct RoleAuthMiddleware {
//!     tokens: HashMap<String, Vec<String>>, // token -> roles
//!     path_roles: HashMap<String, Vec<String>>, // path -> required roles
//! }
//!
//! impl RoleAuthMiddleware {
//!     pub fn new() -> Self {
//!         Self {
//!             tokens: HashMap::new(),
//!             path_roles: HashMap::new(),
//!         }
//!     }
//!
//!     pub fn add_token(mut self, token: String, roles: Vec<String>) -> Self {
//!         self.tokens.insert(token, roles);
//!         self
//!     }
//!
//!     pub fn require_role(mut self, path: String, roles: Vec<String>) -> Self {
//!         self.path_roles.insert(path, roles);
//!         self
//!     }
//! }
//!
//! #[async_trait]
//! impl Middleware for RoleAuthMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         let path = req.uri.path();
//!
//!         // Check if this path requires specific roles
//!         let required_roles = match self.path_roles.get(path) {
//!             Some(roles) => roles,
//!             None => return Ok(()), // No role requirement
//!         };
//!
//!         let auth_header = req.header("Authorization")
//!             .ok_or(Error::Unauthorized)?;
//!
//!         if !auth_header.starts_with("Bearer ") {
//!             return Err(Error::Unauthorized);
//!         }
//!
//!         let token = &auth_header[7..];
//!         let user_roles = self.tokens.get(token)
//!             .ok_or(Error::Unauthorized)?;
//!
//!         // Check if user has any of the required roles
//!         let has_required_role = required_roles.iter()
//!             .any(|required| user_roles.contains(required));
//!
//!         if !has_required_role {
//!             return Err(Error::Forbidden);
//!         }
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ## Security Best Practices
//!
//! ### Token Security
//! - Use cryptographically secure random tokens
//! - Store tokens securely (environment variables, key management services)
//! - Implement token rotation for long-lived applications
//! - Never log tokens in application logs
//!
//! ### Production Considerations
//! ```
//! use ignitia::{Router, AuthMiddleware};
//! use std::env;
//!
//! // ✅ Good: Load from environment
//! let token = env::var("API_SECRET_TOKEN")
//!     .expect("API_SECRET_TOKEN environment variable must be set");
//!
//! // ❌ Bad: Hardcoded token
//! let token = "hardcoded-secret-token";
//!
//! let router = Router::new()
//!     .middleware(AuthMiddleware::new(token).protect_path("/api"));
//! ```
//!
//! ### Rate Limiting
//! Consider combining with rate limiting to prevent brute force attacks:
//! ```
//! use ignitia::Router;
//!
//! let router = Router::new()
//!     .middleware(RateLimitMiddleware::new(100, Duration::from_secs(60))) // 100 requests per minute
//!     .middleware(AuthMiddleware::new(token).protect_path("/api"));
//! ```

use crate::middleware::Middleware;
use crate::{Error, Request, Result};

/// Token-based authentication middleware for protecting API endpoints.
///
/// This middleware provides Bearer token authentication for HTTP requests. It allows
/// you to protect specific paths or entire sections of your API by requiring clients
/// to provide a valid Bearer token in the Authorization header.
///
/// # Authentication Method
/// Uses HTTP Bearer token authentication as specified in RFC 6750. Clients must include
/// an `Authorization: Bearer <token>` header in their requests.
///
/// # Path Protection
/// The middleware uses prefix matching to determine which paths require authentication.
/// If a request path matches or starts with a protected path prefix, authentication
/// is required.
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::{Router, AuthMiddleware};
///
/// let router = Router::new()
///     .middleware(
///         AuthMiddleware::new("secret-token")
///             .protect_path("/api")
///     )
///     .get("/", || async { Ok(ignitia::Response::text("Public")) })
///     .get("/api/data", || async { Ok(ignitia::Response::text("Protected")) });
/// ```
///
/// ## Multiple Protected Paths
/// ```
/// use ignitia::{Router, AuthMiddleware};
///
/// let router = Router::new()
///     .middleware(
///         AuthMiddleware::new("secret-token")
///             .protect_path("/api/admin")
///             .protect_path("/api/users")
///             .protect_path("/api/orders")
///     )
///     .get("/api/admin/dashboard", || async { Ok(ignitia::Response::text("Admin")) });
/// ```
pub struct AuthMiddleware {
    token: String,
    protected_paths: Vec<String>,
}

impl AuthMiddleware {
    /// Creates a new authentication middleware with the specified token.
    ///
    /// The token will be compared against Bearer tokens provided in the
    /// Authorization header of incoming requests.
    ///
    /// # Parameters
    /// - `token`: The secret token that clients must provide for authentication
    ///
    /// # Returns
    /// A new `AuthMiddleware` instance with no protected paths (you must add them)
    ///
    /// # Examples
    /// ```
    /// use ignitia::AuthMiddleware;
    ///
    /// let auth = AuthMiddleware::new("my-secret-token");
    /// ```
    ///
    /// ## With Environment Variable
    /// ```
    /// use ignitia::AuthMiddleware;
    /// use std::env;
    ///
    /// let token = env::var("API_TOKEN").expect("API_TOKEN must be set");
    /// let auth = AuthMiddleware::new(token);
    /// ```
    ///
    /// # Security Note
    /// Use a cryptographically secure random token and store it securely.
    /// Never hardcode tokens in production code.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            protected_paths: Vec::new(),
        }
    }

    /// Adds a path that requires authentication.
    ///
    /// The path will be protected using prefix matching. This means that the
    /// specified path and all sub-paths will require authentication.
    ///
    /// # Parameters
    /// - `path`: The path prefix to protect (e.g., "/api", "/admin")
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::AuthMiddleware;
    ///
    /// let auth = AuthMiddleware::new("token")
    ///     .protect_path("/api")
    ///     .protect_path("/admin");
    /// ```
    ///
    /// # Path Matching
    /// The path "/api" will protect:
    /// - `/api` (exact match)
    /// - `/api/users` (prefix match)
    /// - `/api/users/123` (prefix match)
    /// - `/api/admin/dashboard` (prefix match)
    ///
    /// But will NOT protect:
    /// - `/` (different path)
    /// - `/about` (different path)
    /// - `/apikey` (not a prefix match)
    pub fn protect_path(mut self, path: impl Into<String>) -> Self {
        self.protected_paths.push(path.into());
        self
    }

    /// Adds multiple paths that require authentication.
    ///
    /// This is a convenience method for protecting multiple paths at once.
    ///
    /// # Parameters
    /// - `paths`: A vector of path strings to protect
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::AuthMiddleware;
    ///
    /// let protected_paths = vec!["/api/admin", "/api/users", "/api/orders"];
    /// let auth = AuthMiddleware::new("token")
    ///     .protect_paths(protected_paths);
    /// ```
    ///
    /// ## With Dynamic Paths
    /// ```
    /// use ignitia::AuthMiddleware;
    ///
    /// let mut paths = Vec::new();
    /// paths.push("/api/v1".to_string());
    /// paths.push("/api/v2".to_string());
    ///
    /// let auth = AuthMiddleware::new("token")
    ///     .protect_paths(paths);
    /// ```
    pub fn protect_paths(mut self, paths: Vec<impl Into<String>>) -> Self {
        for path in paths {
            self.protected_paths.push(path.into());
        }
        self
    }

    /// Determines if a request requires authentication based on its path.
    ///
    /// This method checks if the request path matches any of the protected paths
    /// using prefix matching logic.
    ///
    /// # Parameters
    /// - `req`: The incoming HTTP request
    ///
    /// # Returns
    /// - `true` if the request path requires authentication
    /// - `false` if the request path is public
    ///
    /// # Examples
    /// ```
    /// let auth = AuthMiddleware::new("token").protect_path("/api");
    ///
    /// // These would return true:
    /// // req.uri.path() == "/api"
    /// // req.uri.path() == "/api/users"
    /// // req.uri.path() == "/api/admin/dashboard"
    ///
    /// // These would return false:
    /// // req.uri.path() == "/"
    /// // req.uri.path() == "/public"
    /// // req.uri.path() == "/about"
    /// ```
    fn should_authenticate(&self, req: &Request) -> bool {
        let path = req.uri.path();
        self.protected_paths
            .iter()
            .any(|protected| path == protected || path.starts_with(&format!("{}/", protected)))
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    /// Authenticates incoming requests for protected paths.
    ///
    /// This method is called before the request reaches the handler. It performs
    /// the following authentication steps:
    ///
    /// 1. **Path Check**: Determines if the request path requires authentication
    /// 2. **Header Extraction**: Extracts the Authorization header
    /// 3. **Token Format Validation**: Ensures the header uses Bearer token format
    /// 4. **Token Verification**: Compares the provided token with the configured token
    ///
    /// # Parameters
    /// - `req`: Mutable reference to the incoming request
    ///
    /// # Returns
    /// - `Ok(())`: Authentication successful or not required, request continues
    /// - `Err(Error::Unauthorized)`: Authentication failed, request is rejected
    ///
    /// # Error Conditions
    /// Returns `Error::Unauthorized` when:
    /// - The path requires authentication but no Authorization header is present
    /// - The Authorization header doesn't use Bearer token format
    /// - The provided token doesn't match the configured token
    ///
    /// # Examples
    /// ```
    /// # Successful authentication
    /// GET /api/users HTTP/1.1
    /// Authorization: Bearer my-secret-token
    /// -> Request continues to handler
    ///
    /// # Failed authentication - missing header
    /// GET /api/users HTTP/1.1
    /// -> 401 Unauthorized
    ///
    /// # Failed authentication - wrong format
    /// GET /api/users HTTP/1.1
    /// Authorization: Basic dXNlcjpwYXNz
    /// -> 401 Unauthorized
    ///
    /// # Failed authentication - wrong token
    /// GET /api/users HTTP/1.1
    /// Authorization: Bearer wrong-token
    /// -> 401 Unauthorized
    ///
    /// # Public path - no authentication required
    /// GET / HTTP/1.1
    /// -> Request continues to handler (no auth needed)
    /// ```
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Only authenticate if this path requires it
        if !self.should_authenticate(req) {
            return Ok(());
        }

        let auth_header = req.header("Authorization").ok_or(Error::Unauthorized(
            "Missing Authorization header".to_string(),
        ))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(Error::Unauthorized(
                "Invalid Authorization header format".to_string(),
            ));
        }

        let token = &auth_header[7..];
        if token != self.token {
            return Err(Error::Unauthorized("Invalid token".to_string()));
        }

        Ok(())
    }
}
