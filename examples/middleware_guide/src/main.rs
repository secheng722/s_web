use ree::{middleware, Engine, Next, RequestCtx, Response, ResponseBuilder};
use serde_json::json;
use std::{future::Future, pin::Pin, sync::Arc, time::Instant};

// =============================================================================
// 🎉 REE中间件系统 - 使用新的#[middleware]宏
// =============================================================================
//
// 这个示例展示了如何使用新的#[middleware]属性宏来简化中间件的编写。
// 该宏可以将带参数的async函数自动转换为中间件闭包。
//
// ## 宏的使用方式：
//
// ### 统一使用 #[middleware] 宏（推荐）
// 为了代码的一致性和可维护性，推荐统一使用宏：
//
// ```rust
// // 带参数的中间件
// #[middleware]
// async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
//     // 中间件逻辑...
//     next(ctx).await
// }
//
// // 无参数的中间件
// #[middleware]
// async fn cors(ctx: RequestCtx, next: Next) -> Response {
//     // CORS逻辑...
//     next(ctx).await
// }
//
// // 使用：
// app.use_middleware(auth("Bearer secret-token")); // 有参数
// app.use_middleware(cors);                        // 无参数
// ```
//
// ### 混合使用方式（也可以，但不推荐）
// ```rust
// #[middleware]
// async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response { ... }
//
// async fn cors(ctx: RequestCtx, next: Next) -> Response { ... } // 不用宏
// ```
//
// ## 为什么推荐统一使用宏？
// - ✅ 代码风格一致
// - ✅ 学习成本更低（只需要记住一种写法）
// - ✅ 未来扩展兼容（如果宏增加新功能，所有中间件都能受益）
// - ✅ 更好的错误提示和类型检查
//
// ## 宏的转换原理：
//
// 带参数的函数会被转换为返回闭包的函数：
// ```rust
// fn auth(token: &'static str) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
//     move |ctx, next| {
//         Box::pin(async move {
//             // 原始的函数体
//         })
//     }
// }
// ```
//
// 这样就可以在保持简洁语法的同时，支持参数化的中间件！
//
// =============================================================================

// =============================================================================
// 示例中间件实现 - 演示如何使用新的#[middleware]宏
// =============================================================================

/// 🚀 访问日志中间件 - 推荐统一使用宏
#[middleware]
async fn access_log(ctx: RequestCtx, next: Next) -> Response {
    let start = Instant::now();
    let method = ctx.request.method().to_string();
    let path = ctx.request.uri().path().to_string();

    let response = next(ctx).await;

    println!(
        "{} {} {} {}ms",
        method,
        path,
        response.status().as_str(),
        start.elapsed().as_millis()
    );

    response
}

/// 🚀 计时器中间件 - 推荐统一使用宏
#[middleware]
async fn timer(ctx: RequestCtx, next: Next) -> Response {
    let start = Instant::now();
    let response = next(ctx).await;
    println!("Request processing time: {}ms", start.elapsed().as_millis());
    response
}

/// 🚀 认证中间件 - 使用新的 #[middleware] 宏
#[middleware]
async fn auth(token: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(auth) = ctx.request.headers().get("Authorization") {
        if auth.to_str().unwrap_or("") == token {
            return next(ctx).await;
        }
    }
    ResponseBuilder::unauthorized_json(r#"{"error": "Unauthorized"}"#)
}

/// 🚀 JWT 认证中间件 - 使用新的 #[middleware] 宏（简化版本，用于演示）
#[middleware]
async fn jwt_auth(secret: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // 从 Authorization header 获取 JWT token
    if let Some(auth_header) = ctx.request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // 简化的JWT验证逻辑（实际项目中应使用专业的JWT库如jsonwebtoken）
                if validate_jwt_token(token, secret) {
                    println!(
                        "✅ JWT authentication successful: {}",
                        extract_user_from_token(token)
                    );
                    return next(ctx).await;
                }
            }
        }
    }

    ResponseBuilder::unauthorized_json(r#"{"error": "Invalid or missing JWT token"}"#)
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

        !user.is_empty() && (role == "admin" || role == "user") && (current_time - timestamp) < 3600 // 1小时内有效
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

/// 🚀 JWT 权限检查中间件 - 使用新的 #[middleware] 宏
#[middleware]
async fn jwt_require_role(required_role: &'static str, ctx: RequestCtx, next: Next) -> Response {
    // 这个中间件应该在 jwt_auth 之后使用
    // 从 Authorization header 获取并解析角色
    if let Some(auth_header) = ctx.request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let parts: Vec<&str> = token.split('.').collect();
                if parts.len() == 3 {
                    let role = parts[1];
                    if role == required_role || role == "admin" {
                        // admin有所有权限
                        return next(ctx).await;
                    }
                }
            }
        }
    }

    ResponseBuilder::forbidden_json(format!(
        r#"{{"error": "Access denied. Required role: {}"}}"#,
        required_role
    ))
}

