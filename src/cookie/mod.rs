//! # Cookie Handling Module
//!
//! This module provides comprehensive HTTP cookie management for the Ignitia web framework.
//! It includes cookie creation, parsing, security attributes, and seamless integration
//! with request and response handling.
//!
//! ## Features
//!
//! - **Cookie Creation**: Easy-to-use builder pattern for creating cookies
//! - **Cookie Parsing**: Automatic parsing of Cookie headers from requests
//! - **Security Attributes**: Support for Secure, HttpOnly, SameSite attributes
//! - **Cookie Jar**: Container for managing multiple cookies
//! - **Request Integration**: Direct cookie access from Request objects
//! - **Response Integration**: Easy cookie setting on Response objects
//! - **Expiration Handling**: Support for both Max-Age and Expires attributes
//!
//! ## Quick Start
//!
//! ### Creating and Setting Cookies
//!
//! ```
//! use ignitia::{Cookie, SameSite, Response};
//! use std::time::{Duration, SystemTime};
//!
//! // Simple cookie
//! let session_cookie = Cookie::new("session_id", "abc123");
//!
//! // Cookie with security attributes
//! let secure_cookie = Cookie::new("auth_token", "xyz789")
//!     .secure()
//!     .http_only()
//!     .same_site(SameSite::Strict)
//!     .max_age(3600) // 1 hour
//!     .path("/api");
//!
//! // Add to response
//! let response = Response::ok()
//!     .add_cookie(session_cookie)
//!     .add_cookie(secure_cookie);
//! ```
//!
//! ### Reading Cookies from Requests
//!
//! ```
//! use ignitia::Request;
//!
//! fn handle_request(req: Request) {
//!     // Get specific cookie
//!     if let Some(session_id) = req.cookie("session_id") {
//!         println!("Session ID: {}", session_id);
//!     }
//!
//!     // Get all cookies
//!     let jar = req.cookies();
//!     for (name, value) in jar.all() {
//!         println!("Cookie: {} = {}", name, value);
//!     }
//! }
//! ```
//!
//! ## Security Best Practices
//!
//! - Use `secure()` for HTTPS-only cookies
//! - Use `http_only()` to prevent JavaScript access
//! - Set appropriate `SameSite` policies to prevent CSRF attacks
//! - Set `path` and `domain` attributes to limit cookie scope
//! - Use reasonable expiration times

use crate::{Request, Response};
use http::{HeaderName, HeaderValue};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents an HTTP cookie with all its attributes.
///
/// A cookie is a small piece of data stored by the web browser and sent back
/// to the server with each request. This struct provides a comprehensive
/// representation of HTTP cookies as defined in RFC 6265.
///
/// # Examples
///
/// ## Basic Cookie
/// ```
/// use ignitia::Cookie;
///
/// let cookie = Cookie::new("username", "john_doe");
/// ```
///
/// ## Cookie with Security Attributes
/// ```
/// use ignitia::{Cookie, SameSite};
///
/// let secure_cookie = Cookie::new("auth_token", "secure_value_123")
///     .secure()           // HTTPS only
///     .http_only()        // No JavaScript access
///     .same_site(SameSite::Strict)  // CSRF protection
///     .max_age(3600)      // 1 hour expiration
///     .path("/admin")     // Restrict to /admin path
///     .domain(".example.com"); // Domain restriction
/// ```
///
/// ## Session Cookie (no expiration)
/// ```
/// use ignitia::Cookie;
///
/// // Session cookie - expires when browser closes
/// let session = Cookie::new("session_id", "temp_session_123");
/// ```
#[derive(Debug, Clone)]
pub struct Cookie {
    /// The name of the cookie
    pub name: String,
    /// The value of the cookie
    pub value: String,
    /// The path attribute - restricts cookie to specific paths
    pub path: Option<String>,
    /// The domain attribute - restricts cookie to specific domains
    pub domain: Option<String>,
    /// Maximum age in seconds - alternative to expires
    pub max_age: Option<u64>,
    /// Absolute expiration time
    pub expires: Option<SystemTime>,
    /// Secure flag - cookie only sent over HTTPS
    pub secure: bool,
    /// HttpOnly flag - cookie not accessible via JavaScript
    pub http_only: bool,
    /// SameSite attribute for CSRF protection
    pub same_site: Option<SameSite>,
}

