use ree::{RequestCtx, Response, ResponseBuilder, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
    avatar: Option<String>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "操作成功".to_string(),
        }
    }
    
    fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: message.to_string(),
        }
    }
}

/// 首页处理器
pub async fn home_handler(_ctx: RequestCtx) -> Response {
    let html = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Ree框架演示</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; }
        h1 { color: #333; }
        .endpoint { background: #f8f9fa; padding: 10px; margin: 10px 0; border-radius: 4px; }
        .method { font-weight: bold; color: #007bff; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 欢迎使用 Ree HTTP 框架</h1>
        <p>这是一个功能丰富的Rust HTTP框架演示。</p>
        
        <h2>可用端点:</h2>
        <div class="endpoint"><span class="method">GET</span> /health - 健康检查</div>
        <div class="endpoint"><span class="method">GET</span> /about - 关于页面</div>
        <div class="endpoint"><span class="method">GET</span> /docs - API文档</div>
        <div class="endpoint"><span class="method">GET</span> /api/v1/users - 用户列表</div>
        
        <h2>特性:</h2>
        <ul>
            <li>✅ 基于Trie的高效路由</li>
            <li>✅ 中间件支持</li>
            <li>✅ 路由组</li>
            <li>✅ CORS支持</li>
            <li>✅ 中文UTF-8支持</li>
            <li>✅ JSON响应</li>
        </ul>
    </div>
</body>
</html>
"#;
    ResponseBuilder::with_html(html)
}

/// 健康检查处理器
pub async fn health_check_handler(_ctx: RequestCtx) -> Response {
    let response = ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    }));
    
    ResponseBuilder::with_json(serde_json::to_string(&response).unwrap())
}

/// 关于页面处理器
pub async fn about_handler(_ctx: RequestCtx) -> Response {
    ResponseBuilder::with_text("Ree HTTP Framework - 一个简单高效的Rust Web框架")
}

/// 文档页面处理器
pub async fn docs_handler(_ctx: RequestCtx) -> Response {
    let docs = serde_json::json!({
        "name": "Ree API Documentation",
        "version": "1.0.0",
        "endpoints": [
            {
                "method": "GET",
                "path": "/api/v1/users",
                "description": "获取所有用户列表"
            },
            {
                "method": "POST",
                "path": "/api/v1/users",
                "description": "创建新用户"
            },
            {
                "method": "GET",
                "path": "/api/v1/users/:id",
                "description": "根据ID获取用户详情"
            }
        ]
    });
    
    ResponseBuilder::with_json(serde_json::to_string_pretty(&docs).unwrap())
}

/// 获取用户列表
pub async fn list_users_handler(_ctx: RequestCtx) -> Response {
    let users = vec![
        User {
            id: 1,
            name: "张三".to_string(),
            email: "zhangsan@example.com".to_string(),
            avatar: Some("https://avatar.example.com/1.jpg".to_string()),
        },
        User {
            id: 2,
            name: "李四".to_string(),
            email: "lisi@example.com".to_string(),
            avatar: None,
        },
        User {
            id: 3,
            name: "王五".to_string(),
            email: "wangwu@example.com".to_string(),
            avatar: Some("https://avatar.example.com/3.jpg".to_string()),
        },
    ];
    
    let response = ApiResponse::success(users);
    ResponseBuilder::with_json(serde_json::to_string(&response).unwrap())
}

/// 创建用户
pub async fn create_user_handler(_ctx: RequestCtx) -> Response {
    // 这里应该从请求体中解析用户数据，为了演示简化处理
    let new_user = User {
        id: 4,
        name: "新用户".to_string(),
        email: "newuser@example.com".to_string(),
        avatar: None,
    };
    
    let response = ApiResponse::success(new_user);
    ResponseBuilder::with_json(serde_json::to_string(&response).unwrap())
}

/// 根据ID获取用户
pub async fn get_user_handler(ctx: RequestCtx) -> Response {
    if let Some(id_str) = ctx.get_param("id") {
        match id_str.parse::<u32>() {
            Ok(id) => {
                let user = User {
                    id,
                    name: format!("用户{}", id),
                    email: format!("user{}@example.com", id),
                    avatar: Some(format!("https://avatar.example.com/{}.jpg", id)),
                };
                
                let response = ApiResponse::success(user);
                ResponseBuilder::with_json(serde_json::to_string(&response).unwrap())
            }
            Err(_) => {
                let response = ApiResponse::<()>::error("无效的用户ID格式");
                ResponseBuilder::bad_request_json(serde_json::to_string(&response).unwrap())
            }
        }
    } else {
        let response = ApiResponse::<()>::error("缺少用户ID参数");
        ResponseBuilder::bad_request_json(serde_json::to_string(&response).unwrap())
    }
}

/// 更新用户
pub async fn update_user_handler(ctx: RequestCtx) -> Response {
    if let Some(id_str) = ctx.get_param("id") {
        match id_str.parse::<u32>() {
            Ok(id) => {
                let updated_user = User {
                    id,
                    name: format!("更新的用户{}", id),
                    email: format!("updated{}@example.com", id),
                    avatar: Some(format!("https://avatar.example.com/updated{}.jpg", id)),
                };
                
                let response = ApiResponse::success(updated_user);
                ResponseBuilder::with_json(serde_json::to_string(&response).unwrap())
            }
            Err(_) => {
                let response = ApiResponse::<()>::error("无效的用户ID格式");
                ResponseBuilder::bad_request_json(serde_json::to_string(&response).unwrap())
            }
        }
    } else {
        let response = ApiResponse::<()>::error("缺少用户ID参数");
        ResponseBuilder::bad_request_json(serde_json::to_string(&response).unwrap())
    }
}

/// 删除用户
pub async fn delete_user_handler(ctx: RequestCtx) -> Response {
    if let Some(id_str) = ctx.get_param("id") {
        match id_str.parse::<u32>() {
            Ok(id) => {
                let response = ApiResponse::success(serde_json::json!({
                    "deleted_id": id,
                    "message": format!("用户 {} 已被删除", id)
                }));
                ResponseBuilder::with_json(serde_json::to_string(&response).unwrap())
            }
            Err(_) => {
                let response = ApiResponse::<()>::error("无效的用户ID格式");
                ResponseBuilder::bad_request_json(serde_json::to_string(&response).unwrap())
            }
        }
    } else {
        let response = ApiResponse::<()>::error("缺少用户ID参数");
        ResponseBuilder::bad_request_json(serde_json::to_string(&response).unwrap())
    }
}

/// 管理员统计
pub async fn admin_stats_handler(_ctx: RequestCtx) -> Response {
    let stats = serde_json::json!({
        "total_users": 1250,
        "active_sessions": 45,
        "server_uptime": "2 days, 5 hours",
        "memory_usage": "128MB",
        "cpu_usage": "15%"
    });
    
    let response = ApiResponse::success(stats);
    ResponseBuilder::with_json(serde_json::to_string_pretty(&response).unwrap())
}

/// 管理员日志
pub async fn admin_logs_handler(_ctx: RequestCtx) -> Response {
    let logs = vec![
        "2024-01-15 10:30:25 [INFO] Server started successfully",
        "2024-01-15 10:31:02 [INFO] New user registered: user123",
        "2024-01-15 10:32:15 [WARN] High memory usage detected",
        "2024-01-15 10:33:08 [INFO] Database backup completed",
        "2024-01-15 10:34:22 [ERROR] Failed to send email notification",
    ];
    
    let response = ApiResponse::success(logs);
    ResponseBuilder::with_json(serde_json::to_string_pretty(&response).unwrap())
}
