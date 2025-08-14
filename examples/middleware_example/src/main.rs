use ree::{Engine, IntoResponse, Next, RequestCtx, Response, ResponseBuilder};
use serde_json::json;
use std::{future::Future, pin::Pin, sync::Arc, time::Instant};

// =============================================================================
// 🎉 REE中间件系统 - 使用更简洁的中间件写法
// =============================================================================
//
// 这个示例展示了如何使用更简洁的中间件函数来简化中间件的编写。
// 这种方式不需要使用宏，直接使用普通的异步函数即可。
//
// ## 中间件的使用方式：
//
// ```rust
// // 直接使用闭包
// app.use_middleware(|ctx, next| async move {
//     // 中间件逻辑...
//     next(ctx).await
// });
//
// // 使用带参数的中间件函数
// app.use_middleware(|ctx, next| logging("Prefix", ctx, next));
//
// // 定义中间件函数
// async fn logging(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
//     // 中间件逻辑...
//     next(ctx).await
// }
// ```
//
// ## 为什么这种方式更好？
// - ✅ 代码更简洁，没有宏的复杂性
// - ✅ 更直观，容易理解
// - ✅ 更灵活，可以轻松组合中间件
// - ✅ 标准Rust语法，无需特殊处理
//
// =============================================================================

// =============================================================================
// 示例中间件实现
// =============================================================================

/// 🚀 访问日志中间件
async fn _access_log(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let start = Instant::now();
    let method = ctx.request.method().to_string();
    let path = ctx.request.uri().path().to_string();

    println!("[{prefix}] 开始处理请求: {method} {path}");

    let response = next(ctx).await;

    println!(
        "[{}] 完成请求: {} {} {} {}ms",
        prefix,
        method,
        path,
        response.status().as_str(),
        start.elapsed().as_millis()
    );

    response
}

/// 🚀 计时器中间件
async fn timer(name: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let start = Instant::now();
    let response = next(ctx).await;
    println!("[{}] 请求处理时间: {}ms", name, start.elapsed().as_millis());
    response
}

#[allow(dead_code)]
/// 🚀 认证中间件
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization")
        && auth.to_str().unwrap_or("") == token {
            return next(ctx).await;
        }
    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "Unauthorized"}),
    )
        .into_response()
}

/// 🔐 简单认证中间件
async fn auth_simple(token_value: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // 从请求头中获取认证令牌
    let auth_header = match ctx.request.headers().get("Authorization") {
        Some(header) => header,
        None => {
            return ResponseBuilder::new()
                .status(ree::StatusCode::UNAUTHORIZED)
                .header("Content-Type", "application/json")
                .body(json!({"error": "缺少认证头"}).to_string());
        }
    };

    // 验证令牌
    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            return ResponseBuilder::new()
                .status(ree::StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(json!({"error": "无效的认证头"}).to_string());
        }
    };

    // 检查令牌是否有效
    if auth_str != format!("Bearer {token_value}") {
        return ResponseBuilder::new()
            .status(ree::StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "error": "无效的令牌",
                    "message": "提供的认证令牌无效或已过期"
                })
                .to_string(),
            );
    }

    // 认证通过，继续处理请求
    next(ctx).await
}

/// 🚀 JWT 认证中间件 - 使用简洁的函数式风格
async fn jwt_auth(secret: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // 从 Authorization header 获取 JWT token
    if let Some(auth_header) = ctx.request.headers().get("Authorization")
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        // 简化的JWT验证逻辑（实际项目中应使用专业的JWT库如jsonwebtoken）
        if validate_jwt_token(token, secret) {
            println!(
                "✅ JWT authentication successful: {}",
                extract_user_from_token(token)
            );
            return next(ctx).await;
        }
    }

    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "Invalid or missing JWT token"}),
    )
        .into_response()
}

/// 简化的JWT验证函数（仅用于演示）
fn validate_jwt_token(token: &str, _secret: &str) -> bool {
    // 这里是一个简化的验证逻辑
    // 实际项目中应该：
    // 1. 解析JWT的header、payload、signature
    // 2. 验证签名
    // 3. 检查过期时间
    // 4. 验证issuer、audience等claim

    // 演示：假设token格式为 "user.role.timestamp"
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() == 3 {
        // 简单检查：用户名不为空，角色有效，时间戳不太旧
        let user = parts[0];
        let role = parts[1];
        let timestamp = parts[2].parse::<u64>().unwrap_or(0);
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        !user.is_empty() && (role == "admin" || role == "user") && (current_time - timestamp) < 3600
    // 1小时内有效
    } else {
        false
    }
}