/// The SameSite attribute for cookies, providing CSRF protection.
///
/// The SameSite attribute controls when cookies are sent with cross-site requests,
/// helping to prevent Cross-Site Request Forgery (CSRF) attacks.
///
/// # Variants
///
/// - **Strict**: Cookie never sent with cross-site requests
/// - **Lax**: Cookie sent with safe cross-site requests (GET, HEAD, OPTIONS, TRACE)
/// - **None**: Cookie sent with all cross-site requests (requires Secure flag)
///
/// # Examples
///
/// ```
/// use ignitia::{Cookie, SameSite};
///
/// // Strict - maximum protection, may break some legitimate use cases
/// let strict_cookie = Cookie::new("csrf_token", "abc123")
///     .same_site(SameSite::Strict);
///
/// // Lax - good balance of security and usability
/// let lax_cookie = Cookie::new("session_id", "xyz789")
///     .same_site(SameSite::Lax);
///
/// // None - for cross-site functionality (must be secure)
/// let cross_site_cookie = Cookie::new("tracking_id", "def456")
///     .same_site(SameSite::None)
///     .secure();
/// ```
#[derive(Debug, Clone)]
pub enum SameSite {
    /// Never send cookie with cross-site requests
    Strict,
    /// Send cookie with safe cross-site requests only
    Lax,
    /// Send cookie with all cross-site requests (requires Secure)
    None,
}

impl fmt::Display for SameSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SameSite::Strict => write!(f, "Strict"),
            SameSite::Lax => write!(f, "Lax"),
            SameSite::None => write!(f, "None"),
        }
    }
}

impl Cookie {
    /// Creates a new cookie with the specified name and value.
    ///
    /// This creates a basic session cookie (no expiration) without any
    /// additional attributes. Use the builder methods to add attributes.
    ///
    /// # Parameters
    /// - `name`: The cookie name
    /// - `value`: The cookie value
    ///
    /// # Examples
    /// ```
    /// use ignitia::Cookie;
    ///
    /// let cookie = Cookie::new("user_preference", "dark_mode");
    /// let session_cookie = Cookie::new("session_id", "abc123def456");
    /// ```
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            path: None,
            domain: None,
            max_age: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Sets the path attribute for the cookie.
    ///
    /// The path attribute specifies the URL path prefix that must exist
    /// in the requested URL for the browser to send the cookie.
    ///
    /// # Parameters
    /// - `path`: The path prefix (e.g., "/", "/api", "/admin")
    ///
    /// # Examples
    /// ```
    /// use ignitia::Cookie;
    ///
    /// // Cookie only sent for /admin paths
    /// let admin_cookie = Cookie::new("admin_session", "xyz789")
    ///     .path("/admin");
    ///
    /// // Cookie sent for all paths
    /// let global_cookie = Cookie::new("theme", "dark")
    ///     .path("/");
    /// ```
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets the domain attribute for the cookie.
    ///
    /// The domain attribute specifies which hosts can receive the cookie.
    /// If not specified, the cookie is only sent to the exact host that set it.
    ///
    /// # Parameters
    /// - `domain`: The domain (e.g., "example.com", ".example.com")
    ///
    /// # Security Note
    /// Be cautious with domain attributes. A leading dot (e.g., ".example.com")
    /// makes the cookie available to all subdomains.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Cookie;
    ///
    /// // Cookie for specific domain only
    /// let domain_cookie = Cookie::new("user_id", "12345")
    ///     .domain("api.example.com");
    ///
    /// // Cookie for domain and all subdomains
    /// let subdomain_cookie = Cookie::new("site_preference", "compact")
    ///     .domain(".example.com");
    /// ```
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Sets the max-age attribute in seconds.
    ///
    /// Max-Age specifies the maximum lifetime of the cookie as the number
    /// of seconds until the cookie expires. This takes precedence over
    /// the Expires attribute if both are present.
    ///
    /// # Parameters
    /// - `seconds`: Maximum age in seconds
    ///
    /// # Common Values
    /// - 60: 1 minute
    /// - 3600: 1 hour
    /// - 86400: 1 day
    /// - 604800: 1 week
    /// - 2592000: 30 days
    ///
    /// # Examples
    /// ```
    /// use ignitia::Cookie;
    ///
    /// // Short-lived cookie (5 minutes)
    /// let temp_cookie = Cookie::new("temp_token", "abc123")
    ///     .max_age(300);
    ///
    /// // Long-lived cookie (30 days)
    /// let remember_cookie = Cookie::new("remember_me", "true")
    ///     .max_age(30 * 24 * 60 * 60);
    /// ```
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Sets the expires attribute to an absolute time.
    ///
    /// The Expires attribute specifies the exact date and time when
    /// the cookie should expire.
    ///
    /// # Parameters
    /// - `time`: The expiration time as SystemTime
    ///
    /// # Examples
    /// ```
    /// use ignitia::Cookie;
    /// use std::time::{SystemTime, Duration};
    ///
    /// // Cookie expires in 1 hour from now
    /// let future_time = SystemTime::now() + Duration::from_secs(3600);
    /// let expiring_cookie = Cookie::new("session", "temp_123")
    ///     .expires(future_time);
    /// ```
    pub fn expires(mut self, time: SystemTime) -> Self {
        self.expires = Some(time);
        self
    }

