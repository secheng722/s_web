# REE

REE is a lightweight, high-performance HTTP framework for Rust, inspired by Express, Echo, and Koa. It integrates Hyper into a familiar API with a focus on simplicity, flexibility, and performance.

[![Crates.io](https://img.shields.io/crates/v/ree.svg)](https://crates.io/crates/ree)
[![Documentation](https://docs.rs/ree/badge.svg)](https://docs.rs/ree)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://github.com/rust-lib-labs/ree/actions/workflows/rust.yml/badge.svg)](https://github.com/rust-lib-labs/ree/actions/workflows/rust.yml)

## Features

- **Simple API**: Familiar Express/Koa-like API for quick adoption
- **Performance**: Built on Hyper for high performance
- **Middleware**: Elegant, functional middleware system
- **Routing**: Flexible routing including groups and parameters
- **Type Safety**: Strong typing throughout the framework
- **JSON**: Seamless JSON serialization/deserialization
- **Static Files**: Easily serve static files
- **WebSockets**: WebSocket support via Tokio Tungstenite
- **Graceful shutdown**: Handle shutdown signals gracefully
- **Modular Design**: Build only what you need

## Quick Example

```rust
use ree::{
    http::{Request, Response, StatusCode},
    App, Engine, RequestCtx,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new().get("/", hello);
    
    // Simple functional middleware example
    let app = app.use_middleware(|ctx, next| logging("Logger", ctx, next));

    Engine::builder().run(app).await?;
    Ok(())
}

async fn hello(ctx: RequestCtx) -> Response {
    Response::new().text("Hello, World!")
}

// Functional middleware with parameters
async fn logging(prefix: &str, ctx: RequestCtx, next: ree::Next) -> Response {
    println!("{}: {} {}", prefix, ctx.req.method(), ctx.req.uri().path());
    let res = next.run(ctx).await;
    println!("{}: Response status: {}", prefix, res.status());
    res
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ree = { git = "https://github.com/secheng722/ree" }
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

## Middleware System

REE features an elegant functional middleware system with zero overhead:

```rust
// Simple middleware without parameters
async fn logger(ctx: RequestCtx, next: Next) -> Response {
    println!("Request: {} {}", ctx.request.method(), ctx.request.uri().path());
    let start = std::time::Instant::now();
    let response = next(ctx).await;
    println!("Response: {} - {:?}", response.status(), start.elapsed());
    response
}

// Middleware with parameters
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == format!("Bearer {}", token) {
            return next(ctx).await;
        }
    }
    ResponseBuilder::unauthorized_json(r#"{"error":"Unauthorized"}"#)
}

// Usage in application
let mut app = Engine::new();

// Apply middleware directly
app.use_middleware(logger);

// Apply middleware with parameters using closure
app.use_middleware(|ctx, next| auth("secret-token", ctx, next));

// Route group with middleware
let mut api_group = app.group("/api");
api_group.use_middleware(|ctx, next| auth("api-token", ctx, next));
```

This approach is more intuitive, flexible and composable than traditional middleware systems while maintaining type safety and zero runtime overhead.

### Middleware Patterns

REE's functional middleware system is both powerful and flexible:

#### Middleware Composition

```rust
// Chain multiple middlewares together
app.use_middleware(|ctx, next| async move {
    // First middleware logic
    println!("Middleware 1: Pre-processing");
    let response = next(ctx).await;
    println!("Middleware 1: Post-processing");
    response
});

app.use_middleware(|ctx, next| async move {
    // Second middleware runs after the first one
    println!("Middleware 2: Pre-processing");
    let response = next(ctx).await;
    println!("Middleware 2: Post-processing");
    response
});
```

#### Conditional Middleware

```rust
// Apply middleware conditionally
let auth_middleware = if config.is_production {
    |ctx, next| auth("production-token", ctx, next)
} else {
    |ctx, next| auth("development-token", ctx, next)
};

app.use_middleware(auth_middleware);
```

#### Error Handling Middleware

```rust
async fn error_handler(ctx: RequestCtx, next: Next) -> Response {
    let response = next(ctx).await;
    
    if response.status().is_server_error() {
        // Log errors, send alerts, etc.
        println!("Server error: {}", response.status());
    }
    
    response
}

// Apply as early middleware to catch all errors
app.use_middleware(error_handler);
```

This functional approach to middleware provides superior flexibility while maintaining
the performance and type safety that Rust developers expect.
