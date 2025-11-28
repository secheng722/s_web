use bcrypt::{hash, verify};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use s_web::{IntoResponse, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::AppState;
use crate::models::{AuthResponse, CreateUserDto, LoginDto, User, UserResponse};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // User ID
    exp: usize,  // 过期时间
}

// 注册新用户
pub async fn register(state: Arc<AppState>, mut ctx: RequestCtx) -> Response {
    // 解析请求体
    let user_dto = match ctx.json::<CreateUserDto>().await {
        Ok(user) => user,
        Err(e) => {
            return ResponseBuilder::new()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Invalid request body: {e}"));
        }
    };

    // 密码加密
    let password_hash = match hash(&user_dto.password, 10) {
        Ok(hash) => hash,
        Err(_) => {
            return ResponseBuilder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to hash password");
        }
    };

    let now = Utc::now();
    let user_id = Uuid::new_v4();

    // 插入用户
    let result = sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id.to_string())
    .bind(&user_dto.username)
    .bind(&user_dto.email)
    .bind(&password_hash)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            // 创建JWT令牌
            let claims = Claims {
                sub: user_id.to_string(),
                exp: (Utc::now().timestamp() + 60 * 60 * 24) as usize, // 1天过期
            };

            let token = match encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
            ) {
                Ok(t) => t,
                Err(_) => {
                    return ResponseBuilder::new()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Failed to generate token");
                }
            };

            // 构造响应
            let user = UserResponse {
                id: user_id.to_string(),
                username: user_dto.username,
                email: user_dto.email,
            };

            let auth_response = AuthResponse { token, user };

            json!(auth_response).into_response()
        }
        Err(e) => {
            if e.to_string().contains("UNIQUE constraint failed") {
                ResponseBuilder::new()
                    .status(StatusCode::CONFLICT)
                    .body("Username or email already exists")
            } else {
                ResponseBuilder::new()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Database error: {e}"))
            }
        }
    }
}

// 用户登录
pub async fn login(state: Arc<AppState>, mut ctx: RequestCtx) -> Response {
    // 解析请求体
    let login_dto = match ctx.json::<LoginDto>().await {
        Ok(login) => login,
        Err(e) => {
            return ResponseBuilder::new()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("Invalid request body: {e}"));
        }
    };

    // 查找用户
    let user_result = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE username = ?
        "#,
    )
    .bind(&login_dto.username)
    .fetch_optional(&state.db)
    .await;

    let user = match user_result {
        Ok(Some(user)) => user,
        Ok(None) => {
            return ResponseBuilder::new()
                .status(StatusCode::UNAUTHORIZED)
                .body("Invalid credentials");
        }
        Err(e) => {
            println!("Database error: {e}");
            return ResponseBuilder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Database error");
        }
    };

    // 验证密码
    let password_valid = match verify(&login_dto.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(_) => {
            return ResponseBuilder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Password verification error");
        }
    };

    if !password_valid {
        return ResponseBuilder::new()
            .status(StatusCode::UNAUTHORIZED)
            .body("Invalid credentials");
    }

    // 生成JWT令牌
    let claims = Claims {
        sub: user.id.to_string(),
        exp: (Utc::now().timestamp() + 60 * 60 * 24) as usize, // 1天过期
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return ResponseBuilder::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Failed to generate token");
        }
    };

    // 构造响应
    let user_response = UserResponse::from(user);
    let auth_response = AuthResponse {
        token,
        user: user_response,
    };

    json!(auth_response).into_response()
}

// 获取当前用户信息
pub async fn me(state: Arc<AppState>, ctx: RequestCtx) -> Response {
    // 从扩展中获取用户ID
    let user_id = match ctx.get_param("user_id") {
        Some(id) => id,
        None => {
            return ResponseBuilder::new()
                .status(StatusCode::UNAUTHORIZED)
                .body("Not authenticated");
        }
    };

    // 查询用户信息
    let user_result = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE id = ?
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await;

    match user_result {
        Ok(Some(user)) => {
            let user_response = UserResponse::from(user);
            json!(user_response).into_response()
        }
        Ok(None) => ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body("User not found"),
        Err(e) => ResponseBuilder::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("Database error: {e}")),
    }
}
