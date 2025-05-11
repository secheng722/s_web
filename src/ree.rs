use std::{
    collections::HashMap, convert::Infallible, net::SocketAddr, str, sync::Arc, time::Instant,
};

use async_trait::async_trait;
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;

use crate::{context::RequestCtx, router::Router};

type HyperResponse = hyper::Response<BoxBody<Bytes, hyper::Error>>;

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
    async fn handle(&self, ctx: RequestCtx) -> Response;
}

#[async_trait]
impl<F: Send + Sync + 'static, Fut> Handler for F
where
    F: Fn(RequestCtx) -> Fut,
    Fut: std::future::Future<Output = Response> + Send + 'static,
{
    async fn handle(&self, ctx: RequestCtx) -> Response {
        (self)(ctx).await
    }
}

#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response;
}

pub struct Next<'a> {
    pub endpoint: &'a dyn Handler,
    pub next_middleware: &'a [Arc<dyn Middleware>],
}

impl Next<'_> {
    pub async fn run(mut self, ctx: RequestCtx) -> Response {
        if let Some((current, next)) = self.next_middleware.split_first() {
            self.next_middleware = next;
            current.handle(ctx, self).await
        } else {
            (self.endpoint).handle(ctx).await
        }
    }
}

pub struct RouterGroup {
    prefix: String,
    router: Router,
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl RouterGroup {
    pub fn add_route(&mut self, method: &str, pattern: &str, handler: impl Handler) {
        let handler = Box::new(handler);
        let full_pattern = format!("{}{}", self.prefix, pattern);
        self.router.add_route(method, &full_pattern, handler);
    }

    pub fn get(&mut self, path: &str, handler: impl Handler) {
        self.add_route("GET", path, handler);
    }

    pub fn use_middleware(&mut self, middleware: impl Middleware) {
        self.middlewares.push(Arc::new(middleware));
    }

    pub async fn handle_request(&self, ctx: RequestCtx) -> Response {
        self.router.handle_request(ctx).await
    }
}

pub struct AccessLog;

#[async_trait]
impl Middleware for AccessLog {
    async fn handle(&self, ctx: RequestCtx, next: Next<'_>) -> Response {
        let start = Instant::now();
        let method = ctx.request.method().to_string();
        let path = ctx.request.uri().path().to_string();
        let res = next.run(ctx).await;
        println!(
            "{} {:?} {}  {}ms",
            method,
            path,
            res.status().as_str(),
            start.elapsed().as_millis()
        );
        res
    }
}

pub struct Engine {
    // 不属于任何路由组的路由
    router: Router,
    group: HashMap<String, RouterGroup>,
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            router: Router::new(),
            group: HashMap::new(),
            middlewares: Vec::new(),
        }
    }

    pub fn use_middleware(&mut self, middleware: impl Middleware) {
        self.middlewares.push(Arc::new(middleware));
    }

    pub fn group(&mut self, prefix: &str) -> &mut RouterGroup {
        let group = RouterGroup {
            prefix: prefix.to_string(),
            router: Router::new(),
            middlewares: Vec::new(),
        };
        self.group.insert(prefix.to_string(), group);
        self.group.get_mut(prefix).unwrap()
    }

    pub fn add_route(&mut self, method: &str, pattern: &str, handler: impl Handler) {
        let handler = Box::new(handler);
        self.router.add_route(method, pattern, handler);
    }

    pub fn get(&mut self, path: &str, handler: impl Handler) {
        self.add_route("GET", path, handler);
    }

    //self 所有权转移
    pub async fn run(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.parse::<SocketAddr>()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let router = Arc::new(self.router);
        let middlewares = Arc::new(self.middlewares);
        //将group转换为Arc<RouterGroup>类
        let group = Arc::new(
            self.group
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect::<HashMap<_, _>>(),
        );

        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream); // 将TCP流转换为Tokio的IO接口
            let router = Arc::clone(&router); // 克隆路由表的Arc指针以在新任务中使用
            let middlewares = Arc::clone(&middlewares); // 克隆中间件的Arc指针以在新任务中使用
            let group = Arc::clone(&group); // 克隆路由组的Arc指针以在新任务中使用
            tokio::task::spawn(async move {
                // 启动一个新的异步任务来处理连接
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        // 使用HTTP/1协议服务此连接
                        io,
                        service_fn(move |req| {
                            // 创建服务函数来处理每个HTTP请求
                            let router = Arc::clone(&router); // 再次克隆路由表以在请求处理闭包中使用
                            let middlewares = Arc::clone(&middlewares); // 再次克隆中间件以在请求处理闭包中使用
                            let group = Arc::clone(&group); // 再次克隆路由组以在请求处理闭包中使用
                            async move {
                                let group = Arc::clone(&group);
                                let middlewares = Arc::clone(&middlewares);
                                //如果请求的路径以路由组的前缀开头，则使用该路由组
                                let group = group
                                    .iter()
                                    .find(|(_, g)| req.uri().path().starts_with(&g.prefix))
                                    .map(|(_, g)| g.clone());

                                if let Some(group) = group {
                                    let group_middlewares = group.middlewares.clone();
                                    // 创建合并的中间件列表
                                    let mut all_middlewares = Vec::new();
                                    // 先添加组特定的中间件
                                    all_middlewares.extend(group_middlewares.iter().cloned());
                                    // 然后添加全局中间件
                                    all_middlewares.extend(middlewares.iter().cloned());

                                    let ctx = RequestCtx {
                                        request: req,
                                        params: HashMap::new(),
                                    };
                                    let endpoint = Box::new(move |ctx: RequestCtx| {
                                        let group = Arc::clone(&group);
                                        async move { group.handle_request(ctx).await }
                                    });
                                    //所有的的等于默认的加组的中间件

                                    // 使用路由组处理请求
                                    let next = Next {
                                        endpoint: &endpoint,
                                        next_middleware: &all_middlewares,
                                    };

                                    let resp = next.run(ctx).await; // 调用中间件链
                                    return Ok::<_, Infallible>(resp); // 返回响应
                                }
                                let ctx = RequestCtx {
                                    request: req,
                                    params: HashMap::new(),
                                };

                                let endpoint = Box::new(move |ctx: RequestCtx| {
                                    let router = Arc::clone(&router);
                                    async move { router.handle_request(ctx).await }
                                });
                                let next = Next {
                                    endpoint: &endpoint,
                                    next_middleware: &middlewares,
                                };
                                let resp = next.run(ctx).await; // 调用中间件链
                                Ok::<_, Infallible>(resp) // 返回响应
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

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn test_new_group() {
    //     let mut engine = Engine::new();
    //     let group = engine.group("/api");
    //     group.prefix = "/1".to_string();
    //     println!("{:?}", group.prefix);
    //     println!("{:?}", engine.group.get("/api").unwrap().prefix);
    // }
}
