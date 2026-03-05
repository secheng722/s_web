//! # 示例 2：JSON API（初级）
//!
//! 在 Hello World 基础上增加：
//!   - 返回 JSON（serde_json::Value）
//!   - 路径参数解析与类型转换
//!   - URL 查询参数
//!   - 读取 POST 请求体并反序列化
//!   - 不同 HTTP 状态码
//!
//! 运行：
//!   cargo run -p 02_json_api
//!
//! 接口：
//!   GET  /users           → 用户列表
//!   GET  /users/:id       → 单个用户
//!   GET  /search?name=xx  → 按名字搜索
//!   POST /users           → 创建用户

use s_web::{Engine, IntoResponse, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ──────────────────────────────────────────
// 数据模型
// ──────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

/// 静态演示数据
fn demo_users() -> Vec<User> {
    vec![
        User { id: 1, name: "Alice".into(),   email: "alice@example.com".into() },
        User { id: 2, name: "Bob".into(),     email: "bob@example.com".into()   },
        User { id: 3, name: "Charlie".into(), email: "charlie@example.com".into() },
    ]
}

// ──────────────────────────────────────────
// 辅助：快速构造 JSON 错误响应
// ──────────────────────────────────────────

fn json_error(status: StatusCode, msg: &str) -> Response {
    ResponseBuilder::new()
        .status(status)
        .content_type("application/json; charset=utf-8")
        .body(json!({ "error": msg }).to_string())
}

// ──────────────────────────────────────────
// main
// ──────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    // GET /users → 返回全部用户（serde_json::Value 实现了 IntoResponse）
    app.get("/users", |_ctx: RequestCtx| async {
        json!(demo_users())
    });

    // GET /users/:id → 按 id 查询，id 非数字或不存在返回对应错误码
    app.get("/users/:id", |ctx: RequestCtx| async move {
        let id: u32 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
            Some(v) => v,
            None => return json_error(StatusCode::BAD_REQUEST, "id must be a positive integer"),
        };

        match demo_users().into_iter().find(|u| u.id == id) {
            Some(user) => json!(user).into_response(),
            None => json_error(StatusCode::NOT_FOUND, "user not found"),
        }
    });

    // GET /search?name=xx → 模糊匹配名字（大小写不敏感）
    app.get("/search", |ctx: RequestCtx| async move {
        let q = match ctx.query_param("name") {
            Some(v) if !v.is_empty() => v.to_lowercase(),
            _ => return json_error(StatusCode::BAD_REQUEST, "query param `name` is required"),
        };

        let results: Vec<User> = demo_users()
            .into_iter()
            .filter(|u| u.name.to_lowercase().contains(&q))
            .collect();

        json!({ "query": q, "count": results.len(), "results": results }).into_response()
    });

    // POST /users → 解析 JSON 请求体，返回 201 Created
    app.post("/users", |mut ctx: RequestCtx| async move {
        #[derive(Deserialize)]
        struct NewUser { name: String, email: String }

        let body: NewUser = match ctx.json().await {
            Ok(v) => v,
            Err(e) => return json_error(StatusCode::BAD_REQUEST, &format!("invalid JSON: {e}")),
        };

        let created = User { id: 100, name: body.name, email: body.email };

        ResponseBuilder::new()
            .status(StatusCode::CREATED)
            .content_type("application/json; charset=utf-8")
            .body(json!({ "message": "user created", "user": created }).to_string())
    });

    println!("🚀 Example 2 · JSON API  →  http://127.0.0.1:3000");
    println!();
    println!("  curl http://127.0.0.1:3000/users");
    println!("  curl http://127.0.0.1:3000/users/2");
    println!("  curl 'http://127.0.0.1:3000/search?name=ali'");
    println!(
        "  curl -X POST http://127.0.0.1:3000/users \\
       -H 'Content-Type: application/json' \\
       -d '{{\"name\":\"Dave\",\"email\":\"dave@example.com\"}}'"
    );

    app.run("127.0.0.1:3000").await
}
