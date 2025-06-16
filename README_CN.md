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

## 🚀 快速开始

### 添加依赖

```toml
[dependencies]
ree = { git = "https://github.com/secheng722/ree" }
tokio = { version = "1.45.1", features = ["full"] }
serde_json = "1.0"
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
    app.get("/hello/:name", handler(|ctx| async move {
        if let Some(name) = ctx.get_param("name") {
            format!("Hello, {}!", name)
        } else {
            "Hello, Anonymous!".to_string()
        }
    }));
    
    // 返回 Result - 自动处理错误
    app.get("/result", handler(|_| async {
        let result: Result<&str, &str> = Ok("Success!");
        result  // Ok -> 200, Err -> 500
    }));
    
    // 返回 Option - 自动处理 None
    app.get("/option", handler(|_| async {
        let data: Option<&str> = Some("Found!");
        data  // Some -> 200, None -> 404
    }));
    
    // 自定义状态码
    app.get("/created", handler(|_| async {
        (ree::StatusCode::CREATED, "Resource created")
    }));
    
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
    ResponseBuilder::unauthorized_json(r#"{"error": "未授权"}"#)
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
    ResponseBuilder::unauthorized_json(r#"{"error": "无效或缺失的 JWT 令牌"}"#)
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

### 🎨 更多中间件示例

```rust
// 限流中间件
async fn rate_limit(max_requests: usize, ctx: RequestCtx, next: Next) -> Response {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    
    let current = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if current >= max_requests {
        return ResponseBuilder::too_many_requests_json(
            r#"{"error": "请求过于频繁"}"#
        );
    }
    
    next(ctx).await
}

// CORS 中间件
async fn cors(ctx: RequestCtx, next: Next) -> Response {
    let response = next(ctx).await;
    
    let mut builder = hyper::Response::builder()
        .status(response.status())
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization");
    
    if ctx.request.method() == hyper::Method::OPTIONS {
        return builder
            .header("Access-Control-Max-Age", "86400")
            .body(response.into_body())
            .unwrap();
    }
    
    let mut new_response = builder.body(response.into_body()).unwrap();
    for (key, value) in response.headers() {
        if key != "access-control-allow-origin" && 
           key != "access-control-allow-methods" && 
           key != "access-control-allow-headers" {
            new_response.headers_mut().insert(key.clone(), value.clone());
        }
    }
    
    new_response
}

// 自定义来源的 CORS
async fn cors_custom(origin: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    response.headers_mut().insert("Access-Control-Allow-Origin", origin.parse().unwrap());
    response
}

// 请求 ID 中间件
async fn request_id(ctx: RequestCtx, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    println!("🆔 请求 ID: {}", request_id);
    
    let mut response = next(ctx).await;
    response.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());
    response
}

// 使用示例:
let mut app = Engine::new();
app.use_middleware(|ctx, next| cors(ctx, next));
app.use_middleware(request_id);
app.use_middleware(|ctx, next| rate_limit(100, ctx, next));
app.use_middleware(|ctx, next| cors_custom("https://example.com", ctx, next));

// 使用示例
app.use_middleware(cors);                           // 简单 CORS
app.use_middleware(cors_custom("https://app.com")); // 自定义来源 CORS
app.use_middleware(rate_limit(100));                // 100 个请求限制
app.use_middleware(request_id);                     // 添加请求 ID
```

### 高级用法 - 精确控制响应

当需要精确控制响应头、状态码等时，可以直接返回 `Response`：

```rust
use ree::{Engine, ResponseBuilder, RequestCtx, Response};

