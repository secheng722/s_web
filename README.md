# ree 基于gee同样思想的web 框架

## day01

hyper 简单的hello请求
```rust

/// 处理HTTP请求的异步函数
/// 接收一个请求并返回"Hello, World!"响应
async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 设置服务器监听地址为本地的3000端口
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // 创建TCP监听器绑定到指定地址
    let listener = TcpListener::bind(addr).await?;

    // 无限循环持续接受新的连接
    loop {
        // 等待并接受新的连接
        let (stream, _) = listener.accept().await?;

        // 使用适配器将实现`tokio::io`特性的对象转换为实现`hyper::rt` IO特性的对象
        let io = TokioIo::new(stream);

        // 生成一个tokio任务来并发处理多个连接
        tokio::task::spawn(async move {
            // 最终，我们将传入的连接绑定到`hello`服务
            if let Err(err) = http1::Builder::new()
                // `service_fn`将我们的函数转换为一个`Service`
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("连接处理出错: {:?}", err);
            }
        });
    }
}

```

## day02

ree 实现基础的框架

```rust
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
            let io = TokioIo::new(stream);
            let routes = routes.clone();
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        io,
                        service_fn(move |req| {
                            let routes = Arc::clone(&routes);
                            async move {
                                let method = req.method().to_string();
                                let path = req.uri().path().to_string();
                                let key = format!("{}-{}", method, path);
                                if let Some(handler) = routes.get(&key) {
                                    let ctx = RequestCtx { request: req };
                                    handler.handle(ctx).await
                                } else {
                                    let mut not_found = Response::new(
                                        Empty::<Bytes>::new()
                                            .map_err(|never| match never {})
                                            .boxed(),
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

```
## day03

context & router 

- context
  
```rust
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

```
- router

```rust
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
```

## day04

前缀树路由

构建tire树 tire.rs

```rust 
#[derive(Debug)]
pub struct Node {
    pub pattern: String,
    pub part: String,
    pub children: Vec<Node>,
    pub iswild: bool,
}

impl Node {
    pub fn new() -> Self {
        Node {
            pattern: String::new(),
            part: String::new(),
            children: Vec::new(),
            iswild: false,
        }
    }

    fn match_child(&mut self, path: &str) -> Option<&mut Node> {
        self.children
            .iter_mut()
            .find(|child| child.part == path || child.iswild)
    }

    fn match_children(&self, path: &str) -> Vec<&Node> {
        self.children
            .iter()
            .filter(|&child| child.part == path || child.iswild)
            .collect()
    }

    pub fn insert(&mut self, pattern: &str, parts: Vec<&str>, height: usize) {
        if height == parts.len() {
            self.pattern = pattern.to_string();
            return;
        }

        let part = &parts[height];
        if let Some(child) = self.match_child(part) {
            child.insert(pattern, parts, height + 1);
        } else {
            let mut new_node = Node {
                pattern: String::new(),
                part: part.to_string(),
                children: Vec::new(),
                iswild: part.starts_with(':') || part.starts_with('*'),
            };
            new_node.insert(pattern, parts, height + 1);
            self.children.push(new_node);
        }
    }

    pub fn search(&self, parts: &Vec<&str>, height: usize) -> Option<&Node> {
        if height == parts.len() || self.part.starts_with("*") {
            return if self.pattern.is_empty() {
                None
            } else {
                Some(self)
            };
        }

        let part = &parts[height];
        for child in self.match_children(part) {
            if let Some(result) = child.search(parts, height + 1) {
                return Some(result);
            }
        }
        None
    }
}

```

应用到router router.rs

```rust


type HandlerFunc = Box<dyn Handler>;

pub struct Router {
    roots: HashMap<String, Node>,
    handlers: HashMap<String, HandlerFunc>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            roots: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    //only one * is allowed
    pub fn parse_pattern(pattern: &str) -> Vec<&str> {
        let vs = pattern.split('/').collect::<Vec<&str>>();
        let mut part = Vec::new();
        for &item in vs.iter() {
            if !item.is_empty() {
                part.push(item);
                if item.starts_with("*") {
                    break;
                }
            }
        }
        part
    }

    pub fn add_route(&mut self, method: &str, pattern: &str, handler: HandlerFunc) {
        let parts = Self::parse_pattern(pattern);
        let key = format!("{}-{}", method, pattern);
        self.roots
            .entry(method.to_string())
            .or_insert_with(Node::new)
            .insert(pattern, parts, 0);
        self.handlers.insert(key, handler);
    }

    pub fn get_route(&self, method: &str, path: &str) -> (Option<&Node>, HashMap<String, String>) {
        let search_parts = Self::parse_pattern(path);
        let mut params = HashMap::new();
        let root = self.roots.get(method);
        if root.is_none() {
            return (None, HashMap::new());
        }
        if let Some(node) = root.unwrap().search(&search_parts, 0) {
            let parts = Self::parse_pattern(&node.pattern);
            for (index, ele) in parts.iter().enumerate() {
                if let Some(param_name) = ele.strip_prefix(':') {
                    params.insert(param_name.to_string(), search_parts[index].to_string());
                } else if let Some(param_name) = ele.strip_prefix('*') {
                    params.insert(param_name.to_string(), search_parts[index..].join("/"));
                    break;
                }
            }
            return (Some(node), params);
        }
        (None, HashMap::new())
    }

    pub fn handle(&self, key: &str) -> Option<&HandlerFunc> {
        self.handlers.get(key)
    }

    // 新增方法，处理HTTP请求
    pub async fn handle_request(&self, request: HayperRequest) -> Result<Response, hyper::Error> {
        // 提取HTTP方法和路径
        let method = request.method().to_string();
        let path = request.uri().path().to_string();
        let (node, params) = self.get_route(&method, &path);
        if node.is_none() {
            // 路由未找到，返回404 Not Found响应
            return Ok(ResponseBuilder::with_text("404 Not Found"));
        }
        let node = node.unwrap();
        let key = format!("{}-{}", method, node.pattern);
        // 查找对应的路由处理器
        if let Some(handler) = self.handle(&key) {
            // 创建请求上下文
            let ctx = RequestCtx { request, params };
            // 调用处理函数并等待结果
            handler.handle(ctx).await
        } else {
            // 路由未找到，返回404 Not Found响应
            Ok(ResponseBuilder::with_text("404 Not Found"))
        }
    }
}

```

## day05 分组控制

