# Ree HTTP Framework

🚀 现代化、简洁高效的 Rust HTTP 框架，基于 Hyper 构建，具有**零成本函数式中间件**和**优雅的开发**体验。

## ✨ 特性

- **🎯 简洁直观的 API**: 易于使用的路由和处理器系统
- **🔄 自动类型转换**: 直接返回各种类型（String、JSON、Result、Option 等）
- **⚡ 高性能**: 基于 Hyper，利用 Rust 的零成本抽象
- **🧩 强大的中间件系统**: 基于函数的纯粹中间件，简洁优雅
- **📦 路由组**: 使用前缀和组专用中间件组织路由
- **🔒 类型安全**: 编译时保证请求/响应处理的正确性
- **🔗 函数式风格**: 直观的函数式中间件让开发变得轻松自然
- **🛑 优雅停机**： 支持优雅停机，可以安全地关闭 HTTP 服务器，同时确保正在处理的请求能够完成。
- **📖 自动生成的 Swagger 支持**: 所有注册的路由都会自动生成 Swagger 文档，并通过 Swagger UI 提供交互式 API 文档(由于某些限制，可能无法完全反映实际行为，post请求的json数据需要自己测试的时候手动修改)

## 🚀 快速开始

### 添加依赖

```toml
[dependencies]
ree = { git = "https://github.com/secheng722/ree" }
tokio = { version = "1.45.1", features = ["full"] }
```

### 简单处理器示例

```rust
use ree::Engine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 直接返回 &str - 自动转换为 text/plain 响应
    app.get("/hello", |_| async { "Hello, World!" });
    
    // 直接返回 JSON - 自动转换为 application/json 响应
    app.get("/json", |_| async { 
        json!({
            "message": "你好 JSON",
            "framework": "Ree",
            "version": "0.1.0"
        })
    }));
    
    // 使用路径参数
    app.get("/hello/:name", |ctx| async move {
        if let Some(name) = ctx.get_param("name") {
            format!("Hello, {}!", name)
        } else {
            "Hello, Anonymous!".to_string()
        }
    });
    
    // 返回 Result - 自动处理错误
    app.get("/result", |_| async {
        let result: Result<&str, &str> = Ok("Success!");
        result  // Ok -> 200, Err -> 500
    });
    
    // 返回 Option - 自动处理 None
    app.get("/option", |_| async {
        let data: Option<&str> = Some("Found!");
        data  // Some -> 200, None -> 404
    });
    
    // 自定义状态码
    app.get("/created", |_| async {
        (ree::StatusCode::CREATED, "Resource created")
    });

    // 链式调用
    // 因为中间件和路由都支持链式调用，可以更灵活地组织代码
    // 系统会自己处理中间件的执行顺序
    app.get("/chained", |_| async {
        "This is a chained response"
    }).get("/another", |_| async {
        "Another chained response"
    }).use_middleware(|ctx, next| async move {
        println!("Middleware executed");
        next(ctx).await
    });
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

## 🛠 优雅的函数式中间件系统

Ree 引入了一种极其简洁优雅的函数式中间件系统，使得编写和使用中间件变得前所未有的简单：

```rust
use ree::{Engine, RequestCtx, Next, Response, ResponseBuilder};

// 🎯 参数化中间件 - 简洁而强大！
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == format!("Bearer {}", token) {
            return next(ctx).await;
        }
    }
    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "Unauthorized"}),
    )
        .into_response()
}

// 🎯 日志中间件 - 简单直观
async fn logger(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
    println!("[{}] 📨 {} {}", prefix, ctx.request.method(), ctx.request.uri().path());
    let start = std::time::Instant::now();
    let response = next(ctx).await;
    println!("[{}] ✅ 响应: {} ({}ms)", prefix, response.status(), start.elapsed().as_millis());
    response
}

// 🎯 JWT 认证 - 强大而简单
async fn jwt_auth(secret: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth_header) = ctx.request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if validate_jwt_token(token, secret) {
                    return next(ctx).await;
                }
            }
        }
    }

    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "无效或缺失的 JWT 令牌"}),
    )
        .into_response()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 全局中间件 - 使用闭包传递参数
    app.use_middleware(|ctx, next| logger("全局", ctx, next));
    
    // 带简单认证的路由组
    {
        let api = app.group("/api");
        api.use_middleware(|ctx, next| auth("secret-token", ctx, next));
        api.get("/users", |_| async { "受保护的用户数据" });
    }
    
    // JWT 保护的路由
    {
        let secure = app.group("/secure");  
        secure.use_middleware(|ctx, next| jwt_auth("my-secret-key", ctx, next));
        secure.get("/profile", |_| async { "用户个人资料" });
    }
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```
