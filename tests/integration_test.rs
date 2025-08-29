use bytes::Bytes;
use http::Method;
use ignite::{handler_fn, Request, Response, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestUser {
    id: u32,
    name: String,
}

#[tokio::test]
async fn test_basic_routing() {
    let router = Router::new()
        .get("/test", handler_fn(|_| async { Ok(Response::text("GET")) }))
        .post(
            "/test",
            handler_fn(|_| async { Ok(Response::text("POST")) }),
        );

    let get_req = Request::new(
        Method::GET,
        "/test".parse().unwrap(),
        http::Version::HTTP_11,
        Default::default(),
        Bytes::new(),
    );

    let response = router.handle(get_req).await.unwrap();
    assert_eq!(response.status, http::StatusCode::OK);
    assert_eq!(response.body, Bytes::from("GET"));
}

#[tokio::test]
async fn test_route_params() {
    let router = Router::new().get(
        "/users/:id",
        handler_fn(|req| async move {
            let id = req.param("id").unwrap();
            Ok(Response::text(format!("User ID: {}", id)))
        }),
    );

    let req = Request::new(
        Method::GET,
        "/users/123".parse().unwrap(),
        http::Version::HTTP_11,
        Default::default(),
        Bytes::new(),
    );

    let response = router.handle(req).await.unwrap();
    assert_eq!(response.body, Bytes::from("User ID: 123"));
}

#[tokio::test]
async fn test_json_response() {
    let router = Router::new().get(
        "/user",
        handler_fn(|_| async {
            let user = TestUser {
                id: 1,
                name: "Alice".to_string(),
            };
            Response::json(user)
        }),
    );

    let req = Request::new(
        Method::GET,
        "/user".parse().unwrap(),
        http::Version::HTTP_11,
        Default::default(),
        Bytes::new(),
    );

    let response = router.handle(req).await.unwrap();
    assert_eq!(response.status, http::StatusCode::OK);

    let user: TestUser = serde_json::from_slice(&response.body).unwrap();
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice");
}

#[tokio::test]
async fn test_not_found() {
    let router = Router::new().get(
        "/exists",
        handler_fn(|_| async { Ok(Response::text("OK")) }),
    );

    let req = Request::new(
        Method::GET,
        "/nonexistent".parse().unwrap(),
        http::Version::HTTP_11,
        Default::default(),
        Bytes::new(),
    );

    let result = router.handle(req).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_query_params() {
    let router = Router::new().get(
        "/search",
        handler_fn(|req| async move {
            // let query = req.query("q").unwrap_or(&"".to_string());
            let query = req.query("q").map(|s| s.as_str()).unwrap_or("");
            Ok(Response::text(format!("Query: {}", query)))
        }),
    );

    let req = Request::new(
        Method::GET,
        "/search?q=rust".parse().unwrap(),
        http::Version::HTTP_11,
        Default::default(),
        Bytes::new(),
    );

    let response = router.handle(req).await.unwrap();
    assert_eq!(response.body, Bytes::from("Query: rust"));
}
