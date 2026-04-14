mod app;
mod db;
mod dto;
mod error;
mod middleware;
mod models;
mod repository;
mod handlers;

use sqlx::SqlitePool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite:./mini_blog.db?mode=rwc").await?;
    db::init_database(&pool).await?;

    let db = Arc::new(pool);
    let mut app = s_web::Engine::new()
        .on_startup(|| async {
            println!("✅ mini_blog started");
        })
        .on_shutdown(|| async {
            println!("🛑 mini_blog shutdown");
        });

    app::register_routes(&mut app, db);
    app.run("127.0.0.1:3008").await
}