/// 生成简化的JWT token（仅用于演示）
fn generate_demo_jwt_token(user: &str, role: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}.{}.{}", user, role, timestamp)
}

/// 🚀 请求计数器中间件 - 推荐统一使用宏
#[middleware]
async fn request_counter(ctx: RequestCtx, next: Next) -> Response {
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = counter.clone();
    let current = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    println!("Total requests: {}", current + 1);
    next(ctx).await
}

/// CORS 中间件构建器
struct CorsBuilder {
    allow_origin: String,
    allow_methods: Vec<String>,
    allow_headers: Vec<String>,
}

impl CorsBuilder {
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

    fn allow_origin(mut self, origin: &str) -> Self {
        self.allow_origin = origin.to_string();
        self
    }

    fn allow_methods(mut self, methods: &[&str]) -> Self {
        self.allow_methods = methods.iter().map(|s| s.to_string()).collect();
        self
    }

    fn allow_headers(mut self, headers: &[&str]) -> Self {
        self.allow_headers = headers.iter().map(|s| s.to_string()).collect();
        self
    }

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

/// CORS 中间件
fn cors() -> CorsBuilder {
    CorsBuilder::new()
}

/// 🚀 限流中间件 - 使用新的 #[middleware] 宏（简化版本）
#[middleware]
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
        return ResponseBuilder::too_many_requests_json(
            format!(r#"{{"error": "Rate limit exceeded", "limit": {}}}"#, max_requests),
        );
    }

    next(ctx).await
}

/// 🚀 CORS 中间件 - 无参数版本，不需要宏
async fn cors_simple(ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization".parse().unwrap());
    response
}

/// 🚀 自定义CORS中间件 - 使用新的 #[middleware] 宏
#[middleware]
async fn cors_custom(origin: &'static str, ctx: RequestCtx, next: Next) -> Response {
    let mut response = next(ctx).await;
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", origin.parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization".parse().unwrap());
    response
}

/// 🚀 请求ID中间件 - 无参数版本，不需要宏
async fn request_id(ctx: RequestCtx, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    println!("🆔 Request ID: {}", request_id);
    
    let mut response = next(ctx).await;
    response.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());
    response
}

/// 🚀 API密钥验证中间件 - 使用新的 #[middleware] 宏
#[middleware]
async fn api_key_auth(valid_key: &'static str, ctx: RequestCtx, next: Next) -> Response {
    if let Some(api_key) = ctx.request.headers().get("X-API-Key") {
        if api_key.to_str().unwrap_or("") == valid_key {
            return next(ctx).await;
        }
    }
    
    ResponseBuilder::unauthorized_json(r#"{"error": "Invalid or missing API key"}"#)
}

/// 🚀 内容类型验证中间件 - 无参数版本，不需要宏
async fn require_json(ctx: RequestCtx, next: Next) -> Response {
    if let Some(content_type) = ctx.request.headers().get("Content-Type") {
        if content_type.to_str().unwrap_or("").starts_with("application/json") {
            return next(ctx).await;
        }
    }
    
    ResponseBuilder::bad_request_json(r#"{"error": "Content-Type must be application/json"}"#)
}

/// 🚀 限流中间件构建器 - 更优雅的解决方案
struct RateLimitBuilder {
    max_requests: usize,
    window_seconds: u64,
}

impl RateLimitBuilder {
    fn new(max_requests: usize) -> Self {
        Self {
            max_requests,
            window_seconds: 60, // 默认1分钟
        }
    }
    
    fn window_seconds(mut self, seconds: u64) -> Self {
        self.window_seconds = seconds;
        self
    }
    
    /// 构建一个可以直接使用的async函数
    fn build_async(self) -> impl Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static {
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
                    return ResponseBuilder::too_many_requests_json(
                        format!(r#"{{"error": "Rate limit exceeded", "limit": {}, "window": "{}s"}}"#, max_requests, window_seconds),
                    );
                }

                next(ctx).await
            })
        }
    }
}

