use chrono::{DateTime, Utc};
use s_web::{Engine, IntoResponse, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde::{Deserialize, Serialize};

// =============================================================================
// 自定义响应结构体
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub code: u16,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
            timestamp: Utc::now(),
            code: 200,
        }
    }

    pub fn error(message: String, code: u16) -> Self {
        Self {
            success: false,
            data: None,
            message,
            timestamp: Utc::now(),
            code,
        }
    }
}

// 为我们的自定义结构体实现 IntoResponse
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status_code = match self.code {
            200..=299 => StatusCode::from_u16(self.code).unwrap_or(StatusCode::OK),
            400..=499 => StatusCode::from_u16(self.code).unwrap_or(StatusCode::BAD_REQUEST),
            500..=599 => {
                StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            _ => StatusCode::OK,
        };

        match serde_json::to_string(&self) {
            Ok(json) => ResponseBuilder::new()
                .status(status_code)
                .header("Content-Type", "application/json")
                .header("X-Powered-By", "s_web Framework")
                .body(json),
            Err(_) => ResponseBuilder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(r#"{"success":false,"message":"Serialization error"}"#),
        }
    }
}

// =============================================================================
// 分页响应结构体
// =============================================================================

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct Pagination {
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(json) => ResponseBuilder::new()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("X-Total-Count", self.total.to_string())
                .header("X-Page", self.pagination.page.to_string())
                .header("X-Page-Size", self.pagination.page_size.to_string())
                .body(json),
            Err(_) => ResponseBuilder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(r#"{"success":false,"message":"Serialization error"}"#),
        }
    }
}

// =============================================================================
// 业务模型
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub age: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub total_users: usize,
    pub active_users: usize,
    pub new_users_today: usize,
    pub average_age: f32,
}

// =============================================================================
// 错误处理结构体
// =============================================================================

#[derive(Debug)]
pub enum AppError {
    NotFound,
    ValidationError(String),
    DatabaseError,
    Unauthorized,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound => write!(f, "Resource not found"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            AppError::DatabaseError => write!(f, "Database error occurred"),
            AppError::Unauthorized => write!(f, "Unauthorized access"),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (message, code) = match self {
            AppError::NotFound => ("Resource not found".to_string(), 404),
            AppError::ValidationError(msg) => (format!("Validation error: {msg}"), 400),
            AppError::DatabaseError => ("Database error occurred".to_string(), 500),
            AppError::Unauthorized => ("Unauthorized access".to_string(), 401),
        };

        ApiResponse::<()>::error(message, code).into_response()
    }
}

// =============================================================================
// 模拟数据
// =============================================================================

fn get_mock_users() -> Vec<User> {
    vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: Utc::now(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: Utc::now(),
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
            created_at: Utc::now(),
        },
        User {
            id: 4,
            name: "Diana".to_string(),
            email: "diana@example.com".to_string(),
            age: 28,
            created_at: Utc::now(),
        },
    ]
}

// =============================================================================
// 路由处理器
// =============================================================================

async fn get_users(_ctx: RequestCtx) -> ApiResponse<Vec<User>> {
    let users = get_mock_users();
    ApiResponse::success(users)
}

async fn get_user_by_id(ctx: RequestCtx) -> Result<ApiResponse<User>, AppError> {
    let id = ctx
        .get_param("id")
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| AppError::ValidationError("Invalid user ID".to_string()))?;

    let users = get_mock_users();
    let user = users
        .into_iter()
        .find(|u| u.id == id)
        .ok_or(AppError::NotFound)?;

    Ok(ApiResponse::success(user))
}

async fn get_users_paginated(ctx: RequestCtx) -> PaginatedResponse<User> {
    let page = ctx
        .get_param("page")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1);

    let page_size = 2; // 每页2个用户
    let users = get_mock_users();
    let total = users.len();
    let total_pages = total.div_ceil(page_size);

    let start = (page - 1) * page_size;
    let end = (start + page_size).min(total);
    let items = users[start..end].to_vec();

    PaginatedResponse {
        items,
        pagination: Pagination {
            page,
            page_size,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
        total,
    }
}

async fn get_user_stats(_ctx: RequestCtx) -> ApiResponse<UserStats> {
    let users = get_mock_users();
    let stats = UserStats {
        total_users: users.len(),
        active_users: users.len() - 1, // 假设有一个不活跃
        new_users_today: 2,
        average_age: users.iter().map(|u| u.age as f32).sum::<f32>() / users.len() as f32,
    };

    ApiResponse::success(stats)
}

async fn simulate_error(_ctx: RequestCtx) -> AppError {
    AppError::DatabaseError
}

async fn simulate_not_found(_ctx: RequestCtx) -> AppError {
    AppError::NotFound
}

async fn health_check(_ctx: RequestCtx) -> ApiResponse<String> {
    ApiResponse::success("Server is healthy! 🚀".to_string())
}

// =============================================================================
// 主函数
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    println!("🎨 自定义响应类型示例 - s_web Framework");
    println!("===============================================");

    // 基本路由
    app.get("/", |_| async {
        ApiResponse::success("Welcome to Custom Response Example! 🎉".to_string())
    })
    // 健康检查
    .get("/health", health_check)
    // 用户相关路由
    .get("/users", get_users)
    .get("/users/:id", get_user_by_id)
    .get("/users/page/:page", get_users_paginated)
    .get("/stats", get_user_stats)
    // 错误处理示例
    .get("/error", simulate_error)
    .get("/notfound", simulate_not_found);

    println!("\n📚 可用的端点:");
    println!("  GET  /                     - 欢迎消息");
    println!("  GET  /health               - 健康检查");
    println!("  GET  /users                - 获取所有用户");
    println!("  GET  /users/1              - 获取ID为1的用户");
    println!("  GET  /users/999            - 用户不存在示例");
    println!("  GET  /users/invalid        - 无效ID示例");
    println!("  GET  /users/page/1         - 分页用户列表 (第1页)");
    println!("  GET  /users/page/2         - 分页用户列表 (第2页)");
    println!("  GET  /stats                - 用户统计数据");
    println!("  GET  /error                - 模拟数据库错误");
    println!("  GET  /notfound             - 模拟资源不存在");

    println!("\n🚀 服务器启动在: http://127.0.0.1:8080");
    app.run("127.0.0.1:8080").await?;

    Ok(())
}
