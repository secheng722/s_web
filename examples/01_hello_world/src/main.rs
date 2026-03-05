//! # 示例 1：Hello World（入门级）
//!
//! 演示最基本的框架用法：
//!   - 创建 Engine
//!   - 注册 GET 路由
//!   - 路径参数
//!   - 返回纯文本 / HTML
//!
//! 运行：
//!   cargo run -p 01_hello_world
//!
//! 接口：
//!   GET /           → 纯文本问候
//!   GET /hello/:name → 个性化问候
//!   GET /about      → HTML 页面

use s_web::{Engine, RequestCtx, ResponseBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    // 最简路由：直接返回 &str，框架自动包装成 200 text/plain 响应
    app.get("/", |_ctx: RequestCtx| async { "Hello, World!" });

    // 路径参数：`:name` 会被捕获到 ctx.params 中
    app.get("/hello/:name", |ctx: RequestCtx| async move {
        let name = ctx
            .get_param("name")
            .map(|s| s.as_str())
            .unwrap_or("stranger");
        format!("Hello, {}! 👋", name)
    });

    // 返回 HTML：使用 ResponseBuilder::html
    app.get("/about", |_ctx: RequestCtx| async {
        ResponseBuilder::html(
            r#"<!DOCTYPE html>
<html>
  <head><meta charset="utf-8"><title>About</title></head>
  <body>
    <h1>s_web</h1>
    <p>A simple and efficient Rust HTTP framework.</p>
    <ul>
      <li><a href="/">Home</a></li>
      <li><a href="/hello/Rustacean">Greet Rustacean</a></li>
    </ul>
  </body>
</html>"#,
        )
    });

    println!("🚀 Example 1 · Hello World");
    println!("   http://127.0.0.1:3000/");
    println!("   http://127.0.0.1:3000/hello/Rustacean");
    println!("   http://127.0.0.1:3000/about");

    app.run("127.0.0.1:3000").await
}
