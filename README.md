# Ree HTTP Framework

🚀 A modern, simple and efficient Rust HTTP framework built on Hyper, featuring **zero-cost middleware** and **elegant macro-based development**.

## ✨ Features

- **🎯 Simple & Intuitive API**: Easy-to-use routing and handler system
- **🔄 Automatic Type Conversion**: Direct return of various types (String, JSON, Result, Option, etc.)
- **⚡ High Performance**: Built on Hyper, leveraging Rust's zero-cost abstractions
- **🛠 Powerful Middleware System**: Function-based middleware with macro support
- **📦 Route Groups**: Organize routes with prefixes and group-specific middleware
- **🔒 Type Safety**: Compile-time guarantees for request/response handling
- **🎨 Macro Magic**: `#[middleware]` macro for elegant middleware development

## Quick Start

### Add Dependency

```toml
[dependencies]
ree = { git = "https://github.com/secheng722/ree" }
tokio = { version = "1.45.1", features = ["full"] }
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

async fn custom_handler(_ctx: RequestCtx) -> Response {
    let mut response = ResponseBuilder::with_json(r#"{"message": "Custom response"}"#);
    response.headers_mut().insert("X-Custom-Header", "MyValue".parse().unwrap());
    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Precise response control
    app.get("/custom", custom_handler);
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

### 📦 Route Groups

Organize your routes with prefixes and group-specific middleware:

```rust
let mut app = Engine::new();

// API v1 group
let api_v1 = app.group("/api/v1");
api_v1.use_middleware(request_logger);
api_v1.use_middleware(auth("Bearer api-v1-token"));
api_v1.get("/users", handler(|_| async { "API v1 users" }));
api_v1.post("/users", handler(|_| async { "Create user in v1" }));

// API v2 group with different auth
let api_v2 = app.group("/api/v2");
api_v2.use_middleware(jwt_auth("v2-secret"));
api_v2.get("/users", handler(|_| async { "API v2 users" }));

// Admin group with multiple middleware
let admin = app.group("/admin");
admin.use_middleware(jwt_auth("admin-secret"));
admin.use_middleware(require_role("admin"));
admin.get("/users", handler(|_| async { "Admin users list" }));
admin.delete("/users/:id", handler(|ctx| async move {
    if let Some(id) = ctx.get_param("id") {
        format!("Deleted user {}", id)
    } else {
        "Invalid user ID".to_string()
    }
}));
```

## 🛠 Revolutionary Middleware System

### The `#[middleware]` Macro

Ree introduces a game-changing `#[middleware]` macro that makes middleware development incredibly simple and elegant:

```rust
use ree::{middleware, Engine, RequestCtx, Next, Response, ResponseBuilder};

// 🎯 Parameterized middleware - clean and simple!
#[middleware]
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == token {
            return next(ctx).await;
        }
    }
    ResponseBuilder::unauthorized_json(r#"{"error": "Unauthorized"}"#)
}

// 🎯 Simple middleware - consistent style
#[middleware]
async fn request_logger(ctx: RequestCtx, next: Next) -> Response {
    println!("📨 {} {}", ctx.request.method(), ctx.request.uri().path());
    let response = next(ctx).await;
    println!("✅ Response: {}", response.status());
    response
}

// 🎯 JWT Authentication - powerful yet simple
#[middleware]
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
    ResponseBuilder::unauthorized_json(r#"{"error": "Invalid or missing JWT token"}"#)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // Global middleware
    app.use_middleware(request_logger);
    
    // Route group with simple auth
    {
        let api = app.group("/api");
        api.use_middleware(auth("Bearer secret-token"));
        api.get("/users", handler(|_| async { "Protected users data" }));
    }
    
    // JWT protected routes
    {
        let secure = app.group("/secure");  
        secure.use_middleware(jwt_auth("my-secret-key"));
        secure.get("/profile", handler(|_| async { "User profile" }));
    }
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

### 🎨 More Middleware Examples

```rust
// Rate limiting with parameters
#[middleware]
async fn rate_limit(max_requests: usize, ctx: RequestCtx, next: Next) -> Response {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    
    let current = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if current >= max_requests {
        return ResponseBuilder::too_many_requests_json(
            r#"{"error": "Rate limit exceeded"}"#
        );
    }
    
    next(ctx).await
}

