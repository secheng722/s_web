//! HTTP response utilities and type conversions.

use hyper::body::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};

pub type Response = hyper::Response<BoxBody<Bytes, hyper::Error>>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Builder for creating HTTP responses
pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn with_text<T: Into<Bytes>>(chunk: T) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(full(chunk))
            .unwrap()
    }

    pub fn with_json<T: Into<Bytes>>(chunk: T) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(full(chunk))
            .unwrap()
    }

    pub fn with_html<T: Into<Bytes>>(chunk: T) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(full(chunk))
            .unwrap()
    }

    pub fn empty() -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap()
    }

    pub fn with_status_and_content_type<T: Into<Bytes>>(
        status: hyper::StatusCode,
        content_type: &str,
        chunk: T,
    ) -> Response {
        hyper::Response::builder()
            .status(status)
            .header("Content-Type", content_type)
            .body(full(chunk))
            .unwrap()
    }

    pub fn not_found() -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::NOT_FOUND,
            "text/plain; charset=utf-8",
            "404 Not Found",
        )
    }

    pub fn internal_server_error() -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            "text/plain; charset=utf-8",
            "500 Internal Server Error",
        )
    }

    pub fn bad_request_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::BAD_REQUEST,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    pub fn unauthorized_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::UNAUTHORIZED,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    pub fn forbidden_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::FORBIDDEN,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    pub fn too_many_requests_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::TOO_MANY_REQUESTS,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    pub fn created_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::CREATED,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    pub fn no_content() -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::NO_CONTENT)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap()
    }
}

/// Trait for converting types into HTTP responses
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for &str {
    fn into_response(self) -> Response {
        ResponseBuilder::with_text(self.to_string())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        ResponseBuilder::with_text(self)
    }
}

impl IntoResponse for &String {
    fn into_response(self) -> Response {
        ResponseBuilder::with_text(self.clone())
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(json_str) => ResponseBuilder::with_json(json_str),
            Err(_) => ResponseBuilder::internal_server_error(),
        }
    }
}

impl IntoResponse for &serde_json::Value {
    fn into_response(self) -> Response {
        match serde_json::to_string(self) {
            Ok(json_str) => ResponseBuilder::with_json(json_str),
            Err(_) => ResponseBuilder::internal_server_error(),
        }
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .body(full(self))
            .unwrap()
    }
}

impl IntoResponse for &[u8] {
    fn into_response(self) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .body(full(self.to_vec()))
            .unwrap()
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/octet-stream")
            .body(full(self))
            .unwrap()
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        ResponseBuilder::no_content()
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: std::fmt::Display,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => ResponseBuilder::with_status_and_content_type(
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                "text/plain; charset=utf-8",
                format!("Error: {}", err),
            ),
        }
    }
}

impl<T> IntoResponse for Option<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Some(value) => value.into_response(),
            None => ResponseBuilder::not_found(),
        }
    }
}

impl<T> IntoResponse for (hyper::StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let (status, content) = self;
        let mut response = content.into_response();
        *response.status_mut() = status;
        response
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}