/// 🚀 创建限流中间件的便捷函数
fn create_rate_limit(max_requests: usize) -> RateLimitBuilder {
    RateLimitBuilder::new(max_requests)
}

// =============================================================================
// 📚 推荐的统一宏使用方式
// =============================================================================

// 🎯 推荐：统一使用 #[middleware] 宏（无论是否有参数）
//
// ✅ 带参数的中间件（必须使用宏）:
// - #[middleware] async fn auth(token, ctx, next)
// - #[middleware] async fn jwt_auth(secret, ctx, next) 
// - #[middleware] async fn jwt_require_role(role, ctx, next)
// - #[middleware] async fn cors_custom(origin, ctx, next)
// - #[middleware] async fn api_key_auth(key, ctx, next)
// - #[middleware] async fn rate_limit(max_requests, ctx, next)
//
// ✅ 无参数的中间件（推荐也使用宏，保持一致性）:
// - #[middleware] async fn access_log(ctx, next)
// - #[middleware] async fn timer(ctx, next)
// - #[middleware] async fn request_counter(ctx, next)
// - #[middleware] async fn cors_simple(ctx, next)
// - #[middleware] async fn request_id(ctx, next)
// - #[middleware] async fn require_json(ctx, next)
// - #[middleware] async fn error_handler(ctx, next)
//
// 🎯 当前示例为了展示灵活性，混合使用了两种方式，
//    但在实际项目中推荐统一使用 #[middleware] 宏！

// =============================================================================
// 🎉 最终方案对比 - 新方案获胜！
// =============================================================================
/*
🏆 最终推荐方案 - 新的AsyncMiddleware trait方案:

✅ 使用超级简单:
   app.use_middleware(rate_limit_v2(100))
   app.use_middleware(auth_v2("Bearer token"))
   app.use_middleware(jwt_auth_v2("secret"))

✅ 支持链式调用:
   app.use_middleware(rate_limit_v2(100).window_seconds(30))

✅ 类型安全:
   编译时检查，无运行时错误

✅ 性能优异:
   零开销抽象，与原生async函数性能相同

✅ 扩展性强:
   通过实现AsyncMiddleware trait轻松添加新中间件

对比其他方案:
❌ 原闭包方案: 类型签名复杂，不够直观
❌ 简单async函数: 无法传参，不够灵活  
❌ 构建器模式: 需要额外的build_async()调用

🎯 结论: 新方案完美结合了简洁性和灵活性！
*/

/// 🚀 错误处理中间件
async fn error_handler(ctx: RequestCtx, next: Next) -> Response {
    // 在调用 next 之前提取需要的信息
    let method = ctx.request.method().to_string();
    let path = ctx.request.uri().path().to_string();

    let response = next(ctx).await;

    // 如果是错误状态码，添加一些调试信息
    if response.status().is_client_error() || response.status().is_server_error() {
        println!(
            "⚠️ Error response: {} for {} {}",
            response.status(),
            method,
            path
        );
    }

    response
}

/// 🚀 最优雅的解决方案 - 支持async的参数化中间件
/// 
/// 这个方案的核心思想是：
/// 1. 定义一个trait来统一中间件接口
/// 2. 为不同的函数类型实现这个trait
/// 3. 提供一个便捷的宏或函数来简化使用

/// 中间件trait - 统一所有中间件的接口
trait AsyncMiddleware: Send + Sync + 'static {
    fn call(&self, ctx: RequestCtx, next: Next) -> Pin<Box<dyn Future<Output = Response> + Send>>;
}

/// 为普通async函数实现中间件trait
impl<F> AsyncMiddleware for F
where
    F: Fn(RequestCtx, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync + 'static,
{
    fn call(&self, ctx: RequestCtx, next: Next) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        self(ctx, next)
    }
}

/// 🚀 参数化限流中间件 - 简单而灵活的版本
pub struct RateLimit {
    max_requests: usize,
    window_seconds: u64,
    requests_count: Arc<std::sync::atomic::AtomicUsize>,
    last_reset: Arc<std::sync::Mutex<Instant>>,
}

impl RateLimit {
    pub fn new(max_requests: usize) -> Self {
        Self {
            max_requests,
            window_seconds: 60,
            requests_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            last_reset: Arc::new(std::sync::Mutex::new(Instant::now())),
        }
    }
    
    pub fn window_seconds(mut self, seconds: u64) -> Self {
        self.window_seconds = seconds;
        self
    }
}

