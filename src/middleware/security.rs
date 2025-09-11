//! # Security Middleware
//!
//! Comprehensive HTTP security middleware that complements existing CORS and body limit middleware.
//! Focuses on security headers, rate limiting, and Content Security Policy.

use crate::middleware::Middleware;
use crate::{Request, Response, Result};
use http::header::{HeaderName, HeaderValue};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Default rate limit (1000 requests per minute)
const DEFAULT_RATE_LIMIT: u32 = 1000;

/// Content Security Policy configuration
#[derive(Debug, Clone)]
pub struct CspConfig {
    /// default-src directive
    pub default_src: Vec<String>,
    /// script-src directive
    pub script_src: Vec<String>,
    /// style-src directive
    pub style_src: Vec<String>,
    /// img-src directive
    pub img_src: Vec<String>,
    /// connect-src directive
    pub connect_src: Vec<String>,
    /// font-src directive
    pub font_src: Vec<String>,
    /// object-src directive (typically 'none')
    pub object_src: Vec<String>,
    /// Report violations to this URI
    pub report_uri: Option<String>,
}

impl Default for CspConfig {
    fn default() -> Self {
        Self {
            default_src: vec!["'self'".to_string()],
            script_src: vec!["'self'".to_string()],
            style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
            img_src: vec!["'self'".to_string(), "data:".to_string()],
            connect_src: vec!["'self'".to_string()],
            font_src: vec!["'self'".to_string()],
            object_src: vec!["'none'".to_string()],
            report_uri: None,
        }
    }
}

/// Rate limiting bucket for token bucket algorithm
#[derive(Debug, Clone)]
struct RateLimitBucket {
    tokens: u32,
    last_refill: Instant,
    capacity: u32,
}

impl RateLimitBucket {
    fn new(capacity: u32) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
        }
    }

    fn try_consume(&mut self, refill_rate: u32, window: Duration) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        // Refill tokens based on elapsed time
        if elapsed >= window {
            self.tokens = self.capacity;
            self.last_refill = now;
        } else {
            let tokens_to_add =
                (elapsed.as_secs_f64() / window.as_secs_f64() * refill_rate as f64) as u32;
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            if tokens_to_add > 0 {
                self.last_refill = now;
            }
        }

        // Try to consume a token
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }
}

/// Security middleware that adds security headers and rate limiting.
///
/// This middleware complements your existing CORS and BodyLimit middleware
/// by providing additional security layers.
///
/// ## Features
///
/// - **Security Headers**: HSTS, CSP, X-Frame-Options, X-Content-Type-Options
/// - **Rate Limiting**: IP-based request throttling with token bucket algorithm
/// - **Content Security Policy**: Configurable CSP directives
/// - **Server Header Removal**: Removes identifying server headers
///
/// ## Examples
///
/// ```
/// use ignitia::{Router, SecurityMiddleware};
///
/// let app = Router::new()
///     .middleware(SecurityMiddleware::new())
///     .get("/api/users", get_users);
/// ```
#[derive(Debug)]
pub struct SecurityMiddleware {
    /// Enable Strict Transport Security header
    enable_hsts: bool,
    /// HSTS max age in seconds
    hsts_max_age: u32,
    /// Include subdomains in HSTS
    hsts_include_subdomains: bool,
    /// Enable HSTS preload
    hsts_preload: bool,

    /// Enable Content Security Policy
    enable_csp: bool,
    /// CSP configuration
    csp_config: CspConfig,

    /// Enable rate limiting
    enable_rate_limiting: bool,
    /// Rate limiting buckets (IP -> Bucket)
    rate_limit_buckets: Arc<Mutex<HashMap<IpAddr, RateLimitBucket>>>,
    /// Maximum requests per window
    rate_limit_max: u32,
    /// Rate limiting window duration
    rate_limit_window: Duration,

    /// Enable security headers
    enable_security_headers: bool,

    /// Remove server identification headers
    remove_server_header: bool,
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self {
            enable_hsts: true,
            hsts_max_age: 31536000, // 1 year
            hsts_include_subdomains: true,
            hsts_preload: false,

            enable_csp: true,
            csp_config: CspConfig::default(),

            enable_rate_limiting: true,
            rate_limit_buckets: Arc::new(Mutex::new(HashMap::new())),
            rate_limit_max: DEFAULT_RATE_LIMIT,
            rate_limit_window: Duration::from_secs(60),

            enable_security_headers: true,
            remove_server_header: true,
        }
    }
}

impl SecurityMiddleware {
    /// Creates a new `SecurityMiddleware` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables or disables HSTS (HTTP Strict Transport Security).
    pub fn with_hsts(mut self, enabled: bool) -> Self {
        self.enable_hsts = enabled;
        self
    }

    /// Configures HSTS settings.
    pub fn with_hsts_config(
        mut self,
        max_age: u32,
        include_subdomains: bool,
        preload: bool,
    ) -> Self {
        self.hsts_max_age = max_age;
        self.hsts_include_subdomains = include_subdomains;
        self.hsts_preload = preload;
        self
    }

    /// Configures Content Security Policy.
    pub fn with_csp(mut self, config: CspConfig) -> Self {
        self.enable_csp = true;
        self.csp_config = config;
        self
    }

    /// Configures rate limiting.
    pub fn with_rate_limit(mut self, max_requests: u32, window: Duration) -> Self {
        self.enable_rate_limiting = true;
        self.rate_limit_max = max_requests;
        self.rate_limit_window = window;
        self
    }

