use http::Method;
use ignitia::handler::extractor::State;
use ignitia::response::IntoResponse;
use ignitia::{
    middleware::Next, Body, Cookie, Cookies, Error, LayeredHandler, Middleware, Request, Response,
    Result, Router, SameSite, Server,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing_subscriber;

// Simple user database simulation
#[derive(Clone)]
struct UserDB {
    users: HashMap<String, User>,
}

#[derive(Clone)]
struct User {
    username: String,
    password: String, // In production, use hashed passwords!
    email: String,
    role: String,
}

impl UserDB {
    fn new() -> Self {
        let mut users = HashMap::new();
        // Add some test users
        users.insert(
            "admin".to_string(),
            User {
                username: "admin".to_string(),
                password: "admin123".to_string(),
                email: "admin@example.com".to_string(),
                role: "admin".to_string(),
            },
        );
        users.insert(
            "user".to_string(),
            User {
                username: "user".to_string(),
                password: "user123".to_string(),
                email: "user@example.com".to_string(),
                role: "user".to_string(),
            },
        );
        Self { users }
    }

    fn validate(&self, username: &str, password: &str) -> Option<&User> {
        self.users
            .get(username)
            .filter(|user| user.password == password)
    }

    fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }
}

// ============================================================================
// MIDDLEWARE - Updated with Next Pattern
// ============================================================================

// Authentication Middleware - Checks if user is logged in
#[derive(Clone)]
struct AuthMiddleware {
    protected_paths: Vec<String>,
}

impl AuthMiddleware {
    fn new() -> Self {
        Self {
            protected_paths: Vec::new(),
        }
    }

    fn protect_paths(mut self, paths: Vec<impl Into<String>>) -> Self {
        for path in paths {
            self.protected_paths.push(path.into());
        }
        self
    }

    fn requires_auth(&self, path: &str) -> bool {
        self.protected_paths
            .iter()
            .any(|protected_path| path.starts_with(protected_path))
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let path = req.uri.path();

        // Only check auth for protected paths
        if !self.requires_auth(path) {
            return next.run(req).await;
        }

        // Get UserDB from state/extensions
        let user_db = match req.get_extension::<UserDB>() {
            Some(db) => db,
            None => {
                return Error::Internal("UserDB extension not found".to_string()).into_response();
            }
        };

        // Check for session cookie
        let username = match req.cookie("session_user") {
            Some(u) => u,
            None => {
                println!("🔒 Authentication required for {}", path);
                return Error::Unauthorized("Invalid session".to_string()).into_response();
            }
        };

        // Validate user exists in database
        if user_db.get_user(&username).is_none() {
            println!("❌ Invalid session for user: {}", username);
            return Error::Unauthorized("Invalid user".to_string()).into_response();
        }

        println!("✅ Authenticated user: {} accessing {}", username, path);

        // User is authenticated, proceed
        next.run(req).await
    }
}

// Authorization Middleware - Checks user roles
#[derive(Clone)]
struct RoleMiddleware {
    role_paths: HashMap<String, String>, // path -> required_role
}

impl RoleMiddleware {
    fn new() -> Self {
        Self {
            role_paths: HashMap::new(),
        }
    }

    fn require_role(mut self, path: impl Into<String>, role: impl Into<String>) -> Self {
        self.role_paths.insert(path.into(), role.into());
        self
    }
}

