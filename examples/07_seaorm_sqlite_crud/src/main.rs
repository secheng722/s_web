//! # 示例 7：SeaORM + SQLite CRUD（高级）
//!
//! 与示例 6 功能相同，但将 sqlx 替换为 SeaORM：
//!   - SeaORM Entity 定义（派生宏生成 Model / ActiveModel / Column 等）
//!   - Schema::create_table_from_entity 自动建表
//!   - ActiveModel 插入 / find_by_id 查询 / ActiveModel 更新 / delete_by_id
//!   - DatabaseConnection 实现了 Clone（内部 Arc），直接在闭包中 clone 共享
//!   - 统一 JSON 错误格式
//!
//! 数据库文件：./products_seaorm.db（自动创建）
//!
//! 运行：
//!   cargo run -p example_seaorm_sqlite_crud
//!
//! 接口：
//!   GET    /products           → 全部产品（支持 ?name=xx）
//!   POST   /products           → 创建
//!   GET    /products/:id       → 单条
//!   PUT    /products/:id       → 整体更新
//!   DELETE /products/:id       → 删除

mod product;

use product::Entity as ProductEntity;
use s_web::{Engine, IntoResponse, Next, RequestCtx, Response, ResponseBuilder, StatusCode};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Database,
    DatabaseConnection, EntityTrait, PaginatorTrait, Schema,
};
use serde::Deserialize;
use serde_json::json;

// ──────────────────────────────────────────
// 辅助
// ──────────────────────────────────────────

fn json_err(status: StatusCode, msg: &str) -> Response {
    ResponseBuilder::new()
        .status(status)
        .content_type("application/json; charset=utf-8")
        .body(json!({ "error": msg }).to_string())
}

async fn log_middleware(ctx: RequestCtx, next: Next) -> Response {
    let method = ctx.request.method().to_string();
    let path   = ctx.request.uri().path().to_string();
    let resp   = next(ctx).await;
    println!("[seaorm] {} {} → {}", method, path, resp.status());
    resp
}

// ──────────────────────────────────────────
// 数据库初始化
// ──────────────────────────────────────────

async fn setup_db(db: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let backend = db.get_database_backend();
    let schema  = Schema::new(backend);

    // 根据 Entity 定义自动生成建表语句（TableCreateStatement 实现了 StatementBuilder）
    let stmt = schema
        .create_table_from_entity(ProductEntity)
        .if_not_exists()
        .to_owned();
    db.execute(&stmt).await?;

    // 种子数据（仅当表为空时写入）
    let count = ProductEntity::find().count(db).await?;
    if count == 0 {
        for (name, price, stock) in [
            ("Rust Book",           29.99_f64, 50_i32),
            ("Mechanical Keyboard", 89.99,      20),
            ("USB-C Hub",           19.99,     100),
        ] {
            product::ActiveModel {
                name:  Set(name.to_owned()),
                price: Set(price),
                stock: Set(stock),
                ..Default::default()
            }
            .insert(db)
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
    let db = Database::connect("sqlite:./products_seaorm.db?mode=rwc").await?;
    setup_db(&db).await?;

    // DatabaseConnection 内部已是 Arc，clone 是轻量引用计数
    let mut app = Engine::new();
    app.use_middleware(log_middleware);

    // ── GET /products?name=xx ────────────────────────
    {
        let db = db.clone();
        app.get("/products", move |ctx: RequestCtx| {
            let db = db.clone();
            async move {
                let q = ctx.query_param("name").unwrap_or_default().to_lowercase();

                let products = ProductEntity::find()
                    .all(&db)
                    .await
                    .unwrap_or_default();

                let filtered: Vec<_> = if q.is_empty() {
                    products
                } else {
                    products
                        .into_iter()
                        .filter(|p| p.name.to_lowercase().contains(&q))
                        .collect()
                };

                json!({ "count": filtered.len(), "products": filtered }).into_response()
            }
        });
    }

    // ── POST /products ───────────────────────────────
    {
        let db = db.clone();
        app.post("/products", move |mut ctx: RequestCtx| {
            let db = db.clone();
            async move {
                #[derive(Deserialize)]
                struct Payload { name: String, price: f64, stock: i32 }

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

                let model = product::ActiveModel {
                    name:  Set(p.name),
                    price: Set(p.price),
                    stock: Set(p.stock),
                    ..Default::default()
                }
                .insert(&db)
                .await;

                match model {
                    Ok(m) => ResponseBuilder::new()
                        .status(StatusCode::CREATED)
                        .content_type("application/json; charset=utf-8")
                        .body(json!(m).to_string()),
                    Err(e) => {
                        eprintln!("DB insert error: {e}");
                        json_err(StatusCode::INTERNAL_SERVER_ERROR, "database error")
                    }
                }
            }
        });
    }

    // ── GET /products/:id ────────────────────────────
    {
        let db = db.clone();
        app.get("/products/:id", move |ctx: RequestCtx| {
            let db = db.clone();
            async move {
                let id: i32 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };

                match ProductEntity::find_by_id(id).one(&db).await {
                    Ok(Some(m)) => json!(m).into_response(),
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
        let db = db.clone();
        app.put("/products/:id", move |mut ctx: RequestCtx| {
            let db = db.clone();
            async move {
                let id: i32 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };

                #[derive(Deserialize)]
                struct Payload { name: String, price: f64, stock: i32 }

                let p: Payload = match ctx.json().await {
                    Ok(v)  => v,
                    Err(_) => return json_err(StatusCode::BAD_REQUEST, "invalid JSON body"),
                };

                // 先查出已有记录，再转为 ActiveModel 更新
                let existing = match ProductEntity::find_by_id(id).one(&db).await {
                    Ok(Some(m)) => m,
                    Ok(None)    => return json_err(StatusCode::NOT_FOUND, "product not found"),
                    Err(e)      => {
                        eprintln!("DB query error: {e}");
                        return json_err(StatusCode::INTERNAL_SERVER_ERROR, "database error");
                    }
                };

                let mut active: product::ActiveModel = existing.into();
                active.name  = Set(p.name);
                active.price = Set(p.price);
                active.stock = Set(p.stock);

                match active.update(&db).await {
                    Ok(m)  => json!(m).into_response(),
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
        let db = db.clone();
        app.delete("/products/:id", move |ctx: RequestCtx| {
            let db = db.clone();
            async move {
                let id: i32 = match ctx.get_param("id").and_then(|s| s.parse().ok()) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };

                let result = ProductEntity::delete_by_id(id).exec(&db).await;

                match result {
                    Ok(r) if r.rows_affected == 0 => {
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

    println!("🚀 Example 7 · SeaORM SQLite CRUD  →  http://127.0.0.1:3000");
    println!("💾 Database file: ./products_seaorm.db");
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
