mod context;
mod ree;
mod router;
mod tire;

use context::RequestCtx;
use ree::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建一个新的引擎实例
    let mut r = ree::Engine::new();
    // r.use_middleware(AccessLog);
    // 添加路由
    r.get("/", hello);
    r.get("/hello", hello2);
    r.get("/hello/:name", hello_name);
    r.get("/assets/*filepath", hello_path);

    let api = r.group("/api");
    api.use_middleware(AccessLog);
    api.get("/hello", hello);
    

    r.run("127.0.0.1:3000").await.unwrap();
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
