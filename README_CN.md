# s_web HTTP Framework

🚀 现代化、简洁高效的 Rust HTTP 框架，基于 Hyper 构建，具有**零成本函数式中间件**、**优雅的开发体验**和**完整的 API 文档支持**。

## ✨ 特性

- **🎯 统一 API 设计**: 支持直接返回各种类型，自动转换
- **🔄 自动类型转换**: 直接返回 String、JSON、Result、Option、元组等
- **⚡ 高性能**: 基于 Hyper，零成本抽象
- **🧩 函数式中间件系统**: 优雅的参数传递中间件，支持链式调用
- **📦 路由组**: 使用前缀和组专用中间件组织路由
- **� 链式调用**: 中间件和路由都支持链式调用
- **�🔒 类型安全**: 编译时保证请求/响应处理的正确性
- **� 请求体处理**: 轻松解析 POST/PUT 请求体（JSON、文本、字节、表单）
- **🛑 生命周期管理**: 启动和关闭钩子，用于资源管理
- **📖 Swagger 集成**: 内置 Swagger UI，支持自定义文档

## 🚀 快速开始

### 添加依赖

```toml
[dependencies]
s_web = { git = "https://github.com/secheng722/s_web" }
tokio = { version = "1.45.1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 基本用法

```rust
use s_web::Engine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 直接返回类型 - 自动转换
    app.get("/text", |_| async { "Hello, World!" });
    app.get("/json", |_| async { 
        json!({"message": "你好 JSON", "framework": "s_web"})
    });
    
    // 路径参数
    app.get("/greet/:name", |ctx| async move {
        let name = ctx.get_param("name").unwrap_or("访客");
        format!("你好, {}!", name)
    });
    
    // Result 处理 - Ok -> 200, Err -> 500
    app.get("/result", |_| async {
        let result: Result<&str, &str> = Ok("成功!");
        result
    });
    
    // Option 处理 - Some -> 200, None -> 404
    app.get("/option", |_| async {
        let data: Option<&str> = Some("找到了!");
        data
    });
    
    // 自定义状态码
    app.post("/create", |_| async {
        (s_web::StatusCode::CREATED, json!({"id": 123}))
    });
    
    app.run("127.0.0.1:8080").await
}
```

## 🔗 链式调用和中间件

s_web 支持优雅的链式调用，包括路由和中间件：

```rust
use s_web::{Engine, RequestCtx, Next, Response, IntoResponse};
use serde_json::json;

// 带参数的中间件函数
async fn logger(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
    println!("[{}] {} {}", prefix, ctx.request.method(), ctx.request.uri().path());
    let start = std::time::Instant::now();
    let response = next(ctx).await;
    println!("[{}] 响应: {} ({}ms)", prefix, response.status(), start.elapsed().as_millis());
    response
}

async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == format!("Bearer {token}") {
            return next(ctx).await;
        }
    }
    (s_web::StatusCode::UNAUTHORIZED, json!({"error": "未授权"})).into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 全局中间件链式调用
    app.use_middleware(|ctx, next| logger("全局", ctx, next))
       .get("/", |_| async { "欢迎!" })
       .get("/health", |_| async { json!({"status": "ok"}) });
    
    // API 组的链式中间件和路由
    {
        let api = app.group("/api");
        api.use_middleware(|ctx, next| logger("API", ctx, next))
           .use_middleware(|ctx, next| auth("api-token", ctx, next))
           .get("/users", |_| async { json!(["alice", "bob"]) })
           .post("/users", |_| async { json!({"message": "用户已创建"}) })
           .get("/profile", |_| async { json!({"name": "当前用户"}) });
    }
    
    app.run("127.0.0.1:8080").await
}
```

## 📮 请求体处理

轻松解析 POST/PUT 请求体：

```rust
app.post("/json", |ctx: s_web::RequestCtx| async move {
    match ctx.body_json::<serde_json::Value>() {
        Ok(Some(json)) => format!("收到: {}", json),
        Ok(None) => "没有请求体".to_string(),
        Err(e) => format!("解析错误: {}", e),
    }
});

