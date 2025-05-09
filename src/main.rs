mod ree;
mod tire;
mod router;
mod context;

use context::RequestCtx;
use ree::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建一个新的引擎实例
    let mut engine = ree::Engine::new();
    // 添加路由
    engine.get("/", hello);
    engine.get("/hello", hello2);
    engine.run("127.0.0.1:3000").await.unwrap();
    Ok(())
}

async fn hello(_ctx: RequestCtx) -> Result<Response, hyper::Error> {
    Ok(ResponseBuilder::with_text("hello"))
}

async fn hello2(_ctx: RequestCtx) -> Result<Response, hyper::Error> {
    Ok(ResponseBuilder::with_text("hello2"))
}
