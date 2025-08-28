# s_web HTTP Framework

📖 **中文文档**: [README_CN.md](README_CN.md)

🚀 A modern, simple and efficient Rust HTTP framework built on Hyper, featuring **zero-cost functional middleware**, **elegant development experience**, and **comprehensive API documentation**.

## ✨ Features

- **🎯 Unified API Design**: Direct return support for various types with automatic conversion
- **🔄 Automatic Type Conversion**: Return String, JSON, Result, Option, tuples directly
- **⚡ High Performance**: Built on Hyper with zero-cost abstractions
- **🧩 Functional Middleware System**: Elegant parameter-passing middleware with chain calls
- **📦 Route Groups**: Organize routes with prefixes and group-specific middleware
- **� Chain Calls**: Support chain calls for both middleware and routing
- **�🔒 Type Safety**: Compile-time guarantees for request/response handling
- **� Body Handling**: Easy POST/PUT request body parsing (JSON, text, bytes, form)
- **🛑 Lifecycle Management**: Startup and shutdown hooks for resource management
- **📖 Swagger Integration**: Built-in Swagger UI with custom documentation support

## 🚀 Quick Start

### Add Dependencies

```toml
[dependencies]
s_web = { git = "https://github.com/secheng722/s_web" }
tokio = { version = "1.45.1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Basic Usage

```rust
use s_web::Engine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Direct return types - automatic conversion
    app.get("/text", |_| async { "Hello, World!" });
    app.get("/json", |_| async { 
        json!({"message": "Hello JSON", "framework": "s_web"})
    });
    
    // Path parameters
    app.get("/greet/:name", |ctx| async move {
        let name = ctx.get_param("name").unwrap_or("Guest");
        format!("Hello, {}!", name)
    });
    
    // Result handling - Ok -> 200, Err -> 500
    app.get("/result", |_| async {
        let result: Result<&str, &str> = Ok("Success!");
        result
    });
    
    // Option handling - Some -> 200, None -> 404
    app.get("/option", |_| async {
        let data: Option<&str> = Some("Found!");
        data
    });
    
    // Custom status codes
    app.post("/create", |_| async {
        (s_web::StatusCode::CREATED, json!({"id": 123}))
    });
    
    app.run("127.0.0.1:8080").await
}
```

## 🔗 Chain Calls & Middleware

s_web supports elegant chain calls for both routing and middleware:

```rust
use s_web::{Engine, RequestCtx, Next, Response, IntoResponse};
use serde_json::json;

// Middleware functions with parameters
async fn logger(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
    println!("[{}] {} {}", prefix, ctx.request.method(), ctx.request.uri().path());
    let start = std::time::Instant::now();
    let response = next(ctx).await;
    println!("[{}] Response: {} ({}ms)", prefix, response.status(), start.elapsed().as_millis());
    response
}

async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == format!("Bearer {token}") {
            return next(ctx).await;
        }
    }
    (s_web::StatusCode::UNAUTHORIZED, json!({"error": "Unauthorized"})).into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Global middleware with chain calls
    app.use_middleware(|ctx, next| logger("Global", ctx, next))
       .get("/", |_| async { "Welcome!" })
       .get("/health", |_| async { json!({"status": "ok"}) });
    
    // API group with chained middleware and routes
    {
        let api = app.group("/api");
        api.use_middleware(|ctx, next| logger("API", ctx, next))
           .use_middleware(|ctx, next| auth("api-token", ctx, next))
           .get("/users", |_| async { json!(["alice", "bob"]) })
           .post("/users", |_| async { json!({"message": "User created"}) })
           .get("/profile", |_| async { json!({"name": "Current User"}) });
    }
    
    app.run("127.0.0.1:8080").await
}
```

## 📮 Request Body Handling

Easy POST/PUT request body parsing:

```rust
app.post("/json", |ctx: s_web::RequestCtx| async move {
    match ctx.body_json::<serde_json::Value>() {
        Ok(Some(json)) => format!("Received: {}", json),
        Ok(None) => "No body provided".to_string(),
        Err(e) => format!("Parse error: {}", e),
    }
});