app.post("/text", |ctx: s_web::RequestCtx| async move {
    match ctx.body_string() {
        Ok(Some(text)) => format!("文本: {}", text),
        Ok(None) => "没有请求体".to_string(),
        Err(e) => format!("错误: {}", e),
    }
});

app.post("/bytes", |ctx: s_web::RequestCtx| async move {
    match ctx.body_bytes() {
        Some(bytes) => format!("收到 {} 字节", bytes.len()),
        None => "没有请求体".to_string(),
    }
});
```

## 🛑 生命周期管理

内置启动和关闭钩子：

```rust
use s_web::Engine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Engine::new()
        .on_startup(|| async {
            println!("🚀 初始化数据库...");
            // 初始化数据库、缓存等
        })
        .on_startup(|| async {
            println!("🔥 系统预热...");
            // 额外的启动任务
        })
        .on_shutdown(|| async {
            println!("🛑 清理资源...");
            // 清理数据库连接等
        })
        .on_shutdown(|| async {
            println!("✅ 关闭完成");
            // 最终清理
        });
    
    let mut app = app;
    app.get("/", |_| async { "你好!" });
    
    app.run("127.0.0.1:8080").await
}
```

## 📖 Swagger 文档

内置 Swagger UI，支持自定义文档：

```rust
use s_web::{Engine, swagger};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 自动生成文档
    app.get("/users", get_users);
    
    // 自定义 Swagger 文档
    app.post_with_swagger(
        "/users",
        create_user,
        swagger()
            .summary("创建新用户")
            .description("使用提供的数据创建新用户")
            .tag("用户")
            .request_body(json!({"name": "张三", "email": "zhangsan@example.com"}))
            .json_response("201", "用户已创建", Some(json!({"id": 1, "name": "张三"})))
            .build()
    );
    
    // Swagger UI 地址: http://127.0.0.1:3000/swagger-ui
    // OpenAPI JSON: http://127.0.0.1:3000/api-docs
    
    app.run("127.0.0.1:3000").await
}
```

## 🎯 高级用法

### 自定义响应构建器

精确控制 HTTP 响应：

```rust
use s_web::ResponseBuilder;

app.get("/custom", |_| async {
    ResponseBuilder::new()
        .status(s_web::StatusCode::OK)
        .content_type("application/json; charset=utf-8")
        .header("X-Custom-Header", "s_web-框架")
        .body(r#"{"message": "自定义响应"}"#)
});

app.get("/html", |_| async {
    ResponseBuilder::html(r#"
        <h1>来自 s_web 的问候!</h1>
        <p>这是一个 HTML 响应</p>
    "#)
});
```

### 嵌套中间件的路由组

```rust
// 需要认证的管理路由
{
    let admin = app.group("/admin");
    admin.use_middleware(|ctx, next| auth("admin-token", ctx, next))
         .get("/dashboard", |_| async { "管理面板" })
         .delete("/users/:id", |ctx| async move {
             let id = ctx.get_param("id").unwrap_or("0");
             format!("已删除用户 {}", id)
         });
}
```

## 📋 示例

仓库包含全面的示例：

- **`api_example`**: 统一 API 设计，展示自动类型转换
- **`chain_example`**: 中间件和路由的链式调用
- **`lifecycle_example`**: 启动/关闭钩子和资源管理
- **`swagger_custom_example`**: 自定义 Swagger 文档
- **`middleware_example`**: 高级中间件模式
- **`database_example`**: 数据库集成
- **`upload_example`**: 文件上传处理

## 🚦 测试

使用 curl 测试：

```bash
# JSON 请求体
curl -X POST http://127.0.0.1:8080/api/users \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer api-token" \
     -d '{"name": "张三", "email": "zhangsan@example.com"}'

# 表单数据
curl -X POST http://127.0.0.1:8080/form \
     -d "name=张三&email=zhangsan@example.com"

# 文件上传
curl -X POST http://127.0.0.1:8080/upload \
     -F "file=@example.txt"
```

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。