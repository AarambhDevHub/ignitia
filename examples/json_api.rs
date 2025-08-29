use http::StatusCode;
use ignite::{handler_fn, Request, Response, ResponseBuilder, Result, Router, Server};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

type TodoStore = Arc<Mutex<HashMap<u32, Todo>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let store = Arc::new(Mutex::new(HashMap::new()));

    // Add some initial todos
    {
        let mut todos = store.lock().unwrap();
        todos.insert(
            1,
            Todo {
                id: 1,
                title: "Learn Rust".to_string(),
                completed: false,
            },
        );
        todos.insert(
            2,
            Todo {
                id: 2,
                title: "Build web framework".to_string(),
                completed: true,
            },
        );
    }

    let store_clone = store.clone();
    let router = Router::new()
        .get(
            "/api/todos",
            handler_fn(move |req| {
                let store = store.clone();
                async move { list_todos(req, store).await }
            }),
        )
        .post(
            "/api/todos",
            handler_fn(move |req| {
                let store = store_clone.clone();
                async move { create_todo(req, store).await }
            }),
        );

    let addr: SocketAddr = "127.0.0.1:3002".parse().unwrap();
    let server = Server::new(router, addr);

    println!("JSON API server running on http://{}", addr);
    server.run().await.unwrap();

    Ok(())
}

async fn list_todos(_req: Request, store: TodoStore) -> Result<Response> {
    let todos = store.lock().unwrap();
    let todos_vec: Vec<Todo> = todos.values().cloned().collect();
    Response::json(todos_vec)
}

async fn create_todo(req: Request, store: TodoStore) -> Result<Response> {
    let mut todo: Todo = req.json()?;
    let mut todos = store.lock().unwrap();

    let new_id = todos.keys().max().unwrap_or(&0) + 1;
    todo.id = new_id;
    todos.insert(new_id, todo.clone());

    let response = ResponseBuilder::new()
        .status(StatusCode::CREATED)
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&todo).unwrap())
        .build();

    Ok(response)
}
