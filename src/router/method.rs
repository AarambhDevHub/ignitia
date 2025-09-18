//! # HTTP Method Handling and Conversions
//!
//! This module provides utilities for working with HTTP methods in the Ignitia web framework.
//! It includes custom HTTP method types, conversions to/from standard HTTP methods, and string
//! parsing capabilities for flexible method handling in routing and middleware.
//!
//! ## Features
//!
//! - **Custom Method Enum**: Framework-specific HTTP method enumeration
//! - **Standard Conversions**: Seamless conversion to/from `http::Method`
//! - **String Parsing**: Parse HTTP methods from strings with error handling
//! - **Performance Optimized**: Efficient method comparisons and conversions
//! - **Comprehensive Coverage**: Support for all standard HTTP methods
//!
//! ## HTTP Method Overview
//!
//! ### Safe Methods (Idempotent and Safe)
//! - **GET**: Retrieve data without side effects
//! - **HEAD**: Get headers only, no response body
//! - **OPTIONS**: Query supported methods and capabilities
//!
//! ### Idempotent Methods (Can be repeated safely)
//! - **PUT**: Replace or create resource completely
//! - **DELETE**: Remove resource
//! - **GET, HEAD, OPTIONS**: Also idempotent
//!
//! ### Non-Idempotent Methods
//! - **POST**: Create resource or perform action (side effects)
//! - **PATCH**: Partial resource modification
//!
//! ### Connection Methods
//! - **CONNECT**: Establish tunnel (typically for HTTPS proxy)
//! - **TRACE**: Diagnostic trace of request path
//!
//! ## Usage Examples
//!
//! ### Basic Method Usage
//! ```
//! use ignitia::router::method::HttpMethod;
//! use http::Method;
//!
//! // Create custom method
//! let custom_method = HttpMethod::Get;
//!
//! // Convert to standard HTTP method
//! let std_method: Method = custom_method.into();
//! assert_eq!(std_method, Method::GET);
//!
//! // Convert from standard HTTP method
//! let from_std: HttpMethod = Method::POST.into();
//! assert_eq!(from_std, HttpMethod::Post);
//! ```
//!
//! ### String Parsing
//! ```
//! use ignitia::router::method::HttpMethod;
//! use std::str::FromStr;
//!
//! // Parse from string (case insensitive)
//! let method = HttpMethod::from_str("GET").unwrap();
//! assert_eq!(method, HttpMethod::Get);
//!
//! let method = HttpMethod::from_str("post").unwrap();
//! assert_eq!(method, HttpMethod::Post);
//!
//! // Invalid method returns error
//! assert!(HttpMethod::from_str("INVALID").is_err());
//! ```
//!
//! ### Router Integration
//! ```
//! use ignitia::{Router, Response};
//! use ignitia::router::method::HttpMethod;
//! use http::Method;
//!
//! let router = Router::new()
//!     .route_with("/api/data", Method::GET, || async {
//!         Ok(Response::json(serde_json::json!({
//!             "method": "GET",
//!             "message": "Data retrieved"
//!         }))?)
//!     })
//!     .route_with("/api/data", Method::POST, || async {
//!         Ok(Response::json(serde_json::json!({
//!             "method": "POST",
//!             "message": "Data created"
//!         }))?)
//!     });
//! ```
//!
//! ## Method Characteristics
//!
//! ### RESTful API Patterns
//! ```
//! use ignitia::{Router, Response, Path, Json};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Serialize)]
//! struct User {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! let api_router = Router::new()
//!     // GET /users - List all users (safe, idempotent)
//!     .get("/users", || async {
//!         Ok(Response::json(vec![
//!             User { id: 1, name: "Alice".into(), email: "alice@example.com".into() }
//!         ])?)
//!     })
//!     // POST /users - Create new user (not idempotent)
//!     .post("/users", |Json(user): Json<CreateUser>| async move {
//!         let new_user = User {
//!             id: 123, // Would be generated
//!             name: user.name,
//!             email: user.email,
//!         };
//!         Ok(Response::json(new_user)?.with_status_code(201))
//!     })
//!     // GET /users/:id - Get specific user (safe, idempotent)
//!     .get("/users/:id", |Path(id): Path<u32>| async move {
//!         Ok(Response::json(User {
//!             id,
//!             name: "User".into(),
//!             email: "user@example.com".into(),
//!         })?)
//!     })
//!     // PUT /users/:id - Replace user completely (idempotent)
//!     .put("/users/:id", |Path(id): Path<u32>, Json(user): Json<CreateUser>| async move {
//!         Ok(Response::json(User {
//!             id,
//!             name: user.name,
//!             email: user.email,
//!         })?)
//!     })
//!     // PATCH /users/:id - Partial update (not idempotent)
//!     .patch("/users/:id", |Path(id): Path<u32>| async move {
//!         Ok(Response::json(serde_json::json!({
//!             "id": id,
//!             "message": "User updated"
//!         }))?)
//!     })
//!     // DELETE /users/:id - Remove user (idempotent)
//!     .delete("/users/:id", |Path(id): Path<u32>| async move {
//!         Ok(Response::json(serde_json::json!({
//!             "message": format!("User {} deleted", id)
//!         }))?)
//!     })
//!     // OPTIONS /users - Get allowed methods
//!     .options("/users", || async {
//!         Ok(Response::text("")
//!             .with_status_code(204)
//!             .with_header("Allow", "GET, POST, OPTIONS"))
//!     });
//! ```
//!
//! ## Advanced Usage Patterns
//!
//! ### Method-Based Middleware
//! ```
//! use ignitia::{Router, Response, Request, Result, Middleware};
//! use http::Method;
//!
//! struct MethodLoggingMiddleware;
//!
//! #[async_trait::async_trait]
//! impl Middleware for MethodLoggingMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         match req.method {
//!             Method::GET | Method::HEAD | Method::OPTIONS => {
//!                 tracing::info!("Safe method: {}", req.method);
//!             }
//!             Method::PUT | Method::DELETE => {
//!                 tracing::info!("Idempotent method: {}", req.method);
//!             }
//!             Method::POST | Method::PATCH => {
//!                 tracing::warn!("Non-idempotent method: {}", req.method);
//!             }
//!             _ => {
//!                 tracing::debug!("Other method: {}", req.method);
//!             }
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! ### Dynamic Method Routing
//! ```
//! use ignitia::{Router, Response, Request, Method};
//!
//! async fn dynamic_handler(req: Request) -> ignitia::Result<Response> {
//!     let response_data = match req.method {
//!         Method::GET => serde_json::json!({
//!             "action": "retrieve",
//!             "safe": true,
//!             "idempotent": true
//!         }),
//!         Method::POST => serde_json::json!({
//!             "action": "create",
//!             "safe": false,
//!             "idempotent": false
//!         }),
//!         Method::PUT => serde_json::json!({
//!             "action": "replace",
//!             "safe": false,
//!             "idempotent": true
//!         }),
//!         Method::PATCH => serde_json::json!({
//!             "action": "modify",
//!             "safe": false,
//!             "idempotent": false
//!         }),
//!         Method::DELETE => serde_json::json!({
//!             "action": "remove",
//!             "safe": false,
//!             "idempotent": true
//!         }),
//!         _ => serde_json::json!({
//!             "action": "unknown",
//!             "method": req.method.to_string()
//!         })
//!     };
//!
//!     Response::json(response_data)
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Method Comparison Performance
//! Method comparisons are highly optimized:
//! - Enum variants use simple integer comparisons
//! - No string allocations during comparison
//! - Branch prediction friendly for common methods (GET, POST)
//!
//! ### Memory Efficiency
//! - Small enum variants (1 byte each)
//! - No heap allocations for method storage
//! - Efficient conversion between framework and standard types

