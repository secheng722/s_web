use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{Request, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;

type HayperRequest = Request<hyper::body::Incoming>;
type HyperResponse = hyper::Response<BoxBody<Bytes, hyper::Error>>;

pub struct RequestCtx {
    request: HayperRequest,
}

pub type Response = HyperResponse;

pub struct ResponseBuilder;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

impl ResponseBuilder {
    pub fn with_text<T: Into<Bytes>>(chunk: T) -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(full(chunk))
            .unwrap()
    }

    pub fn empty() -> Response {
        hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .body(
                Empty::<Bytes>::new()
                    .map_err(|never| match never {}) // 处理空错误类型
                    .boxed(),
            ) // 转换为BoxBody类型)
            .unwrap()
    }
}

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, ctx: RequestCtx) -> Result<Response, hyper::Error>;
}

#[async_trait]
impl<F: Send + Sync + 'static, Fut> Handler for F
where
    F: Fn(RequestCtx) -> Fut,
    Fut: std::future::Future<Output = Result<Response, hyper::Error>> + Send + 'static,
{
    async fn handle(&self, ctx: RequestCtx) -> Result<Response, hyper::Error> {
        (self)(ctx).await
    }
}

type BoxHandler = Box<dyn Handler>;

struct Router(HashMap<String, BoxHandler>);

impl Router {
    pub fn new() -> Self {
        Router(HashMap::new())
    }

    pub fn add_route(&mut self, key: String, handler: BoxHandler) {
        self.0.insert(key, handler);
    }

    pub fn handle(&self, key: &str) -> Option<&BoxHandler> {
        self.0.get(key)
    }

    // 新增方法，处理HTTP请求
    pub async fn handle_request(&self, req: HayperRequest) -> Result<Response, hyper::Error> {
        // 提取HTTP方法和路径
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let key = format!("{}-{}", method, path);

        // 查找对应的路由处理器
        if let Some(handler) = self.handle(&key) {
            // 创建请求上下文
            let ctx = RequestCtx { request: req };
            // 调用处理函数并等待结果
            handler.handle(ctx).await
        } else {
            // 路由未找到，返回404 Not Found响应
            Ok(ResponseBuilder::with_text("404 Not Found"))
        }
    }
}

pub struct Engine {
    router: Router,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            router: Router::new(),
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: impl Handler) {
        let key = format!("{}-{}", method, path);
        let handler = Box::new(handler);
        self.router.add_route(key, handler);
    }

    pub fn get(&mut self, path: &str, handler: impl Handler) {
        self.add_route("GET", path, handler);
    }

    //self 所有权转移
    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.parse::<SocketAddr>()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let router = Arc::new(self.router);
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream); // 将TCP流转换为Tokio的IO接口
            let router = Arc::clone(&router); // 克隆路由表的Arc指针以在新任务中使用
            tokio::task::spawn(async move {
                // 启动一个新的异步任务来处理连接
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        // 使用HTTP/1协议服务此连接
                        io,
                        service_fn(move |req| {
                            // 创建服务函数来处理每个HTTP请求
                            let router = Arc::clone(&router); // 再次克隆路由表以在请求处理闭包中使用
                            async move { router.handle_request(req).await }
                        }),
                    )
                    .await
                {
                    eprintln!("Error handling connection: {:?}", err);
                }
            });
        }
    }
}
