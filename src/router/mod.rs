//! # HTTP Router Module
//!
//! This module provides the core routing functionality for the Ignitia web framework. It includes
//! a high-performance router with route compilation, middleware support, nested routing capabilities,
//! and optional WebSocket integration. The router is designed for maximum performance with features
//! like route pre-compilation, atomic updates, and efficient path matching.
//!
//! ## Features
//!
//! - **High-Performance Routing**: Pre-compiled routes with regex optimization and fast path matching
//! - **Middleware Support**: Chain middleware with before/after execution hooks
//! - **Nested Routing**: Hierarchical route organization with path prefixing
//! - **WebSocket Integration**: Optional WebSocket support with dedicated handlers
//! - **Thread-Safe Operations**: Lock-free reads with atomic route compilation updates
//! - **Flexible Handler Types**: Support for various handler signatures with automatic extraction
//! - **Route Parameters**: Named parameters (`:id`) and wildcards (`*path`) with type-safe extraction
//!
//! ## Architecture
//!
//! ### Router Design
//! The router uses a two-stage architecture:
//! 1. **Build Stage**: Routes are added using a builder pattern
//! 2. **Compilation Stage**: Routes are compiled into an optimized structure for fast matching
//!
//! ### Performance Features
//! - **Atomic Compilation**: Routes are compiled atomically using `ArcSwap` for lock-free reads
//! - **Route Sorting**: Routes are sorted by specificity for optimal matching order
//! - **Method-Based Grouping**: Routes are grouped by HTTP method for faster lookup
//! - **Regex Caching**: Pre-compiled regex patterns with size limits
//!
//! ## Usage Examples
//!
//! ### Basic Router Setup
//! ```
//! use ignitia::{Router, Response, Result};
//! use http::Method;
//!
//! let router = Router::new()
//!     .get("/", || async {
//!         Ok(Response::text("Hello, World!"))
//!     })
//!     .post("/users", |body: String| async move {
//!         Ok(Response::json(serde_json::json!({
//!             "message": "User created",
//!             "body": body
//!         }))?)
//!     })
//!     .get("/users/:id", |path: ignitia::Path<u32>| async move {
//!         Ok(Response::json(serde_json::json!({
//!             "user_id": path.0
//!         }))?)
//!     });
//! ```
//!
//! ### Advanced Route Patterns
//! ```
//! use ignitia::{Router, Response, Result, Path, Query};
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserParams {
//!     id: u32,
//! }
//!
//! #[derive(Deserialize)]
//! struct QueryParams {
//!     page: Option<u32>,
//!     limit: Option<u32>,
//! }
//!
//! let router = Router::new()
//!     // Named parameters
//!     .get("/users/:id", |path: Path<UserParams>| async move {
//!         Ok(Response::json(serde_json::json!({
//!             "user_id": path.id
//!         }))?)
//!     })
//!     // Multiple parameters
//!     .get("/users/:user_id/posts/:post_id",
//!         |path: Path<serde_json::Value>| async move {
//!             Ok(Response::json(path.into_inner())?)
//!         })
//!     // Wildcard parameters
//!     .get("/files/*path", |path: Path<String>| async move {
//!         Ok(Response::text(format!("File path: {}", path.0)))
//!     })
//!     // Query parameters
//!     .get("/search", |query: Query<QueryParams>| async move {
//!         let page = query.page.unwrap_or(1);
//!         let limit = query.limit.unwrap_or(10);
//!         Ok(Response::json(serde_json::json!({
//!             "page": page,
//!             "limit": limit
//!         }))?)
//!     });
//! ```
//!
//! ### Middleware Integration
//! ```
//! use ignitia::{Router, Response, Result, LoggerMiddleware, CorsMiddleware, AuthMiddleware};
//!
//! let router = Router::new()
//!     // Global middleware
//!     .middleware(LoggerMiddleware)
//!     .middleware(CorsMiddleware::new().allow_origin("https://example.com"))
//!     .middleware(AuthMiddleware::new("secret-token")
//!         .protect_path("/admin")
//!         .protect_path("/api/private"))
//!     // Routes
//!     .get("/public", || async { Ok(Response::text("Public endpoint")) })
//!     .get("/admin", || async { Ok(Response::text("Admin endpoint")) })
//!     .get("/api/private", || async { Ok(Response::text("Private API")) });
//! ```
//!
//! ### Nested Routing
//! ```
//! use ignitia::{Router, Response, Result};
//!
//! // API v1 routes
//! let api_v1 = Router::new()
//!     .get("/users", || async { Ok(Response::text("API v1 users")) })
//!     .get("/posts", || async { Ok(Response::text("API v1 posts")) });
//!
//! // API v2 routes
//! let api_v2 = Router::new()
//!     .get("/users", || async { Ok(Response::text("API v2 users")) })
//!     .get("/posts", || async { Ok(Response::text("API v2 posts")) })
//!     .get("/comments", || async { Ok(Response::text("API v2 comments")) });
//!
//! // Main router with nested routes
//! let router = Router::new()
//!     .get("/", || async { Ok(Response::text("Home")) })
//!     .nest("/api/v1", api_v1)
//!     .nest("/api/v2", api_v2);
//!
//! // This creates routes:
//! // GET /
//! // GET /api/v1/users
//! // GET /api/v1/posts
//! // GET /api/v2/users
//! // GET /api/v2/posts
//! // GET /api/v2/comments
//! ```
//!
//! ### WebSocket Support
//! ```
//! #[cfg(feature = "websocket")]
//! use ignitia::{Router, websocket::WebSocketConnection, Result};
//!
//! #[cfg(feature = "websocket")]
//! let router = Router::new()
//!     .get("/", || async { Ok(ignitia::Response::text("WebSocket Demo")) })
//!     .websocket("/ws", |mut ws: WebSocketConnection| async move {
//!         while let Some(msg) = ws.recv().await {
//!             match msg {
//!                 ignitia::websocket::Message::Text(text) => {
//!                     ws.send_text(format!("Echo: {}", text)).await?;
//!                 }
//!                 ignitia::websocket::Message::Close(_) => break,
//!                 _ => {}
//!             }
//!         }
//!         Ok(())
//!     })
//!     .websocket_fn("/chat", |ws| async move {
//!         // Handle chat WebSocket connection
//!         Ok(())
//!     });
//! ```
//!
//! ## Advanced Features
//!
//! ### Custom Error Handling
//! ```
//! use ignitia::{Router, Response, Result, Error};
//!
//! let router = Router::new()
//!     .get("/users/:id", |path: ignitia::Path<String>| async move {
//!         let id = path.0.parse::<u32>()
//!             .map_err(|_| Error::BadRequest("Invalid user ID".into()))?;
//!
//!         if id == 0 {
//!             return Err(Error::NotFound("User not found".into()));
//!         }
//!
//!         Ok(Response::json(serde_json::json!({
//!             "user_id": id
//!         }))?)
//!     })
//!     .not_found(|| async {
//!         Ok(Response::json(serde_json::json!({
//!             "error": "Endpoint not found",
//!             "suggestion": "Check the API documentation"
//!         }))?.with_status_code(404))
//!     });
//! ```
//!
//! ### Route Groups and Organization
//! ```
//! use ignitia::{Router, Response, Result};
//!
//! fn create_user_routes() -> Router {
//!     Router::new()
//!         .get("/", || async { Ok(Response::text("List users")) })
//!         .post("/", || async { Ok(Response::text("Create user")) })
//!         .get("/:id", || async { Ok(Response::text("Get user")) })
//!         .put("/:id", || async { Ok(Response::text("Update user")) })
//!         .delete("/:id", || async { Ok(Response::text("Delete user")) })
//! }
//!
//! fn create_post_routes() -> Router {
//!     Router::new()
//!         .get("/", || async { Ok(Response::text("List posts")) })
//!         .post("/", || async { Ok(Response::text("Create post")) })
//!         .get("/:id", || async { Ok(Response::text("Get post")) })
//!         .put("/:id", || async { Ok(Response::text("Update post")) })
//!         .delete("/:id", || async { Ok(Response::text("Delete post")) })
//! }
//!
//! let router = Router::new()
//!     .nest("/users", create_user_routes())
//!     .nest("/posts", create_post_routes());
//! ```
//!
//! ## Performance Considerations
//!
//! ### Route Compilation
//! Routes are compiled automatically when first accessed and cached for subsequent requests:
//! - **Lazy Compilation**: Routes are compiled only when needed
//! - **Atomic Updates**: Compilation is atomic and doesn't block request handling
//! - **Memory Efficiency**: Compiled routes are shared across all requests
//!
//! ### Route Ordering
//! Routes are automatically sorted by specificity for optimal matching:
//! ```
//! let router = Router::new()
//!     // These routes will be automatically ordered for best performance:
//!     .get("/*path", || async { Ok(Response::text("Catch all")) })        // Last
//!     .get("/users/:id", || async { Ok(Response::text("User by ID")) })   // Second
//!     .get("/users/me", || async { Ok(Response::text("Current user")) }); // First
//! ```
//!
//! ### Memory Usage
//! - Routes are stored efficiently with minimal memory overhead
//! - Regex patterns have size limits to prevent memory exhaustion
//! - Middleware is shared across routes to minimize duplication
//!
//! ## Testing and Debugging
//!
//! ### Route Testing
//! ```
//! use ignitia::{Router, Method};
//!
//! #[tokio::test]
//! async fn test_router() {
//!     let router = Router::new()
//!         .get("/users/:id", || async { Ok(ignitia::Response::text("User")) });
//!
//!     // Test route matching
//!     assert!(router.matches(&Method::GET, "/users/123"));
//!     assert!(!router.matches(&Method::GET, "/users"));
//!     assert!(!router.matches(&Method::POST, "/users/123"));
//! }
//! ```
//!
//! ### Debug Information
//! ```
//! use ignitia::Router;
//!
//! let router = Router::new()
//!     .get("/users/:id", || async { Ok(ignitia::Response::text("User")) })
//!     .post("/users", || async { Ok(ignitia::Response::text("Create")) });
//!
//! // In debug mode, router provides introspection capabilities
//! #[cfg(debug_assertions)]
//! {
//!     println!("Router has routes for GET and POST methods");
//!     // Additional debug information available in debug builds
//! }
//! ```

pub mod method;
pub mod route;

use crate::handler::{into_handler, IntoHandler};
use crate::middleware::Middleware;
use crate::{Error, Extensions, Handler, HandlerFn, Request, Response, Result};
use arc_swap::ArcSwap;
use http::Method;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub use route::Route;

macro_rules! define_http_method {
    ($name:ident, $method:expr, $doc:expr) => {
        #[doc = $doc]
        pub fn $name<H, T>(self, path: &str, handler: H) -> Self
        where
            H: IntoHandler<T>,
        {
            self.route_with(path, $method, handler)
        }
    };
}

/// A handler with associated middleware layers.
///
/// `LayeredHandler` allows you to attach middleware specifically to individual
/// handlers before they're registered with the router. This provides fine-grained
/// control over which middleware applies to which routes.
///
/// # Examples
///
/// ```
/// use ignitia::{LayeredHandler, Response, AuthMiddleware, RateLimitMiddleware};
///
/// let handler = LayeredHandler::new(|| async { Ok(Response::text("Protected")) })
///     .layer(AuthMiddleware::new("secret"))
///     .layer(RateLimitMiddleware::new(100));
/// ```
#[derive(Clone)]
pub struct LayeredHandler {
    /// The core handler function to execute
    handler: HandlerFn,
    /// Stack of middleware to apply to this handler
    middleware: Vec<Arc<dyn Middleware>>,
}

impl LayeredHandler {
    /// Creates a new layered handler from a handler that implements `IntoHandler`.
    ///
    /// # Type Parameters
    /// - `H`: Handler type that implements `IntoHandler`
    ///
    /// # Parameters
    /// - `handler`: The handler to wrap
    ///
    /// # Returns
    /// A new `LayeredHandler` with no middleware layers
    ///
    /// # Examples
    /// ```
    /// use ignitia::{LayeredHandler, Response};
    ///
    /// let handler = LayeredHandler::new(|| async {
    ///     Ok(Response::text("Hello, World!"))
    /// });
    /// ```
    pub fn new<H, T>(handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        Self {
            handler: into_handler(handler),
            middleware: Vec::new(),
        }
    }

    /// Adds a middleware layer to this handler.
    ///
    /// Middleware is applied in the order it's added. The first middleware added
    /// will be the outermost layer (executed first for `before` hooks and last
    /// for `after` hooks).
    ///
    /// # Type Parameters
    /// - `M`: Middleware type that implements the `Middleware` trait
    ///
    /// # Parameters
    /// - `mw`: The middleware to add
    ///
    /// # Returns
    /// The layered handler for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::{LayeredHandler, Response, LoggerMiddleware, AuthMiddleware};
    ///
    /// let handler = LayeredHandler::new(|| async { Ok(Response::text("Data")) })
    ///     .layer(LoggerMiddleware)
    ///     .layer(AuthMiddleware::new("secret"));
    /// ```
    pub fn layer<M: Middleware + 'static>(mut self, mw: M) -> Self {
        self.middleware.push(Arc::new(mw));
        self
    }

    /// Converts the layered handler into a `HandlerFn` that can be registered with the router.
    ///
    /// This method creates a new handler function that wraps the original handler
    /// with all the configured middleware layers. The middleware is executed in
    /// the correct order (FIFO for `before` hooks, LIFO for `after` hooks).
    ///
    /// # Returns
    /// A `HandlerFn` that executes the handler with its middleware stack
    ///
    /// # Middleware Execution Order
    /// - `before` hooks: Execute in the order middleware was added
    /// - `after` hooks: Execute in reverse order (LIFO)
    ///
    /// # Examples
    /// ```
    /// use ignitia::{LayeredHandler, Router, Response};
    ///
    /// let layered = LayeredHandler::new(|| async { Ok(Response::text("Test")) });
    /// let handler_fn = layered.into_handler();
    ///
    /// let router = Router::new().route("/test", http::Method::GET, handler_fn);
    /// ```
    pub fn into_handler(self) -> HandlerFn {
        let handler = self.handler;
        let middleware = self.middleware;

        Arc::new(move |mut req: Request| {
            let middleware = middleware.clone();
            let handler = handler.clone();

            Box::pin(async move {
                // Run middleware.before() in order
                for mw in &middleware {
                    mw.before(&mut req).await?;
                }

                let mut res = handler.handle(req.clone()).await?;

                // Run middleware.after() in reverse order
                for mw in middleware.iter().rev() {
                    mw.after(&req, &mut res).await?;
                }

                Ok(res)
            })
        })
    }
}