/// 从JWT token中提取用户信息（简化版本）
fn extract_user_from_token(token: &str) -> String {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() == 3 {
        format!("{}({})", parts[0], parts[1])
    } else {
        "unknown".to_string()
    }
}

/// 🚀 JWT 权限检查中间件 - 使用简洁的函数式风格
async fn jwt_require_role(required_role: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // 这个中间件应该在 jwt_auth 之后使用
    // 从 Authorization header 获取并解析角色
    if let Some(auth_header) = ctx.request.headers().get("Authorization")
        && let Ok(auth_str) = auth_header.to_str()
            && let Some(token) = auth_str.strip_prefix("Bearer ") {
                let parts: Vec<&str> = token.split('.').collect();
                if parts.len() == 3 {
                    let role = parts[1];
                    if role == required_role || role == "admin" {
                        // admin有所有权限
                        return next(ctx).await;
                    }
                }
            }

    (
        ree::StatusCode::FORBIDDEN,
        json!({"error": format!("Access denied. Required role: {}", required_role)}),
    )
        .into_response()
}

/// 生成简化的JWT token（仅用于演示）
fn _generate_demo_jwt_token(user: &str, role: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{user}.{role}.{timestamp}")
}

/// 🚀 请求计数器中间件
async fn request_counter(ctx: RequestCtx, next: Next) -> Response {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let current = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    println!("总请求次数: {}", current + 1);
    next(ctx).await
}

/// CORS 中间件构建器
#[allow(dead_code)]
struct CorsBuilder {
    allow_origin: String,
    allow_methods: Vec<String>,
    allow_headers: Vec<String>,
}

impl CorsBuilder {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            allow_origin: "*".to_string(),
            allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allow_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
        }
    }

    #[allow(dead_code)]
    fn allow_origin(mut self, origin: &str) -> Self {
        self.allow_origin = origin.to_string();
        self
    }

    #[allow(dead_code)]
    fn allow_methods(mut self, methods: &[&str]) -> Self {
        self.allow_methods = methods.iter().map(|s| s.to_string()).collect();
        self
    }

    #[allow(dead_code)]
    fn allow_headers(mut self, headers: &[&str]) -> Self {
        self.allow_headers = headers.iter().map(|s| s.to_string()).collect();
        self
    }

    #[allow(dead_code)]
    fn build(
        self,
    ) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
    + Send
    + Sync
    + 'static {
        let origin = self.allow_origin;
        let methods = self.allow_methods.join(", ");
        let headers = self.allow_headers.join(", ");

        move |ctx, next| {
            let origin = origin.clone();
            let methods = methods.clone();
            let headers = headers.clone();
            Box::pin(async move {
                let mut response = next(ctx).await;

                let resp_headers = response.headers_mut();
                resp_headers.insert("Access-Control-Allow-Origin", origin.parse().unwrap());
                resp_headers.insert("Access-Control-Allow-Methods", methods.parse().unwrap());
                resp_headers.insert("Access-Control-Allow-Headers", headers.parse().unwrap());

                response
            })
        }
    }
}

/// 🚀 限流中间件 - 使用简洁的函数式风格
async fn rate_limit(max_requests: usize, ctx: RequestCtx, next: Next) -> Response {
    use std::sync::atomic::{AtomicUsize, Ordering};

    // 使用全局静态计数器（简化实现）
    static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static LAST_RESET: std::sync::OnceLock<std::sync::Mutex<Instant>> = std::sync::OnceLock::new();

    let last_reset = LAST_RESET.get_or_init(|| std::sync::Mutex::new(Instant::now()));

    // 每分钟重置计数器
    {
        let mut last_reset = last_reset.lock().unwrap();
        if last_reset.elapsed().as_secs() > 60 {
            GLOBAL_COUNTER.store(0, Ordering::SeqCst);
            *last_reset = Instant::now();
        }
    }

    let current = GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst);

    if current >= max_requests {
        return ResponseBuilder::new()
            .status(ree::StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "application/json")
            .body(json!({"error": "Rate limit exceeded", "limit": max_requests}).to_string());
    }

    next(ctx).await
}