app.post("/text", |ctx: s_web::RequestCtx| async move {
    match ctx.body_string() {
        Ok(Some(text)) => format!("Text: {}", text),
        Ok(None) => "No body".to_string(),
        Err(e) => format!("Error: {}", e),
    }
});

app.post("/bytes", |ctx: s_web::RequestCtx| async move {
    match ctx.body_bytes() {
        Some(bytes) => format!("Received {} bytes", bytes.len()),
        None => "No body".to_string(),
    }
});
```

## 🛑 Lifecycle Management

Built-in startup and shutdown hooks:

```rust
use s_web::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Engine::new()
        .on_startup(|| async {
            println!("🚀 Initializing database...");
            // Initialize database, cache, etc.
        })
        .on_startup(|| async {
            println!("🔥 Warming up system...");
            // Additional startup tasks
        })
        .on_shutdown(|| async {
            println!("🛑 Cleaning up resources...");
            // Cleanup database connections, etc.
        })
        .on_shutdown(|| async {
            println!("✅ Shutdown complete");
            // Final cleanup
        });
    
    let mut app = app;
    app.get("/", |_| async { "Hello!" });
    
    app.run("127.0.0.1:8080").await
}
```

## 📖 Swagger Documentation

Built-in Swagger UI with custom documentation support:

```rust
use s_web::{Engine, swagger};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Auto-generated documentation
    app.get("/users", get_users);
    
    // Custom Swagger documentation
    app.post_with_swagger(
        "/users",
        create_user,
        swagger()
            .summary("Create a new user")
            .description("Creates a new user with the provided data")
            .tag("Users")
            .request_body(json!({"name": "John", "email": "john@example.com"}))
            .json_response("201", "User created", Some(json!({"id": 1, "name": "John"})))
            .build()
    );
    
    // Swagger UI available at: http://127.0.0.1:3000/swagger-ui
    // OpenAPI JSON at: http://127.0.0.1:3000/api-docs
    
    app.run("127.0.0.1:3000").await
}
```

## 🎯 Advanced Usage

### Custom Response Builder

For precise control over HTTP responses:

```rust
use s_web::ResponseBuilder;

app.get("/custom", |_| async {
    ResponseBuilder::new()
        .status(s_web::StatusCode::OK)
        .content_type("application/json; charset=utf-8")
        .header("X-Custom-Header", "s_web-Framework")
        .body(r#"{"message": "Custom response"}"#)
});

app.get("/html", |_| async {
    ResponseBuilder::html(r#"
        <h1>Hello from s_web!</h1>
        <p>This is an HTML response</p>
    "#)
});
```

### Route Groups with Nested Middleware

```rust
// Admin routes with authentication
{
    let admin = app.group("/admin");
    admin.use_middleware(|ctx, next| auth("admin-token", ctx, next))
         .get("/dashboard", |_| async { "Admin Dashboard" })
         .delete("/users/:id", |ctx| async move {
             let id = ctx.get_param("id").unwrap_or("0");
             format!("Deleted user {}", id)
         });
}
```

## 📋 Examples

The repository includes comprehensive examples:

- **`api_example`**: Unified API design showcasing automatic type conversion
- **`chain_example`**: Chain calls for middleware and routing
- **`lifecycle_example`**: Startup/shutdown hooks and resource management
- **`swagger_custom_example`**: Custom Swagger documentation
- **`middleware_example`**: Advanced middleware patterns
- **`database_example`**: Database integration
- **`upload_example`**: File upload handling

## 🚦 Testing

Test with curl:

```bash
# JSON body
curl -X POST http://127.0.0.1:8080/api/users \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer api-token" \
     -d '{"name": "Alice", "email": "alice@example.com"}'

# Form data
curl -X POST http://127.0.0.1:8080/form \
     -d "name=Alice&email=alice@example.com"

# File upload
curl -X POST http://127.0.0.1:8080/upload \
     -F "file=@example.txt"
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.