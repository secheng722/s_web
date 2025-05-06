use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use http_body_util::{BodyExt, Empty, combinators::BoxBody};
use hyper::{Request, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;

type HayperRequest = Request<hyper::body::Incoming>;
type HyperResponse = hyper::Response<BoxBody<Bytes, hyper::Error>>;

pub struct RequestCtx {
    request: HayperRequest,
}

pub type Response = HyperResponse;

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

type Router = HashMap<String, BoxHandler>;

pub struct Engine {
    routes: Router,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, method: &str, path: &str, handler: impl Handler) {
        let key = format!("{}-{}", method, path);
        self.routes.insert(key, Box::new(handler));
    }

    pub fn get(&mut self, path: &str, handler: impl Handler) {
        self.add_route("GET", path, handler);
    }

    //self 所有权转移
    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.parse::<SocketAddr>()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let routes = Arc::new(self.routes);
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream); // 将TCP流转换为Tokio的IO接口
            let routes = Arc::clone(&routes); // 克隆路由表的Arc指针以在新任务中使用
            tokio::task::spawn(async move {
                // 启动一个新的异步任务来处理连接
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        // 使用HTTP/1协议服务此连接
                        io,
                        service_fn(move |req| {
                            // 创建服务函数来处理每个HTTP请求
                            let routes = Arc::clone(&routes); // 再次克隆路由表以在请求处理闭包中使用
                            async move {
                                let method = req.method().to_string(); // 提取HTTP方法
                                let path = req.uri().path().to_string(); // 提取请求路径
                                let key = format!("{}-{}", method, path); // 构建路由键，格式为"METHOD-path"
                                if let Some(handler) = routes.get(&key) {
                                    // 查找对应的路由处理器
                                    let ctx = RequestCtx { request: req }; // 创建请求上下文
                                    handler.handle(ctx).await // 调用处理函数并等待结果
                                } else {
                                    // 路由未找到，返回404 Not Found响应
                                    let mut not_found = Response::new(
                                        Empty::<Bytes>::new()
                                            .map_err(|never| match never {}) // 处理空错误类型
                                            .boxed(), // 转换为BoxBody类型
                                    );
                                    *not_found.status_mut() = hyper::StatusCode::NOT_FOUND;
                                    Ok(not_found)
                                }
                            }
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
