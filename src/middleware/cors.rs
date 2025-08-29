use crate::middleware::Middleware;
use crate::{Response, Result};
use http::header;

pub struct CorsMiddleware {
    allow_origin: String,
    allow_methods: String,
    allow_headers: String,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            allow_headers: "Content-Type, Authorization".to_string(),
        }
    }

    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allow_origin = origin.into();
        self
    }
}

#[async_trait::async_trait]
impl Middleware for CorsMiddleware {
    async fn after(&self, res: &mut Response) -> Result<()> {
        res.headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            self.allow_origin.parse().unwrap(),
        );
        res.headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            self.allow_methods.parse().unwrap(),
        );
        res.headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            self.allow_headers.parse().unwrap(),
        );
        Ok(())
    }
}