/// Compiled router state for efficient request handling.
///
/// This structure contains the optimized representation of all routes,
/// middleware, and handlers after compilation. It's designed for fast
/// read access during request processing.
#[derive(Clone)]
struct CompiledRouter {
    /// Routes organized by HTTP method for fast lookup
    routes: HashMap<Method, Vec<Route>>,
    /// Middleware stack to apply to requests
    middleware: Vec<Arc<dyn Middleware>>,
    /// Optional custom 404 handler
    not_found_handler: Option<HandlerFn>,
}

/// High-performance HTTP router with middleware support and route compilation.
///
/// The `Router` provides a builder-pattern API for defining routes and middleware.
/// It uses lazy compilation to optimize route matching performance and supports
/// advanced features like nested routing and WebSocket integration.
///
/// # Thread Safety
/// The router is thread-safe and can be shared across multiple threads. Route
/// compilation is performed atomically using `ArcSwap` to ensure consistent
/// state without blocking request handling.
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use ignitia::{Router, Response};
///
/// let router = Router::new()
///     .get("/", || async { Ok(Response::text("Home")) })
///     .get("/about", || async { Ok(Response::text("About")) })
///     .post("/users", || async { Ok(Response::text("Create User")) });
/// ```
///
/// ## With Middleware
/// ```
/// use ignitia::{Router, Response, LoggerMiddleware};
///
/// let router = Router::new()
///     .middleware(LoggerMiddleware)
///     .get("/api/health", || async { Ok(Response::text("OK")) });
/// ```
pub struct Router {
    /// Internal router state protected by RwLock
    inner: Arc<RwLock<RouterInner>>,
    /// Atomically updated compiled router for fast read access
    compiled: ArcSwap<CompiledRouter>,
}