/// 🚀 CORS 中间件 - 无参数版本，不需要宏
async fn cors_simple(ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization".parse().unwrap(),
    );
    response
}

/// 🚀 自定义CORS中间件 - 使用简洁的函数式风格
#[allow(dead_code)]
async fn cors_custom(origin: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", origin.parse().unwrap());
    headers.insert(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization".parse().unwrap(),
    );
    response
}

/// 🚀 请求ID中间件 - 无参数版本，不需要宏
async fn request_id(ctx: RequestCtx, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    println!("🆔 Request ID: {request_id}");

    let mut response = next(ctx).await;
    response
        .headers_mut()
        .insert("X-Request-ID", request_id.parse().unwrap());
    response
}

/// 🚀 API密钥验证中间件 - 使用简洁的函数式风格
async fn api_key_auth(valid_key: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(api_key) = ctx.request.headers().get("X-API-Key")
        && api_key.to_str().unwrap_or("") == valid_key {
            return next(ctx).await;
        }

    (
        ree::StatusCode::UNAUTHORIZED,
        json!({"error": "Invalid or missing API key"}),
    )
        .into_response()
}

/// 🚀 内容类型验证中间件 - 无参数版本，不需要宏
#[allow(dead_code)]
async fn require_json(ctx: RequestCtx, next: Next) -> Response {
    if let Some(content_type) = ctx.request.headers().get("Content-Type")
        && content_type
            .to_str()
            .unwrap_or("")
            .starts_with("application/json")
        {
            return next(ctx).await;
        }

    (
        ree::StatusCode::BAD_REQUEST,
        json!({"error": "Content-Type must be application/json"}),
    )
        .into_response()
}

/// 🚀 限流中间件构建器 - 更优雅的解决方案
#[allow(dead_code)]
struct RateLimitBuilder {
    max_requests: usize,
    window_seconds: u64,
}

impl RateLimitBuilder {
    #[allow(dead_code)]
    fn new(max_requests: usize) -> Self {
        Self {
            max_requests,
            window_seconds: 60, // 默认1分钟
        }
    }

    #[allow(dead_code)]
    fn window_seconds(mut self, seconds: u64) -> Self {
        self.window_seconds = seconds;
        self
    }

    /// 构建一个可以直接使用的async函数
    #[allow(dead_code)]
    fn build_async(
        self,
    ) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>>
    + Send
    + Sync
    + 'static {
        let requests_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let last_reset = Arc::new(std::sync::Mutex::new(Instant::now()));
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;

        move |ctx, next| {
            let requests_count = requests_count.clone();
            let last_reset = last_reset.clone();
            Box::pin(async move {
                // 重置逻辑
                {
                    let mut last_reset = last_reset.lock().unwrap();
                    if last_reset.elapsed().as_secs() > window_seconds {
                        requests_count.store(0, std::sync::atomic::Ordering::SeqCst);
                        *last_reset = Instant::now();
                    }
                }

                let current = requests_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if current >= max_requests {
                    (
                        ree::StatusCode::TOO_MANY_REQUESTS,
                        json!({
                            "error": "Rate limit exceeded",
                            "limit": max_requests,
                            "window": format!("{} seconds", window_seconds)
                        }),
                    )
                        .into_response();
                }

                next(ctx).await
            })
        }
    }
}

/// 🚀 创建限流中间件的便捷函数
#[allow(dead_code)]
fn create_rate_limit(max_requests: usize) -> RateLimitBuilder {
    RateLimitBuilder::new(max_requests)
}

/// 🚀 错误处理中间件
#[allow(dead_code)]
async fn error_handler(ctx: RequestCtx, next: Next) -> Response {
    // 尝试执行下一个处理器，并捕获可能的错误
    let response = next(ctx).await;

    // 检查状态码是否为错误
    if response.status().is_server_error() {
        println!("服务器错误: {}", response.status());

        // 这里可以记录错误，发送告警等

        // 在生产环境中，你可能想要用更友好的错误消息替换原始错误
        // 这里只是简单地返回原始响应
    } else if response.status().is_client_error() {
        println!("客户端错误: {}", response.status());

        // 可以记录客户端错误以分析API使用问题
    }

    response
}

