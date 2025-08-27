use s_web::{Engine, Response, ResponseBuilder, StatusCode};
use std::path::Path;
use tokio::fs;

/// 注册基础路由
pub fn register_routes(app: &mut Engine) {

    // 主页重定向到前端首页
    app.get("/", |_| async {
        serve_static_file("frontend/index.html").await
    });

    // 静态HTML页面路由
    app.get("/login", |_| async {
        serve_static_file("frontend/login.html").await
    });

    app.get("/register", |_| async {
        serve_static_file("frontend/register.html").await
    });

    app.get("/articles", |_| async {
        serve_static_file("frontend/articles.html").await
    });

    app.get("/create-article", |_| async {
        serve_static_file("frontend/create-article.html").await
    });

    // 静态资源路由 - CSS
    app.get("/css/style.css", |_| async {
        serve_static_file("frontend/css/style.css").await
    });

    // 静态资源路由 - JS
    app.get("/js/app.js", |_| async {
        serve_static_file("frontend/js/app.js").await
    });

    app.get("/js/auth.js", |_| async {
        serve_static_file("frontend/js/auth.js").await
    });

    app.get("/js/articles.js", |_| async {
        serve_static_file("frontend/js/articles.js").await
    });

    app.get("/js/create-article.js", |_| async {
        serve_static_file("frontend/js/create-article.js").await
    });
}

/// 读取并提供静态文件
async fn serve_static_file(file_path: &str) -> Response {
    match fs::read_to_string(file_path).await {
        Ok(content) => {
            let content_type = get_content_type(file_path);
            ResponseBuilder::new()
                .status(StatusCode::OK)
                .content_type(content_type)
                .body(content)
        }
        Err(_) => ResponseBuilder::new()
            .status(StatusCode::NOT_FOUND)
            .body("File not found"),
    }
}

/// 根据文件扩展名获取 Content-Type
fn get_content_type(file_path: &str) -> &'static str {
    let path = Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "text/plain; charset=utf-8",
    }
}