use http::Method;
use std::str::FromStr;

/// Custom HTTP method enumeration for the Ignitia web framework.
///
/// This enum provides a framework-specific representation of HTTP methods
/// with efficient conversions to/from the standard `http::Method` type.
/// It's designed for use in routing, middleware, and request handling.
///
/// # Method Categories
///
/// ## Safe Methods (RFC 7231)
/// Safe methods do not have side effects on the server:
/// - `Get`: Retrieve resource representation
/// - `Head`: Get resource metadata (headers only)
/// - `Options`: Query communication options
///
/// ## Idempotent Methods (RFC 7231)
/// Idempotent methods can be called multiple times with the same effect:
/// - `Get`, `Head`, `Options`: Also safe
/// - `Put`: Replace resource (same result each time)
/// - `Delete`: Remove resource (404 after first deletion is fine)
///
/// ## Non-Idempotent Methods
/// These methods may have different effects when repeated:
/// - `Post`: Create resource or trigger action
/// - `Patch`: Modify resource (result may depend on current state)
///
/// ## Special Methods
/// - `Connect`: Establish tunnel connection
/// - `Trace`: Diagnostic message loop-back
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::router::method::HttpMethod;
///
/// let method = HttpMethod::Get;
/// assert_eq!(method, HttpMethod::Get);
///
/// // Pattern matching
/// match method {
///     HttpMethod::Get => println!("Safe GET request"),
///     HttpMethod::Post => println!("Create or action request"),
///     _ => println!("Other method"),
/// }
/// ```
///
/// ## RESTful Patterns
/// ```
/// use ignitia::router::method::HttpMethod;
///
/// fn handle_resource_method(method: HttpMethod) -> &'static str {
///     match method {
///         HttpMethod::Get => "Retrieve resource",
///         HttpMethod::Post => "Create new resource",
///         HttpMethod::Put => "Replace entire resource",
///         HttpMethod::Patch => "Update part of resource",
///         HttpMethod::Delete => "Remove resource",
///         HttpMethod::Options => "Get allowed methods",
///         _ => "Method not typically used for resources"
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    /// GET method - retrieve resource without side effects (safe, idempotent)
    Get,
    /// POST method - create resource or perform action (not safe, not idempotent)
    Post,
    /// PUT method - replace resource completely (not safe, idempotent)
    Put,
    /// DELETE method - remove resource (not safe, idempotent)
    Delete,
    /// PATCH method - partial resource modification (not safe, not idempotent)
    Patch,
    /// HEAD method - get headers only, no body (safe, idempotent)
    Head,
    /// OPTIONS method - query supported methods (safe, idempotent)
    Options,
    /// CONNECT method - establish tunnel connection
    Connect,
    /// TRACE method - diagnostic trace of request path
    Trace,

    /// ANY method - match any method
    Any,
}