#[async_trait::async_trait]
impl Middleware for RoleMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let path = req.uri.path();

        // Check if this path requires a specific role
        if let Some(required_role) = self.role_paths.get(path) {
            // Get UserDB from extensions
            let user_db = match req.get_extension::<UserDB>() {
                Some(db) => db,
                None => {
                    return Error::Internal("UserDB extension not found".to_string())
                        .into_response();
                }
            };

            let username = match req.cookie("session_user") {
                Some(u) => u,
                None => {
                    return Error::Unauthorized("Invalid session".to_string()).into_response();
                }
            };

            let user = match user_db.get_user(&username) {
                Some(u) => u,
                None => {
                    return Error::Unauthorized("Invalid user".to_string()).into_response();
                }
            };

            if user.role != *required_role {
                println!(
                    "🚫 Access denied: {} (role: {}) tried to access {} (requires: {})",
                    username, user.role, path, required_role
                );
                // Return forbidden response
                return Response::html(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>🚫 Access Denied</title>
                        <style>
                            body { font-family: Arial, sans-serif; margin: 40px; max-width: 600px; margin: 40px auto; }
                            .error { background: #f8d7da; padding: 20px; border-radius: 10px; color: #721c24; }
                        </style>
                    </head>
                    <body>
                        <div class="error">
                            <h1>🚫 Access Denied</h1>
                            <p>You need administrator privileges to access this page.</p>
                            <p>This check was performed by role middleware using the Next pattern!</p>
                        </div>
                        <a href="/dashboard" style="background: #007acc; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">← Back to Dashboard</a>
                    </body>
                    </html>
                "#,
                )
                .with_status(http::StatusCode::FORBIDDEN);
            }

            println!(
                "✅ Role authorized: {} ({}) accessing {}",
                username, user.role, path
            );
        }

        // Role check passed or not required
        next.run(req).await
    }
}

// Request Logger Middleware
#[derive(Clone)]
struct RequestLoggerMiddleware;

#[async_trait::async_trait]
impl Middleware for RequestLoggerMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        let user = req
            .cookie("session_user")
            .map(|u| format!("user={}", u))
            .unwrap_or_else(|| "anonymous".to_string());

        println!(
            "📋 {} {} {:?} ({})",
            req.method,
            req.uri.path(),
            req.version,
            user
        );

        // Process request
        let response = next.run(req).await;

        println!("📤 Response: {}", response.status.as_u16());

        response
    }
}

// ============================================================================
// MAIN APPLICATION
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let user_db = UserDB::new();

    // Create middleware instances
    let auth_middleware =
        AuthMiddleware::new().protect_paths(vec!["/dashboard", "/profile", "/admin"]);

    // Build router with Next-pattern middleware
    let router = Router::new()
        // Apply global middleware (executed in order)
        .middleware(RequestLoggerMiddleware)
        .middleware(auth_middleware)
        // Public routes (no auth required)
        .get("/", home)
        .get("/login", login_form)
        .post("/login", login_process)
        // Protected routes (auth middleware will handle authentication)
        .get("/dashboard", dashboard)
        .get("/profile", profile)
        // Admin-only route with per-route middleware
        .route_with_layered(
            "/admin",
            Method::GET,
            LayeredHandler::new(admin_panel)
                .layer(RoleMiddleware::new().require_role("/admin", "admin")),
        )
        // Logout (public)
        .get("/logout", logout)
        // Inject UserDB as application state
        .state(user_db);

    let addr: SocketAddr = "127.0.0.1:3009".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🔐 Login Demo with Middleware running on http://{}", addr);
    println!("🛡️  Middleware Features:");
    println!("   ✅ Axum-style Next pattern middleware");
    println!("   ✅ State-based dependency injection");
    println!("   ✅ Authentication middleware for protected routes");
    println!("   ✅ Role-based authorization middleware");
    println!("   ✅ Request logging middleware");
    println!("   ✅ Per-route middleware with LayeredHandler");
    println!("👤 Test accounts:");
    println!("   admin / admin123 (admin role)");
    println!("   user / user123 (user role)");

    server.ignitia().await.unwrap();
    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

// Helper function to parse form data
fn parse_form_data(body: &[u8]) -> Result<HashMap<String, String>> {
    let body_str =
        String::from_utf8(body.to_vec()).map_err(|_| Error::BadRequest("Invalid UTF-8".into()))?;
    let mut form_data = HashMap::new();
    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // Simple URL decode (replace + with space)
            let key = key.replace('+', " ");
            let value = value.replace('+', " ");
            form_data.insert(key, value);
        }
    }
    Ok(form_data)
}

// ============================================================================
// HANDLER FUNCTIONS
// ============================================================================

