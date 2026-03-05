//! # 示例 6：SQLite CRUD（高级）
//!
//! 演示框架与 sqlx ORM 的完整集成：
//!   - 启动时建立 SQLitePool 连接池
//!   - 在 on_startup 钩子中执行数据库迁移（建表）
//!   - Arc<SqlitePool> 作为共享状态注入每个处理器
//!   - 完整 REST CRUD 对接真实数据库
//!   - 统一错误处理
//!
//! 数据库文件：./products.db（自动创建）
//!
//! 运行：
//!   cargo run -p example_sqlite_crud
//!
//! 接口：
//!   GET    /products           → 查询全部（支持 ?name=xx 过滤）
//!   POST   /products           → 创建（body: {"name":"…","price":9.9,"stock":100}）
//!   GET    /products/:id       → 按 ID 查询
//!   PUT    /products/:id       → 整体更新
//!   DELETE /products/:id       → 删除

use s_web::{Engine, IntoResponse, Next, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, SqlitePool};
use std::sync::Arc;

// ──────────────────────────────────────────
// 数据模型
// ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Product {
    id: i64,
    name: String,
    price: f64,
    stock: i64,
}

// ──────────────────────────────────────────
// 辅助
// ──────────────────────────────────────────

fn json_err(status: StatusCode, msg: &str) -> Response {
    ResponseBuilder::new()
        .status(status)
        .content_type("application/json; charset=utf-8")
        .body(json!({ "error": msg }).to_string())
}

/// 注入 pool 的日志中间件
async fn log_middleware(ctx: RequestCtx, next: Next) -> Response {
    let method = ctx.request.method().to_string();
    let path   = ctx.request.uri().path().to_string();
    let resp   = next(ctx).await;
    println!("[db] {} {} → {}", method, path, resp.status());
    resp
}

// ──────────────────────────────────────────
// 数据库迁移
// ──────────────────────────────────────────

async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT    NOT NULL,
            price REAL    NOT NULL,
            stock INTEGER NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 预置演示数据（仅当表为空时写入）
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        for (name, price, stock) in [
            ("Rust Book",        29.99_f64, 50_i64),
            ("Mechanical Keyboard", 89.99, 20),
            ("USB-C Hub",        19.99, 100),
        ] {
            sqlx::query("INSERT INTO products (name, price, stock) VALUES (?, ?, ?)")
                .bind(name)
                .bind(price)
                .bind(stock)
                .execute(pool)
                .await?;
        }
        println!("✅ Seeded 3 demo products");
    }

    Ok(())
}

