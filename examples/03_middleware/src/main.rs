//! # 示例 3：中间件 & 路由分组（中级）
//!
//! 在 JSON API 基础上增加：
//!   - 全局中间件：请求日志（打印方法、路径、耗时）
//!   - 路由分组 `/api/v1`，绑定 Auth 中间件（X-API-Key 校验）
//!   - 公开路由与受保护路由分离
//!
//! 运行：
//!   cargo run -p 03_middleware
//!
//! 接口（公开）：
//!   GET  /          → 欢迎页
//!   GET  /health    → 健康检查
//!
//! 接口（需要 Header: X-API-Key: secret）：
//!   GET  /api/v1/users        → 用户列表
//!   GET  /api/v1/users/:id    → 单个用户
//!   POST /api/v1/users        → 创建用户

use s_web::{Engine, IntoResponse, Next, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde_json::json;
use std::time::Instant;

// ──────────────────────────────────────────
// 中间件定义
// ──────────────────────────────────────────

/// 全局日志中间件：打印每次请求的方法、路径和响应耗时
async fn logging_middleware(ctx: RequestCtx, next: Next) -> Response {
    let method = ctx.request.method().to_string();
    let path   = ctx.request.uri().path().to_string();
    let start  = Instant::now();

    let response = next(ctx).await;

    println!(
        "[LOG] {} {} → {} ({:.2}ms)",
        method,
        path,
        response.status(),
        start.elapsed().as_secs_f64() * 1000.0,
    );
    response
}

/// 认证中间件：校验 `X-API-Key` 请求头，不匹配则直接返回 401
async fn auth_middleware(ctx: RequestCtx, next: Next) -> Response {
    match ctx.header("x-api-key") {
        Some("secret") => next(ctx).await,
        _ => ResponseBuilder::new()
            .status(StatusCode::UNAUTHORIZED)
            .content_type("application/json; charset=utf-8")
            .body(json!({ "error": "missing or invalid X-API-Key" }).to_string()),
    }
}

// ──────────────────────────────────────────
// main
// ──────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    // ── 全局中间件（对所有路由生效）──────────────
    app.use_middleware(logging_middleware);

    // ── 公开路由 ─────────────────────────────────
    app.get("/", |_ctx: RequestCtx| async {
        ResponseBuilder::html("<h1>Welcome!</h1><p>Protected API is under <code>/api/v1</code>.</p>")
    });

    app.get("/health", |_ctx: RequestCtx| async {
        json!({ "status": "ok" })
    });

    // ── 受保护的路由分组 ──────────────────────────
    // 所有 /api/v1/* 路由均需通过 auth_middleware
    {
        let g = app.group("/api/v1");
        g.use_middleware(auth_middleware);

        g.get("/users", |_ctx: RequestCtx| async {
            json!([
                { "id": 1, "name": "Alice" },
                { "id": 2, "name": "Bob"   },
            ])
        });

        g.get("/users/:id", |ctx: RequestCtx| async move {
            let id = ctx.get_param("id").cloned().unwrap_or_default();
            match id.as_str() {
                "1" => json!({ "id": 1, "name": "Alice", "role": "admin" }).into_response(),
                "2" => json!({ "id": 2, "name": "Bob",   "role": "user"  }).into_response(),
                _   => ResponseBuilder::new()
                    .status(StatusCode::NOT_FOUND)
                    .content_type("application/json; charset=utf-8")
                    .body(json!({ "error": "not found" }).to_string()),
            }
        });

        g.post("/users", |mut ctx: RequestCtx| async move {
            let body: serde_json::Value = match ctx.json().await {
                Ok(v) => v,
                Err(_) => return ResponseBuilder::new()
                    .status(StatusCode::BAD_REQUEST)
                    .content_type("application/json; charset=utf-8")
                    .body(json!({ "error": "invalid JSON body" }).to_string()),
            };
            ResponseBuilder::new()
                .status(StatusCode::CREATED)
                .content_type("application/json; charset=utf-8")
                .body(json!({ "message": "created", "data": body }).to_string())
        });
    }

    println!("🚀 Example 3 · Middleware & Groups  →  http://127.0.0.1:3000");
    println!();
    println!("Public:");
    println!("  curl http://127.0.0.1:3000/health");
    println!();
    println!("Protected (add -H 'X-API-Key: secret'):");
    println!("  curl -H 'X-API-Key: secret' http://127.0.0.1:3000/api/v1/users");
    println!("  curl -H 'X-API-Key: secret' http://127.0.0.1:3000/api/v1/users/1");
    println!("  curl -X POST http://127.0.0.1:3000/api/v1/users \\");
    println!("       -H 'X-API-Key: secret' -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"name\":\"Charlie\"}}'");

    app.run("127.0.0.1:3000").await
}
