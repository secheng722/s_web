use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn init_db() -> Result<Pool<Sqlite>, sqlx::Error> {
    // 确保 data 目录存在
    std::fs::create_dir_all("data").ok();
    
    let database_url = "sqlite:examples/article_system_example/data/app.db";
    
    // 创建数据库连接
    let pool = SqlitePool::connect(database_url).await?;
    
    // 运行迁移
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS articles (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            author_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (author_id) REFERENCES users (id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