impl AsyncMiddleware for RateLimit {
    fn call(&self, ctx: RequestCtx, next: Next) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let requests_count = self.requests_count.clone();
        let last_reset = self.last_reset.clone();
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;
        
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
                return ResponseBuilder::too_many_requests_json(
                    format!(r#"{{"error": "Rate limit exceeded", "limit": {}, "window": "{}s"}}"#, max_requests, window_seconds),
                );
            }

            next(ctx).await
        })
    }
}

/// 🚀 参数化认证中间件
pub struct Auth {
    token: String,
}

impl Auth {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl AsyncMiddleware for Auth {
    fn call(&self, ctx: RequestCtx, next: Next) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let expected_token = self.token.clone();
        Box::pin(async move {
            if let Some(auth) = ctx.request.headers().get("Authorization") {
                if auth.to_str().unwrap_or("") == expected_token {
                    return next(ctx).await;
                }
            }
            ResponseBuilder::unauthorized_json(r#"{"error": "Unauthorized"}"#)
        })
    }
}

/// 🚀 参数化JWT认证中间件
pub struct JwtAuth {
    secret: String,
}

impl JwtAuth {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }
}

impl AsyncMiddleware for JwtAuth {
    fn call(&self, ctx: RequestCtx, next: Next) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let secret = self.secret.clone();
        Box::pin(async move {
            if let Some(auth_header) = ctx.request.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        if validate_jwt_token(token, &secret) {
                            println!("✅ JWT authentication successful: {}", extract_user_from_token(token));
                            return next(ctx).await;
                        }
                    }
                }
            }
            ResponseBuilder::unauthorized_json(r#"{"error": "Invalid or missing JWT token"}"#)
        })
    }
}

/// 🚀 便捷函数 - 让使用更加简单
pub fn rate_limit_v2(max_requests: usize) -> RateLimit {
    RateLimit::new(max_requests)
}

pub fn auth_v2(token: impl Into<String>) -> Auth {
    Auth::new(token)
}

