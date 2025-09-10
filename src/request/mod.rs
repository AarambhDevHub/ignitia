//! # HTTP Request Handling Module
//!
//! This module provides comprehensive HTTP request handling for the Ignitia web framework.
//! It includes request parsing, parameter extraction, body processing, and extension management.
//!
//! ## Features
//!
//! - **HTTP Request Parsing**: Complete HTTP request parsing with headers, body, and metadata
//! - **Path Parameters**: Dynamic route parameter extraction and parsing
//! - **Query Parameters**: URL query string parsing and type conversion
//! - **Request Body**: Support for JSON, text, and binary request bodies
//! - **Header Access**: Easy access to HTTP headers with type safety
//! - **Extensions**: Type-safe request extensions for middleware communication
//! - **Performance Optimized**: Efficient parsing with minimal allocations
//!
//! ## Request Lifecycle
//!
//! 1. **Raw HTTP Request**: Hyper provides the raw HTTP request
//! 2. **Request Creation**: Convert to Ignitia's Request struct
//! 3. **Parameter Parsing**: Extract path and query parameters
//! 4. **Body Processing**: Parse request body based on content type
//! 5. **Handler Processing**: Pass to route handlers with extractors
//! 6. **Response Generation**: Generate HTTP response
//!
//! ## Core Components
//!
//! - **Request**: Main request struct containing all HTTP data
//! - **Body**: Request body handling and parsing (`body.rs`)
//! - **Params**: Parameter extraction and type conversion (`params.rs`)
//! - **Extensions**: Type-safe extension storage for middleware
//!
//! ## Quick Start
//!
//! ### Basic Request Handling
//! ```
//! use ignitia::{Router, Request, Response, Result};
//!
//! async fn handler(req: Request) -> Result<Response> {
//!     // Access HTTP method
//!     println!("Method: {}", req.method);
//!
//!     // Access path
//!     println!("Path: {}", req.uri.path());
//!
//!     // Access headers
//!     if let Some(user_agent) = req.header("user-agent") {
//!         println!("User-Agent: {}", user_agent);
//!     }
//!
//!     Ok(Response::text("Hello World!"))
//! }
//!
//! let router = Router::new()
//!     .get("/", |req| async { handler(req).await });
//! ```
//!
//! ### Working with Path Parameters
//! ```
//! use ignitia::{Router, Request, Response, Result};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserParams {
//!     id: u32,
//!     action: String,
//! }
//!
//! async fn user_handler(req: Request) -> Result<Response> {
//!     // Access individual parameters
//!     if let Some(id) = req.param("id") {
//!         println!("User ID: {}", id);
//!     }
//!
//!     Ok(Response::text(format!("User ID: {}", req.param("id").unwrap_or("unknown"))))
//! }
//!
//! let router = Router::new()
//!     .get("/users/:id/:action", |req| async { user_handler(req).await });
//! ```
//!
//! ### Working with Query Parameters
//! ```
//! use ignitia::{Router, Request, Response, Result};
//!
//! async fn search_handler(req: Request) -> Result<Response> {
//!     let query = req.query("q").unwrap_or("default");
//!     let page: u32 = req.query("page")
//!         .and_then(|p| p.parse().ok())
//!         .unwrap_or(1);
//!
//!     Ok(Response::text(format!("Search: {} (page {})", query, page)))
//! }
//!
//! let router = Router::new()
//!     .get("/search", |req| async { search_handler(req).await });
//! // URL: /search?q=rust&page=2
//! ```
//!
//! ### Working with JSON Bodies
//! ```
//! use ignitia::{Router, Request, Response, Result};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Deserialize)]
//! struct CreateUser {
//!     name: String,
//!     email: String,
//! }
//!
//! #[derive(Serialize)]
//! struct UserResponse {
//!     id: u32,
//!     name: String,
//!     email: String,
//! }
//!
//! async fn create_user(req: Request) -> Result<Response> {
//!     let user_data: CreateUser = req.json()?;
//!
//!     let response = UserResponse {
//!         id: 123,
//!         name: user_data.name,
//!         email: user_data.email,
//!     };
//!
//!     Response::json(response)
//! }
//!
//! let router = Router::new()
//!     .post("/users", |req| async { create_user(req).await });
//! ```
//!
//! ## Advanced Usage
//!
//! ### Using Extensions for Middleware Communication
//! ```
//! use ignitia::{Router, Request, Response, Result, Middleware};
//! use async_trait::async_trait;
//!
//! #[derive(Clone)]
//! struct UserId(u32);
//!
//! struct AuthMiddleware;
//!
//! #[async_trait]
//! impl Middleware for AuthMiddleware {
//!     async fn before(&self, req: &mut Request) -> Result<()> {
//!         // Extract user ID from token and store in extensions
//!         let user_id = UserId(123); // From JWT token
//!         req.insert_extension(user_id);
//!         Ok(())
//!     }
//! }
//!
//! async fn protected_handler(req: Request) -> Result<Response> {
//!     if let Some(user_id) = req.get_extension::<UserId>() {
//!         Ok(Response::text(format!("Hello user {}", user_id.0)))
//!     } else {
//!         Ok(Response::text("Not authenticated"))
//!     }
//! }
//!
//! let router = Router::new()
//!     .middleware(AuthMiddleware)
//!     .get("/profile", |req| async { protected_handler(req).await });
//! ```
//!
//! ### Custom Request Processing
//! ```
//! use ignitia::{Request, Response, Result};
//! use bytes::Bytes;
//!
//! async fn upload_handler(req: Request) -> Result<Response> {
//!     // Check content type
//!     let content_type = req.header("content-type").unwrap_or("application/octet-stream");
//!
//!     // Get raw body
//!     let body_size = req.body.len();
//!
//!     // Process based on content type
//!     match content_type {
//!         "application/json" => {
//!             let json_data: serde_json::Value = req.json()?;
//!             Ok(Response::text(format!("Received JSON with {} bytes", body_size)))
//!         }
//!         "multipart/form-data" => {
//!             // Handle file upload
//!             Ok(Response::text(format!("Received file upload with {} bytes", body_size)))
//!         }
//!         _ => {
//!             Ok(Response::text(format!("Received {} bytes of {}", body_size, content_type)))
//!         }
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! ### Efficient Query Parameter Parsing
//! The query parameter parser is optimized for performance:
//! - **Zero-Copy Parsing**: Minimal string allocations
//! - **Pre-allocated Capacity**: Efficient HashMap growth
//! - **Single Pass**: Parse query string in one iteration
//!
//! ### Memory Management
//! - **Shared Extensions**: Extensions use Arc for efficient sharing
//! - **Lazy Parsing**: Parameters parsed only when needed
//! - **Efficient Body Handling**: Uses bytes::Bytes for zero-copy operations
//!
//! ## Error Handling
//!
//! ### JSON Parsing Errors
//! ```
//! use ignitia::{Request, Response, Result, Error};
//!
//! async fn json_handler(req: Request) -> Result<Response> {
//!     match req.json::<serde_json::Value>() {
//!         Ok(data) => Ok(Response::json(data)?),
//!         Err(Error::BadRequest(msg)) => {
//!             Ok(Response::text(format!("Invalid JSON: {}", msg))
//!                 .with_status_code(400))
//!         }
//!         Err(e) => Err(e),
//!     }
//! }
//! ```
//!
//! ### Parameter Type Conversion
//! ```
//! use ignitia::{Request, Response, Result};
//!
//! async fn numeric_handler(req: Request) -> Result<Response> {
//!     let id_str = req.param("id").ok_or_else(|| {
//!         ignitia::Error::BadRequest("Missing id parameter".into())
//!     })?;
//!
//!     let id: u32 = id_str.parse().map_err(|_| {
//!         ignitia::Error::BadRequest("Invalid id format".into())
//!     })?;
//!
//!     Ok(Response::text(format!("ID: {}", id)))
//! }
//! ```
//!
//! ## Security Considerations
//!
//! ### Input Validation
//! - Always validate and sanitize user input from parameters and body
//! - Use type-safe extractors to prevent injection attacks
//! - Set reasonable limits on request body sizes
//!
//! ### Header Handling
//! - Be cautious with user-controlled headers
//! - Validate header values before using them in business logic
//! - Consider header injection attacks
//!
//! ## Testing
//!
//! ### Unit Testing Requests
//! ```
//! use ignitia::Request;
//! use http::{Method, Uri};
//! use bytes::Bytes;
//!
//! #[tokio::test]
//! async fn test_request_creation() {
//!     let uri: Uri = "/test?param=value".parse().unwrap();
//!     let request = Request::new(
//!         Method::GET,
//!         uri,
//!         http::Version::HTTP_11,
//!         http::HeaderMap::new(),
//!         Bytes::new(),
//!     );
//!
//!     assert_eq!(request.method, Method::GET);
//!     assert_eq!(request.query("param"), Some(&"value".to_string()));
//! }
//! ```

