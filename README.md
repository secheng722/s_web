# Ree HTTP Framework

A simple and efficient Rust HTTP framework built on Hyper, providing clean APIs and powerful type conversion features.

## Features

- **Simple & Intuitive API**: Easy-to-use routing and handler system
- **Automatic Type Conversion**: Direct return of various types (String, JSON, Result, Option, etc.)
- **High Performance**: Built on Hyper, leveraging Rust's zero-cost abstractions
- **Middleware Support**: Flexible middleware system for cross-cutting concerns
- **Route Groups**: Organize routes with prefixes and group-specific middleware
- **Type Safety**: Compile-time guarantees for request/response handling

## Quick Start

### Add Dependency

```toml
[dependencies]
ree = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### Simple Handler Approach (Recommended)

```rust
use ree::{Engine, handler};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Return &str directly - automatically converted to text/plain response
    app.get("/hello", handler(|_| async { "Hello, World!" }));
    
    // Return String directly
    app.get("/time", handler(|_| async { 
        format!("Current time: {}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs())
    }));
    
    // Return JSON directly - automatically converted to application/json response
    app.get("/json", handler(|_| async { 
        json!({
            "message": "Hello JSON",
            "status": "success"
        })
    }));
    
    // Use path parameters
    app.get("/hello/:name", handler(|ctx| async move {
        if let Some(name) = ctx.get_param("name") {
            format!("Hello, {}!", name)
        } else {
            "Hello, Anonymous!".to_string()
        }
    }));
    
    // Return Result - automatically handle errors
    app.get("/result", handler(|_| async {
        let result: Result<&str, &str> = Ok("Success!");
        result  // Ok -> 200, Err -> 500
    }));
    
    // Return Option - automatically handle None
    app.get("/option", handler(|_| async {
        let data: Option<&str> = Some("Found!");
        data  // Some -> 200, None -> 404
    }));
    
    // Custom status code
    app.get("/created", handler(|_| async {
        (ree::StatusCode::CREATED, "Resource created")
    }));
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

### Advanced Usage - Precise Response Control

When you need precise control over response headers, status codes, etc., you can return `Response` directly:

```rust
use ree::{Engine, ResponseBuilder, RequestCtx, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Precise response control
    app.get("/custom", custom_handler);
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}

async fn custom_handler(_ctx: RequestCtx) -> Response {
    let mut response = ResponseBuilder::with_json(r#"{"message": "Custom response"}"#);
    response.headers_mut().insert("X-Custom-Header", "MyValue".parse().unwrap());
    response
}
```

### Middleware

```rust
use ree::{Engine, AccessLog};

let mut app = Engine::new();
app.use_middleware(AccessLog);
```

### Route Groups

```rust
let api_group = app.group("/api");
api_group.get("/users", get_users_handler);
api_group.get("/users/:id", get_user_by_id_handler);
```

## Running Examples

```bash
cargo run --example hello_world
```

Then visit:
- http://127.0.0.1:8080/ - Basic greeting
- http://127.0.0.1:8080/hello/John - Greeting with parameter
- http://127.0.0.1:8080/api/users - Get user list
- http://127.0.0.1:8080/api/users/1 - Get specific user

## API Documentation

### Engine

The main application structure for configuring routes and middleware.

#### Methods

- `new()` - Create a new Engine instance
- `get(path, handler)` - Add GET route
- `post(path, handler)` - Add POST route
- `put(path, handler)` - Add PUT route
- `delete(path, handler)` - Add DELETE route
- `group(prefix)` - Create route group
- `use_middleware(middleware)` - Add middleware
- `run(addr)` - Start the server

### ResponseBuilder

Utility for building HTTP responses.

#### Methods

- `with_text(content)` - Create text response
- `with_json(content)` - Create JSON response
- `empty()` - Create empty response
- `not_found()` - Create 404 response
- `internal_server_error()` - Create 500 response

### RequestCtx

Request context containing request information and path parameters.

#### Methods

- `get_param(key)` - Get path parameter
- `request` - Access to the underlying HTTP request

## Design Philosophy

### Simple First
Use `handler()` for automatic type conversion in 99% of cases. The framework handles the complexity of HTTP response generation for you.

### Flexible Control
Return `Response` directly when you need precise control over headers, status codes, or response format.

### Type Safety
Compile-time guarantees ensure your request/response handling is correct, reducing runtime errors.

## Examples

The repository includes comprehensive examples:

- **Basic Usage**: Simple routing and handlers
- **Middleware Guide**: Comprehensive middleware examples including auth, logging, CORS, rate limiting
- **Large App Example**: Modular application structure for scalable projects

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License.
