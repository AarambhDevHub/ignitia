use crate::middleware::Middleware;
use crate::{Error, Request, Result};

pub struct AuthMiddleware {
    token: String,
    protected_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            protected_paths: Vec::new(),
        }
    }

    pub fn protect_path(mut self, path: impl Into<String>) -> Self {
        self.protected_paths.push(path.into());
        self
    }

    pub fn protect_paths(mut self, paths: Vec<impl Into<String>>) -> Self {
        for path in paths {
            self.protected_paths.push(path.into());
        }
        self
    }

    fn should_authenticate(&self, req: &Request) -> bool {
        let path = req.uri.path();
        self.protected_paths
            .iter()
            .any(|protected| path == protected || path.starts_with(&format!("{protected}/")))
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        // Only authenticate if this path requires it
        if !self.should_authenticate(req) {
            return Ok(());
        }

        let auth_header = req.header("Authorization").ok_or(Error::Unauthorized)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(Error::Unauthorized);
        }

        let token = &auth_header[7..];
        if token != self.token {
            return Err(Error::Unauthorized);
        }

        Ok(())
    }
}