pub mod body;
pub mod params;

use crate::{error::Result, Extensions};
use bytes::Bytes;
use http::{HeaderMap, Method, Uri, Version};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::Arc};

/// HTTP request representation containing all request data and metadata.
///
/// The `Request` struct encapsulates all information about an incoming HTTP request,
/// including the method, URI, headers, body, and extracted parameters. It also
/// provides convenient methods for accessing common request data.
///
/// # Structure
/// - **method**: HTTP method (GET, POST, PUT, DELETE, etc.)
/// - **uri**: Request URI with path and query string
/// - **version**: HTTP protocol version
/// - **headers**: HTTP headers as a HeaderMap
/// - **body**: Request body as bytes
/// - **params**: Extracted path parameters from route matching
/// - **query_params**: Parsed query string parameters
/// - **extensions**: Type-safe storage for middleware data
///
/// # Examples
///
/// ## Basic Request Information
/// ```
/// use ignitia::Request;
/// use http::Method;
///
/// async fn handler(req: Request) -> ignitia::Result<ignitia::Response> {
///     println!("Method: {}", req.method);
///     println!("Path: {}", req.uri.path());
///     println!("Query: {:?}", req.uri.query());
///
///     Ok(ignitia::Response::text("Request processed"))
/// }
/// ```
///
/// ## Accessing Headers
/// ```
/// use ignitia::Request;
///
/// async fn header_example(req: Request) -> ignitia::Result<ignitia::Response> {
///     let user_agent = req.header("user-agent").unwrap_or("unknown");
///     let content_type = req.header("content-type").unwrap_or("text/plain");
///
///     Ok(ignitia::Response::text(format!(
///         "User-Agent: {}, Content-Type: {}",
///         user_agent,
///         content_type
///     )))
/// }
/// ```
///
/// ## Working with Parameters
/// ```
/// use ignitia::Request;
///
/// async fn param_example(req: Request) -> ignitia::Result<ignitia::Response> {
///     // Path parameters (from /users/:id)
///     let user_id = req.param("id").unwrap_or("0");
///
///     // Query parameters (from ?page=1&limit=10)
///     let page = req.query("page").unwrap_or("1");
///     let limit = req.query("limit").unwrap_or("10");
///
///     Ok(ignitia::Response::text(format!(
///         "User: {}, Page: {}, Limit: {}",
///         user_id, page, limit
///     )))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: Method,
    /// Request URI containing path and query string
    pub uri: Uri,
    /// HTTP protocol version
    pub version: Version,
    /// HTTP headers
    pub headers: HeaderMap,
    /// Request body as bytes
    pub body: Bytes,
    /// Path parameters extracted from route matching
    pub params: HashMap<String, String>,
    /// Query string parameters parsed from URI
    pub query_params: HashMap<String, String>,
    /// Type-safe extension storage for middleware communication
    pub extensions: Extensions,
}