/// 🌐 CORS中间件
#[allow(dead_code)]
async fn cors(ctx: RequestCtx, next: Next) -> Response {
    let response = next(ctx).await;

    // 添加CORS头
    ResponseBuilder::new()
        .status(response.status())
        .header("Access-Control-Allow-Origin", "*")
        .header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE, OPTIONS",
        )
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .empty_body()
}

// =============================================================================
// 📚 函数式中间件使用指南
// =============================================================================

// 🎯 函数式中间件的几种使用方式:
//
// ✅ 直接使用简单的闭包:
// ```rust
// app.use_middleware(|ctx, next| async move {
//     println!("处理请求前");
//     let response = next(ctx).await;
//     println!("处理请求后");
//     response
// });
// ```
//
// ✅ 使用带参数的辅助函数:
// ```rust
// // 定义带参数的中间件函数
// async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
//     // 验证逻辑...
//     next(ctx).await
// }
//
// // 使用方式
// app.use_middleware(|ctx, next| auth("Bearer token", ctx, next));
// ```
//
// ✅ 使用无参数的辅助函数:
// ```rust
// // 定义无参数的中间件函数
// async fn cors(ctx: RequestCtx, next: Next) -> Response {
//     // CORS逻辑...
//     next(ctx).await
// }
//
// // 使用方式
// app.use_middleware(cors);
// ```
//
// 🎯 优势:
// - 简洁直观的语法
// - 标准Rust语法，无需宏
// - 类型安全，编译时检查
// - 易于测试单独的中间件函数
// - 极高的灵活性和组合性
// - 零运行时开销
//
// 🎯 最佳实践:
// - 对有共性的中间件逻辑提取为函数
// - 有参数的中间件使用|ctx, next|闭包包装
// - 无参数的中间件可以直接传递函数名
// - 参数用&'static str保证静态生命周期
// - 复杂的中间件考虑使用构建器模式

/// 增强版可配置限流中间件
/// 这个函数允许自定义窗口时间周期
pub fn advanced_rate_limit(
    max_requests: usize,
    window_seconds: u64,
) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static
{
    // 使用静态计数器和上次重置时间
    let requests_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let last_reset = Arc::new(std::sync::Mutex::new(Instant::now()));

    move |ctx, next| {
        let requests_count = requests_count.clone();
        let last_reset = last_reset.clone();

        Box::pin(async move {
            // 检查是否需要重置计数器
            {
                let mut last_reset_guard = last_reset.lock().unwrap();
                if last_reset_guard.elapsed().as_secs() > window_seconds {
                    requests_count.store(0, std::sync::atomic::Ordering::SeqCst);
                    *last_reset_guard = Instant::now();
                }
            }

            // 增加计数并检查限制
            let current = requests_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if current >= max_requests {
                (
                    ree::StatusCode::TOO_MANY_REQUESTS,
                    json!({
                        "error": "Rate limit exceeded",
                        "limit": max_requests,
                        "window": format!("{} seconds", window_seconds)
                    }),
                )
                    .into_response();
            }

            next(ctx).await
        })
    }
}

