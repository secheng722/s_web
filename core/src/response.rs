//! HTTP response utilities and type conversions.

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Bytes;

pub type Response = hyper::Response<BoxBody<Bytes, hyper::Error>>;

/// Create a full body from any type that can convert to Bytes
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Create an empty body
fn empty() -> BoxBody<Bytes, hyper::Error> {
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
        self.builder.body(full(body)).unwrap_or_else(|_| {
            hyper::Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("500 Internal Server Error"))
                .expect("static fallback response is always valid")
        })
    }

    /// Build response with empty body
    pub fn empty_body(self) -> Response {
        self.builder.body(empty()).unwrap_or_else(|_| {
            hyper::Response::builder()
                .status(hyper::StatusCode::NO_CONTENT)
                .body(empty())
                .expect("static fallback response is always valid")
        })
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

// --- Helper constructors to reduce duplication ---

fn text_response(body: impl Into<Bytes>) -> Response {
    ResponseBuilder::new()
        .status(hyper::StatusCode::OK)
        .content_type("text/plain; charset=utf-8")
        .body(body)
}

fn binary_response(body: impl Into<Bytes>) -> Response {
    ResponseBuilder::new()
        .status(hyper::StatusCode::OK)
        .content_type("application/octet-stream")
        .body(body)
}

fn json_response(body: String) -> Response {
    ResponseBuilder::new()
        .status(hyper::StatusCode::OK)
        .content_type("application/json; charset=utf-8")
        .body(body)
}

// --- Text types ---

impl IntoResponse for &str {
    fn into_response(self) -> Response {
        text_response(self.to_string())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        text_response(self)
    }
}

impl IntoResponse for &String {
    fn into_response(self) -> Response {
        text_response(self.clone())
    }
}

// --- JSON types ---

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(json_str) => json_response(json_str),
            Err(_) => ResponseBuilder::internal_error(),
        }
    }
}

impl IntoResponse for &serde_json::Value {
    fn into_response(self) -> Response {
        match serde_json::to_string(self) {
            Ok(json_str) => json_response(json_str),
            Err(_) => ResponseBuilder::internal_error(),
        }
    }
}

// --- Binary types ---

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        binary_response(self)
    }
}

impl IntoResponse for &[u8] {
    fn into_response(self) -> Response {
        binary_response(self.to_vec())
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        binary_response(self)
    }
}

impl<const N: usize> IntoResponse for [u8; N] {
    fn into_response(self) -> Response {
        binary_response(self.to_vec())
    }
}

// --- Special types ---

impl IntoResponse for () {
    fn into_response(self) -> Response {
        ResponseBuilder::no_content()
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: std::fmt::Debug,
{
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => {
                eprintln!("[s_web] handler error: {:?}", err);
                ResponseBuilder::internal_error()
            }
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

impl<T> IntoResponse for (hyper::StatusCode, &'static str, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        let (status, content_type, content) = self;
        let mut response = content.into_response();
        *response.status_mut() = status;
        response.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static(content_type),
        );
        response
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}
