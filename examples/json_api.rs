use http::StatusCode;
use ignitia::middleware::Next;
use ignitia::{
    Extension, Json, Middleware, Request, Response, ResponseBuilder, Result, Router, Server,
};
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

// Middleware to inject TodoStore as an extension into each request
struct TodoStoreMiddleware {
    store: TodoStore,
}

impl TodoStoreMiddleware {
    fn new(store: TodoStore) -> Self {
        Self { store }
    }
}

#[ignitia::async_trait]
impl Middleware for TodoStoreMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        req.insert_extension(self.store.clone());
        next.run(req).await
    }
}

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

    let router = Router::new()
        .middleware(TodoStoreMiddleware::new(store.clone()))
        .get("/todos", list_todos)
        .post("/todos", create_todo);

    let app = Router::new().nest("/api", router);

    let addr: SocketAddr = "127.0.0.1:3002".parse().unwrap();
    let server = Server::new(app, addr);

    println!("JSON API server running on http://{}", addr);
    server.ignitia().await.unwrap();

    Ok(())
}

// Handler using Extension extractor
async fn list_todos(Extension(store): Extension<TodoStore>) -> Result<Response> {
    let todos = store.lock().unwrap();
    let todos_vec: Vec<Todo> = todos.values().cloned().collect();
    Ok(Response::json(todos_vec))
}

// Handler using both Json<T> and Extension extractors
async fn create_todo(
    Json(mut todo): Json<Todo>,
    Extension(store): Extension<TodoStore>,
) -> Result<Response> {
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
