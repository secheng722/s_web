# s_web

📖 **中文文档**: [README_CN.md](README_CN.md)

A modern, lightweight Rust HTTP framework built on top of [Hyper](https://github.com/hyperium/hyper).  
Designed around plain async functions — no macros, no boilerplate, no magic.

```toml
[dependencies]
s_web  = { git = "https://github.com/secheng722/s_web" }
tokio  = { version = "1", features = ["full"] }
serde_json = "1"
```

---

## Features

| | |
|---|---|
| **Zero-boilerplate handlers** | Return `&str`, `String`, `serde_json::Value`, `(StatusCode, T)`, `Result`, `Option` directly |
| **Functional middleware** | Plain `async fn(ctx, next) -> Response` — no traits, no wrappers |
| **Route groups** | Prefix-scoped groups with per-group middleware |
| **Lifecycle hooks** | `on_startup` / `on_shutdown` for resource init & cleanup |
| **Swagger UI** | Built-in `/docs/` UI with `swagger()` builder for documentation |
| **Graceful shutdown** | Ctrl-C signal handling with configurable drain timeout |

---

## Quick Start

```rust
use s_web::{Engine, RequestCtx};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    app.get("/",           |_: RequestCtx| async { "Hello, World!" });
    app.get("/hello/:name", |ctx: RequestCtx| async move {
        format!("Hello, {}!", ctx.get_param("name").map(|s| s.as_str()).unwrap_or("stranger"))
    });
    app.get("/json", |_: RequestCtx| async {
        json!({ "framework": "s_web", "status": "ok" })
    });

    app.run("127.0.0.1:3000").await
}
```

---

## Middleware

Middleware is a plain async function — capture whatever you need via closures:

```rust
use s_web::{Engine, Next, RequestCtx, Response};
use std::time::Instant;

async fn logging(ctx: RequestCtx, next: Next) -> Response {
    let path  = ctx.request.uri().path().to_owned();
    let start = Instant::now();
    let resp  = next(ctx).await;
    println!("{} → {} ({:.1}ms)", path, resp.status(), start.elapsed().as_secs_f64() * 1000.0);
    resp
}

async fn require_api_key(ctx: RequestCtx, next: Next) -> Response {
    use s_web::{IntoResponse, StatusCode};
    use serde_json::json;
    match ctx.header("x-api-key") {
        Some(k) if k == "secret" => next(ctx).await,
        _ => (StatusCode::UNAUTHORIZED, json!({ "error": "invalid API key" })).into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    // Global middleware
    app.use_middleware(logging);

    // Route group with its own middleware
    {
        let api = app.group("/api/v1");
        api.use_middleware(require_api_key);
        api.get("/users", |_: RequestCtx| async { serde_json::json!(["alice", "bob"]) });
    }

    app.run("127.0.0.1:3000").await
}
```

---

## Route Groups

```rust
{
    let admin = app.group("/admin");
    admin.use_middleware(require_api_key);
    admin.get("/dashboard", |_: RequestCtx| async { "Dashboard" });
    admin.delete("/users/:id", |ctx: RequestCtx| async move {
        format!("deleted {}", ctx.get_param("id").unwrap())
    });
}
```

---

## Request Body

```rust
// JSON
app.post("/users", |mut ctx: RequestCtx| async move {
    #[derive(serde::Deserialize)]
    struct Payload { name: String }

    let p: Payload = ctx.json().await?;
    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(format!("Hello, {}", p.name))
});

// Query params
app.get("/search", |ctx: RequestCtx| async move {
    let q = ctx.query_param("q").unwrap_or_default();
    format!("search: {q}")
});
```

---

## Lifecycle Hooks

```rust
let app = Engine::new()
    .on_startup(|| async { println!("DB connected") })
    .on_shutdown(|| async { println!("DB closed") });
```

---

## Swagger UI

```rust
use s_web::{Engine, RequestCtx, swagger};
use serde_json::json;

let mut app = Engine::new();
app.enable_swagger();   // mounts /docs/ and /docs/swagger.json

app.get_with_swagger(
    "/users",
    |_: RequestCtx| async { json!([]) },
    swagger()
        .summary("List users")
        .tag("Users")
        .json_response("200", "User list", Some(json!([])))
        .build(),
);
// Open http://127.0.0.1:3000/docs/
```

---

## Custom Responses

```rust
use s_web::{ResponseBuilder, StatusCode};

// HTML
ResponseBuilder::html("<h1>Hello</h1>");

// Full control
ResponseBuilder::new()
    .status(StatusCode::CREATED)
    .content_type("application/json; charset=utf-8")
    .header("X-Request-Id", "abc123")
    .body(r#"{"id":1}"#);
```

---

## Examples

Eight runnable projects, each a self-contained Cargo package — copy any one out and use it standalone.

| # | Directory | Topics | Run |
|---|-----------|--------|-----|
| 1 | [01_hello_world](examples/01_hello_world) | Routes, path params, HTML | `cargo run -p example_hello_world` |
| 2 | [02_json_api](examples/02_json_api) | JSON, query params, POST body | `cargo run -p example_json_api` |
| 3 | [03_middleware](examples/03_middleware) | Global & group middleware, auth | `cargo run -p example_middleware` |
| 4 | [04_todo_app](examples/04_todo_app) | Shared state, full CRUD, lifecycle hooks | `cargo run -p example_todo_app` |
| 5 | [05_swagger](examples/05_swagger) | Swagger UI, bearer auth docs | `cargo run -p example_swagger` |
| 6 | [06_sqlx_sqlite_crud](examples/06_sqlx_sqlite_crud) | sqlx + SQLite, connection pool | `cargo run -p example_sqlx_sqlite_crud` |
| 7 | [07_seaorm_sqlite_crud](examples/07_seaorm_sqlite_crud) | SeaORM entity, auto migration | `cargo run -p example_seaorm_sqlite_crud` |
| 8 | [08_mini_blog](examples/08_mini_blog) | Multi-file mini blog, layered architecture, sqlx + SQLite | `cargo run -p mini_blog` |

---

## Contributing

Pull requests are welcome.