/// Internal router state that can be modified during route building.
struct RouterInner {
    /// Routes organized by HTTP method
    routes: HashMap<Method, Vec<Route>>,
    /// Middleware stack
    middleware: Vec<Arc<dyn Middleware>>,
    /// Custom 404 handler
    not_found_handler: Option<HandlerFn>,
    /// Nested routers with their path prefixes
    nested_routers: Vec<(String, Router)>,
    /// Flag indicating if router needs recompilation
    dirty: bool,
    /// Extensions for state management
    extensions: Extensions,
    /// WebSocket route handlers (when feature is enabled)
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    websocket_routes: HashMap<String, Arc<dyn crate::websocket::WebSocketHandler>>,
}

impl Router {
    /// Creates a new empty router.
    ///
    /// # Returns
    /// A new `Router` instance ready for route configuration
    ///
    /// # Examples
    /// ```
    /// use ignitia::Router;
    ///
    /// let router = Router::new();
    /// ```
    pub fn new() -> Self {
        let inner = RouterInner {
            routes: HashMap::new(),
            middleware: Vec::new(),
            not_found_handler: None,
            nested_routers: Vec::new(),
            extensions: Extensions::new(),
            dirty: true,
            #[cfg(feature = "websocket")]
            #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
            websocket_routes: HashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
            compiled: ArcSwap::new(Arc::new(CompiledRouter {
                routes: HashMap::new(),
                middleware: Vec::new(),
                not_found_handler: None,
            })),
        }
    }