    /// Marks the cookie as secure (HTTPS only).
    ///
    /// When the Secure attribute is set, the cookie is only sent to the server
    /// over HTTPS connections. This prevents the cookie from being transmitted
    /// over unencrypted HTTP connections, protecting it from network eavesdropping.
    ///
    /// # Security Note
    /// Always use the Secure flag for sensitive cookies in production environments.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Cookie;
    ///
    /// let auth_cookie = Cookie::new("auth_token", "sensitive_token_123")
    ///     .secure()
    ///     .http_only();
    /// ```
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Marks the cookie as HTTP-only (not accessible via JavaScript).
    ///
    /// When the HttpOnly attribute is set, the cookie is not accessible
    /// through client-side scripts. This helps prevent Cross-Site Scripting (XSS)
    /// attacks from stealing cookies.
    ///
    /// # Security Note
    /// Use HttpOnly for all cookies that don't need to be accessed by client-side JavaScript.
    ///
    /// # Examples
    /// ```
    /// use ignitia::Cookie;
    ///
    /// // Session cookie protected from XSS
    /// let session_cookie = Cookie::new("session_id", "secure_session_123")
    ///     .http_only()
    ///     .secure();
    ///
    /// // Theme cookie accessible to JavaScript (no http_only)
    /// let theme_cookie = Cookie::new("theme", "dark_mode");
    /// ```
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Sets the SameSite attribute for CSRF protection.
    ///
    /// The SameSite attribute controls when cookies are sent with cross-site requests,
    /// providing protection against Cross-Site Request Forgery (CSRF) attacks.
    ///
    /// # Parameters
    /// - `same_site`: The SameSite policy to apply
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Cookie, SameSite};
    ///
    /// // Maximum protection
    /// let csrf_token = Cookie::new("csrf_token", "abc123")
    ///     .same_site(SameSite::Strict);
    ///
    /// // Balanced protection
    /// let session_cookie = Cookie::new("session_id", "xyz789")
    ///     .same_site(SameSite::Lax);
    /// ```
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Creates a cookie that immediately expires (for cookie removal).
    ///
    /// This is a convenience method for creating cookies that instruct
    /// the browser to delete an existing cookie. The cookie will have
    /// an expiration date in the past.
    ///
    /// # Parameters
    /// - `name`: The name of the cookie to remove
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Cookie, Response};
    ///
    /// // Remove a session cookie
    /// let remove_session = Cookie::removal("session_id");
    ///
    /// let response = Response::ok()
    ///     .add_cookie(remove_session);
    /// ```
    pub fn removal(name: impl Into<String>) -> Self {
        Self::new(name, "").expires(UNIX_EPOCH).path("/")
    }

    /// Converts the cookie to a Set-Cookie header value string.
    ///
    /// This method generates the complete Set-Cookie header value according
    /// to RFC 6265, including all cookie attributes.
    ///
    /// # Returns
    /// A string representation suitable for use in HTTP Set-Cookie headers
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Cookie, SameSite};
    ///
    /// let cookie = Cookie::new("session_id", "abc123")
    ///     .secure()
    ///     .http_only()
    ///     .same_site(SameSite::Strict)
    ///     .max_age(3600)
    ///     .path("/");
    ///
    /// let header_value = cookie.to_header_value();
    /// // Result: "session_id=abc123; Path=/; Max-Age=3600; Secure; HttpOnly; SameSite=Strict"
    /// ```
    pub fn to_header_value(&self) -> String {
        let mut cookie_str = format!("{}={}", self.name, self.value);

        if let Some(path) = &self.path {
            cookie_str.push_str(&format!("; Path={path}"));
        }

        if let Some(domain) = &self.domain {
            cookie_str.push_str(&format!("; Domain={domain}"));
        }

        if let Some(max_age) = self.max_age {
            cookie_str.push_str(&format!("; Max-Age={max_age}"));
        }

        if let Some(expires) = self.expires {
            if let Ok(duration) = expires.duration_since(UNIX_EPOCH) {
                // Format as HTTP date
                let timestamp = duration.as_secs();
                let datetime =
                    httpdate::HttpDate::from(UNIX_EPOCH + Duration::from_secs(timestamp));
                cookie_str.push_str(&format!("; Expires={datetime}"));
            }
        }

        if self.secure {
            cookie_str.push_str("; Secure");
        }

        if self.http_only {
            cookie_str.push_str("; HttpOnly");
        }

        if let Some(same_site) = &self.same_site {
            cookie_str.push_str(&format!("; SameSite={same_site}"));
        }

        cookie_str
    }
}