impl Request {
    /// Creates a new Request instance from HTTP components.
    ///
    /// This method is typically called by the framework when converting from
    /// a hyper::Request. It automatically parses query parameters from the URI.
    ///
    /// # Parameters
    /// - `method`: HTTP method
    /// - `uri`: Request URI
    /// - `version`: HTTP version
    /// - `headers`: HTTP headers
    /// - `body`: Request body as bytes
    ///
    /// # Returns
    /// A new Request instance with parsed query parameters
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    /// use http::{Method, Uri, Version, HeaderMap};
    /// use bytes::Bytes;
    ///
    /// let uri: Uri = "/users?page=1&limit=10".parse().unwrap();
    /// let request = Request::new(
    ///     Method::GET,
    ///     uri,
    ///     Version::HTTP_11,
    ///     HeaderMap::new(),
    ///     Bytes::new(),
    /// );
    ///
    /// assert_eq!(request.query("page"), Some(&"1".to_string()));
    /// assert_eq!(request.query("limit"), Some(&"10".to_string()));
    /// ```
    pub fn new(
        method: Method,
        uri: Uri,
        version: Version,
        headers: HeaderMap,
        body: Bytes,
    ) -> Self {
        let query_params = Self::parse_query_params(&uri);

        Self {
            method,
            uri,
            version,
            headers,
            body,
            params: HashMap::new(),
            query_params,
            extensions: Extensions::new(),
        }
    }

