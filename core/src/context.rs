//! Context for handling HTTP requests in a web application.

use http_body_util::BodyExt;
use hyper::body::Bytes;

pub type HayperRequest = hyper::Request<hyper::body::Incoming>;

pub struct RequestCtx {
    pub request: hyper::Request<()>, // Request without body
    pub params: std::collections::HashMap<String, String>,
    body: Option<Bytes>, // Cached body
    body_stream: Option<hyper::body::Incoming>, // Original body stream
}

impl RequestCtx {
    /// Create a new RequestCtx from a hyper request
    pub async fn new(request: HayperRequest) -> Result<Self, hyper::Error> {
        let (parts, body) = request.into_parts();
        
        Ok(RequestCtx {
            request: hyper::Request::from_parts(parts, ()),
            params: std::collections::HashMap::new(),
            body: None,
            body_stream: Some(body),
        })
    }

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

    /// Check if a parameter exists
    pub fn has_param(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Get the request body as bytes (lazy loading)
    pub async fn body_bytes(&mut self) -> Result<Option<&Bytes>, hyper::Error> {
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
    pub async fn body_string(&mut self) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        match self.body_bytes().await? {
            Some(bytes) => Ok(Some(String::from_utf8(bytes.to_vec())?)),
            None => Ok(None),
        }
    }

    /// Parse JSON from the request body
    pub async fn body_json<T>(&mut self) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.body_bytes().await? {
            Some(bytes) => Ok(Some(serde_json::from_slice(bytes)?)),
            None => Ok(None),
        }
    }

    /// Parse JSON from the request body, returning an error if body is missing
    /// Use this when the request body is required
    pub async fn json<T>(&mut self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.body_json().await? {
            Some(value) => Ok(value),
            None => Err("Request body is required".into()),
        }
    }
    
    /// Parse JSON from the request body, returning default value if body is missing
    pub async fn json_or_default<T>(&mut self) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned + Default,
    {
        match self.body_json().await? {
            Some(value) => Ok(value),
            None => Ok(T::default()),
        }
    }
    
    /// Take the raw body stream (for large file handling)
    /// Note: This will consume the body, making subsequent calls to body_bytes/json return None
    pub fn take_body_stream(&mut self) -> Option<hyper::body::Incoming> {
        self.body_stream.take()
    }
}