    /// Adds a route with a specific HTTP method and handler.
    ///
    /// This is the low-level route registration method. Most users should prefer
    /// the convenience methods like `get()`, `post()`, etc.
    ///
    /// # Parameters
    /// - `path`: The route path pattern (e.g., "/users/:id")
    /// - `method`: The HTTP method for this route
    /// - `handler`: The handler function to execute for matching requests
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Path Patterns
    /// - Static paths: `/users`, `/api/health`
    /// - Named parameters: `/users/:id`, `/posts/:slug`
    /// - Wildcards: `/files/*path`
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Response, Method, handler_fn};
    ///
    /// let handler = handler_fn(|req| async move {
    ///     Ok(Response::text("Custom handler"))
    /// });
    ///
    /// let router = Router::new()
    ///     .route("/custom", Method::GET, handler);
    /// ```
    pub fn route(self, path: &str, method: Method, handler: HandlerFn) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;

            let full_path = normalize_path(path);
            let routes = inner.routes.entry(method.clone()).or_insert_with(Vec::new);

            // Pre-compile the route for better performance
            let route = Route::new(&full_path, method, handler);
            routes.push(route);
        }

        self
    }

    /// Adds a route with automatic handler conversion.
    ///
    /// This method automatically converts handlers that implement `IntoHandler`
    /// into the appropriate handler function format.
    ///
    /// # Type Parameters
    /// - `H`: Handler type that implements `IntoHandler<T>`
    /// - `T`: Handler signature marker type
    ///
    /// # Parameters
    /// - `path`: The route path pattern
    /// - `method`: The HTTP method for this route
    /// - `handler`: The handler to convert and register
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Response, Method, Path};
    ///
    /// let router = Router::new()
    ///     .route_with("/users/:id", Method::GET, |path: Path<u32>| async move {
    ///         Ok(Response::text(format!("User ID: {}", path.0)))
    ///     });
    /// ```
    pub fn route_with<H, T>(self, path: &str, method: Method, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        self.route(path, method, into_handler(handler))
    }

    /// Adds a route with a pre-layered handler.
    ///
    /// This method allows you to register a `LayeredHandler` that already has
    /// middleware attached. The layered handler will be converted to a regular
    /// handler function that includes the middleware execution.
    ///
    /// # Parameters
    /// - `path`: The route path pattern
    /// - `method`: The HTTP method for this route
    /// - `lh`: The layered handler with its middleware stack
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Response, LayeredHandler, AuthMiddleware};
    ///
    /// let layered = LayeredHandler::new(|| async { Ok(Response::text("Protected")) })
    ///     .layer(AuthMiddleware::new("secret"));
    ///
    /// let router = Router::new()
    ///     .route_with_layered("/admin", http::Method::GET, layered);
    /// ```
    pub fn route_with_layered(self, path: &str, method: http::Method, lh: LayeredHandler) -> Self {
        self.route(path, method, lh.into_handler())
    }

    // HTTP method convenience functions

    define_http_method!(
        get,
        Method::GET,
        "Adds a GET route.\n\n\
             # Parameters\n\
             - `path`: The route path pattern\n\
             - `handler`: The handler function for GET requests\n\n\
             # Examples\n\
             ```\n\
             use ignitia::{Router, Response};\n\n\
             let router = Router::new()\n\
                 .get(\"/\", || async { Ok(Response::text(\"Home\")) })\n\
                 .get(\"/users/:id\", |path: ignitia::Path<u32>| async move {\n\
                     Ok(Response::text(format!(\"User {}\", path.0)))\n\
                 });\n\
             ```"
    );

    define_http_method!(
        post,
        Method::POST,
        "Adds a POST route.\n\n\
             # Parameters\n\
             - `path`: The route path pattern\n\
             - `handler`: The handler function for POST requests\n\n\
             # Examples\n\
             ```\n\
             use ignitia::{Router, Response, Json};\n\
             use serde::Deserialize;\n\n\
             #[derive(Deserialize)]\n\
             struct CreateUser {\n\
                 name: String,\n\
                 email: String,\n\
             }\n\n\
             let router = Router::new()\n\
                 .post(\"/users\", |Json(user): Json<CreateUser>| async move {\n\
                     Ok(Response::json(serde_json::json!({\n\
                         \"message\": \"User created\",\n\
                         \"name\": user.name\n\
                     }))?)\n\
                 });\n\
             ```"
    );

    define_http_method!(
        put,
        Method::PUT,
        "Adds a PUT route.\n\n\
             # Parameters\n\
             - `path`: The route path pattern\n\
             - `handler`: The handler function for PUT requests"
    );

    define_http_method!(
        delete,
        Method::DELETE,
        "Adds a DELETE route.\n\n\
             # Parameters\n\
             - `path`: The route path pattern\n\
             - `handler`: The handler function for DELETE requests"
    );

    define_http_method!(
        patch,
        Method::PATCH,
        "Adds a PATCH route.\n\n\
             # Parameters\n\
             - `path`: The route path pattern\n\
             - `handler`: The handler function for PATCH requests"
    );

    define_http_method!(
        head,
        Method::HEAD,
        "Adds a HEAD route.\n\n\
             # Parameters\n\
             - `path`: The route path pattern\n\
             - `handler`: The handler function for HEAD requests"
    );

    define_http_method!(
        options,
        Method::OPTIONS,
        "Adds an OPTIONS route.\n\n\
             # Parameters\n\
             - `path`: The route path pattern\n\
             - `handler`: The handler function for OPTIONS requests"
    );

    /// Adds middleware to the router.
    ///
    /// Middleware is executed in the order it's added. Each middleware can
    /// process requests before they reach handlers and responses after
    /// handlers return.
    ///
    /// # Type Parameters
    /// - `M`: Middleware type that implements the `Middleware` trait
    ///
    /// # Parameters
    /// - `middleware`: The middleware instance to add
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Response, LoggerMiddleware, CorsMiddleware};
    ///
    /// let router = Router::new()
    ///     .middleware(LoggerMiddleware)
    ///     .middleware(CorsMiddleware::new().allow_origin("*"))
    ///     .get("/api/data", || async { Ok(Response::text("Data")) });
    /// ```
    pub fn middleware<M: Middleware + 'static>(self, middleware: M) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.middleware.push(Arc::new(middleware));
        }
        self
    }

    /// Sets a custom 404 (Not Found) handler.
    ///
    /// This handler will be called when no routes match the incoming request.
    /// If not set, the router will return a default 404 error.
    ///
    /// # Type Parameters
    /// - `H`: Handler type that implements `IntoHandler<T>`
    /// - `T`: Handler signature marker type
    ///
    /// # Parameters
    /// - `handler`: The handler to execute for unmatched requests
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Response};
    ///
    /// let router = Router::new()
    ///     .get("/", || async { Ok(Response::text("Home")) })
    ///     .not_found(|| async {
    ///         Ok(Response::html(r#"
    ///             <h1>Page Not Found</h1>
    ///             <p>The requested page could not be found.</p>
    ///         "#).with_status_code(404))
    ///     });
    /// ```
    pub fn not_found<H, T>(self, handler: H) -> Self
    where
        H: IntoHandler<T>,
    {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.not_found_handler = Some(into_handler(handler));
        }
        self
    }

    /// Nests another router under a path prefix.
    ///
    /// This allows for modular route organization by grouping related routes
    /// under a common prefix. The nested router's routes will be prefixed
    /// with the specified path.
    ///
    /// # Parameters
    /// - `path`: The path prefix for the nested router
    /// - `router`: The router to nest under the prefix
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Response};
    ///
    /// // Create API v1 routes
    /// let api_v1 = Router::new()
    ///     .get("/users", || async { Ok(Response::text("API v1 users")) })
    ///     .get("/posts", || async { Ok(Response::text("API v1 posts")) });
    ///
    /// // Create API v2 routes
    /// let api_v2 = Router::new()
    ///     .get("/users", || async { Ok(Response::text("API v2 users")) })
    ///     .get("/posts", || async { Ok(Response::text("API v2 posts")) });
    ///
    /// // Nest both API versions
    /// let router = Router::new()
    ///     .get("/", || async { Ok(Response::text("Home")) })
    ///     .nest("/api/v1", api_v1)
    ///     .nest("/api/v2", api_v2);
    /// ```
    pub fn nest(self, path: &str, router: Router) -> Self {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.nested_routers.push((normalize_path(path), router));
        }
        self
    }

    /// Adds a WebSocket route (requires "websocket" feature).
    ///
    /// This method registers a WebSocket handler for the specified path.
    /// When a WebSocket upgrade request is received for this path, the
    /// handler will be invoked to manage the WebSocket connection.
    ///
    /// # Type Parameters
    /// - `H`: WebSocket handler type that implements `WebSocketHandler`
    ///
    /// # Parameters
    /// - `path`: The route path for the WebSocket endpoint
    /// - `handler`: The WebSocket handler instance
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Examples
    /// ```
    /// #[cfg(feature = "websocket")]
    /// use ignitia::{Router, websocket::WebSocketConnection};
    ///
    /// #[cfg(feature = "websocket")]
    /// let router = Router::new()
    ///     .websocket("/ws", |mut ws: WebSocketConnection| async move {
    ///         while let Some(msg) = ws.recv().await {
    ///             match msg {
    ///                 ignitia::websocket::Message::Text(text) => {
    ///                     ws.send_text(format!("Echo: {}", text)).await?;
    ///                 }
    ///                 ignitia::websocket::Message::Close(_) => break,
    ///                 _ => {}
    ///             }
    ///         }
    ///         Ok(())
    ///     });
    /// ```
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub fn websocket<H>(self, path: &str, handler: H) -> Self
    where
        H: crate::websocket::WebSocketHandler + 'static,
    {
        let normalized_path = normalize_path(path);
        tracing::debug!("Storing WebSocket handler for path: {}", normalized_path);
        let ws_handler: Arc<dyn crate::websocket::WebSocketHandler> = Arc::new(handler);

        // Store the WebSocket handler
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner
                .websocket_routes
                .insert(normalized_path.clone(), Arc::clone(&ws_handler));
        }

        // Create a regular HTTP handler that handles WebSocket upgrades
        let http_handler = Arc::new(move |req: Request| {
            Box::pin(async move {
                if crate::websocket::is_websocket_request(&req) {
                    crate::websocket::upgrade_connection(req)
                } else {
                    Err(crate::Error::BadRequest(
                        "This endpoint only accepts WebSocket connections".into(),
                    ))
                }
            }) as crate::handler::BoxFuture<'static, crate::Result<Response>>
        });

        self.route(&normalized_path, Method::GET, http_handler)
    }

    /// Adds a WebSocket route with a closure handler (requires "websocket" feature).
    ///
    /// This is a convenience method for adding WebSocket routes using closures
    /// instead of implementing the `WebSocketHandler` trait.
    ///
    /// # Type Parameters
    /// - `F`: Closure type that takes a WebSocketConnection
    /// - `Fut`: Future type returned by the closure
    ///
    /// # Parameters
    /// - `path`: The route path for the WebSocket endpoint
    /// - `f`: The closure to handle WebSocket connections
    ///
    /// # Returns
    /// The router instance for method chaining
    ///
    /// # Examples
    /// ```
    /// #[cfg(feature = "websocket")]
    /// use ignitia::Router;
    ///
    /// #[cfg(feature = "websocket")]
    /// let router = Router::new()
    ///     .websocket_fn("/echo", |ws| async move {
    ///         // Handle WebSocket connection
    ///         Ok(())
    ///     });
    /// ```
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub fn websocket_fn<F, Fut>(self, path: &str, f: F) -> Self
    where
        F: Fn(crate::websocket::WebSocketConnection) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = crate::Result<()>> + Send + 'static,
    {
        use crate::websocket::websocket_handler;
        self.websocket(path, websocket_handler(f))
    }

    /// Gets all registered WebSocket handlers (requires "websocket" feature).
    ///
    /// This method returns a map of path patterns to their corresponding
    /// WebSocket handlers. It's primarily used internally by the server
    /// for WebSocket upgrade handling.
    ///
    /// # Returns
    /// A HashMap mapping paths to WebSocket handlers
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub fn get_websocket_handlers(
        &self,
    ) -> HashMap<String, Arc<dyn crate::websocket::WebSocketHandler>> {
        let inner = self.inner.read();
        inner.websocket_routes.clone()
    }

    /// WebSocket route method when WebSocket feature is disabled.
    ///
    /// This method will panic if called when the "websocket" feature is not enabled.
    #[cfg(not(feature = "websocket"))]
    pub fn websocket<H>(self, _path: &str, _handler: H) -> Self {
        panic!("WebSocket support is not enabled. Add 'websocket' feature to your Cargo.toml");
    }

    /// WebSocket function route method when WebSocket feature is disabled.
    ///
    /// This method will panic if called when the "websocket" feature is not enabled.
    #[cfg(not(feature = "websocket"))]
    pub fn websocket_fn<F>(self, _path: &str, _f: F) -> Self {
        panic!("WebSocket support is not enabled. Add 'websocket' feature to your Cargo.toml");
    }

    /// Add application state that will be available to all handlers in this router.
    ///
    /// The state can be extracted in handlers using the `State<T>` extractor.
    /// State is shared efficiently using `Arc<T>` internally.
    ///
    /// # Requirements
    ///
    /// The state type must implement `Clone + Send + Sync + 'static`.
    ///
    /// # Example
    ///
    /// ```
    /// #[derive(Clone)]
    /// struct AppState {
    ///     db: DatabasePool,
    ///     config: Config,
    /// }
    ///
    /// let state = AppState {
    ///     db: create_pool(),
    ///     config: load_config(),
    /// };
    ///
    /// let app = Router::new()
    ///     .route("/users", get_users)
    ///     .state(state);
    /// ```
    ///
    /// # Multiple States
    ///
    /// You can add multiple different state types:
    ///
    /// ```
    /// let app = Router::new()
    ///     .state(database_pool)
    ///     .state(redis_client)
    ///     .state(app_config);
    /// ```
    pub fn state<T>(self, state: T) -> Self
    where
        T: Clone + Send + Sync + 'static,
    {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.extensions.insert(state);
        }
        self
    }

    /// Add application state using an `Arc<T>` for maximum efficiency.
    ///
    /// This method is preferred when you already have your state wrapped in `Arc<T>`
    /// or when you want to share the exact same state instance across multiple routers.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let shared_state = Arc::new(AppState::new());
    ///
    /// let api_router = Router::new()
    ///     .route("/v1/users", get_users)
    ///     .state_arc(shared_state.clone());
    ///
    /// let admin_router = Router::new()
    ///     .route("/admin/stats", get_stats)
    ///     .state_arc(shared_state.clone());
    /// ```
    pub fn state_arc<T>(self, state: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.extensions.insert(state);
        }
        self
    }

    /// Add state using a factory function that will be called once during router setup.
    ///
    /// Useful for state that requires async initialization or depends on other router configuration.
    ///
    /// # Example
    ///
    /// ```
    /// let app = Router::new()
    ///     .route("/data", get_data)
    ///     .state_factory(|| {
    ///         AppState {
    ///             created_at: std::time::SystemTime::now(),
    ///             id: uuid::Uuid::new_v4(),
    ///         }
    ///     });
    /// ```
    pub fn state_factory<T, F>(self, factory: F) -> Self
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let state = factory();
        {
            let mut inner = self.inner.write();
            inner.dirty = true;
            inner.extensions.insert(state);
        }
        self
    }

    /// Check if a specific state type has been added to this router.
    ///
    /// Useful for debugging state configuration issues.
    ///
    /// # Example
    ///
    /// ```
    /// let app = Router::new().state(app_config);
    ///
    /// assert!(app.has_state::<AppConfig>());
    /// assert!(!app.has_state::<DatabasePool>());
    /// ```
    pub fn has_state<T: Send + Sync + Clone + 'static>(&self) -> bool {
        let inner = self.inner.read();
        inner.extensions.get::<T>().is_some()
    }

    /// Get a reference to state if it exists (for debugging/testing).
    ///
    /// # Example
    ///
    /// ```
    /// let config = AppConfig { debug: true };
    /// let app = Router::new().state(config);
    ///
    /// if let Some(state) = app.get_state::<AppConfig>() {
    ///     println!("Debug mode: {}", state.debug);
    /// }
    /// ```
    pub fn get_state<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        let inner = self.inner.read();
        // Extensions store Arc<T>, so we get the Arc and then clone the inner value
        inner
            .extensions
            .get::<T>()
            .map(|arc_t| arc_t.as_ref().clone())
    }

    /// Ensures the router is compiled and returns the compiled version.
    ///
    /// This method performs lazy compilation of routes. If the router hasn't
    /// changed since the last compilation, it returns the cached version.
    /// Otherwise, it compiles the routes and caches the result.
    ///
    /// # Returns
    /// An `Arc<CompiledRouter>` containing the optimized route structure
    fn ensure_compiled(&self) -> Arc<CompiledRouter> {
        // Fast path: check if compilation is needed without holding the lock
        {
            let inner = self.inner.read();
            if !inner.dirty {
                return self.compiled.load_full();
            }
        }

        // Now get write lock for compilation
        let compiled = {
            let inner = self.inner.read();
            self.compile_inner(&inner)
        };

        // Store the compiled version
        let compiled_arc = Arc::new(compiled);
        self.compiled.store(Arc::clone(&compiled_arc));

        // Mark as clean
        {
            let mut inner = self.inner.write();
            inner.dirty = false;
        }

        compiled_arc
    }

    /// Compiles the router's internal state into an optimized structure.
    ///
    /// This method processes all routes, nested routers, and middleware to
    /// create an optimized structure for fast request matching.
    ///
    /// # Parameters
    /// - `inner`: The internal router state to compile
    ///
    /// # Returns
    /// A `CompiledRouter` with optimized route matching structures
    fn compile_inner(&self, inner: &RouterInner) -> CompiledRouter {
        let mut routes = inner.routes.clone();
        let mut middleware = inner.middleware.clone();
        let mut not_found_handler = inner.not_found_handler.clone();

        // Process nested routers
        for (prefix, nested_router) in &inner.nested_routers {
            let nested_compiled = nested_router.ensure_compiled();

            // Merge routes with prefix
            for (method, nested_routes) in &nested_compiled.routes {
                for route in nested_routes {
                    let full_path = if route.path == "/" {
                        prefix.clone()
                    } else {
                        format!("{}{}", prefix, route.path)
                    };

                    let mut new_route = route.clone();
                    new_route.path = full_path.clone();
                    new_route.regex = Route::compile_regex(&full_path);

                    routes
                        .entry(method.clone())
                        .or_insert_with(Vec::new)
                        .push(new_route);
                }
            }

            // Merge middleware (nested first)
            let mut combined = nested_compiled.middleware.clone();
            combined.extend(middleware.drain(..));
            middleware = combined;

            // Use nested not found handler if we don't have one
            if not_found_handler.is_none() {
                not_found_handler = nested_compiled.not_found_handler.clone();
            }
        }

        // Sort routes by specificity for faster matching
        for routes in routes.values_mut() {
            routes.sort_by(|a, b| {
                // Sort by number of path segments (more specific first)
                let a_segments = a.path.matches('/').count();
                let b_segments = b.path.matches('/').count();

                // Then by number of parameters (fewer parameters first)
                let a_params = a.param_names.len() + a.wildcard_names.len();
                let b_params = b.param_names.len() + b.wildcard_names.len();

                b_segments.cmp(&a_segments).then(a_params.cmp(&b_params))
            });
        }

        CompiledRouter {
            routes,
            middleware,
            not_found_handler,
        }
    }

    /// Handles an incoming HTTP request.
    ///
    /// This is the main request processing method. It applies middleware,
    /// finds matching routes, executes handlers, and processes responses.
    ///
    /// # Parameters
    /// - `req`: The incoming HTTP request to handle
    ///
    /// # Returns
    /// A `Result<Response>` containing either the response or an error
    ///
    /// # Request Processing Flow
    /// 1. Compile routes if needed
    /// 2. Apply middleware `before` hooks
    /// 3. Find matching route and extract parameters
    /// 4. Execute route handler
    /// 5. Apply middleware `after` hooks
    /// 6. Return response or 404 error
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Request, Method};
    /// use http::Uri;
    /// use bytes::Bytes;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::new()
    ///     .get("/test", || async { Ok(ignitia::Response::text("Test")) });
    ///
    /// let request = Request::new(
    ///     Method::GET,
    ///     "/test".parse::<Uri>()?,
    ///     http::Version::HTTP_11,
    ///     http::HeaderMap::new(),
    ///     Bytes::new(),
    /// );
    ///
    /// let response = router.handle(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle(&self, mut req: Request) -> Result<Response> {
        let compiled = self.ensure_compiled();

        {
            let inner = self.inner.read();
            req.extensions = inner.extensions.clone();
        }

        // Apply middleware in order
        for mw in &compiled.middleware {
            mw.before(&mut req).await?;
        }

        // Get routes for this method only
        if let Some(routes) = compiled.routes.get(&req.method) {
            for route in routes {
                if let Some(params) = route.matches(&req) {
                    req.params = params;

                    // Apply route middleware before handler
                    for mw in &route.middleware {
                        mw.before(&mut req).await?;
                    }

                    let mut response = route.handler.handle(req.clone()).await?;

                    // Apply route middleware after handler in reverse order
                    for mw in route.middleware.iter().rev() {
                        mw.after(&req, &mut response).await?;
                    }

                    // Apply global middleware after handler in reverse order
                    for mw in compiled.middleware.iter().rev() {
                        mw.after(&req, &mut response).await?;
                    }

                    return Ok(response);
                }
            }
        }

        // Handle not found
        if let Some(handler) = &compiled.not_found_handler {
            handler.handle(req).await
        } else {
            Err(Error::NotFound(req.uri.path().to_string()))
        }
    }

    /// Checks if a route exists for the given method and path.
    ///
    /// This is a utility method primarily used for testing and debugging.
    /// It checks if any route would match the given method and path without
    /// actually processing a full request.
    ///
    /// # Parameters
    /// - `method`: The HTTP method to check
    /// - `path`: The path to check for matches
    ///
    /// # Returns
    /// `true` if a route matches, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Router, Method};
    ///
    /// let router = Router::new()
    ///     .get("/users/:id", || async { Ok(ignitia::Response::text("User")) });
    ///
    /// assert!(router.matches(&Method::GET, "/users/123"));
    /// assert!(!router.matches(&Method::GET, "/users"));
    /// assert!(!router.matches(&Method::POST, "/users/123"));
    /// ```
    pub fn matches(&self, method: &Method, path: &str) -> bool {
        let compiled = self.ensure_compiled();
        if let Some(routes) = compiled.routes.get(method) {
            for route in routes {
                // Create a mock request for matching
                let mock_req = Request::new(
                    method.clone(),
                    path.parse().unwrap_or_default(),
                    http::Version::HTTP_11,
                    http::HeaderMap::new(),
                    bytes::Bytes::new(),
                );

                if route.matches(&mock_req).is_some() {
                    return true;
                }
            }
        }
        false
    }
}