impl From<Method> for HttpMethod {
    /// Converts from the standard `http::Method` to framework `HttpMethod`.
    ///
    /// This conversion handles all standard HTTP methods and provides a
    /// fallback to `Get` for any unrecognized methods.
    ///
    /// # Parameters
    /// - `method`: The standard HTTP method to convert
    ///
    /// # Returns
    /// The corresponding `HttpMethod` variant
    ///
    /// # Examples
    /// ```
    /// use ignitia::router::method::HttpMethod;
    /// use http::Method;
    ///
    /// let std_method = Method::POST;
    /// let custom_method: HttpMethod = std_method.into();
    /// assert_eq!(custom_method, HttpMethod::Post);
    ///
    /// // Extension methods are mapped to Get as fallback
    /// let extension_method = Method::from_bytes(b"CUSTOM").unwrap();
    /// let custom_method: HttpMethod = extension_method.into();
    /// assert_eq!(custom_method, HttpMethod::Get);
    /// ```
    fn from(method: Method) -> Self {
        match method {
            Method::GET => HttpMethod::Get,
            Method::POST => HttpMethod::Post,
            Method::PUT => HttpMethod::Put,
            Method::DELETE => HttpMethod::Delete,
            Method::PATCH => HttpMethod::Patch,
            Method::HEAD => HttpMethod::Head,
            Method::OPTIONS => HttpMethod::Options,
            Method::CONNECT => HttpMethod::Connect,
            Method::TRACE => HttpMethod::Trace,
            _ => HttpMethod::Get, // Default fallback for extension methods
        }
    }
}