// =============================================================================
// 主应用程序
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    println!("🛠 Ree HTTP Framework - Function-based Middleware System");
    println!("════════════════════════════════════════════════════════");
    println!("✨ Modern function-based middleware API, zero-cost and composable!");
    println!("🎯 使用简洁的函数式中间件，更直观、更灵活！");

    // 1. 全局中间件 - 使用简洁的函数式写法
    println!("1️⃣ Global middleware - Simple function-based middleware");
    app.use_middleware(|ctx, next| logging("访问日志", ctx, next));
    app.use_middleware(|ctx, next| timer("请求计时器", ctx, next));

    // 2. CORS中间件和请求计数器
    println!("2️⃣ CORS middleware and request counter");
    app.use_middleware(cors_simple); // 无参数的中间件
    app.use_middleware(request_counter); // 无参数的中间件

    // 3. 其他全局中间件 - 混合方式
    println!("3️⃣ Other global middleware - Mixed for demonstration");
    app.use_middleware(request_id); // 💫 不使用宏的版本

    // 4. 路由组中间件 - 展示带参数函数式中间件的使用
    println!("4️⃣ Route group middleware - With parameters using function-based approach");
    {
        let api_group = app.group("/api");

        // 使用函数式中间件写法，更加简洁直观
        api_group.use_middleware(|ctx, next| auth_simple("Bearer secret-token", ctx, next));
        api_group.use_middleware(|ctx, next| rate_limit(50, ctx, next));
        api_group.use_middleware(|ctx, next| api_key_auth("my-secret-api-key", ctx, next));
        api_group.use_middleware(|ctx, next| {
            Box::pin(async move {
                println!("🚦 API Group: Validating request format");
                // 这里可以添加请求格式验证逻辑
                next(ctx).await
            })
        });

        // API 路由
        api_group.get("/users", |_ctx: RequestCtx| async move {
            json!({
                "users": [
                    {"id": 1, "name": "Alice", "role": "admin"},
                    {"id": 2, "name": "Bob", "role": "user"}
                ],
                "total": 2
            })
        });

        api_group.post("/users", |_ctx: RequestCtx| async move {
            json!({"message": "用户创建成功", "id": 3, "status": "created"})
        });

        api_group.get("/stats", |_ctx: RequestCtx| async move {
            json!({
                "api_version": "v1.0",
                "uptime": "1 day",
                "requests_today": 1234,
                "middleware_chain": [
                    "global: access_log",
                    "global: timer",
                    "global: request_counter",
                    "global: cors",
                    "global: error_handler",
                    "global: rate_limit(100)",
                    "global: custom_logger",
                    "api_group: auth",
                    "api_group: rate_limit(10)",
                    "api_group: request_validator"
                ]
            })
        });
    }

    // 5. JWT 认证路由组 - 展示带参数中间件的使用
    println!("5️⃣ JWT authentication route group - Function-based middleware");
    {
        let jwt_group = app.group("/jwt");

        // 使用函数式中间件写法，更加简洁直观
        jwt_group.use_middleware(|ctx, next| jwt_auth("my-secret-key", ctx, next));

        // JWT路由
        jwt_group.get("/profile", |_ctx: RequestCtx| async move {
            json!({
                "message": "用户个人资料",
                "user": "从JWT token中解析的用户信息",
                "auth_method": "JWT",
                "note": "使用简洁的函数式中间件实现"
            })
        });

        jwt_group.get("/dashboard", |_ctx: RequestCtx| async move {
            json!({
                "message": "用户仪表板",
                "data": ["图表1", "图表2", "图表3"],
                "auth_method": "JWT",
                "note": "使用简洁的函数式中间件实现"
            })
        });
    }

    // 6. JWT + 角色权限路由组演示 - 使用新的函数式中间件
    println!("6️⃣ JWT + Role-based permissions route group - Function-based middleware");
    {
        let admin_group = app.group("/admin");

        // JWT认证 + 管理员角色要求 - 使用函数式中间件
        admin_group.use_middleware(|ctx, next| jwt_auth("my-secret-key", ctx, next));
        admin_group.use_middleware(|ctx, next| jwt_require_role("admin", ctx, next));

        admin_group.get("/users", |_ctx: RequestCtx| async move {
            json!({
                "message": "管理员：用户列表",
                "users": [
                    {"id": 1, "name": "Alice", "role": "admin"},
                    {"id": 2, "name": "Bob", "role": "user"},
                    {"id": 3, "name": "Charlie", "role": "user"}
                ],
                "auth_method": "JWT + Role"
            })
        });

        admin_group.post("/users", |_ctx: RequestCtx| async move {
            json!({
                "message": "管理员：创建用户成功",
                "auth_method": "JWT + Role"
            })
        });
    }

    // 7. 高级限流中间件演示
    println!("7️⃣ Advanced rate limiter with configurable window");
    {
        let limiter_group = app.group("/limiter");

        // 使用高级限流中间件 - 配置10秒窗口，最多5个请求
        limiter_group.use_middleware(advanced_rate_limit(5, 10));

        // 限流测试路由
        limiter_group.get("/test", |_ctx: RequestCtx| async move {
            // 模拟处理时间
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            json!({
                "message": "限流测试成功",
                "limit": "5个请求/10秒",
                "info": "快速多次访问此接口会触发限流"
            })
        });
    }

    // 添加最终的日志中间件演示
    app.get("/demo/logging", |_ctx: RequestCtx| async move {
        json!({
            "message": "这是一个演示各种日志中间件的端点",
            "timestamp": "2025-06-16T12:00:00Z"
        })
    });

    // 使用自定义日志中间件
    app.use_middleware(|ctx, next| logging("全局日志中间件", ctx, next));

    println!("\n🚀 Server starting...");
    println!("📍 Address: http://127.0.0.1:3000");
    println!("\n📋 测试路由:");
    println!("  GET  /                    - 主页");
    println!("  GET  /api/users           - 需要认证 (Bearer secret-token)");
    println!("  GET  /api/stats           - API统计信息");
    println!("  GET  /jwt/profile         - JWT认证用户信息");
    println!("  GET  /admin/users         - 需要admin角色权限");
    println!("  GET  /limiter/test        - 限流测试 (5次/10秒)");
    println!("  GET  /demo/logging        - 日志中间件演示");

    println!("\n💡 测试日志中间件:");
    println!("  curl http://127.0.0.1:3000/demo/logging");
    println!("\n💡 测试认证API:");
    println!("  curl -H 'Authorization: Bearer secret-token' http://127.0.0.1:3000/api/users");
    println!("\n💡 测试限流:");
    println!("  快速多次执行: curl http://127.0.0.1:3000/limiter/test");

    println!("\n🔥 新的函数式中间件让开发更简洁高效！");

    app.run("127.0.0.1:3000").await?;
    Ok(())
}

