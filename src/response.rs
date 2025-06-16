//! HTTP response utilities and type conversions.

use hyper::body::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};

pub type Response = hyper::Response<BoxBody<Bytes, hyper::Error>>;

/// Create a full body from any type that can convert to Bytes
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Create an empty body
pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

/// A builder for creating HTTP responses with method chaining
pub struct ResponseBuilder {
    builder: hyper::http::response::Builder,
}

impl ResponseBuilder {
    /// Start building a new response
    pub fn new() -> Self {
        Self {
            builder: hyper::Response::builder(),
        }
    }

    /// Set the status code
    pub fn status(mut self, status: hyper::StatusCode) -> Self {
        self.builder = self.builder.status(status);
        self
    }

    /// Add a header
    pub fn header<V>(mut self, key: &str, value: V) -> Self
    where
        V: AsRef<str>,
    {
        self.builder = self.builder.header(key, value.as_ref());
        self
    }

    /// Set content type
    pub fn content_type(self, content_type: &str) -> Self {
        self.header("Content-Type", content_type)
    }

    /// Build response with body
    pub fn body<T: Into<Bytes>>(self, body: T) -> Response {
        self.builder.body(full(body)).unwrap()
    }

    /// Build response with empty body
    pub fn empty_body(self) -> Response {
        self.builder.body(empty()).unwrap()
    }

    /// Build an HTML response
    pub fn html<T: Into<Bytes>>(body: T) -> Response {
        Self::new()
            .content_type("text/html; charset=utf-8")
            .body(body)
    }

    /// Build a 404 response
    pub fn not_found() -> Response {
        Self::new()
            .status(hyper::StatusCode::NOT_FOUND)
            .content_type("text/plain; charset=utf-8")
            .body("404 Not Found")
    }

    /// Build a 500 response
    pub fn internal_error() -> Response {
        Self::new()
            .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("text/plain; charset=utf-8")
            .body("500 Internal Server Error")
    }

    /// Build a 204 No Content response
    pub fn no_content() -> Response {
        Self::new()
            .status(hyper::StatusCode::NO_CONTENT)
            .empty_body()
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for converting types into HTTP responses
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for &str {
    fn into_response(self) -> Response {
        ResponseBuilder::new()
            .status(hyper::StatusCode::OK)
            .content_type("text/plain; charset=utf-8")
            .body(self.to_string())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        ResponseBuilder::new()
            .status(hyper::StatusCode::OK)
            .content_type("text/plain; charset=utf-8")
            .body(self)
    }
}

impl IntoResponse for &String {
    fn into_response(self) -> Response {
        ResponseBuilder::new()
            .status(hyper::StatusCode::OK)
            .content_type("text/plain; charset=utf-8")
            .body(self.clone())
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(json_str) => ResponseBuilder::new()
                .status(hyper::StatusCode::OK)
                .content_type("application/json; charset=utf-8")
                .body(json_str),
            Err(_) => ResponseBuilder::internal_error(),
        }
    }
}

impl IntoResponse for &serde_json::Value {
    fn into_response(self) -> Response {
        match serde_json::to_string(self) {
            Ok(json_str) => ResponseBuilder::new()
                .status(hyper::StatusCode::OK)
                .content_type("application/json; charset=utf-8")
                .body(json_str),
            Err(_) => ResponseBuilder::internal_error(),
        }
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        ResponseBuilder::new()
            .status(hyper::StatusCode::OK)
            .content_type("application/octet-stream")
            .body(self)
    }
}

impl IntoResponse for &[u8] {
    fn into_response(self) -> Response {
        ResponseBuilder::new()
            .status(hyper::StatusCode::OK)
            .content_type("application/octet-stream")
            .body(self.to_vec())
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        ResponseBuilder::new()
            .status(hyper::StatusCode::OK)
            .content_type("application/octet-stream")
            .body(self)
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
            Err(err) => ResponseBuilder::new()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/plain; charset=utf-8")
                .body(format!("Error: {}", err)),
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
