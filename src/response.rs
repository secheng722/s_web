//! HTTP response utilities and builders.

use hyper::body::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};

pub type Response = hyper::Response<BoxBody<Bytes, hyper::Error>>;

/// Helper function to create a full body
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Builder for creating HTTP responses
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Create a plain text response with UTF-8 encoding
    pub fn with_text<T: Into<Bytes>>(chunk: T) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(full(chunk))
            .unwrap()
    }

    /// Create a JSON response with UTF-8 encoding
    pub fn with_json<T: Into<Bytes>>(chunk: T) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(full(chunk))
            .unwrap()
    }

    /// Create an HTML response with UTF-8 encoding
    pub fn with_html<T: Into<Bytes>>(chunk: T) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")
            .body(full(chunk))
            .unwrap()
    }

    /// Create an empty response
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

    /// Create a custom response with specified status and content type
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

    /// Create a 404 Not Found response
    pub fn not_found() -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::NOT_FOUND,
            "text/plain; charset=utf-8",
            "404 Not Found",
        )
    }

    /// Create a 500 Internal Server Error response
    pub fn internal_server_error() -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::INTERNAL_SERVER_ERROR,
            "text/plain; charset=utf-8",
            "500 Internal Server Error",
        )
    }

    /// Create a 400 Bad Request response with JSON
    pub fn bad_request_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::BAD_REQUEST,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    /// Create a 401 Unauthorized response with JSON
    pub fn unauthorized_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::UNAUTHORIZED,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    /// Create a 403 Forbidden response with JSON
    pub fn forbidden_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::FORBIDDEN,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    /// Create a 429 Too Many Requests response with JSON
    pub fn too_many_requests_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::TOO_MANY_REQUESTS,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    /// Create a 201 Created response with JSON
    pub fn created_json<T: Into<Bytes>>(chunk: T) -> Response {
        Self::with_status_and_content_type(
            hyper::StatusCode::CREATED,
            "application/json; charset=utf-8",
            chunk,
        )
    }

    /// Create a 204 No Content response
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
