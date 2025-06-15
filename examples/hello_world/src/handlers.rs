use ree::{RequestCtx, Response, ResponseBuilder};

/// 基本的问候处理器
pub async fn hello_handler(_ctx: RequestCtx) -> Response {
    ResponseBuilder::with_text("Hello, World! 这是一个使用 ree 框架的示例!")
}

/// 带参数的问候处理器
pub async fn hello_name_handler(ctx: RequestCtx) -> Response {
    if let Some(name) = ctx.get_param("name") {
        ResponseBuilder::with_text(format!("Hello, {}! 欢迎使用 ree 框架!", name))
    } else {
        ResponseBuilder::with_text("Hello, Anonymous!")
    }
}

/// 获取用户列表
pub async fn get_users_handler(_ctx: RequestCtx) -> Response {
    let users_json = r#"[
    {"id": 1, "name": "张三", "email": "zhangsan@example.com"},
    {"id": 2, "name": "李四", "email": "lisi@example.com"},
    {"id": 3, "name": "王五", "email": "wangwu@example.com"}
]"#;
    
    ResponseBuilder::with_json(users_json)
}

/// 根据ID获取特定用户
pub async fn get_user_by_id_handler(ctx: RequestCtx) -> Response {
    if let Some(id) = ctx.get_param("id") {
        match id.parse::<u32>() {
            Ok(user_id) => {
                let user_json = format!(
                    r#"{{"id": {}, "name": "用户{}", "email": "user{}@example.com"}}"#,
                    user_id, user_id, user_id
                );
                ResponseBuilder::with_json(user_json)
            }
            Err(_) => {
                ResponseBuilder::with_json(r#"{"error": "无效的用户ID格式"}"#)
            }
        }
    } else {
        ResponseBuilder::with_json(r#"{"error": "用户ID未提供"}"#)
    }
}
