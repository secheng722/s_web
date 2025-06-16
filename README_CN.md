# Ree HTTP Framework

🚀 现代化、简洁高效的 Rust HTTP 框架，基于 Hyper 构建，具有**零成本中间件**和**优雅的宏驱动开发**体验。

## ✨ 特性

- **🎯 简洁直观的 API**: 易于使用的路由和处理器系统
- **🔄 自动类型转换**: 直接返回各种类型（String、JSON、Result、Option 等）
- **⚡ 高性能**: 基于 Hyper，利用 Rust 的零成本抽象
- **� 强大的中间件系统**: 基于函数的中间件，支持宏开发
- **📦 路由组**: 使用前缀和组专用中间件组织路由
- **🔒 类型安全**: 编译时保证请求/响应处理的正确性
- **🎨 宏魔法**: `#[middleware]` 宏让中间件开发变得优雅

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
use ree::{Engine, handler};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 直接返回 &str - 自动转换为 text/plain 响应
    app.get("/hello", handler(|_| async { "Hello, World!" }));
    
    // 直接返回 JSON - 自动转换为 application/json 响应
    app.get("/json", handler(|_| async { 
        json!({
            "message": "你好 JSON",
            "framework": "Ree",
            "version": "0.1.0"
        })
    }));
    
    // 使用路径参数
    app.get("/hello/:name", handler(|ctx| async move {
        if let Some(name) = ctx.get_param("name") {
            format!("你好, {}!", name)
        } else {
            "你好, 匿名用户!".to_string()
        }
    }));
    
    // 返回 Result - 自动处理错误
    app.get("/result", handler(|_| async {
        let result: Result<&str, &str> = Ok("成功!");
        result  // Ok -> 200, Err -> 500
    }));
    
    // 返回 Option - 自动处理 None
    app.get("/option", handler(|_| async {
        let data: Option<&str> = Some("找到了!");
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

## 🛠 革命性的中间件系统

### `#[middleware]` 宏

Ree 引入了颠覆性的 `#[middleware]` 宏，让中间件开发变得极其简单和优雅：

```rust
use ree::{middleware, Engine, RequestCtx, Next, Response, ResponseBuilder};

// 🎯 参数化中间件 - 简洁而强大！
#[middleware]
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == token {
            return next(ctx).await;
        }
    }
    ResponseBuilder::unauthorized_json(r#"{"error": "未授权"}"#)
}

// 🎯 简单中间件 - 保持一致的风格
#[middleware]
async fn request_logger(ctx: RequestCtx, next: Next) -> Response {
    println!("📨 {} {}", ctx.request.method(), ctx.request.uri().path());
    let response = next(ctx).await;
    println!("✅ 响应: {}", response.status());
    response
}

// 🎯 JWT 认证 - 强大而简单
#[middleware]
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
    
    // 全局中间件
    app.use_middleware(request_logger);
    
    // 带简单认证的路由组
    {
        let api = app.group("/api");
        api.use_middleware(auth("Bearer secret-token"));
        api.get("/users", handler(|_| async { "受保护的用户数据" }));
    }
    
    // JWT 保护的路由
    {
        let secure = app.group("/secure");  
        secure.use_middleware(jwt_auth("my-secret-key"));
        secure.get("/profile", handler(|_| async { "用户个人资料" }));
    }
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}
```

### 🎨 更多中间件示例

```rust
// 限流中间件
#[middleware]
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
#[middleware]
async fn cors(ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE".parse().unwrap());
    response
}

// 自定义来源的 CORS
#[middleware]
async fn cors_custom(origin: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    response.headers_mut().insert("Access-Control-Allow-Origin", origin.parse().unwrap());
    response
}

// 请求 ID 中间件
#[middleware]
async fn request_id(ctx: RequestCtx, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    println!("🆔 请求 ID: {}", request_id);
    
    let mut response = next(ctx).await;
    response.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());
    response
}

// 使用示例
app.use_middleware(cors);                           // 简单 CORS
app.use_middleware(cors_custom("https://app.com")); // 自定义来源 CORS
app.use_middleware(rate_limit(100));                // 100 个请求限制
app.use_middleware(request_id);                     // 添加请求 ID
```
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

**使用宏后（简洁）:**
```rust
#[middleware]
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // 清晰、可读的异步函数
    // 只需自然地编写你的逻辑！
}
```

`#[middleware]` 宏自动处理：
- ✅ **复杂的返回类型** - 不再需要 `Pin<Box<dyn Future<...>>>`
- ✅ **参数绑定** - 清晰的参数传递
- ✅ **Send/Sync 约束** - 自动实现 trait
- ✅ **类型推导** - Rust 编译器理解一切

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

### 📦 路由组

使用前缀和组专用中间件组织你的路由：

```rust
let mut app = Engine::new();

// API v1 组
let api_v1 = app.group("/api/v1");
api_v1.use_middleware(request_logger);
api_v1.use_middleware(auth("Bearer api-v1-token"));
api_v1.get("/users", handler(|_| async { "API v1 用户" }));
api_v1.post("/users", handler(|_| async { "在 v1 中创建用户" }));

// 带不同认证的 API v2 组
let api_v2 = app.group("/api/v2");
api_v2.use_middleware(jwt_auth("v2-secret"));
api_v2.get("/users", handler(|_| async { "API v2 用户" }));

// 带多个中间件的管理组
let admin = app.group("/admin");
admin.use_middleware(jwt_auth("admin-secret"));
admin.use_middleware(require_role("admin"));
admin.get("/users", handler(|_| async { "管理员用户列表" }));
admin.delete("/users/:id", handler(|ctx| async move {
    if let Some(id) = ctx.get_param("id") {
        format!("已删除用户 {}", id)
    } else {
        "无效的用户 ID".to_string()
    }
}));
```

## 🏃‍♂️ 运行示例

仓库包含展示框架不同方面的综合示例：

```bash
# 基本 API 用法
cargo run --example api_guide

# 综合中间件示例
cargo run --example middleware_guide

# 大型应用结构
cargo run --example large_app_example
```

然后访问：
- **基本路由**: 
  - http://127.0.0.1:3000/ - 框架主页
  - http://127.0.0.1:3000/hello/张三 - 带参数的问候
  - http://127.0.0.1:3000/health - 健康检查端点
  
- **中间件示例**:
  - http://127.0.0.1:3000/api/users - 简单认证保护的路由
  - http://127.0.0.1:3000/jwt/profile - JWT 保护的路由
  - http://127.0.0.1:3000/admin/users - 需要管理员角色
  
- **认证测试**:
  ```bash
  # 获取 JWT 令牌
  curl -X POST http://127.0.0.1:3000/auth/login
  
  # 测试简单认证
  curl -H 'Authorization: Bearer secret-token' http://127.0.0.1:3000/api/users
  
  # 测试 JWT 认证
  curl -H 'Authorization: Bearer <jwt_token>' http://127.0.0.1:3000/jwt/profile
  ```

## 🎯 设计理念

### 简洁优先
- 在 99% 的用例中使用 `handler()` 和自动类型转换
- 框架为你处理 HTTP 响应的复杂性
- 编写自然的 Rust 代码，自动获得 HTTP 响应

### 强大而灵活
- 直接返回 `Response` 以精确控制头部和状态码
- 带宏支持的灵活中间件系统
- 零成本抽象 - 只为你使用的功能付出代价

### 开发者体验
- **宏魔法**: `#[middleware]` 消除复杂的类型签名
- **类型安全**: 编译时保证减少运行时错误
- **直观的 API**: 如果看起来正确，那可能就是正确的
- **综合示例**: 通过查看真实模式来学习

## 📚 示例概述

### 🔰 基本用法 (`examples/api_guide`)
- 简单路由和处理器
- 自动类型转换
- 路径参数和查询处理
- JSON 请求/响应处理

### 🛠 中间件指南 (`examples/middleware_guide`) 
- **认证**: 简单令牌和 JWT 认证
- **授权**: 基于角色的访问控制
- **限流**: 可配置限制的请求节流
- **CORS**: 跨域资源共享设置
- **日志**: 请求/响应日志和计时
- **错误处理**: 集中错误处理
- **请求 ID**: 跟踪和调试支持

### 🏗 大型应用示例 (`examples/large_app_example`)
- 模块化应用结构
- 职责分离
- 路由组织模式
- 中间件组合策略
- 配置管理

## 🚀 性能

- **零成本抽象**: 中间件编译为高效代码
- **基于 Hyper**: 利用 Rust 中最快的 HTTP 实现之一
- **最小开销**: 直接函数调用，无动态分发
- **内存高效**: 尽可能使用栈分配的中间件链

## 🤝 贡献

欢迎贡献！我们特别感兴趣：

- **新的中间件示例** - 展示 `#[middleware]` 宏的创造性用法
- **性能改进** - 保持快速和零成本
- **文档** - 帮助他人学习框架
- **测试** - 确保可靠性和正确性

请随时提交问题和拉取请求。

## 📝 许可证

本项目在 MIT 许可证下授权 - 详见 [LICENSE](LICENSE) 文件。

---

**用 Rust 构建，充满 ❤️** 🦀
