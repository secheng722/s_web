use chrono::{DateTime, Utc};
use ree::{Engine, RequestCtx, ResponseBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

// 应用状态，包含数据库连接池
#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Ree Framework - Database Example");
    println!("═══════════════════════════════════");
    println!("📊 SQLite + CRUD API Demo");
    println!();

    // 初始化数据库
    let db = init_database().await?;
    let state = AppState { db };

    let mut app = Engine::new();

    // 添加CORS中间件（简单版本）
    app.use_middleware(|ctx, next| async move {
        let mut response = next(ctx).await;
        response
            .headers_mut()
            .insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        response.headers_mut().insert(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE".parse().unwrap(),
        );
        response.headers_mut().insert(
            "Access-Control-Allow-Headers",
            "Content-Type".parse().unwrap(),
        );
        response
    });


    // 创建API路由组
    let api = app.group("/api/v1");

    // 用户CRUD端点
    api.get("/users", {
        let state = state.clone();
        move |_ctx| {
            let state = state.clone();
            async move { get_users(state).await }
        }
    });

    api.post("/users", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { create_user(ctx, state).await }
        }
    });

    api.get("/users/:id", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { get_user(ctx, state).await }
        }
    });

    api.put("/users/:id", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { update_user(ctx, state).await }
        }
    });

    api.delete("/users/:id", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { delete_user(ctx, state).await }
        }
    });

    // 健康检查端点
    app.get("/health", |_| async {
        json!({
            "status": "healthy",
            "timestamp": Utc::now(),
            "service": "ree-database-example"
        })
    });

    // 根路径 - API文档
    app.get("/", |_| async {
        ResponseBuilder::new()
            .status(StatusCode::OK)
            .content_type("text/html; charset=utf-8")
            .body(include_str!("../templates/index.html"))
    });

    println!("🚀 服务器启动中...");
    println!("📖 API文档: http://127.0.0.1:3000");
    println!("🔗 API端点:");
    println!("  GET    /api/v1/users     - 获取所有用户");
    println!("  POST   /api/v1/users     - 创建新用户");
    println!("  GET    /api/v1/users/:id - 获取特定用户");
    println!("  PUT    /api/v1/users/:id - 更新用户");
    println!("  DELETE /api/v1/users/:id - 删除用户");
    println!("  GET    /health           - 健康检查");
    println!();
    println!("📝 示例请求:");
    println!("curl -X POST http://127.0.0.1:3000/api/v1/users \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"name\":\"张三\",\"email\":\"zhangsan@example.com\"}}'");
    println!();

    app.run("127.0.0.1:3000").await
}

async fn init_database() -> Result<SqlitePool, sqlx::Error> {
    // 获取当前工作目录并构建数据库路径
    let current_dir = std::env::current_dir()
        .map_err(|e| sqlx::Error::Configuration(format!("无法获取当前目录: {}", e).into()))?;

    let db_path = current_dir.join("database.db");

    // 添加 create 参数以允许创建数据库文件
    let db_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

    println!("📁 数据库路径: {}", db_path.display());

    // 确保数据库文件的父目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| sqlx::Error::Configuration(format!("无法创建数据库目录: {}", e).into()))?;
    }

    // 如果数据库文件不存在，创建一个空文件
    if !db_path.exists() {
        std::fs::File::create(&db_path)
            .map_err(|e| sqlx::Error::Configuration(format!("无法创建数据库文件: {}", e).into()))?;
        println!("✅ 创建新的数据库文件: {}", db_path.display());
    }

    // 创建数据库连接池
    let pool = SqlitePool::connect(&db_url).await?;

    // 运行数据库迁移
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    println!("✅ 数据库初始化完成");
    Ok(pool)
}

// 获取所有用户
async fn get_users(state: AppState) -> Result<serde_json::Value, String> {
    let rows =
        sqlx::query("SELECT id, name, email, created_at FROM users ORDER BY created_at DESC")
            .fetch_all(&state.db)
            .await
            .map_err(|e| format!("数据库查询错误: {}", e))?;

    let users: Vec<User> = rows
        .into_iter()
        .map(|row| User {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
            created_at: row
                .get::<String, _>("created_at")
                .parse()
                .unwrap_or(Utc::now()),
        })
        .collect();

    Ok(json!({
        "success": true,
        "data": users,
        "count": users.len()
    }))
}

// 创建新用户
async fn create_user(ctx: RequestCtx, state: AppState) -> Result<serde_json::Value, String> {
    let req: CreateUserRequest = ctx.json().map_err(|e| format!("请求体解析错误: {}", e))?;

    let user_id = Uuid::new_v4().to_string();
    let created_at = Utc::now();

    sqlx::query("INSERT INTO users (id, name, email, created_at) VALUES (?, ?, ?, ?)")
        .bind(&user_id)
        .bind(&req.name)
        .bind(&req.email)
        .bind(created_at.to_rfc3339())
        .execute(&state.db)
        .await
        .map_err(|e| format!("创建用户失败: {}", e))?;

    let user = User {
        id: user_id,
        name: req.name,
        email: req.email,
        created_at,
    };

    Ok(json!({
        "success": true,
        "message": "用户创建成功",
        "data": user
    }))
}

// 获取特定用户
async fn get_user(ctx: RequestCtx, state: AppState) -> Result<serde_json::Value, String> {
    let user_id = ctx.get_param("id").ok_or("缺少用户ID参数")?;

    let row = sqlx::query("SELECT id, name, email, created_at FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("数据库查询错误: {}", e))?;

    match row {
        Some(row) => {
            let user = User {
                id: row.get("id"),
                name: row.get("name"),
                email: row.get("email"),
                created_at: row
                    .get::<String, _>("created_at")
                    .parse()
                    .unwrap_or(Utc::now()),
            };

            Ok(json!({
                "success": true,
                "data": user
            }))
        }
        None => Err("用户不存在".to_string()),
    }
}

// 更新用户
async fn update_user(ctx: RequestCtx, state: AppState) -> Result<serde_json::Value, String> {
    let user_id = ctx.get_param("id").ok_or("缺少用户ID参数")?;
    let req: UpdateUserRequest = ctx.json().map_err(|e| format!("请求体解析错误: {}", e))?;

    // 检查用户是否存在
    let existing = sqlx::query("SELECT id FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("数据库查询错误: {}", e))?;

    if existing.is_none() {
        return Err("用户不存在".to_string());
    }

    // 动态构建更新查询
    let mut query_parts = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(name) = req.name {
        query_parts.push("name = ?");
        bind_values.push(name);
    }

    if let Some(email) = req.email {
        query_parts.push("email = ?");
        bind_values.push(email);
    }

    if query_parts.is_empty() {
        return Err("没有提供要更新的字段".to_string());
    }

    let query = format!("UPDATE users SET {} WHERE id = ?", query_parts.join(", "));
    bind_values.push(user_id.to_string());

    let mut sqlx_query = sqlx::query(&query);
    for value in &bind_values {
        sqlx_query = sqlx_query.bind(value);
    }

    sqlx_query
        .execute(&state.db)
        .await
        .map_err(|e| format!("更新用户失败: {}", e))?;

    // 返回更新后的用户
    get_user(ctx, state).await
}

// 删除用户
async fn delete_user(ctx: RequestCtx, state: AppState) -> Result<serde_json::Value, String> {
    let user_id = ctx.get_param("id").ok_or("缺少用户ID参数")?;

    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| format!("删除用户失败: {}", e))?;

    if result.rows_affected() == 0 {
        return Err("用户不存在".to_string());
    }

    Ok(json!({
        "success": true,
        "message": "用户删除成功"
    }))
}