/// A container for managing multiple cookies from HTTP requests.
///
/// CookieJar provides a convenient way to work with all cookies sent
/// by the client. It automatically parses Cookie headers and provides
/// methods for accessing individual cookies or iterating over all cookies.
///
/// # Examples
///
/// ## Manual Creation
/// ```
/// use ignitia::CookieJar;
///
/// // Create empty jar
/// let mut jar = CookieJar::new();
///
/// // Parse from Cookie header string
/// let jar = CookieJar::from_header("session_id=abc123; theme=dark; lang=en");
/// ```
///
/// ## Accessing Cookies
/// ```
/// use ignitia::CookieJar;
///
/// let jar = CookieJar::from_header("user_id=12345; preferences=compact");
///
/// // Check if cookie exists
/// if jar.contains("user_id") {
///     println!("User is logged in");
/// }
///
/// // Get specific cookie
/// if let Some(user_id) = jar.get("user_id") {
///     println!("User ID: {}", user_id);
/// }
///
/// // Iterate over all cookies
/// for (name, value) in jar.all() {
///     println!("Cookie: {} = {}", name, value);
/// }
/// ```
#[derive(Debug, Default)]
pub struct CookieJar {
    cookies: HashMap<String, String>,
}

impl CookieJar {
    /// Creates a new empty cookie jar.
    ///
    /// # Examples
    /// ```
    /// use ignitia::CookieJar;
    ///
    /// let jar = CookieJar::new();
    /// assert!(jar.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Parses cookies from an HTTP Cookie header string.
    ///
    /// This method parses the Cookie header format as specified in RFC 6265.
    /// It handles multiple cookies separated by semicolons and properly
    /// trims whitespace.
    ///
    /// # Parameters
    /// - `cookie_header`: The Cookie header value string
    ///
    /// # Format
    /// The expected format is: `name1=value1; name2=value2; name3=value3`
    ///
    /// # Examples
    /// ```
    /// use ignitia::CookieJar;
    ///
    /// let jar = CookieJar::from_header("session_id=abc123; theme=dark; lang=en-US");
    /// assert_eq!(jar.len(), 3);
    /// assert_eq!(jar.get("session_id"), Some(&"abc123".to_string()));
    /// assert_eq!(jar.get("theme"), Some(&"dark".to_string()));
    /// ```
    ///
    /// # Note
    /// This method only parses cookie names and values. Cookie attributes
    /// (like Path, Domain, etc.) are not present in request Cookie headers.
    pub fn from_header(cookie_header: &str) -> Self {
        let mut jar = Self::new();

        for cookie_pair in cookie_header.split(';') {
            let cookie_pair = cookie_pair.trim();
            if let Some((name, value)) = cookie_pair.split_once('=') {
                jar.cookies
                    .insert(name.trim().to_string(), value.trim().to_string());
            }
        }

        jar
    }

