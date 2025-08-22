//! Context for handling HTTP requests in a web application.

use http_body_util::BodyExt;
use hyper::body::Bytes;

pub type HayperRequest = hyper::Request<hyper::body::Incoming>;

pub struct RequestCtx {
    pub request: hyper::Request<()>, // Request without body
    pub params: std::collections::HashMap<String, String>,
    pub body: Option<Bytes>, // Pre-read body
}

impl RequestCtx {
    /// Create a new RequestCtx from a hyper request
    pub async fn new(request: HayperRequest) -> Result<Self, hyper::Error> {
        let (parts, body) = request.into_parts();
        let body_bytes = body.collect().await?.to_bytes();

        Ok(RequestCtx {
            request: hyper::Request::from_parts(parts, ()),
            params: std::collections::HashMap::new(),
            body: if body_bytes.is_empty() {
                None
            } else {
                Some(body_bytes)
            },
        })
    }

    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// Get a parameter value as a string (convenience method)
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// Add a parameter to the context
    pub fn add_param(&mut self, key: String, value: String) {
        self.params.insert(key, value);
    }

    /// Add multiple parameters to the context
    pub fn add_params(&mut self, params: std::collections::HashMap<String, String>) {
        self.params.extend(params);
    }

    /// Check if a parameter exists
    pub fn has_param(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Get the request body as bytes
    pub fn body_bytes(&self) -> Option<&Bytes> {
        self.body.as_ref()
    }

    /// Get the request body as a UTF-8 string
    pub fn body_string(&self) -> Result<Option<String>, std::string::FromUtf8Error> {
        match &self.body {
            Some(bytes) => Ok(Some(String::from_utf8(bytes.to_vec())?)),
            None => Ok(None),
        }
    }

    /// Parse JSON from the request body
    pub fn body_json<T>(&self) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.body_string()? {
            Some(body_str) => Ok(Some(serde_json::from_str(&body_str)?)),
            None => Ok(None),
        }
    }

    /// Parse JSON from the request body, returning an error if body is missing
    /// Use this when the request body is required
    pub fn json<T>(&self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.body_json()? {
            Some(value) => Ok(value),
            None => Err("Request body is required".into()),
        }
    }
}