// =============================================================================
// 路由处理器
// =============================================================================

/// 首页处理器
#[allow(dead_code)]
async fn index(_ctx: RequestCtx) -> Response {
    json!({
        "message": "欢迎使用REE框架",
        "version": "1.0.0",
        "description": "一个简单、高效的Rust Web框架"
    })
    .into_response()
}

/// 用户信息处理器
#[allow(dead_code)]
async fn user_info(ctx: RequestCtx) -> Response {
    // 获取URL参数
    if let Some(user_id) = ctx.get_param("id") {
        json!({
            "id": user_id,
            "name": "测试用户",
                "email": "test@example.com",
                "created_at": "2025-01-01T00:00:00Z"
        })
        .into_response()
    } else {
        json!({
            "error": "缺少用户ID"
        })
        .into_response()
    }
}

/// 模拟API处理器
#[allow(dead_code)]
async fn api_handler(_ctx: RequestCtx) -> Response {
    // 故意延迟一点来测试计时器中间件
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    json!({
        "data": {
            "items": [
                { "id": 1, "name": "项目1" },
                { "id": 2, "name": "项目2" },
                { "id": 3, "name": "项目3" }
            ],
            "total": 3
        }
    })
    .into_response()
}

/// 受保护的API处理器
#[allow(dead_code)]
async fn protected_api(_ctx: RequestCtx) -> Response {
    json!({
        "message": "认证成功，你已访问受保护的资源",
        "data": {
            "sensitive": true,
            "value": "这是一个需要认证才能访问的秘密数据",
            "timestamp": "2025-06-16T10:00:00Z"
        }
    })
    .into_response()
}

/// 模拟错误处理器
#[allow(dead_code)]
async fn error_demo(_ctx: RequestCtx) -> Response {
    json!({
        "error": "这是一个模拟的服务器错误",
        "code": "SERVER_ERROR_DEMO"
    })
    .into_response()
}

/// 日志中间件的辅助函数 - 使用函数式风格
async fn logging(prefix: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let start = Instant::now();
    let path = ctx.request.uri().path().to_string();
    let method = ctx.request.method().clone();

    println!("[{prefix}] 📝 处理请求: {method} {path}");

    let response = next(ctx).await;

    let status = response.status();
    let status_str = if status.is_success() {
        format!("✅ {status}")
    } else if status.is_client_error() {
        format!("⚠️ {status}")
    } else if status.is_server_error() {
        format!("❌ {status}")
    } else {
        format!("ℹ️ {status}")
    };

    println!(
        "[{}] 🏁 请求完成: {} {} {} ({}ms)",
        prefix,
        method,
        path,
        status_str,
        start.elapsed().as_millis()
    );

    response
}
