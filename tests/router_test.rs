use bytes::Bytes;
use http::Method;
use ignite::{handler_fn, Request, Response, Router};

#[tokio::test]
async fn test_multiple_params() {
    let router = Router::new().get(
        "/users/:id/posts/:post_id",
        handler_fn(|req| async move {
            let user_id = req.param("id").unwrap();
            let post_id = req.param("post_id").unwrap();
            Ok(Response::text(format!(
                "User: {}, Post: {}",
                user_id, post_id
            )))
        }),
    );

    let req = Request::new(
        Method::GET,
        "/users/123/posts/456".parse().unwrap(),
        http::Version::HTTP_11,
        Default::default(),
        Bytes::new(),
    );

    let response = router.handle(req).await.unwrap();
    assert_eq!(response.body, Bytes::from("User: 123, Post: 456"));
}

#[tokio::test]
async fn test_method_matching() {
    let router = Router::new()
        .get("/test", handler_fn(|_| async { Ok(Response::text("GET")) }))
        .post(
            "/test",
            handler_fn(|_| async { Ok(Response::text("POST")) }),
        )
        .put("/test", handler_fn(|_| async { Ok(Response::text("PUT")) }))
        .delete(
            "/test",
            handler_fn(|_| async { Ok(Response::text("DELETE")) }),
        );

    for (method, expected) in [
        (Method::GET, "GET"),
        (Method::POST, "POST"),
        (Method::PUT, "PUT"),
        (Method::DELETE, "DELETE"),
    ] {
        let req = Request::new(
            method,
            "/test".parse().unwrap(),
            http::Version::HTTP_11,
            Default::default(),
            Bytes::new(),
        );

        let response = router.handle(req).await.unwrap();
        assert_eq!(response.body, Bytes::from(expected));
    }
}

#[tokio::test]
async fn test_custom_not_found_handler() {
    let router = Router::new()
        .get(
            "/exists",
            handler_fn(|_| async { Ok(Response::text("OK")) }),
        )
        .not_found(handler_fn(|_| async {
            Ok(Response::html("<h1>Custom 404</h1>"))
        }));

    let req = Request::new(
        Method::GET,
        "/nonexistent".parse().unwrap(),
        http::Version::HTTP_11,
        Default::default(),
        Bytes::new(),
    );

    let response = router.handle(req).await.unwrap();
    assert_eq!(response.body, Bytes::from("<h1>Custom 404</h1>"));
}
