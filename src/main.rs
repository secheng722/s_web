mod context;
mod ree;
mod router;
mod tire;

use context::RequestCtx;
use ree::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建一个新的引擎实例
    let mut engine = ree::Engine::new();
    engine.use_middleware(AccessLog);
    // 添加路由
    engine.get("/", hello);
    engine.get("/hello", hello2);
    engine.get("/hello/:name", hello_name);
    engine.get("/assets/*filepath", hello_path);
    engine.run("127.0.0.1:3000").await.unwrap();
    Ok(())
}

async fn hello(_ctx: RequestCtx) -> Response {
    ResponseBuilder::with_text("hello")
}

async fn hello2(_ctx: RequestCtx) -> Response {
    ResponseBuilder::with_text("hello2")
}

async fn hello_name(_ctx: RequestCtx) -> Response {
    ResponseBuilder::with_text(format!("hello {}", _ctx.params.get("name").unwrap()))
}

async fn hello_path(_ctx: RequestCtx) -> Response {
    ResponseBuilder::with_text(format!("hello {}", _ctx.params.get("filepath").unwrap()))
}