    /// Gets the value of a cookie by name.
    ///
    /// # Parameters
    /// - `name`: The name of the cookie to retrieve
    ///
    /// # Returns
    /// `Some(&String)` if the cookie exists, `None` otherwise
    ///
    /// # Examples
    /// ```
    /// use ignitia::CookieJar;
    ///
    /// let jar = CookieJar::from_header("user_id=12345; session=active");
    ///
    /// if let Some(user_id) = jar.get("user_id") {
    ///     println!("User ID: {}", user_id);
    /// }
    ///
    /// assert_eq!(jar.get("nonexistent"), None);
    /// ```
    pub fn get(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }

    /// Checks if a cookie with the given name exists.
    ///
    /// # Parameters
    /// - `name`: The name of the cookie to check
    ///
    /// # Returns
    /// `true` if the cookie exists, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// use ignitia::CookieJar;
    ///
    /// let jar = CookieJar::from_header("auth_token=xyz789");
    ///
    /// if jar.contains("auth_token") {
    ///     println!("User is authenticated");
    /// }
    /// ```
    pub fn contains(&self, name: &str) -> bool {
        self.cookies.contains_key(name)
    }

    /// Gets a reference to all cookies as a HashMap.
    ///
    /// # Returns
    /// A reference to the internal HashMap containing all cookies
    ///
    /// # Examples
    /// ```
    /// use ignitia::CookieJar;
    ///
    /// let jar = CookieJar::from_header("a=1; b=2; c=3");
    /// let all_cookies = jar.all();
    ///
    /// for (name, value) in all_cookies {
    ///     println!("{}: {}", name, value);
    /// }
    /// ```
    pub fn all(&self) -> &HashMap<String, String> {
        &self.cookies
    }

    /// Returns the number of cookies in the jar.
    ///
    /// # Examples
    /// ```
    /// use ignitia::CookieJar;
    ///
    /// let jar = CookieJar::from_header("a=1; b=2");
    /// assert_eq!(jar.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    /// Checks if the cookie jar is empty.
    ///
    /// # Examples
    /// ```
    /// use ignitia::CookieJar;
    ///
    /// let empty_jar = CookieJar::new();
    /// assert!(empty_jar.is_empty());
    ///
    /// let jar = CookieJar::from_header("session=abc123");
    /// assert!(!jar.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

/// Request extensions for cookie handling.
///
/// These methods are implemented directly on the Request struct to provide
/// convenient access to cookies without requiring manual header parsing.
impl Request {
    /// Gets all cookies from the request as a CookieJar.
    ///
    /// This method automatically parses the Cookie header (if present)
    /// and returns a CookieJar containing all cookies sent by the client.
    ///
    /// # Returns
    /// A CookieJar containing all cookies from the request
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    ///
    /// fn handle_request(req: Request) {
    ///     let jar = req.cookies();
    ///
    ///     if jar.contains("session_id") {
    ///         println!("Session found");
    ///     }
    ///
    ///     println!("Total cookies: {}", jar.len());
    /// }
    /// ```
    pub fn cookies(&self) -> CookieJar {
        if let Some(cookie_header) = self.header("Cookie") {
            CookieJar::from_header(cookie_header)
        } else {
            CookieJar::new()
        }
    }

