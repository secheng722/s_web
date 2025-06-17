mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod routes;

use ree::Engine;

use crate::middleware::logging_middleware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化数据库
    let pool = db::init_db().await?;

    // 创建应用状态
    let state = config::AppState::new(
        pool,
        "your_jwt_secret_key_here".to_string(), // 在实际应用中应该从环境变量读取
    );

    // 创建应用
    let mut app = Engine::new();

    // 注册全局中间件
    app.use_middleware(|ctx, next| logging_middleware("BlogAPI", ctx, next));

    // 注册所有路由
    routes::register_all_routes(&mut app, state);

    // 添加错误处理

    // 启动应用
    app.run("127.0.0.1:3000").await?;

    Ok(())
}
