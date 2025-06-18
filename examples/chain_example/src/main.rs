use ree::{Engine, IntoResponse, Next, RequestCtx, Response};
use serde_json::json;

// 日志中间件
async fn logger(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
    println!("[{}] 📨 {} {}", prefix, ctx.request.method(), ctx.request.uri().path());
    let start = std::time::Instant::now();
    let response = next(ctx).await;
    println!("[{}] ✅ Response: {} ({}ms)", prefix, response.status(), start.elapsed().as_millis());
    response
}

// 认证中间件
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == format!("Bearer {}", token) {
            return next(ctx).await;
        }
    }
    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "Unauthorized"}),
    ).into_response()
}

// CORS 中间件  
async fn cors(_ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(_ctx).await;
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        "*".parse().unwrap(),
    );
    response.headers_mut().insert(
        "Access-Control-Allow-Methods", 
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 全局中间件链式调用
    app.use_middleware(|ctx, next| logger("Global", ctx, next))
       .use_middleware(cors)
       // 路由链式调用
       .get("/", |_| async { "Welcome to Ree!" })
       .get("/health", |_| async { json!({"status": "ok"}) });

    // API 路由组，支持链式调用
    {
        let api = app.group("/api");
        api.use_middleware(|ctx, next| logger("API", ctx, next))
           .use_middleware(|ctx, next| auth("api-token", ctx, next))
           .get("/users", |_| async { json!({"users": ["alice", "bob"]}) })
           .post("/users", |_| async { json!({"message": "User created"}) })
           .get("/profile", |_| async { json!({"name": "Current User"}) });
    }

    // 管理员路由组
    {
        let admin = app.group("/admin"); 
        admin.use_middleware(|ctx, next| logger("Admin", ctx, next))
             .use_middleware(|ctx, next| auth("admin-token", ctx, next))
             .get("/dashboard", |_| async { "Admin Dashboard" })
             .delete("/users/:id", |ctx: RequestCtx| async move {
                 if let Some(id) = ctx.get_param("id") {
                     format!("Deleted user {}", id)
                 } else {
                     "User ID not found".to_string()
                 }
             });
    }

    println!("🚀 链式调用示例服务器启动在 http://127.0.0.1:8080");
    println!("📚 试试这些端点:");
    println!("  GET  /                    - 公开端点");
    println!("  GET  /health              - 健康检查");
    println!("  GET  /api/users           - 需要 Bearer api-token");
    println!("  POST /api/users           - 需要 Bearer api-token");
    println!("  GET  /admin/dashboard     - 需要 Bearer admin-token");
    println!("  DELETE /admin/users/123   - 需要 Bearer admin-token");

    app.run("127.0.0.1:8080").await?;
    Ok(())
}
