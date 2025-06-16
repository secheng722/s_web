# Ree HTTP Framework

一个简单高效的 Rust HTTP 框架，基于 Hyper 构建，提供简洁的 API 和强大的类型转换功能。

## 特性

- 🚀 基于 Tokio 的异步处理
- 🛣️ 灵活的路由系统，支持路径参数和通配符
- 🔧 中间件支持
- 📦 路由组支持
- ✨ **自动类型转换** - 支持直接返回 `&str`、`String`、`serde_json::Value` 等类型
- 🎯 简洁易用的 API 设计

## 快速开始

### 添加依赖

```toml
[dependencies]
ree = { git = "https://github.com/your-username/ree.git" }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"  # 如果需要 JSON 支持
```

### 简洁的处理器写法（推荐）

```rust
use ree::{Engine, handler};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 直接返回 &str - 自动转换为 text/plain 响应
    app.get("/hello", handler(|_| async { "Hello, World!" }));
    
    // 直接返回 String
    app.get("/time", handler(|_| async { 
        format!("Current time: {}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs())
    }));
    
    // 直接返回 JSON - 自动转换为 application/json 响应
    app.get("/json", handler(|_| async { 
        json!({
            "message": "Hello JSON",
            "status": "success"
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

### 高级用法 - 精确控制响应

当需要精确控制响应头、状态码等时，可以直接返回 `Response`：

```rust
use ree::{Engine, ResponseBuilder, RequestCtx, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();
    
    // 精确控制响应
    app.get("/custom", custom_handler);
    
    app.run("127.0.0.1:8080").await?;
    Ok(())
}

async fn custom_handler(_ctx: RequestCtx) -> Response {
    let mut response = ResponseBuilder::with_json(r#"{"message": "Custom response"}"#);
    response.headers_mut().insert("X-Custom-Header", "MyValue".parse().unwrap());
    response
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
