//! Context for handling HTTP requests in a web application.

use http_body_util::BodyExt;
use hyper::body::Bytes;
use std::net::SocketAddr;

/// Type alias for the raw incoming hyper request
pub type HyperRequest = hyper::Request<hyper::body::Incoming>;

pub struct RequestCtx {
    pub request: hyper::Request<()>, // Request without body
    pub params: std::collections::HashMap<String, String>,
    body: Option<Bytes>,                      // Cached body
    body_stream: Option<hyper::body::Incoming>, // Original body stream
    pub remote_addr: Option<SocketAddr>,      // Remote address
}

impl RequestCtx {
    /// Create a new RequestCtx from a hyper request (infallible, body is lazy-loaded)
    pub fn new(request: HyperRequest) -> Self {
        let (parts, body) = request.into_parts();
        RequestCtx {
            request: hyper::Request::from_parts(parts, ()),
            params: std::collections::HashMap::new(),
            body: None,
            body_stream: Some(body),
            remote_addr: None,
        }
    }

    /// Attach the remote address (called by the engine after construction)
    pub fn with_remote_addr(mut self, addr: SocketAddr) -> Self {
        self.remote_addr = Some(addr);
        self
    }

    /// Get a path parameter by key
    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// Add a parameter to the context
    pub fn add_param(&mut self, key: String, value: String) {
        self.params.insert(key, value);
    }

    /// Add multiple parameters to the context
    pub fn add_params(&mut self, params: std::collections::HashMap<String, String>) {
        self.params.extend(params);
    }

    /// Check if a path parameter exists
    pub fn has_param(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Get a URL query parameter by key (e.g. `?foo=bar`).
    /// Values are percent-decoded automatically.
    pub fn query_param(&self, key: &str) -> Option<String> {
        let query = self.request.uri().query()?;
        form_urlencoded::parse(query.as_bytes())
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.into_owned())
    }

    /// Get a request header value by name (case-insensitive)
    pub fn header(&self, key: &str) -> Option<&str> {
        self.request
            .headers()
            .get(key)
            .and_then(|v| v.to_str().ok())
    }

    /// Get the request body as bytes (lazy loading)
    pub async fn body_bytes(
        &mut self,
    ) -> Result<Option<&Bytes>, Box<dyn std::error::Error + Send + Sync>> {
        if self.body.is_some() {
            return Ok(self.body.as_ref());
        }

        if let Some(body) = self.body_stream.take() {
            let bytes = body.collect().await?.to_bytes();
            if !bytes.is_empty() {
                self.body = Some(bytes);
            }
        }

        Ok(self.body.as_ref())
    }

    /// Get the request body as a UTF-8 string
    pub async fn body_string(
        &mut self,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match self.body_bytes().await? {
            Some(bytes) => Ok(Some(std::str::from_utf8(bytes)?.to_owned())),
            None => Ok(None),
        }
    }

    /// Parse JSON from the request body
    pub async fn body_json<T>(
        &mut self,
    ) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.body_bytes().await? {
            Some(bytes) => Ok(Some(serde_json::from_slice(bytes)?)),
            None => Ok(None),
        }
    }

    /// Parse JSON from the request body, returning an error if body is missing.
    /// Use this when the request body is required.
    pub async fn json<T>(&mut self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.body_json().await? {
            Some(value) => Ok(value),
            None => Err("Request body is required".into()),
        }
    }

    /// Parse JSON from the request body, returning the default value if body is missing.
    pub async fn json_or_default<T>(
        &mut self,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        match self.body_json().await? {
            Some(value) => Ok(value),
            None => Ok(T::default()),
        }
    }

    /// Take the raw body stream (for large file / streaming handling).
    /// Note: This consumes the body; subsequent calls to body_bytes/json will return None.
    pub fn take_body_stream(&mut self) -> Option<hyper::body::Incoming> {
        self.body_stream.take()
    }
}
