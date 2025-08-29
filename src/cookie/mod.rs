use crate::{Request, Response};
use http::{HeaderName, HeaderValue};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub max_age: Option<u64>, // seconds
    pub expires: Option<SystemTime>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
}

#[derive(Debug, Clone)]
pub enum SameSite {
    Strict,
    Lax,
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
    /// Create a new cookie with name and value
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

    /// Set the path for this cookie
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the domain for this cookie
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Set max age in seconds
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Set expiration time
    pub fn expires(mut self, time: SystemTime) -> Self {
        self.expires = Some(time);
        self
    }

    /// Make this cookie secure (HTTPS only)
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Make this cookie HTTP only (not accessible via JavaScript)
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Set SameSite attribute
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Create a removal cookie (expires in the past)
    pub fn removal(name: impl Into<String>) -> Self {
        Self::new(name, "").expires(UNIX_EPOCH).path("/")
    }

    /// Convert cookie to Set-Cookie header value
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

#[derive(Debug, Default)]
pub struct CookieJar {
    cookies: HashMap<String, String>,
}

impl CookieJar {
    /// Create a new empty cookie jar
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Parse cookies from a Cookie header string
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

    /// Get a cookie value by name
    pub fn get(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }

    /// Check if a cookie exists
    pub fn contains(&self, name: &str) -> bool {
        self.cookies.contains_key(name)
    }

    /// Get all cookies as a HashMap
    pub fn all(&self) -> &HashMap<String, String> {
        &self.cookies
    }

    /// Get the number of cookies
    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    /// Check if jar is empty
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

/// Cookie utilities for Request
impl Request {
    /// Get cookies from the request
    pub fn cookies(&self) -> CookieJar {
        if let Some(cookie_header) = self.header("Cookie") {
            CookieJar::from_header(cookie_header)
        } else {
            CookieJar::new()
        }
    }

    /// Get a specific cookie value
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.cookies().get(name).cloned()
    }
}

/// Cookie utilities for Response
impl Response {
    /// Add a cookie to the response
    pub fn add_cookie(mut self, cookie: Cookie) -> Self {
        let header_value = HeaderValue::from_str(&cookie.to_header_value())
            .unwrap_or_else(|_| HeaderValue::from_static(""));

        self.headers
            .append(HeaderName::from_static("set-cookie"), header_value);
        self
    }

    /// Add multiple cookies to the response
    pub fn add_cookies(mut self, cookies: Vec<Cookie>) -> Self {
        for cookie in cookies {
            self = self.add_cookie(cookie);
        }
        self
    }

    /// Remove a cookie (by setting it to expire in the past)
    pub fn remove_cookie(self, name: impl Into<String>) -> Self {
        self.add_cookie(Cookie::removal(name))
    }
}
