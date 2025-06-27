# Ree HTTP Framework

📖 **中文文档**: [README_CN.md](README_CN.md)

🚀 A modern, simple and efficient Rust HTTP framework built on Hyper, featuring **zero-cost functional middleware** and an **elegant development** experience.

## ✨ Features

- **🎯 Simple and Intuitive API**: Easy-to-use routing and handler system
- **🔄 Automatic Type Conversion**: Directly return various types (String, JSON, Result, Option, etc.)
- **⚡ High Performance**: Built on Hyper, leveraging Rust's zero-cost abstractions
- **🧩 Powerful Middleware System**: Function-based pure middleware, simple and elegant
- **📦 Route Groups**: Organize routes with prefixes and group-specific middleware
- **🔒 Type Safety**: Compile-time guarantees for request/response handling correctness
- **🔗 Functional Style**: Intuitive functional middleware makes development effortless and natural
- **🛑 Graceful Shutdown**: Supports graceful shutdown to safely close HTTP server while ensuring in-flight requests can complete
- **📖 Automatic Swagger Support**: Automatically generates Swagger documentation for all registered routes and provides an interactive API documentation via Swagger UI (due to certain limitations, POST request JSON data may need manual adjustment during testing).

## 🚀 Quick Start

### Add Dependencies

```toml
[dependencies]
ree = { git = "https://github.com/secheng722/ree" }
tokio = { version = "1.45.1", features = ["full"] }
```

### Simple Handler Examples

```rust
use ree::Engine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Return &str directly - auto-converts to text/plain response
    app.get("/hello", |_| async { "Hello, World!" });
    
    // Return JSON directly - auto-converts to application/json response
    app.get("/json", |_| async { 
        json!({
            "message": "Hello JSON",
            "framework": "Ree",
            "version": "0.1.0"
        })
    });
    
    // Use path parameters
    app.get("/hello/:name", |ctx| async move {
        if let Some(name) = ctx.get_param("name") {
            format!("Hello, {}!", name)
        } else {
            "Hello, Anonymous!".to_string()
        }
    });
    
    // Return Result - auto-handles errors
    app.get("/result", |_| async {
        let result: Result<&str, &str> = Ok("Success!");
        result  // Ok -> 200, Err -> 500
    });

    app.get("/option", |_| async {
        let data: Option<&str> = Some("Found!");
        data  // Some -> 200, None -> 404
    });

    app.get("/created", |_| async {
        (ree::StatusCode::CREATED, "Resource created")
    });


    //Chain call
    //Because middleware and routing support chain calls, you can organize your code more flexibly
    //The system will handle the execution order of the middleware itself
    app.get("/chained", handler(|_| async {
        "This is a chained response"
    }).get("/another", |_| async {
        "Another chained response"
    }).use_middleware(|ctx, next| async move {
        println!("Middleware executed");
        next(ctx).await
    });
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

## 🛠 Elegant Functional Middleware System

Ree introduces an extremely simple and elegant functional middleware system that makes writing and using middleware unprecedentedly easy:

```rust
use ree::{Engine, RequestCtx, Next, Response, ResponseBuilder};

// 🎯 Parameterized middleware - simple yet powerful!
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == format!("Bearer {}", token) {
            return next(ctx).await;
        }
    }
    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "Unauthorized"}),
    )
        .into_response()
}

// 🎯 Logger middleware - simple and intuitive
async fn logger(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
    println!("[{}] 📨 {} {}", prefix, ctx.request.method(), ctx.request.uri().path());
    let start = std::time::Instant::now();
    let response = next(ctx).await;
    println!("[{}] ✅ Response: {} ({}ms)", prefix, response.status(), start.elapsed().as_millis());
    response
}

// 🎯 JWT authentication - powerful yet simple
async fn jwt_auth(secret: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth_header) = ctx.request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if validate_jwt_token(token, secret) {
                    return next(ctx).await;
                }
            }
        }
    }

    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "Invalid or missing JWT token"}),
    )
        .into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Global middleware - use closures to pass parameters
    app.use_middleware(|ctx, next| logger("Global", ctx, next));
    
    // Route group with simple auth
    {
        let api = app.group("/api");
        api.use_middleware(|ctx, next| auth("secret-token", ctx, next));
        api.get("/users", |_| async { "Protected user data" });
    }
    
    // JWT protected routes
    {
        let secure = app.group("/secure");  
        secure.use_middleware(|ctx, next| jwt_auth("my-secret-key", ctx, next));
        secure.get("/profile", |_| async { "User profile" });
    }
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```