// ──────────────────────────────────────────
// main
// ──────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 建立 SQLite 连接池
    let pool = Arc::new(
        SqlitePool::connect("sqlite:./products.db?mode=rwc").await?,
    );

    // 迁移 & 种子数据
    migrate(&pool).await?;

    let mut app = Engine::new();

    // ── 全局日志中间件 ───────────────────────────────
    app.use_middleware(log_middleware);

    // ── GET /products?name=xx ────────────────────────
    {
        let pool = pool.clone();
        app.get("/products", move |ctx: RequestCtx| {
            let pool = pool.clone();
            async move {
                let name_filter = ctx.query_param("name").unwrap_or_default();

                let products: Vec<Product> = if name_filter.is_empty() {
                    sqlx::query_as("SELECT id, name, price, stock FROM products ORDER BY id")
                        .fetch_all(pool.as_ref())
                        .await
                        .unwrap_or_default()
                } else {
                    let pattern = format!("%{}%", name_filter);
                    sqlx::query_as(
                        "SELECT id, name, price, stock FROM products WHERE name LIKE ? ORDER BY id",
                    )
                    .bind(pattern)
                    .fetch_all(pool.as_ref())
                    .await
                    .unwrap_or_default()
                };

                json!({ "count": products.len(), "products": products }).into_response()
            }
        });
    }

    // ── POST /products ───────────────────────────────
    {
        let pool = pool.clone();
        app.post("/products", move |mut ctx: RequestCtx| {
            let pool = pool.clone();
            async move {
                #[derive(Deserialize)]
                struct Payload { name: String, price: f64, stock: i64 }

                let p: Payload = match ctx.json().await {
                    Ok(v)  => v,
                    Err(_) => return json_err(StatusCode::BAD_REQUEST, "invalid JSON body"),
                };

                if p.name.trim().is_empty() {
                    return json_err(StatusCode::BAD_REQUEST, "name must not be empty");
                }
                if p.price < 0.0 {
                    return json_err(StatusCode::BAD_REQUEST, "price must be non-negative");
                }

                let row: (i64,) = match sqlx::query_as(
                    "INSERT INTO products (name, price, stock) VALUES (?, ?, ?) RETURNING id",
                )
                .bind(&p.name)
                .bind(p.price)
                .bind(p.stock)
                .fetch_one(pool.as_ref())
                .await
                {
                    Ok(r)  => r,
                    Err(e) => {
                        eprintln!("DB insert error: {e}");
                        return json_err(StatusCode::INTERNAL_SERVER_ERROR, "database error");
                    }
                };

                let product = Product { id: row.0, name: p.name, price: p.price, stock: p.stock };
                ResponseBuilder::new()
                    .status(StatusCode::CREATED)
                    .content_type("application/json; charset=utf-8")
                    .body(json!(product).to_string())
            }
        });
    }

    // ── GET /products/:id ────────────────────────────
    {
        let pool = pool.clone();
        app.get("/products/:id", move |ctx: RequestCtx| {
            let pool = pool.clone();
            async move {
                let id: i64 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };

                match sqlx::query_as::<_, Product>(
                    "SELECT id, name, price, stock FROM products WHERE id = ?",
                )
                .bind(id)
                .fetch_optional(pool.as_ref())
                .await
                {
                    Ok(Some(p)) => json!(p).into_response(),
                    Ok(None)    => json_err(StatusCode::NOT_FOUND, "product not found"),
                    Err(e)      => {
                        eprintln!("DB query error: {e}");
                        json_err(StatusCode::INTERNAL_SERVER_ERROR, "database error")
                    }
                }
            }
        });
    }

    // ── PUT /products/:id ────────────────────────────
    {
        let pool = pool.clone();
        app.put("/products/:id", move |mut ctx: RequestCtx| {
            let pool = pool.clone();
            async move {
                let id: i64 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };

                #[derive(Deserialize)]
                struct Payload { name: String, price: f64, stock: i64 }

                let p: Payload = match ctx.json().await {
                    Ok(v)  => v,
                    Err(_) => return json_err(StatusCode::BAD_REQUEST, "invalid JSON body"),
                };

                let result = sqlx::query(
                    "UPDATE products SET name = ?, price = ?, stock = ? WHERE id = ?",
                )
                .bind(&p.name)
                .bind(p.price)
                .bind(p.stock)
                .bind(id)
                .execute(pool.as_ref())
                .await;

                match result {
                    Ok(r) if r.rows_affected() == 0 => {
                        json_err(StatusCode::NOT_FOUND, "product not found")
                    }
                    Ok(_) => {
                        let product = Product { id, name: p.name, price: p.price, stock: p.stock };
                        json!(product).into_response()
                    }
                    Err(e) => {
                        eprintln!("DB update error: {e}");
                        json_err(StatusCode::INTERNAL_SERVER_ERROR, "database error")
                    }
                }
            }
        });
    }

    // ── DELETE /products/:id ─────────────────────────
    {
        let pool = pool.clone();
        app.delete("/products/:id", move |ctx: RequestCtx| {
            let pool = pool.clone();
            async move {
                let id: i64 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };

                let result = sqlx::query("DELETE FROM products WHERE id = ?")
                    .bind(id)
                    .execute(pool.as_ref())
                    .await;

                match result {
                    Ok(r) if r.rows_affected() == 0 => {
                        json_err(StatusCode::NOT_FOUND, "product not found")
                    }
                    Ok(_) => ResponseBuilder::new()
                        .status(StatusCode::NO_CONTENT)
                        .empty_body(),
                    Err(e) => {
                        eprintln!("DB delete error: {e}");
                        json_err(StatusCode::INTERNAL_SERVER_ERROR, "database error")
                    }
                }
            }
        });
    }

    println!("🚀 Example 6 · SQLite CRUD  →  http://127.0.0.1:3000");
    println!("💾 Database file: ./products.db");
    println!();
    println!("  curl http://127.0.0.1:3000/products");
    println!("  curl http://127.0.0.1:3000/products/1");
    println!("  curl 'http://127.0.0.1:3000/products?name=rust'");
    println!("  curl -X POST http://127.0.0.1:3000/products \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"name\":\"Cargo Plushie\",\"price\":14.99,\"stock\":200}}'");
    println!("  curl -X PUT http://127.0.0.1:3000/products/1 \\");
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"name\":\"Rust Book v2\",\"price\":34.99,\"stock\":30}}'");
    println!("  curl -X DELETE http://127.0.0.1:3000/products/3");

    app.run("127.0.0.1:3000").await
}
