mod ree;

use std::{convert::Infallible, net::SocketAddr};

// 导入HTTP相关模块
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Method, Request, Response, StatusCode,
    body::{Body, Bytes, Frame},
    server::conn::http1,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use ree::RequestCtx;
use tokio::net::TcpListener;

/// 处理HTTP请求的异步函数
/// 接收一个请求并返回"Hello, World!"响应
async fn hello1(
    _: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    Ok(Response::new(full("Hello, World!")))
}

async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full("Try POSTing data to /echo"))),
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body().boxed())),
        (&Method::POST, "/echo/uppercase") => {
            // Map this body's frame to a different type
            let frame_stream = req.into_body().map_frame(|frame| {
                let frame = if let Ok(data) = frame.into_data() {
                    // 将数据帧中的每个字节转换为大写字母
                    data.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Bytes>()
                } else {
                    Bytes::new()
                };

                Frame::data(frame)
            });

            Ok(Response::new(frame_stream.boxed()))
        }
        // 在match块中的另一个路由处理...
        (&Method::POST, "/echo/reversed") => {
            // 保护服务器不受大型请求体的影响
            let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if upper > 1024 * 64 {
                // 如果请求体超过64KB，返回"请求体过大"的错误
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            // 等待整个请求体被收集到一个单一的`Bytes`对象中...
            let whole_body = req.collect().await?.to_bytes();

            // 按照相反的顺序迭代整个请求体并收集到一个新的Vec中
            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();

            Ok(Response::new(full(reversed_body)))
        }
        // 对于其他路由返回404 Not Found
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建一个新的引擎实例
    let mut engine = ree::Engine::new();
    // 添加路由
    engine.get("/", hello);
    engine.run("127.0.0.1:3000").await.unwrap();
    Ok(())
}

async fn hello(_ctx: RequestCtx) -> Result<ree::Response, hyper::Error> {
    Ok(Response::new(full("Hello, World!")))
}
