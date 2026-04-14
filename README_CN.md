# s_web

📖 **English Docs**: [README.md](README.md)

现代化、轻量级的 Rust HTTP 框架，基于 [Hyper](https://github.com/hyperium/hyper) 构建。  
以普通 async 函数为核心设计——无宏、无模板、无黑魔法。

```toml
[dependencies]
s_web      = { git = "https://github.com/secheng722/s_web" }
tokio      = { version = "1", features = ["full"] }
serde_json = "1"
```

---

## 特性

| | |
|---|---|
| **零模板处理器** | 直接返回 `&str`、`String`、`serde_json::Value`、`(StatusCode, T)`、`Result`、`Option` |
| **函数式中间件** | 普通 `async fn(ctx, next) -> Response`——无 trait、无包装器 |
| **路由分组** | 前缀作用域分组，支持组级独立中间件 |
| **生命周期钩子** | `on_startup` / `on_shutdown` 用于资源初始化与清理 |
| **Swagger UI** | 内置 `/docs/` 界面，搭配 `swagger()` builder 生成文档 |
| **优雅关闭** | Ctrl-C 信号处理，带可配置排空超时 |

---

## 快速开始

```rust
use s_web::{Engine, RequestCtx};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    app.get("/",            |_: RequestCtx| async { "Hello, World!" });
    app.get("/hello/:name", |ctx: RequestCtx| async move {
        format!("你好, {}!", ctx.get_param("name").map(|s| s.as_str()).unwrap_or("陌生人"))
    });
    app.get("/json", |_: RequestCtx| async {
        json!({ "framework": "s_web", "status": "ok" })
    });

    app.run("127.0.0.1:3000").await
}
```

---

## 中间件

中间件是普通 async 函数，通过闭包捕获所需上下文：

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
        _ => (StatusCode::UNAUTHORIZED, json!({ "error": "无效的 API Key" })).into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    // 全局中间件
    app.use_middleware(logging);

    // 路由分组，绑定独立中间件
    {
        let api = app.group("/api/v1");
        api.use_middleware(require_api_key);
        api.get("/users", |_: RequestCtx| async { serde_json::json!(["alice", "bob"]) });
    }

    app.run("127.0.0.1:3000").await
}
```

---

## 路由分组

```rust
{
    let admin = app.group("/admin");
    admin.use_middleware(require_api_key);
    admin.get("/dashboard", |_: RequestCtx| async { "管理面板" });
    admin.delete("/users/:id", |ctx: RequestCtx| async move {
        format!("已删除用户 {}", ctx.get_param("id").unwrap())
    });
}
```

---

## 请求体解析

```rust
// JSON
app.post("/users", |mut ctx: RequestCtx| async move {
    #[derive(serde::Deserialize)]
    struct Payload { name: String }

    let p: Payload = ctx.json().await?;
    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(format!("你好, {}", p.name))
});

// 查询参数
app.get("/search", |ctx: RequestCtx| async move {
    let q = ctx.query_param("q").unwrap_or_default();
    format!("搜索: {q}")
});
```

---

## 生命周期钩子

```rust
let app = Engine::new()
    .on_startup(|| async { println!("数据库已连接") })
    .on_shutdown(|| async { println!("数据库已关闭") });
```

---

## Swagger UI

```rust
use s_web::{Engine, RequestCtx, swagger};
use serde_json::json;

let mut app = Engine::new();
app.enable_swagger();   // 挂载 /docs/ 和 /docs/swagger.json

app.get_with_swagger(
    "/users",
    |_: RequestCtx| async { json!([]) },
    swagger()
        .summary("获取用户列表")
        .tag("Users")
        .json_response("200", "用户列表", Some(json!([])))
        .build(),
);
// 访问 http://127.0.0.1:3000/docs/
```

---

## 自定义响应

```rust
use s_web::{ResponseBuilder, StatusCode};

// HTML
ResponseBuilder::html("<h1>你好</h1>");

// 完全控制
ResponseBuilder::new()
    .status(StatusCode::CREATED)
    .content_type("application/json; charset=utf-8")
    .header("X-Request-Id", "abc123")
    .body(r#"{"id":1}"#);
```

---

## 示例

八个可运行项目，每个都是独立的 Cargo 包——可以单独复制出去使用。

| # | 目录 | 主题 | 运行方式 |
|---|------|------|---------|
| 1 | [01_hello_world](examples/01_hello_world) | 路由、路径参数、HTML | `cargo run -p example_hello_world` |
| 2 | [02_json_api](examples/02_json_api) | JSON、查询参数、POST 请求体 | `cargo run -p example_json_api` |
| 3 | [03_middleware](examples/03_middleware) | 全局与组级中间件、认证 | `cargo run -p example_middleware` |
| 4 | [04_todo_app](examples/04_todo_app) | 共享状态、完整 CRUD、生命周期钩子 | `cargo run -p example_todo_app` |
| 5 | [05_swagger](examples/05_swagger) | Swagger UI、Bearer Auth 文档 | `cargo run -p example_swagger` |
| 6 | [06_sqlx_sqlite_crud](examples/06_sqlx_sqlite_crud) | sqlx + SQLite，连接池 | `cargo run -p example_sqlx_sqlite_crud` |
| 7 | [07_seaorm_sqlite_crud](examples/07_seaorm_sqlite_crud) | SeaORM Entity，自动建表 | `cargo run -p example_seaorm_sqlite_crud` |
| 8 | [08_mini_blog](examples/08_mini_blog) | 多文件 mini blog、分层结构、sqlx + SQLite | `cargo run -p mini_blog` |

---

## 贡献

欢迎提交 Pull Request。