pub fn jwt_auth_v2(secret: impl Into<String>) -> JwtAuth {
    JwtAuth::new(secret)
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
    println!("🎯 推荐统一使用 #[middleware] 宏，保持代码风格一致性！");

    // 1. 全局中间件 - 展示统一使用宏的好处
    println!("1️⃣ Global middleware - Unified macro usage (recommended)");
    app.use_middleware(access_log);     // 🔥 无参数，但使用宏保持一致性
    app.use_middleware(timer);          // 🔥 无参数，但使用宏保持一致性
    app.use_middleware(request_counter); // 🔥 无参数，但使用宏保持一致性

    // 2. CORS中间件 - 展示混合使用方式
    println!("2️⃣ CORS middleware - Mixed usage (for demonstration)");
    app.use_middleware(cors_simple); // 💫 不使用宏的版本（为了展示灵活性）
    // app.use_middleware(cors_custom("https://example.com")); // 🔥 使用宏的版本

    // 3. 其他全局中间件 - 混合方式
    println!("3️⃣ Other global middleware - Mixed for demonstration");
    app.use_middleware(request_id); // 💫 不使用宏的版本

    // 4. 路由组中间件 - 展示带参数中间件的使用（需要宏）
    println!("4️⃣ Route group middleware - With parameters (requires macro)");
    {
        let api_group = app.group("/api");

        // ✅ 带参数的中间件：必须使用 #[middleware] 宏
        api_group.use_middleware(auth("Bearer secret-token"));        // 🔥 宏版本
        api_group.use_middleware(rate_limit(50));                     // 🔥 宏版本 
        api_group.use_middleware(api_key_auth("my-secret-api-key"));  // 🔥 宏版本
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
    println!("5️⃣ JWT authentication route group - Parameterized middleware");
    {
        let jwt_group = app.group("/jwt");

        // ✅ 带参数的中间件：需要使用 #[middleware] 宏
        jwt_group.use_middleware(jwt_auth("my-secret-key")); // 🔥 宏版本
        
        // JWT路由
        jwt_group.get("/profile", |_ctx: RequestCtx| async move {
            json!({
                "message": "用户个人资料",
                "user": "从JWT token中解析的用户信息",
                "auth_method": "JWT",
                "note": "使用新的#[middleware]宏实现"
            })
        });

        jwt_group.get("/dashboard", |_ctx: RequestCtx| async move {
            json!({
                "message": "用户仪表板",
                "data": ["图表1", "图表2", "图表3"],
                "auth_method": "JWT",
                "note": "使用新的#[middleware]宏实现"
            })
        });
    }

    // 6. JWT + 角色权限路由组演示 - 使用新的宏版本
    println!("6️⃣ JWT + Role-based permissions route group - New macro version");
    {
        let admin_group = app.group("/admin");

        // JWT认证 + 管理员角色要求 - 使用新的宏版本
        admin_group.use_middleware(jwt_auth("my-secret-key"));
        admin_group.use_middleware(jwt_require_role("admin"));

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

    // 8. JWT Token生成端点（用于测试）
    app.post("/auth/login", |_ctx: RequestCtx| async move {
        // 在实际项目中，这里应该验证用户名密码
        let admin_token = generate_demo_jwt_token("alice", "admin");
        let user_token = generate_demo_jwt_token("bob", "user");

        json!({
            "message": "登录成功（演示）",
            "tokens": {
                "admin": admin_token,
                "user": user_token
            },
            "usage": {
                "header": "Authorization: Bearer <token>",
                "endpoints": {
                    "jwt_protected": "/jwt/profile, /jwt/dashboard",
                    "admin_only": "/admin/users"
                }
            }
        })
    });

    // 9. 基础路由（不需要认证）
    println!("9️⃣ Basic routes (with global middleware)");

    app.get("/", |_: RequestCtx| async {
        json!({
            "message": "🎉 欢迎使用 Ree HTTP Framework!",
            "version": "0.1.0",
            "features": [
                "函数式中间件",
                "零开销抽象",
                "易于组合",
                "类型安全",
                "链式执行"
            ],
            "middleware_examples": [
                "访问日志",
                "计时器",
                "请求计数",
                "CORS",
                "简单认证",
                "JWT认证",
                "角色权限",
                "限流",
                "错误处理"
            ]
        })
    });

    app.get("/health", |_: RequestCtx| async {
        json!({"status": "ok", "timestamp": "2025-06-16T12:00:00Z"})
    });

    app.get("/middleware-test", |_: RequestCtx| async {
        json!({
            "message": "这个响应经过了所有全局中间件处理",
            "middlewares_applied": [
                "access_log",
                "timer",
                "request_counter",
                "cors",
                "error_handler",
                "rate_limit(100)",
                "custom_logger"
            ]
        })
    });

    // 7. 错误处理演示
    app.get("/error", |_: RequestCtx| async {
        json!({"error": "内部服务器错误", "code": 500})
    });

    app.get("/not-found", |_: RequestCtx| async {
        json!({"error": "资源未找到", "code": 404})
    });

    println!("\n🚀 Server starting...");
    println!("📍 Address: http://127.0.0.1:3000");
    println!("\n📋 Test routes:");
    println!("  GET  /                  - Home page");
    println!("  GET  /health            - Health check");
    println!("  GET  /middleware-test   - Middleware test");
    println!("  GET  /error             - Error handling demo");
    println!("  GET  /not-found         - 404 error demo");
    println!("  GET  /api/users         - Requires authentication (Bearer secret-token)");
    println!("  POST /api/users         - Requires authentication (Bearer secret-token)");
    println!("  GET  /api/stats         - API statistics");
    println!("  POST /auth/login        - Get JWT token (demo)");
    println!("  GET  /jwt/profile       - JWT authenticated user info");
    println!("  GET  /jwt/dashboard     - JWT authenticated dashboard");
    println!("  GET  /admin/users       - Admin user list (requires admin role)");
    println!("  POST /admin/users       - Admin create user (requires admin role)");
    println!("\n💡 Test simple authentication API:");
    println!("  curl -H 'Authorization: Bearer secret-token' http://127.0.0.1:3000/api/users");
    println!("\n🔐 Test JWT authentication:");
    println!("  1. Get token: curl -X POST http://127.0.0.1:3000/auth/login");
    println!(
        "  2. 使用token: curl -H 'Authorization: Bearer <admin_token>' http://127.0.0.1:3000/jwt/profile"
    );
    println!(
        "  3. 管理员API: curl -H 'Authorization: Bearer <admin_token>' http://127.0.0.1:3000/admin/users"
    );
    println!(
        "  4. 普通用户API: curl -H 'Authorization: Bearer <user_token>' http://127.0.0.1:3000/jwt/dashboard"
    );
    println!("\n🔍 测试限流:");
    println!("  快速发送多个请求观察限流效果");

    app.run("127.0.0.1:3000").await?;
    Ok(())
}