// CORS - simple version
#[middleware]
async fn cors(ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE".parse().unwrap());
    response
}

// CORS with custom origin
#[middleware]
async fn cors_custom(origin: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    response.headers_mut().insert("Access-Control-Allow-Origin", origin.parse().unwrap());
    response
}

// Request ID middleware
#[middleware]
async fn request_id(ctx: RequestCtx, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    println!("🆔 Request ID: {}", request_id);
    
    let mut response = next(ctx).await;
    response.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());
    response
}

// Usage examples
app.use_middleware(cors);                           // Simple CORS
app.use_middleware(cors_custom("https://app.com")); // Custom origin CORS
app.use_middleware(rate_limit(100));                // 100 requests limit
app.use_middleware(request_id);                     // Add request IDs
```

### 🚀 Why This Matters

**Before (traditional approach):**
```rust
fn auth(token: &'static str) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
    move |ctx, next| {
        Box::pin(async move {
            // Complex nested structure
            // Hard to read and write
        })
    }
}
```

**After (with `#[middleware]`):**
```rust
#[middleware]
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // Clean, readable async function
    // Just write your logic naturally!
}
```

The `#[middleware]` macro automatically handles:
- ✅ **Complex return types** - No more `Pin<Box<dyn Future<...>>>`
- ✅ **Parameter binding** - Clean parameter passing
- ✅ **Send/Sync bounds** - Automatic trait implementations
- ✅ **Type inference** - Rust compiler understands everything
```

## 🏃‍♂️ Running Examples

The repository includes comprehensive examples showcasing different aspects of the framework:

```bash
# Basic API usage
cargo run --example api_guide

# Comprehensive middleware examples
cargo run --example middleware_guide

# Large application structure
cargo run --example large_app_example
```

Then visit:
- **Basic Routes**: 
  - http://127.0.0.1:3000/ - Framework home page
  - http://127.0.0.1:3000/hello/John - Greeting with parameter
  - http://127.0.0.1:3000/health - Health check endpoint
  
- **Middleware Examples**:
  - http://127.0.0.1:3000/api/users - Simple auth protected route
  - http://127.0.0.1:3000/jwt/profile - JWT protected route
  - http://127.0.0.1:3000/admin/users - Admin role required
  
- **Authentication Testing**:
  ```bash
  # Get JWT tokens
  curl -X POST http://127.0.0.1:3000/auth/login
  
  # Test simple auth
  curl -H 'Authorization: Bearer secret-token' http://127.0.0.1:3000/api/users
  
  # Test JWT auth
  curl -H 'Authorization: Bearer <jwt_token>' http://127.0.0.1:3000/jwt/profile
  ```

## 📖 API Documentation

### Engine

The main application structure for configuring routes and middleware.

#### Methods

- `new()` - Create a new Engine instance
- `get(path, handler)` - Add GET route  
- `post(path, handler)` - Add POST route
- `put(path, handler)` - Add PUT route
- `delete(path, handler)` - Add DELETE route
- `group(prefix)` - Create route group with prefix
- `use_middleware(middleware)` - Add global middleware
- `run(addr)` - Start the server

### Middleware Macro

#### `#[middleware]`

Transform async functions into middleware with parameter support:

```rust
// Parameterized middleware
#[middleware]
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response { ... }

// Simple middleware  
#[middleware]
async fn logger(ctx: RequestCtx, next: Next) -> Response { ... }
```

**Requirements:**
- Function must be `async`
- Must have `ctx: RequestCtx` and `next: Next` parameters
- Must return `Response`
- Can have additional parameters before `ctx` and `next`

### ResponseBuilder

Utility for building HTTP responses with convenience methods.