/// Normalizes a path by ensuring it starts with '/' and doesn't end with '/' (except for root).
///
/// This function standardizes path formats to ensure consistent route matching.
///
/// # Parameters
/// - `path`: The path to normalize
///
/// # Returns
/// The normalized path as a String
///
/// # Examples
/// ```
/// # use ignitia::router::normalize_path;
/// assert_eq!(normalize_path("users"), "/users");
/// assert_eq!(normalize_path("/users/"), "/users");
/// assert_eq!(normalize_path("/"), "/");
/// ```
fn normalize_path(path: &str) -> String {
    let mut normalized = path.to_string();

    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }

    if normalized != "/" && normalized.ends_with('/') {
        normalized.pop();
    }

    normalized
}

// Implement Clone for Router
impl Clone for Router {
    /// Creates a deep clone of the router.
    ///
    /// This creates a new router with the same configuration but independent
    /// compilation state. The cloned router will need to be recompiled on
    /// first use.
    fn clone(&self) -> Self {
        let inner = self.inner.read();
        Self {
            inner: Arc::new(RwLock::new(RouterInner {
                routes: inner.routes.clone(),
                middleware: inner.middleware.clone(),
                not_found_handler: inner.not_found_handler.clone(),
                nested_routers: inner.nested_routers.clone(),
                dirty: inner.dirty,
                extensions: inner.extensions.clone(),
                #[cfg(feature = "websocket")]
                #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
                websocket_routes: inner.websocket_routes.clone(),
            })),
            compiled: ArcSwap::new(Arc::new(CompiledRouter {
                routes: HashMap::new(),
                middleware: Vec::new(),
                not_found_handler: None,
            })),
        }
    }
}

// Default implementation
impl Default for Router {
    /// Creates a new empty router (same as `Router::new()`).
    fn default() -> Self {
        Self::new()
    }
}
