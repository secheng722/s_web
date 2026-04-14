use sqlx::SqlitePool;

pub async fn init_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            is_published INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER NOT NULL,
            author TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY(post_id) REFERENCES posts(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        let draft = sqlx::query("INSERT INTO posts (title, content, is_published) VALUES (?, ?, 0)")
            .bind("第一篇草稿")
            .bind("这是草稿内容，尚未发布")
            .execute(pool)
            .await?;

        let published = sqlx::query("INSERT INTO posts (title, content, is_published) VALUES (?, ?, 1)")
            .bind("Rust Web 实战")
            .bind("这是一篇已发布文章，展示 SQLite + 多文件结构")
            .execute(pool)
            .await?;

        let published_id = published.last_insert_rowid();
        let draft_id = draft.last_insert_rowid();

        sqlx::query("INSERT INTO comments (post_id, author, content) VALUES (?, ?, ?)")
            .bind(published_id)
            .bind("Alice")
            .bind("写得很清晰，感谢示例")
            .execute(pool)
            .await?;

        sqlx::query("INSERT INTO comments (post_id, author, content) VALUES (?, ?, ?)")
            .bind(draft_id)
            .bind("Bob")
            .bind("草稿也很有价值，期待发布")
            .execute(pool)
            .await?;

        println!("✅ mini_blog seeded with demo data");
    }

    Ok(())
}