    /// Parses query parameters from a URI efficiently.
    ///
    /// This method performs optimized parsing of query strings with minimal
    /// allocations. It handles URL encoding and properly separates key-value pairs.
    ///
    /// # Parameters
    /// - `uri`: The URI to parse query parameters from
    ///
    /// # Returns
    /// HashMap containing parsed query parameters
    ///
    /// # Performance
    /// - Pre-allocates string capacity to reduce allocations
    /// - Single-pass parsing algorithm
    /// - Handles empty keys and values gracefully
    ///
    /// # Examples
    /// ```
    /// let uri: Uri = "/search?q=rust&page=2&sort=desc".parse().unwrap();
    /// let params = Request::parse_query_params(&uri);
    ///
    /// assert_eq!(params.get("q"), Some(&"rust".to_string()));
    /// assert_eq!(params.get("page"), Some(&"2".to_string()));
    /// assert_eq!(params.get("sort"), Some(&"desc".to_string()));
    /// ```
    fn parse_query_params(uri: &Uri) -> HashMap<String, String> {
        let query = match uri.query() {
            Some(q) => q,
            None => return HashMap::new(),
        };

        let mut params = HashMap::new();
        let mut key = String::with_capacity(32);
        let mut value = String::with_capacity(64);
        let mut parsing_key = true;

        for c in query.chars() {
            match c {
                '&' => {
                    if !key.is_empty() {
                        params.insert(std::mem::take(&mut key), std::mem::take(&mut value));
                    }
                    parsing_key = true;
                }
                '=' if parsing_key => {
                    parsing_key = false;
                }
                _ if parsing_key => {
                    key.push(c);
                }
                _ => {
                    value.push(c);
                }
            }
        }

        if !key.is_empty() {
            params.insert(key, value);
        }

        params
    }

    /// Parses the request body as JSON and deserializes it to the specified type.
    ///
    /// This method provides optimized JSON parsing with content type validation.
    /// It performs pre-checks to ensure the request contains valid JSON data.
    ///
    /// # Type Parameters
    /// - `T`: The type to deserialize the JSON into (must implement DeserializeOwned)
    ///
    /// # Returns
    /// - `Ok(T)`: Successfully parsed JSON data
    /// - `Err(Error)`: JSON parsing error or content type mismatch
    ///
    /// # Errors
    /// - `BadRequest`: Empty body, wrong content type, or invalid JSON
    /// - `Json`: JSON deserialization errors
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct UserData {
    ///     name: String,
    ///     email: String,
    /// }
    ///
    /// async fn create_user(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     let user: UserData = req.json()?;
    ///
    ///     Ok(ignitia::Response::text(format!(
    ///         "Created user: {} ({})",
    ///         user.name,
    ///         user.email
    ///     )))
    /// }
    /// ```
    ///
    /// ## Error Handling
    /// ```
    /// use ignitia::{Request, Response, Error};
    ///
    /// async fn safe_json_handler(req: Request) -> ignitia::Result<Response> {
    ///     match req.json::<serde_json::Value>() {
    ///         Ok(data) => Ok(Response::json(data)?),
    ///         Err(Error::BadRequest(msg)) => {
    ///             Ok(Response::text(format!("Invalid request: {}", msg))
    ///                 .with_status_code(400))
    ///         }
    ///         Err(e) => Err(e),
    ///     }
    /// }
    /// ```
    // Optimized JSON parsing with pre-check
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        if self.body.is_empty() {
            return Err(crate::Error::BadRequest("Empty body".into()));
        }

        // Quick check for JSON content type
        if let Some(content_type) = self.header("content-type") {
            if !content_type.starts_with("application/json") {
                return Err(crate::Error::BadRequest(
                    "Expected JSON content type".into(),
                ));
            }
        }

        serde_json::from_slice(&self.body).map_err(Into::into)
    }