async fn home(cookies: Cookies, State(db): State<UserDB>) -> Result<Response> {
    // Check if user is logged in (this is just for display, not security)
    let current_user = cookies.get("session_user").map(|s| s.clone());

    let user_info = if let Some(username) = &current_user {
        if let Some(user) = db.get_user(username) {
            format!(
                r#"
                <div style="background: #d4edda; padding: 15px; border-radius: 5px; margin: 20px 0;">
                    <h3>👋 Welcome back, {}!</h3>
                    <p><strong>Email:</strong> {}</p>
                    <p><strong>Role:</strong> {}</p>
                    <p><strong>Status:</strong> ✅ Logged in</p>
                    <div style="margin-top: 10px;">
                        <a href="/dashboard" style="background: #007acc; color: white; padding: 8px 16px; text-decoration: none; border-radius: 3px; margin-right: 10px;">Dashboard</a>
                        <a href="/profile" style="background: #28a745; color: white; padding: 8px 16px; text-decoration: none; border-radius: 3px; margin-right: 10px;">Profile</a>
                        {}
                        <a href="/logout" style="background: #dc3545; color: white; padding: 8px 16px; text-decoration: none; border-radius: 3px;">Logout</a>
                    </div>
                </div>
            "#,
                user.username,
                user.email,
                user.role,
                if user.role == "admin" {
                    r#"<a href="/admin" style="background: #ffc107; color: black; padding: 8px 16px; text-decoration: none; border-radius: 3px; margin-right: 10px;">Admin Panel</a>"#
                } else {
                    ""
                }
            )
        } else {
            "<p>⚠️ Invalid session</p>".to_string()
        }
    } else {
        r#"
            <div style="background: #f8d7da; padding: 15px; border-radius: 5px; margin: 20px 0;">
                <p>🔒 You are not logged in</p>
                <a href="/login" style="background: #007acc; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">Login</a>
            </div>
        "#.to_string()
    };

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>🔐 Login Demo with Next Middleware</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; max-width: 800px; margin: 40px auto; }}
                h1 {{ color: #333; }}
                .test-accounts {{ background: #e7f3ff; padding: 20px; border-radius: 10px; margin: 20px 0; }}
                .middleware-info {{ background: #f0f8ff; padding: 20px; border-radius: 10px; margin: 20px 0; }}
            </style>
        </head>
        <body>
            <h1>🔐 Login Demo - Axum-Style Next Middleware</h1>
            {}
            <div class="middleware-info">
                <h3>🛡️ Middleware Protection (Next Pattern):</h3>
                <ul>
                    <li><strong>Authentication Middleware:</strong> Protects /dashboard, /profile, /admin</li>
                    <li><strong>Role Middleware:</strong> /admin requires 'admin' role (per-route)</li>
                    <li><strong>Request Logger:</strong> Logs all requests with user info</li>
                    <li><strong>Next Pattern:</strong> Clean middleware chaining like Axum</li>
                </ul>
            </div>
            <div class="test-accounts">
                <h3>🧪 Test Accounts:</h3>
                <ul>
                    <li><strong>admin</strong> / admin123 (Administrator - can access admin panel)</li>
                    <li><strong>user</strong> / user123 (Regular User - cannot access admin panel)</li>
                </ul>
            </div>
            <h3>🛠️ Features Demonstrated:</h3>
            <ul>
                <li>✅ Axum-style Next pattern middleware</li>
                <li>✅ State-based dependency injection</li>
                <li>✅ Authentication middleware (automatic session checking)</li>
                <li>✅ Role-based authorization middleware</li>
                <li>✅ Per-route middleware with LayeredHandler</li>
                <li>✅ Request logging middleware</li>
                <li>✅ Clean handler functions with extractors</li>
                <li>✅ Type-safe state management</li>
            </ul>
        </body>
        </html>
    "#,
        user_info
    );

    Ok(Response::html(html))
}

async fn login_form(cookies: Cookies) -> Result<Response> {
    // Redirect if already logged in
    if cookies.get("session_user").is_some() {
        return Ok(Response::html(
            r#"
            <h1>Already Logged In</h1>
            <p>You are already logged in!</p>
            <a href="/">← Go to Home</a>
        "#,
        ));
    }

    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>🔐 Login</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; max-width: 400px; margin: 40px auto; }
                .login-form { background: #f8f9fa; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
                input { width: 100%; padding: 12px; margin: 10px 0; border: 1px solid #ddd; border-radius: 5px; box-sizing: border-box; }
                button { width: 100%; padding: 12px; background: #007acc; color: white; border: none; border-radius: 5px; cursor: pointer; font-size: 16px; }
                button:hover { background: #005c99; }
                .test-info { background: #e7f3ff; padding: 15px; border-radius: 5px; margin: 20px 0; font-size: 14px; }
            </style>
        </head>
        <body>
            <div class="login-form">
                <h1>🔐 Login</h1>
                <form action="/login" method="POST">
                    <input type="text" name="username" placeholder="Username" required autofocus>
                    <input type="password" name="password" placeholder="Password" required>
                    <button type="submit">Login</button>
                </form>
                <div class="test-info">
                    <strong>🧪 Test Accounts:</strong><br>
                    admin / admin123<br>
                    user / user123
                </div>
                <p style="text-align: center; margin-top: 20px;">
                    <a href="/">← Back to Home</a>
                </p>
            </div>
        </body>
        </html>
    "#;

    Ok(Response::html(html))
}

async fn login_process(body: Body, State(db): State<UserDB>) -> Result<Response> {
    let form_data = parse_form_data(&body)?;
    let username = form_data.get("username").unwrap_or(&"".to_string()).clone();
    let password = form_data.get("password").unwrap_or(&"".to_string()).clone();

    if username.is_empty() || password.is_empty() {
        return Ok(Response::html(
            r#"
            <h1>❌ Login Failed</h1>
            <p>Username and password are required.</p>
            <a href="/login">← Try Again</a>
        "#,
        ));
    }

    if let Some(user) = db.validate(&username, &password) {
        // Create session cookie
        let session_cookie = Cookie::new("session_user", &user.username)
            .path("/")
            .max_age(3600) // 1 hour
            .http_only()
            .same_site(SameSite::Lax);

        let response = Response::html(format!(
            r#"
            <h1>✅ Login Successful!</h1>
            <p>Welcome, <strong>{}</strong>!</p>
            <p>Role: <strong>{}</strong></p>
            <p>Next middleware will now protect your session automatically!</p>
            <p>Redirecting to dashboard...</p>
            <script>
                setTimeout(function() {{
                    window.location.href = '/dashboard';
                }}, 2000);
            </script>
            <a href="/dashboard">Go to Dashboard</a>
        "#,
            user.username, user.role
        ));

        Ok(response.add_cookie(session_cookie))
    } else {
        Ok(Response::html(
            r#"
            <h1>❌ Login Failed</h1>
            <p>Invalid username or password.</p>
            <a href="/login">← Try Again</a>
        "#,
        ))
    }
}

async fn dashboard(cookies: Cookies, State(db): State<UserDB>) -> Result<Response> {
    // Middleware guarantees we have a valid session here
    let username = cookies.get("session_user").unwrap().clone();
    let user = db.get_user(&username).unwrap();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>📊 Dashboard</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; }}
                .dashboard {{ max-width: 800px; margin: 0 auto; }}
                .user-info {{ background: #d4edda; padding: 20px; border-radius: 10px; margin: 20px 0; }}
                .middleware-info {{ background: #f0f8ff; padding: 20px; border-radius: 10px; margin: 20px 0; }}
                .actions {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; margin: 20px 0; }}
                .actions a {{ display: block; padding: 15px; text-align: center; text-decoration: none; border-radius: 5px; color: white; }}
                .btn-primary {{ background: #007acc; }}
                .btn-success {{ background: #28a745; }}
                .btn-warning {{ background: #ffc107; color: black; }}
                .btn-danger {{ background: #dc3545; }}
            </style>
        </head>
        <body>
            <div class="dashboard">
                <h1>📊 Dashboard (Protected by Next Middleware)</h1>

                <div class="middleware-info">
                    <h3>🛡️ Next Pattern Protection Active</h3>
                    <p>This page is automatically protected by authentication middleware!</p>
                    <p>Middleware used <code>next.run(req).await</code> to proceed.</p>
                    <p>No manual session checks needed in the handler.</p>
                </div>

                <div class="user-info">
                    <h2>👤 User Information</h2>
                    <p><strong>Username:</strong> {}</p>
                    <p><strong>Email:</strong> {}</p>
                    <p><strong>Role:</strong> {}</p>
                    <p><strong>Session:</strong> ✅ Validated by middleware</p>
                </div>

                <div class="actions">
                    <a href="/profile" class="btn-primary">👤 Profile</a>
                    {}
                    <a href="/" class="btn-success">🏠 Home</a>
                    <a href="/logout" class="btn-danger">🚪 Logout</a>
                </div>
            </div>
        </body>
        </html>
    "#,
        user.username,
        user.email,
        user.role,
        if user.role == "admin" {
            r#"<a href="/admin" class="btn-warning">⚙️ Admin Panel</a>"#
        } else {
            ""
        }
    );

    Ok(Response::html(html))
}

async fn profile(cookies: Cookies, State(db): State<UserDB>) -> Result<Response> {
    let username = cookies.get("session_user").unwrap().clone();
    let user = db.get_user(&username).unwrap();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>👤 Profile</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; max-width: 600px; margin: 40px auto; }}
                .profile {{ background: #f8f9fa; padding: 30px; border-radius: 10px; }}
                .middleware-info {{ background: #f0f8ff; padding: 20px; border-radius: 10px; margin: 20px 0; }}
            </style>
        </head>
        <body>
            <h1>👤 User Profile (Protected by Next Middleware)</h1>

            <div class="middleware-info">
                <p><strong>🛡️ Protection:</strong> Middleware called <code>next.run()</code> after validation.</p>
            </div>

            <div class="profile">
                <h2>{}</h2>
                <p><strong>Email:</strong> {}</p>
                <p><strong>Role:</strong> {}</p>
                <p><strong>Session:</strong> ✅ Validated by Next middleware</p>
            </div>

            <div style="text-align: center; margin-top: 20px;">
                <a href="/dashboard" style="background: #007acc; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">← Back to Dashboard</a>
            </div>
        </body>
        </html>
    "#,
        user.username, user.email, user.role
    );

    Ok(Response::html(html))
}

async fn admin_panel(cookies: Cookies, State(db): State<UserDB>) -> Result<Response> {
    let username = cookies.get("session_user").unwrap().clone();
    let user = db.get_user(&username).unwrap();

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>⚙️ Admin Panel</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; max-width: 800px; margin: 40px auto; }}
                .admin-panel {{ background: #fff3cd; padding: 20px; border-radius: 10px; margin: 20px 0; }}
                .user-stats {{ background: #d1ecf1; padding: 20px; border-radius: 10px; margin: 20px 0; }}
                .middleware-info {{ background: #f0f8ff; padding: 20px; border-radius: 10px; margin: 20px 0; }}
            </style>
        </head>
        <body>
            <h1>⚙️ Admin Panel (Protected by Per-Route Middleware)</h1>

            <div class="middleware-info">
                <p><strong>🛡️ LayeredHandler + RoleMiddleware:</strong></p>
                <p>This route uses per-route middleware with the Next pattern!</p>
                <p>Global auth middleware + route-specific role middleware.</p>
                <p>Both use <code>next.run()</code> to chain execution.</p>
            </div>

            <p>Welcome to the admin panel, <strong>{}</strong>!</p>

            <div class="user-stats">
                <h3>👥 User Statistics</h3>
                <ul>
                    <li>Total Users: {}</li>
                    <li>Admin Users: {}</li>
                    <li>Regular Users: {}</li>
                </ul>
            </div>

            <div style="text-align: center; margin-top: 20px;">
                <a href="/dashboard" style="background: #007acc; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">← Back to Dashboard</a>
            </div>
        </body>
        </html>
    "#,
        user.username,
        db.users.len(),
        db.users.values().filter(|u| u.role == "admin").count(),
        db.users.values().filter(|u| u.role == "user").count()
    );

    Ok(Response::html(html))
}

async fn logout() -> Result<Response> {
    let response = Response::html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>👋 Logged Out</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; max-width: 600px; margin: 40px auto; }
                .logout-message { background: #d4edda; padding: 20px; border-radius: 10px; text-align: center; }
            </style>
        </head>
        <body>
            <div class="logout-message">
                <h1>👋 Logged Out</h1>
                <p>You have been successfully logged out.</p>
                <p>Your session cookies have been cleared.</p>
                <p>Next middleware will no longer recognize you as authenticated.</p>
            </div>
            <div style="text-align: center; margin-top: 20px;">
                <a href="/" style="background: #007acc; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px;">← Back to Home</a>
            </div>
        </body>
        </html>
    "#,
    );

    Ok(response.remove_cookie("session_user"))
}