#### Methods

- `with_text(content)` - Create text/plain response
- `with_json(content)` - Create application/json response
- `with_html(content)` - Create text/html response
- `empty()` - Create empty response (204 No Content)
- `not_found()` - Create 404 response
- `not_found_json(content)` - Create 404 JSON response
- `unauthorized_json(content)` - Create 401 JSON response
- `forbidden_json(content)` - Create 403 JSON response
- `bad_request_json(content)` - Create 400 JSON response
- `internal_server_error()` - Create 500 response
- `too_many_requests_json(content)` - Create 429 JSON response

### RequestCtx

Request context containing request information and extracted parameters.

#### Methods

- `get_param(key)` - Get path parameter by name
- `request` - Access to the underlying `hyper::Request<hyper::body::Incoming>`

### Route Groups

Organize routes with common prefixes and middleware:

```rust
let api = app.group("/api");
api.use_middleware(auth("Bearer token"));
api.get("/users", handler);
api.post("/users", handler);
```

## 🎯 Design Philosophy

### Simplicity First
- Use `handler()` with automatic type conversion for 99% of use cases
- The framework handles HTTP response complexity for you
- Write natural Rust code, get HTTP responses automatically

### Powerful When Needed
- Return `Response` directly for precise control over headers and status codes
- Flexible middleware system with macro support
- Zero-cost abstractions - pay only for what you use

### Developer Experience
- **Macro Magic**: `#[middleware]` eliminates complex type signatures
- **Type Safety**: Compile-time guarantees reduce runtime errors  
- **Intuitive API**: If it looks right, it probably works
- **Comprehensive Examples**: Learn by seeing real patterns

### Zero-Cost Middleware
- Function-based middleware compiles to efficient code
- No dynamic dispatch overhead
- Composable and reusable middleware components

## 📚 Examples Overview

The repository includes comprehensive examples demonstrating real-world usage patterns:

### 🔰 Basic Usage (`examples/api_guide`)
- Simple routing and handlers
- Automatic type conversion
- Path parameters and query handling
- JSON request/response handling

### 🛠 Middleware Guide (`examples/middleware_guide`) 
- **Authentication**: Simple token and JWT authentication
- **Authorization**: Role-based access control
- **Rate Limiting**: Request throttling with configurable limits
- **CORS**: Cross-origin resource sharing setup
- **Logging**: Request/response logging and timing
- **Error Handling**: Centralized error processing
- **Request ID**: Tracing and debugging support

### 🏗 Large App Example (`examples/large_app_example`)
- Modular application structure
- Separation of concerns
- Route organization patterns  
- Middleware composition strategies
- Configuration management

### Key Middleware Patterns Shown

```rust
// Authentication patterns
#[middleware]
async fn simple_auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response { ... }

#[middleware] 
async fn jwt_auth(secret: &'static str, ctx: RequestCtx, next: Next) -> Response { ... }

// Rate limiting
#[middleware]
async fn rate_limit(max_requests: usize, ctx: RequestCtx, next: Next) -> Response { ... }

// CORS handling
#[middleware]
async fn cors_custom(origin: &'static str, ctx: RequestCtx, next: Next) -> Response { ... }

// Request processing
#[middleware]
async fn request_logger(ctx: RequestCtx, next: Next) -> Response { ... }
```

## 🚀 Performance

- **Zero-cost abstractions**: Middleware compiles to efficient code
- **Built on Hyper**: Leverages one of the fastest HTTP implementations in Rust
- **Minimal overhead**: Direct function calls, no dynamic dispatch
- **Memory efficient**: Stack-allocated middleware chains when possible

## 🤝 Contributing

Contributions are welcome! We're especially interested in:

- **New middleware examples** - Show off creative uses of the `#[middleware]` macro
- **Performance improvements** - Keep it fast and zero-cost
- **Documentation** - Help others learn the framework
- **Testing** - Ensure reliability and correctness

Please feel free to submit issues and pull requests.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ in Rust** 🦀
