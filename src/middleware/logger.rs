use crate::middleware::Middleware;
use crate::{Request, Response, Result};
use tracing::info;

pub struct LoggerMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggerMiddleware {
    async fn before(&self, req: &mut Request) -> Result<()> {
        let version_str = match req.version {
            http::Version::HTTP_09 => "HTTP/0.9",
            http::Version::HTTP_10 => "HTTP/1.0",
            http::Version::HTTP_11 => "HTTP/1.1",
            http::Version::HTTP_2 => "HTTP/2.0",
            http::Version::HTTP_3 => "HTTP/3.0",
            _ => "UNKNOWN",
        };

        info!("{} {} {}", req.method, req.uri.path(), version_str);
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        info!("Response: {}", res.status.as_u16());
        Ok(())
    }
}