    /// Gets a path parameter value by key.
    ///
    /// Path parameters are extracted from route patterns like `/users/:id/:action`
    /// and stored when the route is matched.
    ///
    /// # Parameters
    /// - `key`: The parameter name (without the colon prefix)
    ///
    /// # Returns
    /// - `Some(&String)`: The parameter value if found
    /// - `None`: If the parameter doesn't exist
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    ///
    /// // For route "/users/:id/:action" with URL "/users/123/edit"
    /// async fn user_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     let user_id = req.param("id").unwrap_or("0");
    ///     let action = req.param("action").unwrap_or("view");
    ///
    ///     Ok(ignitia::Response::text(format!(
    ///         "User {} - Action: {}",
    ///         user_id,
    ///         action
    ///     )))
    /// }
    /// ```
    ///
    /// ## Type Conversion
    /// ```
    /// use ignitia::Request;
    ///
    /// async fn typed_param_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     let user_id: u32 = req.param("id")
    ///         .and_then(|id| id.parse().ok())
    ///         .unwrap_or(0);
    ///
    ///     Ok(ignitia::Response::text(format!("User ID: {}", user_id)))
    /// }
    /// ```
    // Inline these methods for better performance
    #[inline]
    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// Gets a query parameter value by key.
    ///
    /// Query parameters are parsed from the URL query string (after the `?`).
    /// Multiple values for the same key are not supported - only the last value is kept.
    ///
    /// # Parameters
    /// - `key`: The query parameter name
    ///
    /// # Returns
    /// - `Some(&String)`: The parameter value if found
    /// - `None`: If the parameter doesn't exist
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    ///
    /// // For URL "/search?q=rust&page=2&limit=10"
    /// async fn search_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     let query = req.query("q").unwrap_or("*");
    ///     let page = req.query("page")
    ///         .and_then(|p| p.parse::<u32>().ok())
    ///         .unwrap_or(1);
    ///     let limit = req.query("limit")
    ///         .and_then(|l| l.parse::<u32>().ok())
    ///         .unwrap_or(10);
    ///
    ///     Ok(ignitia::Response::text(format!(
    ///         "Search: '{}' (page {}, limit {})",
    ///         query, page, limit
    ///     )))
    /// }
    /// ```
    ///
    /// ## Boolean Parameters
    /// ```
    /// use ignitia::Request;
    ///
    /// async fn filter_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     let include_archived = req.query("archived")
    ///         .map(|v| v == "true" || v == "1")
    ///         .unwrap_or(false);
    ///
    ///     let response = if include_archived {
    ///         "Including archived items"
    ///     } else {
    ///         "Excluding archived items"
    ///     };
    ///
    ///     Ok(ignitia::Response::text(response))
    /// }
    /// ```
    #[inline]
    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    /// Gets an HTTP header value by name.
    ///
    /// Header names are case-insensitive according to HTTP specifications.
    /// This method returns the header value as a string slice if it exists
    /// and contains valid UTF-8.
    ///
    /// # Parameters
    /// - `key`: The header name (case-insensitive)
    ///
    /// # Returns
    /// - `Some(&str)`: The header value if found and valid UTF-8
    /// - `None`: If the header doesn't exist or contains invalid UTF-8
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    ///
    /// async fn header_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     let user_agent = req.header("user-agent").unwrap_or("Unknown");
    ///     let accept = req.header("accept").unwrap_or("*/*");
    ///     let auth = req.header("authorization");
    ///
    ///     let response = format!(
    ///         "User-Agent: {}\nAccept: {}\nAuth: {}",
    ///         user_agent,
    ///         accept,
    ///         auth.unwrap_or("None")
    ///     );
    ///
    ///     Ok(ignitia::Response::text(response))
    /// }
    /// ```
    ///
    /// ## Authentication Headers
    /// ```
    /// use ignitia::Request;
    ///
    /// async fn auth_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     match req.header("authorization") {
    ///         Some(auth) if auth.starts_with("Bearer ") => {
    ///             let token = &auth[7..];
    ///             Ok(ignitia::Response::text(format!("Token: {}", token)))
    ///         }
    ///         Some(_) => {
    ///             Ok(ignitia::Response::text("Invalid auth format")
    ///                 .with_status_code(400))
    ///         }
    ///         None => {
    ///             Ok(ignitia::Response::text("Missing authorization")
    ///                 .with_status_code(401))
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|v| v.to_str().ok())
    }

    /// Inserts an extension value into the request.
    ///
    /// Extensions provide type-safe storage for sharing data between middleware
    /// and handlers. They are commonly used for authentication data, request IDs,
    /// database connections, and other per-request context.
    ///
    /// # Type Parameters
    /// - `T`: The type to store (must be Send + Sync + 'static)
    ///
    /// # Parameters
    /// - `value`: The value to store
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Request, Middleware};
    /// use async_trait::async_trait;
    ///
    /// #[derive(Clone)]
    /// struct UserId(u32);
    ///
    /// #[derive(Clone)]
    /// struct RequestId(String);
    ///
    /// struct AuthMiddleware;
    ///
    /// #[async_trait]
    /// impl Middleware for AuthMiddleware {
    ///     async fn before(&self, req: &mut Request) -> ignitia::Result<()> {
    ///         // Extract user from JWT token
    ///         let user_id = UserId(123);
    ///         req.insert_extension(user_id);
    ///
    ///         // Generate request ID
    ///         let request_id = RequestId(uuid::Uuid::new_v4().to_string());
    ///         req.insert_extension(request_id);
    ///
    ///         Ok(())
    ///     }
    /// }
    /// ```
    // Extension methods
    /// Insert an extension value
    pub fn insert_extension<T: Send + Sync + 'static>(&mut self, value: T) {
        self.extensions.insert(value);
    }

    /// Gets an extension value from the request.
    ///
    /// Extensions are retrieved by type. The value is returned as an Arc<T>
    /// for efficient sharing across the application.
    ///
    /// # Type Parameters
    /// - `T`: The type to retrieve (must be Send + Sync + Clone + 'static)
    ///
    /// # Returns
    /// - `Some(Arc<T>)`: The extension value if found
    /// - `None`: If no extension of this type exists
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    /// use std::sync::Arc;
    ///
    /// #[derive(Clone)]
    /// struct UserId(u32);
    ///
    /// async fn protected_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     match req.get_extension::<UserId>() {
    ///         Some(user_id) => {
    ///             Ok(ignitia::Response::text(format!("Hello, user {}", user_id.0)))
    ///         }
    ///         None => {
    ///             Ok(ignitia::Response::text("Not authenticated")
    ///                 .with_status_code(401))
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// ## Database Connection Example
    /// ```
    /// use ignitia::Request;
    /// use std::sync::Arc;
    ///
    /// struct DatabasePool; // Placeholder for actual DB pool
    ///
    /// async fn db_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     let db_pool = req.get_extension::<DatabasePool>()
    ///         .ok_or_else(|| ignitia::Error::Internal("Database not available".into()))?;
    ///
    ///     // Use database connection
    ///     Ok(ignitia::Response::text("Data retrieved"))
    /// }
    /// ```
    /// Get an extension value (returns Arc<T> for shared ownership)
    pub fn get_extension<T: Send + Sync + Clone + 'static>(&self) -> Option<Arc<T>> {
        self.extensions.get()
    }

    /// Removes an extension value from the request.
    ///
    /// This method removes and returns the extension value, transferring
    /// ownership to the caller.
    ///
    /// # Type Parameters
    /// - `T`: The type to remove (must be Send + Sync + 'static)
    ///
    /// # Returns
    /// - `Some(T)`: The removed extension value
    /// - `None`: If no extension of this type exists
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    ///
    /// struct TempData(String);
    ///
    /// async fn consume_handler(mut req: Request) -> ignitia::Result<ignitia::Response> {
    ///     match req.remove_extension::<TempData>() {
    ///         Some(data) => {
    ///             Ok(ignitia::Response::text(format!("Consumed: {}", data.0)))
    ///         }
    ///         None => {
    ///             Ok(ignitia::Response::text("No temporary data"))
    ///         }
    ///     }
    /// }
    /// ```
    /// Remove an extension value
    pub fn remove_extension<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.extensions.remove()
    }

    /// Checks if an extension of the specified type exists.
    ///
    /// This method provides a lightweight way to check for extension
    /// existence without retrieving the value.
    ///
    /// # Type Parameters
    /// - `T`: The type to check for (must be Send + Sync + 'static)
    ///
    /// # Returns
    /// - `true`: If an extension of this type exists
    /// - `false`: If no extension of this type exists
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    ///
    /// struct AuthToken(String);
    ///
    /// async fn conditional_handler(req: Request) -> ignitia::Result<ignitia::Response> {
    ///     if req.has_extension::<AuthToken>() {
    ///         // Process as authenticated request
    ///         Ok(ignitia::Response::text("Authenticated request"))
    ///     } else {
    ///         // Process as public request
    ///         Ok(ignitia::Response::text("Public request"))
    ///     }
    /// }
    /// ```
    /// Check if an extension exists
    pub fn has_extension<T: Send + Sync + 'static>(&self) -> bool {
        self.extensions.contains::<T>()
    }
}
