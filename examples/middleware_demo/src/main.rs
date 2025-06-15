use async_trait::async_trait;
use ree::{AccessLog, Cors, Engine, Middleware, Next, RequestCtx, Response};
use serde_json::json;
use std::time::Instant;

// 自定义中间件：计时器中间件
struct Timer;

#[async_trait]
impl Middleware for Timer {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        let start = Instant::now();
        let response = next.run(ctx).await;
        println!("请求处理耗时: {}ms", start.elapsed().as_millis());
        response
    }
}

// 自定义中间件：认证中间件
struct Auth;

#[async_trait]
impl Middleware for Auth {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        if let Some(auth) = ctx.request.headers().get("Authorization") {
            if auth == "Bearer secret" {
                return next.run(ctx).await;
            }
        }
        // 未认证返回401
        ree::ResponseBuilder::unauthorized_json(r#"{"error": "Unauthorized"}"#)
    }
}

// 自定义中间件：请求计数器
struct RequestCounter {
    count: std::sync::atomic::AtomicUsize,
}

impl RequestCounter {
    fn new() -> Self {
        Self {
            count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Middleware for RequestCounter {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        let current = self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("总请求数: {}", current + 1);
        next.run(ctx).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    println!("🛠 Ree HTTP Framework - 中间件使用指南");
    println!("════════════════════════════════════");

    // 1. 全局中间件 - 应用到所有路由
    println!("1️⃣ 全局中间件 - 应用到所有路由");
    app.use_middleware(AccessLog); // 访问日志
    app.use_middleware(Timer); // 计时器
    app.use_middleware(RequestCounter::new()); // 请求计数

    // 2. 路由组中间件 - 仅应用到特定组
    println!("2️⃣ 路由组中间件 - 仅应用到特定组");
    let admin = app.group("/admin");
    admin.use_middleware(Auth); // 认证中间件

    // 添加受保护的管理路由
    admin.get("/stats", |_| async {
        json!({
            "status": "ok",
            "message": "这是受保护的管理统计接口"
        })
    });

    admin.post("/update", |_| async {
        json!({
            "status": "ok",
            "message": "更新成功"
        })
    });

    // 3. CORS中间件 - 配置跨域
    println!("3️⃣ CORS中间件 - 跨域设置");
    let api = app.group("/api");
    api.use_middleware(
        Cors::new()
            .allow_origin("http://localhost:3000")
            .allow_methods("GET, POST, PUT, DELETE")
            .allow_headers("Content-Type, Authorization"),
    );

    // API路由
    api.get("/hello", |_| async {
        json!({
            "message": "Hello from API",
            "cors": "enabled"
        })
    });

    // 公开路由 - 不需要认证
    app.get("/", |_| async { "欢迎访问中间件演示!" });

    app.get("/public/hello", |_| async {
        json!({
            "message": "这是公开API，无需认证"
        })
    });

    println!("\n📋 测试指南:");
    println!("   1. 访问日志和计时器（所有请求）:");
    println!("      curl http://localhost:8080/");
    println!();
    println!("   2. 认证中间件测试（需要token）:");
    println!("      curl -H 'Authorization: Bearer secret' http://localhost:8080/admin/stats");
    println!("      curl -H 'Authorization: wrong-token' http://localhost:8080/admin/stats");
    println!();
    println!("   3. CORS中间件测试:");
    println!("      curl -H 'Origin: http://localhost:3000' http://localhost:8080/api/hello");
    println!();
    println!("   4. 公开API测试:");
    println!("      curl http://localhost:8080/public/hello");

    println!("\n🚀 服务启动在 http://localhost:8080");
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
