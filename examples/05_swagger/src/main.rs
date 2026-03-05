//! # 示例 5：Swagger UI 文档（中级）
//!
//! 演示如何为每条路由添加 Swagger 文档注解：
//!   - `enable_swagger()` 开启内置 Swagger UI
//!   - `swagger()` builder 配置摘要、标签、参数、响应
//!   - `get_with_swagger` / `post_with_swagger` 等带注解的路由注册
//!   - 路径参数、查询参数、请求体、响应码文档化
//!   - Bearer token 安全方案标注
//!
//! 运行：
//!   cargo run -p example_swagger
//!
//! 然后打开 http://127.0.0.1:3000/docs/ 查看 Swagger UI

use s_web::{
    Engine, IntoResponse, RequestCtx, Response, ResponseBuilder, StatusCode,
    swagger,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ──────────────────────────────────────────
// 数据模型（内存演示数据）
// ──────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
struct Article {
    id: u32,
    title: String,
    body: String,
    author: String,
}

fn demo_articles() -> Vec<Article> {
    vec![
        Article { id: 1, title: "Hello Rust".into(),      body: "Rust is awesome.".into(),        author: "Alice".into() },
        Article { id: 2, title: "Async in Rust".into(),   body: "Tokio makes async easy.".into(), author: "Bob".into()   },
        Article { id: 3, title: "s_web Framework".into(), body: "Build APIs with s_web.".into(),  author: "Charlie".into() },
    ]
}

fn json_err(status: StatusCode, msg: &str) -> Response {
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

    // 开启内置 Swagger UI（挂载到 /docs/）
    app.enable_swagger();

    // ── GET /articles ───────────────────────────────────────────
    app.get_with_swagger(
        "/articles",
        |_ctx: RequestCtx| async { json!(demo_articles()) },
        swagger()
            .summary("获取文章列表")
            .description("返回所有文章，支持按 author 过滤")
            .tag("Articles")
            .query_param("author", "按作者过滤（可选）", false)
            .json_response(
                "200",
                "文章列表",
                Some(json!([{ "id": 1, "title": "Hello Rust", "body": "...", "author": "Alice" }])),
            )
            .response("500", "服务器内部错误")
            .build(),
    );

    // ── GET /articles/:id ───────────────────────────────────────
    app.get_with_swagger(
        "/articles/:id",
        |ctx: RequestCtx| async move {
            let id: u32 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                Some(v) => v,
                None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
            };
            match demo_articles().into_iter().find(|a| a.id == id) {
                Some(a) => json!(a).into_response(),
                None    => json_err(StatusCode::NOT_FOUND, "article not found"),
            }
        },
        swagger()
            .summary("获取单篇文章")
            .tag("Articles")
            .path_param("id", "文章 ID")
            .json_response(
                "200",
                "文章详情",
                Some(json!({ "id": 1, "title": "Hello Rust", "body": "...", "author": "Alice" })),
            )
            .response("404", "文章不存在")
            .build(),
    );

    // ── POST /articles ──────────────────────────────────────────
    app.post_with_swagger(
        "/articles",
        |mut ctx: RequestCtx| async move {
            #[derive(Deserialize)]
            struct Payload { title: String, body: String, author: String }

            let p: Payload = match ctx.json().await {
                Ok(v)  => v,
                Err(_) => return json_err(StatusCode::BAD_REQUEST, "invalid JSON body"),
            };

            let created = Article { id: 100, title: p.title, body: p.body, author: p.author };
            ResponseBuilder::new()
                .status(StatusCode::CREATED)
                .content_type("application/json; charset=utf-8")
                .body(json!(created).to_string())
        },
        swagger()
            .summary("创建文章")
            .description("提交 JSON 请求体创建一篇新文章，需要 Bearer Token 认证")
            .tag("Articles")
            .request_body(json!({ "title": "My Title", "body": "Content here.", "author": "Dave" }))
            .json_response(
                "201",
                "创建成功，返回新文章",
                Some(json!({ "id": 100, "title": "My Title", "body": "Content here.", "author": "Dave" })),
            )
            .response("400", "请求体格式错误")
            .bearer_auth()
            .build(),
    );

    // ── PUT /articles/:id ───────────────────────────────────────
    app.put_with_swagger(
        "/articles/:id",
        |mut ctx: RequestCtx| async move {
            let id: u32 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                Some(v) => v,
                None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
            };

            #[derive(Deserialize)]
            struct Payload { title: String, body: String, author: String }

            let p: Payload = match ctx.json().await {
                Ok(v)  => v,
                Err(_) => return json_err(StatusCode::BAD_REQUEST, "invalid JSON body"),
            };

            let updated = Article { id, title: p.title, body: p.body, author: p.author };
            json!(updated).into_response()
        },
        swagger()
            .summary("更新文章")
            .tag("Articles")
            .path_param("id", "文章 ID")
            .request_body(json!({ "title": "Updated Title", "body": "Updated body.", "author": "Alice" }))
            .crud_responses()
            .bearer_auth()
            .build(),
    );

    // ── DELETE /articles/:id ────────────────────────────────────
    app.delete_with_swagger(
        "/articles/:id",
        |ctx: RequestCtx| async move {
            let id: u32 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                Some(v) => v,
                None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
            };
            if id <= 3 {
                ResponseBuilder::new().status(StatusCode::NO_CONTENT).empty_body()
            } else {
                json_err(StatusCode::NOT_FOUND, "article not found")
            }
        },
        swagger()
            .summary("删除文章")
            .tag("Articles")
            .path_param("id", "文章 ID")
            .response("204", "删除成功（无内容）")
            .response("404", "文章不存在")
            .bearer_auth()
            .build(),
    );

    println!("🚀 Example 5 · Swagger UI  →  http://127.0.0.1:3000");
    println!("📖 Swagger UI             →  http://127.0.0.1:3000/docs/");

    app.run("127.0.0.1:3000").await
}
