use mini_web_framework::{
    Request, Response, Result, Router, Server, handler_fn,
    middleware::{AuthMiddleware, CorsMiddleware, LoggerMiddleware},
};
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .middleware(LoggerMiddleware)
        .middleware(CorsMiddleware::new())
        .middleware(AuthMiddleware::new("secret-token").protect_path("/protected"))
        .get("/public", handler_fn(public_route))
        .get("/protected", handler_fn(protected_route));

    let addr: SocketAddr = "127.0.0.1:3001".parse().unwrap();
    let server = Server::new(router, addr);

    println!("Server with middleware running on http://{}", addr);
    server.run().await.unwrap();

    Ok(())
}

async fn public_route(_req: Request) -> Result<Response> {
    Ok(Response::text("This is a public route"))
}

async fn protected_route(_req: Request) -> Result<Response> {
    Ok(Response::text(
        "This is a protected route - you're authenticated!",
    ))
}
