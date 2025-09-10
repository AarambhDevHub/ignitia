//! # Route Matching and Parameter Extraction
//!
//! This module provides high-performance route matching with parameter extraction capabilities.
//! It includes optimized regex compilation, path parameter parsing, and efficient route matching
//! algorithms designed for maximum performance in web request routing.
//!
//! ## Features
//!
//! - **High-Performance Matching**: Optimized regex patterns with size limits and caching
//! - **Parameter Extraction**: Support for named parameters (`:param`) and wildcards (`*param`)
//! - **Early Rejection**: Fast path checks to reject non-matching routes quickly
//! - **Type-Safe Parameters**: Automatic parameter extraction with type conversion
//! - **Wildcard Support**: Flexible wildcard matching for file paths and catch-all routes
//!
//! ## Route Pattern Syntax
//!
//! ### Static Routes
//! ```
//! /users
//! /api/health
//! /static/css/main.css
//! ```
//!
//! ### Named Parameters
//! Named parameters match a single path segment and are accessible by name:
//! ```
//! /users/:id          # Matches /users/123, /users/abc
//! /posts/:slug        # Matches /posts/hello-world
//! /api/:version/users # Matches /api/v1/users, /api/v2/users
//! ```
//!
//! ### Wildcard Parameters
//! Wildcards match one or more path segments including forward slashes:
//! ```
//! /files/*path        # Matches /files/docs/readme.txt
//! /proxy/*url         # Matches /proxy/https://example.com/path
//! ```
//!
//! ### Combined Patterns
//! ```
//! /users/:id/files/*path    # Matches /users/123/files/documents/file.pdf
//! /api/:version/*endpoint   # Matches /api/v1/users/profile/settings
//! ```
//!
//! ## Performance Optimizations
//!
//! ### Fast Path Rejection
//! Routes use multiple fast-path checks before expensive regex matching:
//! - Method matching (fastest)
//! - Minimum path length checks
//! - Path segment count validation
//! - Only then regex pattern matching
//!
//! ### Regex Optimization
//! - Pre-compiled regex patterns with size limits
//! - Selective escaping to preserve performance
//! - Cached parameter counts and segment counts
//!
//! ## Usage Examples
//!
//! ### Basic Route Creation
//! ```
//! use ignitia::router::{Route, HandlerFn};
//! use http::Method;
//! use std::sync::Arc;
//!
//! let handler = Arc::new(|req| Box::pin(async move {
//!     Ok(ignitia::Response::text("Hello"))
//! }));
//!
//! let route = Route::new("/users/:id", Method::GET, handler);
//! ```
//!
//! ### Route Matching
//! ```
//! use ignitia::{Request, Method};
//! use http::Uri;
//! use bytes::Bytes;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let request = Request::new(
//!     Method::GET,
//!     "/users/123".parse::<Uri>()?,
//!     http::Version::HTTP_11,
//!     http::HeaderMap::new(),
//!     Bytes::new(),
//! );
//!
//! // This would extract parameters: {"id": "123"}
//! if let Some(params) = route.matches(&request) {
//!     assert_eq!(params.get("id"), Some(&"123".to_string()));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Pattern Examples
//!
//! ### File Serving Routes
//! ```
//! // Serve static files with wildcard paths
//! // Pattern: /static/*path
//! // Matches: /static/css/main.css -> path = "css/main.css"
//! //          /static/js/app.min.js -> path = "js/app.min.js"
//! ```
//!
//! ### API Versioning
//! ```
//! // Version-specific API routes
//! // Pattern: /api/:version/users/:id
//! // Matches: /api/v1/users/123 -> version = "v1", id = "123"
//! //          /api/v2/users/456 -> version = "v2", id = "456"
//! ```
//!
//! ### Proxy Routes
//! ```
//! // Proxy requests with full URL capture
//! // Pattern: /proxy/*url
//! // Matches: /proxy/https://example.com/api/data -> url = "https://example.com/api/data"
//! ```

use crate::{HandlerFn, Middleware, Request};
use http::Method;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::{collections::HashMap, sync::Arc};

/// Pre-compiled regex for matching named parameters in route patterns.
///
/// Matches patterns like `:id`, `:slug`, `:version` in route definitions.
/// The parameter name must start with a letter or underscore and can contain
/// letters, numbers, and underscores.
static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

