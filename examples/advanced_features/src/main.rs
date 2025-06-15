mod handlers;
mod middleware;

use ree::{Engine, AccessLog, Cors};
use handlers::*;
use middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 全局中间件
    app.use_middleware(AccessLog);
    app.use_middleware(Cors::new().allow_origin("*"));
    app.use_middleware(CustomLogger::new("REE-APP"));
    
    // 基本路由
    app.get("/", home_handler);
    app.get("/health", health_check_handler);
    
    // 静态内容
    app.get("/about", about_handler);
    app.get("/docs", docs_handler);
    
    // API 路由组
    let api_v1 = app.group("/api/v1");
    api_v1.use_middleware(RateLimiter::new(100)); // 100 requests per minute
    api_v1.get("/users", list_users_handler);
    api_v1.post("/users", create_user_handler);
    api_v1.get("/users/:id", get_user_handler);
    api_v1.put("/users/:id", update_user_handler);
    api_v1.delete("/users/:id", delete_user_handler);
    
    // 管理员路由组
    let admin = app.group("/admin");
    admin.use_middleware(AuthMiddleware::new());
    admin.get("/stats", admin_stats_handler);
    admin.get("/logs", admin_logs_handler);
    
    println!("🚀 Ree框架高级功能演示服务器启动在 http://127.0.0.1:3000");
    println!("📖 可访问以下端点:");
    println!("   基础:");
    println!("     GET  /              - 首页");
    println!("     GET  /health        - 健康检查");
    println!("     GET  /about         - 关于页面");
    println!("     GET  /docs          - 文档页面");
    println!("   API v1:");
    println!("     GET    /api/v1/users     - 获取用户列表");
    println!("     POST   /api/v1/users     - 创建用户");
    println!("     GET    /api/v1/users/:id - 获取用户详情");
    println!("     PUT    /api/v1/users/:id - 更新用户");
    println!("     DELETE /api/v1/users/:id - 删除用户");
    println!("   管理员:");
    println!("     GET  /admin/stats   - 系统统计");
    println!("     GET  /admin/logs    - 系统日志");
    
    app.run("127.0.0.1:3000").await?;
    
    Ok(())
}