async fn custom_handler(_ctx: RequestCtx) -> Response {
    let mut response = ResponseBuilder::with_json(r#"{"message": "自定义响应"}"#);
    response.headers_mut().insert("X-Custom-Header", "MyValue".parse().unwrap());
    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 精确响应控制
    app.get("/custom", custom_handler);
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

### 中间件

```rust
use ree::{Engine, AccessLog};

let mut app = Engine::new();
app.use_middleware(AccessLog);
```

### 路由组

```rust
let api_group = app.group("/api");
api_group.get("/users", get_users_handler);
api_group.get("/users/:id", get_user_by_id_handler);
```

## 运行示例

```bash
cargo run --example hello_world
```

然后访问：
- http://127.0.0.1:8080/ - 基本问候
- http://127.0.0.1:8080/hello/张三 - 带参数的问候
- http://127.0.0.1:8080/api/users - 获取用户列表
- http://127.0.0.1:8080/api/users/1 - 获取特定用户

## API 文档

### Engine

主要的应用程序结构，用于配置路由和中间件。

#### 方法

- `new()` - 创建新的 Engine 实例
- `get(path, handler)` - 添加 GET 路由
- `group(prefix)` - 创建路由组
- `use_middleware(middleware)` - 添加中间件
- `run(addr)` - 启动服务器

### ResponseBuilder

用于构建 HTTP 响应的工具。

#### 方法

- `with_text(content)` - 创建文本响应
- `empty()` - 创建空响应

### RequestCtx

请求上下文，包含请求信息和路径参数。

#### 方法

- `get_param(key)` - 获取路径参数

### 🚀 为什么这很重要

**传统写法（复杂）:**
```rust
fn auth(token: &'static str) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
    move |ctx, next| {
        Box::pin(async move {
            // 复杂的嵌套结构
            // 难以阅读和编写
        })
    }
}
```

**使用函数式中间件（更简洁）:**
```rust
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // 清晰、可读的异步函数
    // 只需自然地编写你的逻辑！
}

// 使用时通过闭包传递参数
app.use_middleware(|ctx, next| auth("secret-token", ctx, next));
```

函数式中间件的优点：
- ✅ **标准Rust语法** - 使用普通异步函数，无需特殊语法
- ✅ **直观灵活** - 参数传递一目了然
- ✅ **结构清晰** - 中间件逻辑和使用分离
- ✅ **类型安全** - 完全利用Rust的类型系统

### 🌟 中间件系统的优势

这种函数式中间件风格相比传统中间件系统和其他框架实现有显著优势：

- **极简语法** - 使用标准的Rust函数，无需特殊宏或trait
- **灵活参数** - 可以轻松传递任意参数给中间件
- **类型安全** - 充分利用Rust的类型系统进行编译时检查
- **零运行开销** - 编译器优化确保最佳性能
- **直观易懂** - 降低学习曲线，新手也能快速掌握
- **易于测试** - 中间件函数可以单独测试
- **出色的组合性** - 中间件可以轻松组合或嵌套使用

#### 🔄 函数式中间件使用模式

```rust
// 1. 无参数中间件 - 直接传递函数名
app.use_middleware(cors);

// 2. 带参数中间件 - 使用闭包包装
app.use_middleware(|ctx, next| logging("API", ctx, next));

// 3. 内联中间件 - 直接编写闭包
app.use_middleware(|ctx, next| async move {
    println!("开始处理请求");
    let res = next(ctx).await;
    println!("请求处理完毕");
    res
});

// 4. 条件中间件 - 根据条件选择不同中间件
let auth_middleware = if config.is_secure {
    |ctx, next| auth("secure-token", ctx, next)
} else {
    |ctx, next| next(ctx)
};
app.use_middleware(auth_middleware);
```

### 高级用法 - 精确响应控制

当你需要精确控制响应头、状态码等时，可以直接返回 `Response`：

```rust
use ree::{Engine, ResponseBuilder, RequestCtx, Response};

async fn custom_handler(_ctx: RequestCtx) -> Response {
    let mut response = ResponseBuilder::with_json(r#"{"message": "自定义响应"}"#);
    response.headers_mut().insert("X-Custom-Header", "MyValue".parse().unwrap());
    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 精确响应控制
    app.get("/custom", custom_handler);
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

### 🧩 常见中间件示例

#### CORS中间件

```rust
async fn cors(ctx: RequestCtx, next: Next) -> Response {
    let response = next(ctx).await;
    
    let mut builder = hyper::Response::builder()
        .status(response.status())
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization");
    
    if ctx.request.method() == hyper::Method::OPTIONS {
        return builder
            .header("Access-Control-Max-Age", "86400")
            .body(response.into_body())
            .unwrap();
    }
    
    let mut new_response = builder.body(response.into_body()).unwrap();
    for (key, value) in response.headers() {
        if key != "access-control-allow-origin" && 
           key != "access-control-allow-methods" && 
           key != "access-control-allow-headers" {
            new_response.headers_mut().insert(key.clone(), value.clone());
        }
    }
    
    new_response
}
```

#### 限流中间件

```rust
async fn rate_limit(max_requests: usize, ctx: RequestCtx, next: Next) -> Response {
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    // 使用静态计数器（简化示例）
    static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static LAST_RESET: std::sync::OnceLock<std::sync::Mutex<std::time::Instant>> = 
        std::sync::OnceLock::new();
    
    let last_reset = LAST_RESET.get_or_init(|| std::sync::Mutex::new(std::time::Instant::now()));
    
    // 每分钟重置计数器
    {
        let mut last_reset = last_reset.lock().unwrap();
        if last_reset.elapsed().as_secs() > 60 {
            GLOBAL_COUNTER.store(0, Ordering::SeqCst);
            *last_reset = std::time::Instant::now();
        }
    }
    
    let current = GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    if current >= max_requests {
        return ResponseBuilder::too_many_requests_json(
            r#"{"error": "请求频率超限"}"#,
        );
    }

    next(ctx).await
}

// 使用方式
app.use_middleware(|ctx, next| rate_limit(100, ctx, next));
```

#### 错误处理中间件

```rust
async fn error_handler(ctx: RequestCtx, next: Next) -> Response {
    // 尝试执行下一个处理器，并捕获可能的错误
    let response = next(ctx).await;
    
    // 检查状态码是否为错误
    if response.status().is_server_error() {
        println!("服务器错误: {}", response.status());
        
        // 这里可以记录错误，发送告警等
        
        // 也可以替换为用户友好的错误响应
        return ResponseBuilder::new()
            .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "application/json")
            .json(json!({
                "error": "服务器内部错误",
                "message": "我们正在处理这个问题"
            }));
    }
    
    response
}
```
