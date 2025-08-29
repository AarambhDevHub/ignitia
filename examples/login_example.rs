use ignitia::{handler_fn, Cookie, Error, Request, Response, Result, Router, SameSite, Server};
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
                password: "admin123".to_string(), // Never store plain passwords in production!
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let user_db = UserDB::new();

    let router = Router::new()
        .get(
            "/",
            handler_fn({
                let db = user_db.clone();
                move |req| home(req, db.clone())
            }),
        )
        .get("/login", handler_fn(login_form))
        .post(
            "/login",
            handler_fn({
                let db = user_db.clone();
                move |req| login_process(req, db.clone())
            }),
        )
        .get(
            "/dashboard",
            handler_fn({
                let db = user_db.clone();
                move |req| dashboard(req, db.clone())
            }),
        )
        .get(
            "/profile",
            handler_fn({
                let db = user_db.clone();
                move |req| profile(req, db.clone())
            }),
        )
        .get(
            "/admin",
            handler_fn({
                let db = user_db.clone();
                move |req| admin_panel(req, db.clone())
            }),
        )
        .get("/logout", handler_fn(logout));

    let addr: SocketAddr = "127.0.0.1:3008".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🔐 Login Demo Server running on http://{}", addr);
    println!("👤 Test accounts:");
    println!("   admin / admin123 (admin role)");
    println!("   user / user123 (user role)");

    server.run().await.unwrap();
    Ok(())
}

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

async fn home(req: Request, db: UserDB) -> Result<Response> {
    // Check if user is logged in
    let current_user = req.cookie("session_user");

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
            <title>🔐 Login Demo</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; max-width: 800px; margin: 40px auto; }}
                h1 {{ color: #333; }}
                .test-accounts {{ background: #e7f3ff; padding: 20px; border-radius: 10px; margin: 20px 0; }}
            </style>
        </head>
        <body>
            <h1>🔐 Mini Web Framework - Login Demo</h1>

            {}

            <div class="test-accounts">
                <h3>🧪 Test Accounts:</h3>
                <ul>
                    <li><strong>admin</strong> / admin123 (Administrator)</li>
                    <li><strong>user</strong> / user123 (Regular User)</li>
                </ul>
            </div>

            <h3>🛠️ Features Demonstrated:</h3>
            <ul>
                <li>✅ Session-based authentication</li>
                <li>✅ Role-based access control</li>
                <li>✅ Protected routes</li>
                <li>✅ Cookie management</li>
                <li>✅ Form processing</li>
                <li>✅ User sessions</li>
            </ul>
        </body>
        </html>
    "#,
        user_info
    );

    Ok(Response::html(html))
}

async fn login_form(req: Request) -> Result<Response> {
    // Redirect if already logged in
    if req.cookie("session_user").is_some() {
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

async fn login_process(req: Request, db: UserDB) -> Result<Response> {
    let form_data = parse_form_data(&req.body)?;

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

        // Create role cookie (for authorization)
        let role_cookie = Cookie::new("user_role", &user.role)
            .path("/")
            .max_age(3600)
            .http_only()
            .same_site(SameSite::Lax);

        let response = Response::html(format!(
            r#"
            <h1>✅ Login Successful!</h1>
            <p>Welcome, <strong>{}</strong>!</p>
            <p>Role: <strong>{}</strong></p>
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

        Ok(response.add_cookie(session_cookie).add_cookie(role_cookie))
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

async fn dashboard(req: Request, db: UserDB) -> Result<Response> {
    // Check authentication
    let username = req
        .cookie("session_user")
        .ok_or_else(|| Error::Unauthorized)?;

    let user = db.get_user(&username).ok_or_else(|| Error::Unauthorized)?;

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
                <h1>📊 Dashboard</h1>

                <div class="user-info">
                    <h2>👤 User Information</h2>
                    <p><strong>Username:</strong> {}</p>
                    <p><strong>Email:</strong> {}</p>
                    <p><strong>Role:</strong> {}</p>
                    <p><strong>Session:</strong> ✅ Active</p>
                </div>

                <div class="actions">
                    <a href="/profile" class="btn-primary">👤 Profile</a>
                    {}
                    <a href="/" class="btn-success">🏠 Home</a>
                    <a href="/logout" class="btn-danger">🚪 Logout</a>
                </div>

                <h3>🔒 Protected Content</h3>
                <p>This is a protected page that requires authentication.</p>
                <p>Only logged-in users can see this content.</p>
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

async fn profile(req: Request, db: UserDB) -> Result<Response> {
    let username = req
        .cookie("session_user")
        .ok_or_else(|| Error::Unauthorized)?;

    let user = db.get_user(&username).ok_or_else(|| Error::Unauthorized)?;

    let html = format!(
        r#"
        <h1>👤 User Profile</h1>
        <div style="background: #f8f9fa; padding: 20px; border-radius: 10px; max-width: 500px;">
            <h2>{}</h2>
            <p><strong>Email:</strong> {}</p>
            <p><strong>Role:</strong> {}</p>
            <p><strong>Account Status:</strong> ✅ Active</p>
        </div>
        <div style="margin-top: 20px;">
            <a href="/dashboard">← Back to Dashboard</a>
        </div>
    "#,
        user.username, user.email, user.role
    );

    Ok(Response::html(html))
}

async fn admin_panel(req: Request, db: UserDB) -> Result<Response> {
    let username = req
        .cookie("session_user")
        .ok_or_else(|| Error::Unauthorized)?;

    let user = db.get_user(&username).ok_or_else(|| Error::Unauthorized)?;

    // Check admin role
    if user.role != "admin" {
        return Ok(Response::html(
            r#"
            <h1>🚫 Access Denied</h1>
            <p>You need administrator privileges to access this page.</p>
            <a href="/dashboard">← Back to Dashboard</a>
        "#,
        ));
    }

    let html = format!(
        r#"
        <h1>⚙️ Admin Panel</h1>
        <p>Welcome to the admin panel, <strong>{}</strong>!</p>

        <div style="background: #fff3cd; padding: 20px; border-radius: 10px; margin: 20px 0;">
            <h3>👥 User Management</h3>
            <ul>
                <li>Total Users: {}</li>
                <li>Admin Users: {}</li>
                <li>Regular Users: {}</li>
            </ul>
        </div>

        <div style="background: #d1ecf1; padding: 20px; border-radius: 10px; margin: 20px 0;">
            <h3>🔧 Admin Actions</h3>
            <p>This is where admin-only functionality would go.</p>
            <p>Only users with 'admin' role can access this area.</p>
        </div>

        <a href="/dashboard">← Back to Dashboard</a>
    "#,
        user.username,
        db.users.len(),
        db.users.values().filter(|u| u.role == "admin").count(),
        db.users.values().filter(|u| u.role == "user").count()
    );

    Ok(Response::html(html))
}

async fn logout(_req: Request) -> Result<Response> {
    let response = Response::html(
        r#"
        <h1>👋 Logged Out</h1>
        <p>You have been successfully logged out.</p>
        <p>Your session has been cleared.</p>
        <a href="/">← Back to Home</a>
    "#,
    );

    // Clear session cookies
    Ok(response
        .remove_cookie("session_user")
        .remove_cookie("user_role"))
}
