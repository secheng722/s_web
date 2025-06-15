mod handlers;

use ree::{Engine, AccessLog};
use handlers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 添加访问日志中间件
    app.use_middleware(AccessLog);
    
    // 基本路由
    app.get("/", hello_handler);
    app.get("/hello/:name", hello_name_handler);
    
    // 路由组示例
    let api_group = app.group("/api");
    api_group.get("/users", get_users_handler);
    api_group.get("/users/:id", get_user_by_id_handler);
    
    println!("🚀 服务器启动在 http://127.0.0.1:8080");
    println!("📝 可以访问以下端点:");
    println!("   - GET /");
    println!("   - GET /hello/:name");
    println!("   - GET /api/users");
    println!("   - GET /api/users/:id");
    
    app.run("127.0.0.1:8080").await?;
    
    Ok(())
}