impl From<HttpMethod> for Method {
    /// Converts from framework `HttpMethod` to standard `http::Method`.
    ///
    /// This conversion is lossless for all supported HTTP methods.
    ///
    /// # Parameters
    /// - `method`: The framework HTTP method to convert
    ///
    /// # Returns
    /// The corresponding standard `Method`
    ///
    /// # Examples
    /// ```
    /// use ignitia::router::method::HttpMethod;
    /// use http::Method;
    ///
    /// let custom_method = HttpMethod::Delete;
    /// let std_method: Method = custom_method.into();
    /// assert_eq!(std_method, Method::DELETE);
    /// ```
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
            HttpMethod::Connect => Method::CONNECT,
            HttpMethod::Trace => Method::TRACE,
            HttpMethod::Any => Method::from_bytes(b"ANY").unwrap_or(Method::GET),
        }
    }
}

impl FromStr for HttpMethod {
    type Err = ();

    /// Parses an HTTP method from a string representation.
    ///
    /// This parser is case-insensitive and handles all standard HTTP methods.
    /// Unknown methods return an error rather than falling back to a default.
    ///
    /// # Parameters
    /// - `s`: The string to parse as an HTTP method
    ///
    /// # Returns
    /// - `Ok(HttpMethod)`: Successfully parsed method
    /// - `Err(())`: Invalid or unrecognized method string
    ///
    /// # Supported Strings
    /// Case-insensitive parsing of:
    /// - "GET", "get", "Get" → `HttpMethod::Get`
    /// - "POST", "post", "Post" → `HttpMethod::Post`
    /// - "PUT", "put", "Put" → `HttpMethod::Put`
    /// - "DELETE", "delete", "Delete" → `HttpMethod::Delete`
    /// - "PATCH", "patch", "Patch" → `HttpMethod::Patch`
    /// - "HEAD", "head", "Head" → `HttpMethod::Head`
    /// - "OPTIONS", "options", "Options" → `HttpMethod::Options`
    /// - "CONNECT", "connect", "Connect" → `HttpMethod::Connect`
    /// - "TRACE", "trace", "Trace" → `HttpMethod::Trace`
    ///
    /// # Examples
    /// ```
    /// use ignitia::router::method::HttpMethod;
    /// use std::str::FromStr;
    ///
    /// // Standard cases
    /// assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::Get);
    /// assert_eq!(HttpMethod::from_str("POST").unwrap(), HttpMethod::Post);
    ///
    /// // Case insensitive
    /// assert_eq!(HttpMethod::from_str("get").unwrap(), HttpMethod::Get);
    /// assert_eq!(HttpMethod::from_str("Post").unwrap(), HttpMethod::Post);
    /// assert_eq!(HttpMethod::from_str("DELETE").unwrap(), HttpMethod::Delete);
    ///
    /// // Invalid methods
    /// assert!(HttpMethod::from_str("INVALID").is_err());
    /// assert!(HttpMethod::from_str("").is_err());
    /// assert!(HttpMethod::from_str("HTTP").is_err());
    /// ```
    ///
    /// ## Usage in Request Processing
    /// ```
    /// use ignitia::router::method::HttpMethod;
    /// use std::str::FromStr;
    ///
    /// fn process_method_header(method_str: &str) -> Result<String, &'static str> {
    ///     match HttpMethod::from_str(method_str) {
    ///         Ok(HttpMethod::Get) => Ok("Safe read operation".to_string()),
    ///         Ok(HttpMethod::Post) => Ok("Create operation".to_string()),
    ///         Ok(HttpMethod::Put) => Ok("Replace operation".to_string()),
    ///         Ok(HttpMethod::Delete) => Ok("Remove operation".to_string()),
    ///         Ok(_) => Ok("Other operation".to_string()),
    ///         Err(_) => Err("Unknown HTTP method"),
    ///     }
    /// }
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "CONNECT" => Ok(HttpMethod::Connect),
            "TRACE" => Ok(HttpMethod::Trace),
            "ANY" => Ok(HttpMethod::Any),
            _ => Err(()),
        }
    }
}
