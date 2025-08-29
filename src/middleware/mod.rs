pub mod auth;
pub mod cors;
pub mod logger;

use crate::{Request, Response, Result};

#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, req: &mut Request) -> Result<()> {
        Ok(())
    }

    async fn after(&self, res: &mut Response) -> Result<()> {
        Ok(())
    }
}

pub use self::auth::AuthMiddleware;
pub use self::cors::CorsMiddleware;
pub use self::logger::LoggerMiddleware;