    /// Gets the value of a specific cookie by name.
    ///
    /// This is a convenience method that combines cookie parsing and lookup
    /// into a single operation.
    ///
    /// # Parameters
    /// - `name`: The name of the cookie to retrieve
    ///
    /// # Returns
    /// `Some(String)` if the cookie exists, `None` otherwise
    ///
    /// # Examples
    /// ```
    /// use ignitia::Request;
    ///
    /// fn check_authentication(req: Request) -> bool {
    ///     req.cookie("auth_token").is_some()
    /// }
    ///
    /// fn get_user_preferences(req: Request) -> String {
    ///     req.cookie("theme").unwrap_or_else(|| "light".to_string())
    /// }
    /// ```
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.cookies().get(name).cloned()
    }
}

/// Response extensions for cookie handling.
///
/// These methods are implemented directly on the Response struct to provide
/// convenient cookie management when building HTTP responses.
impl Response {
    /// Adds a cookie to the response.
    ///
    /// This method adds a Set-Cookie header to the response for the given cookie.
    /// Multiple cookies can be added by calling this method multiple times.
    ///
    /// # Parameters
    /// - `cookie`: The Cookie to add to the response
    ///
    /// # Returns
    /// Self (for method chaining)
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Cookie, Response, SameSite};
    ///
    /// let auth_cookie = Cookie::new("auth_token", "xyz789")
    ///     .secure()
    ///     .http_only()
    ///     .same_site(SameSite::Strict)
    ///     .max_age(3600);
    ///
    /// let response = Response::ok()
    ///     .add_cookie(auth_cookie);
    /// ```
    pub fn add_cookie(mut self, cookie: Cookie) -> Self {
        let header_value = HeaderValue::from_str(&cookie.to_header_value())
            .unwrap_or_else(|_| HeaderValue::from_static(""));

        self.headers
            .append(HeaderName::from_static("set-cookie"), header_value);
        self
    }

    /// Adds multiple cookies to the response.
    ///
    /// This is a convenience method for adding several cookies at once.
    ///
    /// # Parameters
    /// - `cookies`: A vector of Cookies to add to the response
    ///
    /// # Returns
    /// Self (for method chaining)
    ///
    /// # Examples
    /// ```
    /// use ignitia::{Cookie, Response, SameSite};
    ///
    /// let cookies = vec![
    ///     Cookie::new("session_id", "abc123").http_only(),
    ///     Cookie::new("theme", "dark"),
    ///     Cookie::new("lang", "en-US").max_age(86400),
    /// ];
    ///
    /// let response = Response::ok()
    ///     .add_cookies(cookies);
    /// ```
    pub fn add_cookies(mut self, cookies: Vec<Cookie>) -> Self {
        for cookie in cookies {
            self = self.add_cookie(cookie);
        }
        self
    }

    /// Removes a cookie by setting it to expire immediately.
    ///
    /// This method creates a removal cookie (with past expiration date)
    /// that instructs the browser to delete the specified cookie.
    ///
    /// # Parameters
    /// - `name`: The name of the cookie to remove
    ///
    /// # Returns
    /// Self (for method chaining)
    ///
    /// # Examples
    /// ```
    /// use ignitia::Response;
    ///
    /// // User logout - remove session cookie
    /// let response = Response::ok()
    ///     .remove_cookie("session_id")
    ///     .remove_cookie("auth_token");
    /// ```
    ///
    /// # Note
    /// The removal cookie will have the path set to "/" to ensure it matches
    /// the original cookie. If the original cookie had a different path or domain,
    /// you may need to create a custom removal cookie with matching attributes.
    pub fn remove_cookie(self, name: impl Into<String>) -> Self {
        self.add_cookie(Cookie::removal(name))
    }
}
