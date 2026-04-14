use s_web::{Response, ResponseBuilder, StatusCode};
use serde_json::json;

pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Database(String),
}

impl AppError {
    pub fn to_response(self) -> Response {
        let (status, msg) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        ResponseBuilder::new()
            .status(status)
            .content_type("application/json; charset=utf-8")
            .body(json!({ "error": msg }).to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => AppError::NotFound("资源不存在".to_string()),
            other => AppError::Database(format!("数据库错误: {other}")),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
