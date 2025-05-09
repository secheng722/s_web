use hyper::Request;

pub type HayperRequest = Request<hyper::body::Incoming>;

pub struct RequestCtx {
    pub request: HayperRequest,
    pub params: std::collections::HashMap<String, String>,
}

impl RequestCtx {
    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
} 