    /// Gets client IP address from request.
    fn get_client_ip(&self, req: &Request) -> Option<IpAddr> {
        // Try X-Forwarded-For header first (for load balancers/proxies)
        if let Some(forwarded) = req.headers.get("x-forwarded-for") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    if let Ok(ip) = first_ip.trim().parse() {
                        return Some(ip);
                    }
                }
            }
        }

        // Try X-Real-IP header
        if let Some(real_ip) = req.headers.get("x-real-ip") {
            if let Ok(ip_str) = real_ip.to_str() {
                if let Ok(ip) = ip_str.parse() {
                    return Some(ip);
                }
            }
        }

        // Fallback to default for development
        Some("127.0.0.1".parse().unwrap())
    }

    /// Checks rate limit for the given IP address.
    fn check_rate_limit(&self, ip: IpAddr) -> bool {
        if !self.enable_rate_limiting {
            return true;
        }

        let mut buckets = self.rate_limit_buckets.lock().unwrap();
        let bucket = buckets
            .entry(ip)
            .or_insert_with(|| RateLimitBucket::new(self.rate_limit_max));

        bucket.try_consume(self.rate_limit_max, self.rate_limit_window)
    }

    /// Builds CSP header value from configuration.
    fn build_csp_header(&self) -> String {
        let mut directives = Vec::new();

        if !self.csp_config.default_src.is_empty() {
            directives.push(format!(
                "default-src {}",
                self.csp_config.default_src.join(" ")
            ));
        }
        if !self.csp_config.script_src.is_empty() {
            directives.push(format!(
                "script-src {}",
                self.csp_config.script_src.join(" ")
            ));
        }
        if !self.csp_config.style_src.is_empty() {
            directives.push(format!("style-src {}", self.csp_config.style_src.join(" ")));
        }
        if !self.csp_config.img_src.is_empty() {
            directives.push(format!("img-src {}", self.csp_config.img_src.join(" ")));
        }
        if !self.csp_config.connect_src.is_empty() {
            directives.push(format!(
                "connect-src {}",
                self.csp_config.connect_src.join(" ")
            ));
        }
        if !self.csp_config.font_src.is_empty() {
            directives.push(format!("font-src {}", self.csp_config.font_src.join(" ")));
        }
        if !self.csp_config.object_src.is_empty() {
            directives.push(format!(
                "object-src {}",
                self.csp_config.object_src.join(" ")
            ));
        }
        if let Some(report_uri) = &self.csp_config.report_uri {
            directives.push(format!("report-uri {}", report_uri));
        }

        directives.join("; ")
    }
}

#[async_trait::async_trait]
impl Middleware for SecurityMiddleware {
    /// Processes the request and applies rate limiting validation.
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Check rate limiting
        if let Some(client_ip) = self.get_client_ip(req) {
            if !self.check_rate_limit(client_ip) {
                warn!(ip = %client_ip, "Rate limit exceeded");
                return Err(crate::Error::BadRequest("Rate limit exceeded".to_string()));
            }
        }

        debug!("Security validations passed");
        Ok(())
    }

    /// Processes the response and adds security headers.
    async fn after(&self, _req: &Request, res: &mut Response) -> Result<()> {
        // Add HSTS header
        if self.enable_hsts {
            let mut hsts_value = format!("max-age={}", self.hsts_max_age);
            if self.hsts_include_subdomains {
                hsts_value.push_str("; includeSubDomains");
            }
            if self.hsts_preload {
                hsts_value.push_str("; preload");
            }
            res.headers.insert(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_str(&hsts_value).unwrap(),
            );
        }

        // Add Content Security Policy
        if self.enable_csp {
            let csp_value = self.build_csp_header();
            res.headers.insert(
                HeaderName::from_static("content-security-policy"),
                HeaderValue::from_str(&csp_value).unwrap(),
            );
        }

        // Add standard security headers
        if self.enable_security_headers {
            res.headers.insert(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );
            res.headers.insert(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
            res.headers.insert(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            res.headers.insert(
                HeaderName::from_static("permissions-policy"),
                HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
            );
            res.headers.insert(
                HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("0"), // Disabled as CSP is preferred
            );
        }

        // Remove server identification
        if self.remove_server_header {
            res.headers.remove("server");
            res.headers.remove("x-powered-by");
        }

        debug!("Security headers added to response");
        Ok(())
    }
}

// Preset configurations for common use cases
impl SecurityMiddleware {
    /// Creates security middleware optimized for API services.
    pub fn for_api() -> Self {
        Self::new().with_rate_limit(2000, Duration::from_secs(60)) // Higher limit for APIs
    }

    /// Creates security middleware for web applications.
    pub fn for_web() -> Self {
        Self::new()
            .with_rate_limit(500, Duration::from_secs(60))
            .with_csp(CspConfig {
                default_src: vec!["'self'".to_string()],
                script_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
                style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
                img_src: vec![
                    "'self'".to_string(),
                    "data:".to_string(),
                    "https:".to_string(),
                ],
                font_src: vec!["'self'".to_string(), "https:".to_string()],
                ..Default::default()
            })
    }

    /// Creates security middleware with maximum protection.
    pub fn high_security() -> Self {
        Self::new()
            .with_rate_limit(100, Duration::from_secs(60)) // Very restrictive
            .with_hsts_config(63072000, true, true) // 2 years with preload
            .with_csp(CspConfig {
                default_src: vec!["'none'".to_string()],
                script_src: vec!["'self'".to_string()],
                style_src: vec!["'self'".to_string()],
                img_src: vec!["'self'".to_string()],
                connect_src: vec!["'self'".to_string()],
                font_src: vec!["'self'".to_string()],
                object_src: vec!["'none'".to_string()],
                ..Default::default()
            })
    }

    /// Creates security middleware optimized for development.
    pub fn for_development() -> Self {
        Self::new()
            .with_rate_limit(10000, Duration::from_secs(60)) // Very permissive
            .with_hsts(false) // Disabled for HTTP development
    }
}
