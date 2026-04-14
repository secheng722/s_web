use s_web::{Next, RequestCtx, Response};

pub async fn access_log(ctx: RequestCtx, next: Next) -> Response {
    let method = ctx.request.method().to_string();
    let path = ctx.request.uri().path().to_string();
    let resp = next(ctx).await;
    println!("[mini_blog] {} {} -> {}", method, path, resp.status());
    resp
}