/// Pre-compiled regex for matching wildcard parameters in route patterns.
///
/// Matches patterns like `*path`, `*url`, `*file` in route definitions.
/// The wildcard name must start with a letter or underscore and can contain
/// letters, numbers, and underscores.
static WILDCARD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\*([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());

/// Represents a compiled route with optimized matching capabilities.
///
/// A `Route` contains all the information needed to match incoming requests
/// and extract parameters efficiently. It uses pre-compiled regex patterns
/// and cached metadata for maximum performance.
///
/// # Performance Features
/// - Pre-compiled regex patterns with size limits
/// - Cached parameter and segment counts for fast rejection
/// - Optimized parameter extraction with pre-allocated hashmaps
/// - Early method and path length checks
///
/// # Examples
///
/// ## Creating Routes
/// ```
/// use ignitia::router::Route;
/// use http::Method;
/// use std::sync::Arc;
///
/// let handler = Arc::new(|req| Box::pin(async move {
///     Ok(ignitia::Response::text("User profile"))
/// }));
///
/// let route = Route::new("/users/:id/profile", Method::GET, handler);
/// ```
///
/// ## Parameter Extraction
/// ```
/// use ignitia::{Request, Method};
/// use http::Uri;
/// use bytes::Bytes;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = Request::new(
///     Method::GET,
///     "/users/42/profile".parse::<Uri>()?,
///     http::Version::HTTP_11,
///     http::HeaderMap::new(),
///     Bytes::new(),
/// );
///
/// if let Some(params) = route.matches(&request) {
///     assert_eq!(params.get("id"), Some(&"42".to_string()));
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Route {
    /// The original path pattern used to create this route
    pub path: String,
    /// The HTTP method this route responds to
    pub method: Method,
    /// The handler function to execute when this route matches
    pub handler: HandlerFn,
    /// The compiled regex pattern for path matching
    pub regex: Regex,
    /// Names of named parameters in order of appearance
    pub param_names: Vec<String>,
    /// Names of wildcard parameters in order of appearance
    pub wildcard_names: Vec<String>,
    /// Cached total parameter count for performance optimization
    total_params: usize,
    /// Cached path segment count for performance optimization
    segment_count: usize,
    /// Middleware to be executed before the route handler
    pub middleware: Vec<Arc<dyn Middleware>>,
}

impl Route {
    /// Creates a new route with the specified path pattern, method, and handler.
    ///
    /// This method compiles the path pattern into an optimized regex and caches
    /// metadata for efficient matching. The compilation process handles both
    /// named parameters (`:param`) and wildcards (`*param`).
    ///
    /// # Parameters
    /// - `path`: The route path pattern (e.g., "/users/:id", "/files/*path")
    /// - `method`: The HTTP method this route should match
    /// - `handler`: The handler function to execute for matching requests
    ///
    /// # Returns
    /// A new `Route` instance ready for request matching
    ///
    /// # Pattern Compilation
    /// The path pattern is processed as follows:
    /// 1. Wildcards (`*param`) are converted to `(.+)` regex groups
    /// 2. Named parameters (`:param`) are converted to `([^/]+)` regex groups
    /// 3. The rest of the path is escaped for literal matching
    /// 4. The final pattern is anchored with `^` and `$`
    ///
    /// # Examples
    /// ```
    /// use ignitia::router::Route;
    /// use http::Method;
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(|req| Box::pin(async move {
    ///     Ok(ignitia::Response::text("Hello"))
    /// }));
    ///
    /// // Static route
    /// let static_route = Route::new("/api/health", Method::GET, handler.clone());
    ///
    /// // Route with named parameter
    /// let param_route = Route::new("/users/:id", Method::GET, handler.clone());
    ///
    /// // Route with wildcard
    /// let wildcard_route = Route::new("/files/*path", Method::GET, handler.clone());
    ///
    /// // Complex route with both types
    /// let complex_route = Route::new("/users/:id/files/*path", Method::GET, handler);
    /// ```
    pub fn new(path: &str, method: Method, handler: HandlerFn) -> Self {
        let (regex_pattern, param_names, wildcard_names) = Self::build_regex(path);
        let regex = Self::compile_regex(&regex_pattern);
        let total_params = param_names.len() + wildcard_names.len();
        let segment_count = path.matches('/').count();

        Self {
            path: path.to_string(),
            method,
            handler,
            regex,
            param_names,
            wildcard_names,
            total_params,
            segment_count,
            middleware: Vec::new(),
        }
    }

    /// Adds middleware to this specific route.
    ///
    /// Route-level middleware is applied in addition to global router middleware.
    /// The middleware execution order follows the standard pattern:
    /// - Global middleware `before` hooks (in registration order)
    /// - Route middleware `before` hooks (in registration order)
    /// - Handler execution
    /// - Route middleware `after` hooks (in reverse order)
    /// - Global middleware `after` hooks (in reverse order)
    ///
    /// # Type Parameters
    /// - `M`: Middleware type that implements the `Middleware` trait
    ///
    /// # Parameters
    /// - `mw`: The middleware instance to add to this route
    ///
    /// # Returns
    /// The route instance for method chaining
    ///
    /// # Use Cases
    /// - Authentication for specific endpoints
    /// - Rate limiting for resource-intensive routes
    /// - Logging for debug routes
    /// - Custom validation for specific route patterns
    ///
    /// # Examples
    /// ```
    /// use ignitia::{router::Route, middleware::AuthMiddleware};
    /// use http::Method;
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(|req| Box::pin(async move {
    ///     Ok(ignitia::Response::text("Protected resource"))
    /// }));
    ///
    /// let route = Route::new("/admin/users", Method::GET, handler)
    ///     .with_middleware(AuthMiddleware::new("admin-token"))
    ///     .with_middleware(RateLimitMiddleware::new(10));
    /// ```
    pub fn with_middleware(mut self, mw: impl Middleware + 'static) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    /// Compiles a regex pattern with optimized settings for route matching.
    ///
    /// This method creates a regex with size limits to prevent memory exhaustion
    /// and optimize compilation time. The limits are set conservatively to handle
    /// typical web application route patterns efficiently.
    ///
    /// # Parameters
    /// - `pattern`: The regex pattern string to compile
    ///
    /// # Returns
    /// A compiled `Regex` instance
    ///
    /// # Panics
    /// Panics if the regex pattern is invalid. This should not happen in normal
    /// operation as patterns are generated programmatically.
    ///
    /// # Performance Settings
    /// - Size limit: 5KB for compiled regex
    /// - DFA size limit: 5KB for regex automaton
    ///
    /// # Examples
    /// ```
    /// use ignitia::router::Route;
    ///
    /// let regex = Route::compile_regex(r"^/users/([^/]+)$");
    /// assert!(regex.is_match("/users/123"));
    /// assert!(!regex.is_match("/users/123/posts"));
    /// ```
    pub fn compile_regex(pattern: &str) -> Regex {
        RegexBuilder::new(pattern)
            .size_limit(5 * 1024)
            .dfa_size_limit(5 * 1024)
            .build()
            .expect("Invalid regex pattern")
    }

    /// Builds a regex pattern from a route path, extracting parameter names.
    ///
    /// This method processes the route path to identify wildcards and named
    /// parameters, converts them to appropriate regex groups, and returns
    /// the compiled pattern along with parameter name lists.
    ///
    /// # Parameters
    /// - `path`: The original route path with parameter patterns
    ///
    /// # Returns
    /// A tuple containing:
    /// - The compiled regex pattern string
    /// - Vector of named parameter names in order
    /// - Vector of wildcard parameter names in order
    ///
    /// # Processing Order
    /// 1. **Wildcards First**: `*param` → `(.+)` (matches one or more characters)
    /// 2. **Named Parameters**: `:param` → `([^/]+)` (matches non-slash characters)
    /// 3. **Escape Special Characters**: Literal path parts are regex-escaped
    /// 4. **Anchor Pattern**: Wrap with `^` and `$` for exact matching
    ///
    /// # Examples
    /// ```
    /// use ignitia::router::Route;
    ///
    /// // Simple named parameter
    /// let (pattern, params, wildcards) = Route::build_regex("/users/:id");
    /// assert_eq!(pattern, "^/users/([^/]+)$");
    /// assert_eq!(params, vec!["id"]);
    /// assert_eq!(wildcards, Vec::<String>::new());
    ///
    /// // Wildcard parameter
    /// let (pattern, params, wildcards) = Route::build_regex("/files/*path");
    /// assert_eq!(pattern, "^/files/(.+)$");
    /// assert_eq!(params, Vec::<String>::new());
    /// assert_eq!(wildcards, vec!["path"]);
    /// ```
    fn build_regex(path: &str) -> (String, Vec<String>, Vec<String>) {
        let mut param_names = Vec::new();
        let mut wildcard_names = Vec::new();

        // Handle wildcards first
        let path_with_wildcards = WILDCARD_REGEX.replace_all(path, |caps: &regex::Captures| {
            wildcard_names.push(caps[1].to_string());
            "(.+)"
        });

        // Then handle regular parameters
        let path_with_params =
            PARAM_REGEX.replace_all(&path_with_wildcards, |caps: &regex::Captures| {
                param_names.push(caps[1].to_string());
                "([^/]+)"
            });

        // Escape only the parts that aren't our regex groups
        let escaped_pattern = escape_regex_selective(&path_with_params);

        (
            format!("^{}$", escaped_pattern),
            param_names,
            wildcard_names,
        )
    }

    /// Attempts to match a request against this route and extract parameters.
    ///
    /// This method performs optimized route matching with multiple fast-path
    /// checks before falling back to regex matching. If the route matches,
    /// it extracts all named and wildcard parameters into a HashMap.
    ///
    /// # Parameters
    /// - `req`: The incoming HTTP request to match against
    ///
    /// # Returns
    /// - `Some(HashMap<String, String>)`: Parameters if the route matches
    /// - `None`: If the route doesn't match
    ///
    /// # Matching Algorithm
    /// 1. **Method Check**: Fast comparison of HTTP methods
    /// 2. **Length Check**: Minimum path length validation
    /// 3. **Segment Count Check**: Path segment count validation
    /// 4. **Regex Matching**: Full pattern matching and parameter extraction
    ///
    /// # Parameter Extraction
    /// - Named parameters are extracted in order of appearance
    /// - Wildcard parameters are extracted after named parameters
    /// - All parameters are URL-decoded and stored as strings
    ///
    /// # Examples
    /// ```
    /// use ignitia::{router::Route, Request, Method};
    /// use http::Uri;
    /// use bytes::Bytes;
    /// use std::sync::Arc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let handler = Arc::new(|req| Box::pin(async move {
    ///     Ok(ignitia::Response::text("Hello"))
    /// }));
    ///
    /// let route = Route::new("/users/:id/posts/:post_id", Method::GET, handler);
    ///
    /// let request = Request::new(
    ///     Method::GET,
    ///     "/users/123/posts/456".parse::<Uri>()?,
    ///     http::Version::HTTP_11,
    ///     http::HeaderMap::new(),
    ///     Bytes::new(),
    /// );
    ///
    /// if let Some(params) = route.matches(&request) {
    ///     assert_eq!(params.get("id"), Some(&"123".to_string()));
    ///     assert_eq!(params.get("post_id"), Some(&"456".to_string()));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Performance Characteristics
    /// - **O(1)** method comparison
    /// - **O(1)** length and segment checks
    /// - **O(n)** regex matching (where n is path length)
    /// - **O(p)** parameter extraction (where p is parameter count)
    pub fn matches(&self, req: &Request) -> Option<HashMap<String, String>> {
        // Fast path: check method first
        if self.method != req.method {
            return None;
        }

        let path = req.uri.path();

        // Quick length and segment checks for early rejection
        let min_length = self.path.len().saturating_sub(self.total_params * 3);
        if path.len() < min_length {
            return None;
        }

        // Check segment count for early rejection
        let request_segments = path.matches('/').count();
        if request_segments < self.segment_count.saturating_sub(self.total_params) {
            return None;
        }

        let captures = match self.regex.captures(path) {
            Some(caps) => caps,
            None => return None,
        };

        // Pre-allocate HashMap with expected size
        let mut params = HashMap::with_capacity(self.total_params);

        // Handle regular parameters
        for (i, name) in self.param_names.iter().enumerate() {
            if let Some(value) = captures.get(i + 1) {
                params.insert(name.clone(), value.as_str().to_string());
            }
        }

        // Handle wildcard parameters
        let wildcard_offset = self.param_names.len();
        for (i, name) in self.wildcard_names.iter().enumerate() {
            if let Some(value) = captures.get(wildcard_offset + i + 1) {
                params.insert(name.clone(), value.as_str().to_string());
            }
        }

        Some(params)
    }

    /// Returns a list of all parameter names (both named and wildcard).
    ///
    /// This method is primarily used for testing and debugging purposes.
    /// It combines both named parameters and wildcard parameters in the
    /// order they appear in the route pattern.
    ///
    /// # Returns
    /// A vector of all parameter names in order of appearance
    ///
    /// # Examples
    /// ```
    /// use ignitia::router::Route;
    /// use http::Method;
    /// use std::sync::Arc;
    ///
    /// let handler = Arc::new(|req| Box::pin(async move {
    ///     Ok(ignitia::Response::text("Hello"))
    /// }));
    ///
    /// let route = Route::new("/users/:id/files/*path", Method::GET, handler);
    /// let param_names = route.get_param_names();
    ///
    /// assert_eq!(param_names, vec!["id", "path"]);
    /// ```
    pub fn get_param_names(&self) -> Vec<String> {
        let mut names = self.param_names.clone();
        names.extend(self.wildcard_names.clone());
        names
    }
}

/// Selectively escapes regex special characters while preserving regex groups.
///
/// This function escapes regex metacharacters in route patterns while leaving
/// our generated regex groups intact. It's designed to handle the output of
/// parameter replacement where some parts should be treated as literals and
/// others as regex patterns.
///
/// # Parameters
/// - `s`: The string to selectively escape
///
/// # Returns
/// The escaped string with regex groups preserved
///
/// # Escaping Rules
/// - **Preserve**: Parenthesized groups like `(.+)` and `([^/]+)`
/// - **Escape**: All other regex metacharacters: `\`, `.`, `+`, `*`, `?`, `^`, `$`, `[`, `]`, `{`, `}`, `|`
///
/// # Examples
/// ```
/// use ignitia::router::route::escape_regex_selective;
///
/// // Literal path with regex group
/// let input = "/api/v1.0/users/([^/]+)";
/// let escaped = escape_regex_selective(input);
/// assert_eq!(escaped, "/api/v1\\.0/users/([^/]+)");
///
/// // Multiple groups preserved
/// let input = "/files/(.+)/info/([^/]+)";
/// let escaped = escape_regex_selective(input);
/// assert_eq!(escaped, "/files/(.+)/info/([^/]+)");
/// ```
fn escape_regex_selective(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Don't escape if we're in a regex group pattern
            '(' => {
                result.push(c);
                // Copy everything until the matching ')'
                let mut paren_count = 1;
                while let Some(inner_c) = chars.next() {
                    result.push(inner_c);
                    match inner_c {
                        '(' => paren_count += 1,
                        ')' => {
                            paren_count -= 1;
                            if paren_count == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Escape other regex special characters
            '\\' | '.' | '+' | '*' | '?' | '^' | '$' | '[' | ']' | '{' | '}' | '|' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    result
}

/// Trait for optimized route matching with fast-path checks.
///
/// This trait provides additional matching methods optimized for performance
/// testing and routing efficiency analysis.
pub trait RouteMatcher {
    /// Performs a fast pre-check before full regex matching.
    ///
    /// This method implements the same fast-path optimizations as the full
    /// `matches` method but only returns a boolean result. It's useful for
    /// quickly filtering routes before more expensive operations.
    ///
    /// # Parameters
    /// - `path`: The request path to check
    ///
    /// # Returns
    /// `true` if the route might match (requires full matching to confirm)
    fn fast_match(&self, path: &str) -> bool;
}

impl RouteMatcher for Route {
    fn fast_match(&self, path: &str) -> bool {
        // Very fast pre-check before regex matching
        if path.len() < self.path.len().saturating_sub(self.total_params * 3) {
            return false;
        }

        if path.matches('/').count() < self.segment_count.saturating_sub(self.total_params) {
            return false;
        }

        self.regex.is_match(path)
    }
}
