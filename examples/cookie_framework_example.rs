use ignitia::{handler_fn, Cookie, Error, Request, Response, Result, Router, SameSite, Server};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .get("/", handler_fn(home))
        .get("/set", handler_fn(set_cookies))
        .get("/get", handler_fn(get_cookies))
        .get("/secure", handler_fn(set_secure_cookie))
        .get("/remove", handler_fn(remove_cookie))
        .post("/login", handler_fn(login))
        .get("/dashboard", handler_fn(dashboard))
        .get("/logout", handler_fn(logout));

    let addr: SocketAddr = "127.0.0.1:3006".parse().unwrap();
    let server = Server::new(router, addr);

    println!("🍪 Framework Cookie Server running on http://{}", addr);
    println!("📋 Built-in Cookie Functionality Demo");
    println!("🔗 Try: http://127.0.0.1:3006/");

    server.run().await.unwrap();
    Ok(())
}

async fn home(req: Request) -> Result<Response> {
    let cookies = req.cookies();
    let cookie_count = cookies.len();

    let mut cookie_list = String::new();
    for (name, value) in cookies.all() {
        cookie_list.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", name, value));
    }

    if cookie_list.is_empty() {
        cookie_list = "<tr><td colspan='2'><em>No cookies found</em></td></tr>".to_string();
    }

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>🍪 Built-in Cookie Framework</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 40px; }}
                .container {{ max-width: 900px; margin: 0 auto; }}
                .cookie-display {{ background: #f0f8ff; padding: 20px; border-radius: 10px; margin: 20px 0; }}
                .links {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 10px; margin: 20px 0; }}
                .links a {{ display: block; padding: 12px; background: #007acc; color: white; text-decoration: none; border-radius: 5px; text-align: center; }}
                .links a:hover {{ background: #005c99; }}
                table {{ width: 100%; border-collapse: collapse; margin: 15px 0; }}
                th, td {{ padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }}
                th {{ background: #f2f2f2; }}
                .form-section {{ background: #f8f9fa; padding: 20px; border-radius: 10px; margin: 20px 0; }}
                input, button {{ padding: 10px; margin: 5px; border: 1px solid #ddd; border-radius: 5px; }}
                button {{ background: #28a745; color: white; border: none; cursor: pointer; }}
                button:hover {{ background: #218838; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>🍪 Built-in Cookie Framework</h1>
                <p><strong>Framework Feature:</strong> Cookies are now built into your mini web framework!</p>

                <div class="cookie-display">
                    <h2>Current Cookies ({} total):</h2>
                    <table>
                        <thead>
                            <tr><th>Name</th><th>Value</th></tr>
                        </thead>
                        <tbody>{}</tbody>
                    </table>
                </div>

                <div class="links">
                    <a href="/set">🍪 Set Demo Cookies</a>
                    <a href="/get">📋 Get Cookies (JSON)</a>
                    <a href="/secure">🔒 Set Secure Cookie</a>
                    <a href="/remove">🗑️ Remove Cookies</a>
                    <a href="/dashboard">👤 Dashboard (Protected)</a>
                </div>

                <div class="form-section">
                    <h3>🔐 Login Form</h3>
                    <form action="/login" method="POST">
                        <input type="text" name="username" placeholder="Username" required>
                        <input type="password" name="password" placeholder="Password" required>
                        <button type="submit">Login</button>
                    </form>
                    <p><em>Try any username/password combo</em></p>
                </div>

                <div class="form-section">
                    <h3>🛠️ Framework Features Demonstrated:</h3>
                    <ul>
                        <li>✅ <code>request.cookies()</code> - Get all cookies</li>
                        <li>✅ <code>request.cookie("name")</code> - Get specific cookie</li>
                        <li>✅ <code>response.add_cookie(cookie)</code> - Add cookie to response</li>
                        <li>✅ <code>Cookie::new("name", "value")</code> - Create cookies</li>
                        <li>✅ Cookie attributes: path, domain, secure, http_only, same_site</li>
                        <li>✅ <code>Cookie::removal("name")</code> - Remove cookies</li>
                    </ul>
                </div>
            </div>
        </body>
        </html>
    "#,
        cookie_count, cookie_list
    );

    Ok(Response::html(html))
}

async fn set_cookies(_req: Request) -> Result<Response> {
    let response = Response::html(
        r#"
        <h1>✅ Demo Cookies Set!</h1>
        <p>Multiple cookies with different attributes have been set using the built-in framework:</p>
        <ul>
            <li><strong>demo_cookie</strong> - Basic cookie (1 hour)</li>
            <li><strong>user_pref</strong> - User preference (1 day)</li>
            <li><strong>session_temp</strong> - Session cookie (browser close)</li>
            <li><strong>theme</strong> - Theme setting (30 days)</li>
        </ul>
        <a href="/">← Back to Home</a>
    "#,
    );

    // Using the built-in cookie functionality
    let cookies = vec![
        Cookie::new("demo_cookie", "framework_test")
            .path("/")
            .max_age(3600), // 1 hour
        Cookie::new("user_pref", "dark_mode")
            .path("/")
            .max_age(86400) // 1 day
            .same_site(SameSite::Lax),
        Cookie::new("session_temp", "temp_session").path("/"), // No max_age = session cookie
        Cookie::new("theme", "blue")
            .path("/")
            .max_age(2592000) // 30 days
            .http_only(),
    ];

    Ok(response.add_cookies(cookies))
}

async fn get_cookies(req: Request) -> Result<Response> {
    let cookies = req.cookies();
    Response::json(cookies.all())
}

async fn set_secure_cookie(_req: Request) -> Result<Response> {
    let response = Response::html(
        r#"
        <h1>🔒 Secure Cookie Set!</h1>
        <p>A secure, HTTP-only cookie has been set with SameSite=Strict.</p>
        <p>This cookie demonstrates security best practices.</p>
        <a href="/">← Back to Home</a>
    "#,
    );

    let secure_cookie = Cookie::new("secure_token", "super_secret_value")
        .path("/")
        .max_age(1800) // 30 minutes
        .secure()
        .http_only()
        .same_site(SameSite::Strict);

    Ok(response.add_cookie(secure_cookie))
}

async fn remove_cookie(_req: Request) -> Result<Response> {
    let response = Response::html(
        r#"
        <h1>🗑️ Cookies Removed!</h1>
        <p>Demo cookies have been removed using the framework's removal functionality.</p>
        <a href="/">← Back to Home</a>
    "#,
    );

    // Remove multiple cookies
    Ok(response
        .remove_cookie("demo_cookie")
        .remove_cookie("user_pref")
        .remove_cookie("session_temp"))
}

async fn login(req: Request) -> Result<Response> {
    // Simple form parsing
    let body = String::from_utf8(req.body.to_vec())
        .map_err(|_| Error::BadRequest("Invalid form data".into()))?;

    let mut username = String::new();
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "username" {
                username = value.replace('+', " ");
                break;
            }
        }
    }

    if username.is_empty() {
        return Ok(Response::html(
            r#"
            <h1>❌ Login Failed!</h1>
            <p>Username is required.</p>
            <a href="/">← Back to Home</a>
        "#,
        ));
    }

    let response = Response::html(format!(
        r#"
        <h1>✅ Login Successful!</h1>
        <p>Welcome, <strong>{}</strong>!</p>
        <p>A session cookie has been set using the framework.</p>
        <a href="/dashboard">Go to Dashboard</a> | <a href="/">Home</a>
    "#,
        username
    ));

    // Create a session cookie
    let session_cookie = Cookie::new("user_session", &username)
        .path("/")
        .max_age(3600) // 1 hour
        .http_only()
        .same_site(SameSite::Lax);

    Ok(response.add_cookie(session_cookie))
}

async fn dashboard(req: Request) -> Result<Response> {
    // Check for session using built-in cookie functionality
    if let Some(username) = req.cookie("user_session") {
        Ok(Response::html(format!(
            r#"
            <h1>👤 Dashboard</h1>
            <p>Welcome back, <strong>{}</strong>!</p>
            <p>This protected page was accessed using framework cookies.</p>

            <div style="background: #d4edda; padding: 15px; border-radius: 5px; margin: 20px 0;">
                <h3>🍪 Session Info:</h3>
                <p><strong>Username:</strong> {}</p>
                <p><strong>Authentication:</strong> ✅ Valid session cookie</p>
                <p><strong>Method:</strong> Framework built-in cookies</p>
            </div>

            <a href="/logout">Logout</a> | <a href="/">Home</a>
        "#,
            username, username
        )))
    } else {
        Ok(Response::html(
            r#"
            <h1>🔒 Access Denied</h1>
            <p>Please log in to access the dashboard.</p>
            <p>This protection is implemented using framework cookies.</p>
            <a href="/">← Login</a>
        "#,
        ))
    }
}

async fn logout(_req: Request) -> Result<Response> {
    let response = Response::html(
        r#"
        <h1>👋 Logged Out!</h1>
        <p>You have been logged out. Session cookie cleared.</p>
        <a href="/">← Back to Home</a>
    "#,
    );

    // Remove the session cookie
    Ok(response.remove_cookie("user_session"))
}
