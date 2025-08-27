use s_web::{Engine, IntoResponse, RequestCtx, Response, ResponseBuilder};
use std::path::{Path, PathBuf};
use tokio::fs;

async fn upload_handler(ctx: RequestCtx) -> Response {
    // Expecting raw body bytes as file content for simplicity
    if let Some(bytes) = ctx.body_bytes() {
        // Optional: get filename from query like ?name=foo.bin
        let name = ctx
            .request
            .uri()
            .query()
            .and_then(|q| q.split('&').find(|kv| kv.starts_with("name=")))
            .and_then(|kv| kv.split('=').nth(1))
            .unwrap_or("upload.bin");

        let safe_name = sanitize_filename::sanitize(name);
        let save_dir = Path::new("uploads");
        let save_path: PathBuf = save_dir.join(&safe_name);

        if let Err(e) = fs::create_dir_all(save_dir).await {
            return (
                s_web::StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({"ok": false, "error": format!("Failed to create upload dir: {e}")}),
            ).into_response()
        }

        if let Err(e) = fs::write(&save_path, bytes.clone()).await {
            return (
                s_web::StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({"ok": false, "error": format!("Failed to write file: {e}")}),
            )
                .into_response();
        }

        return serde_json::json!({"ok": true, "filename": safe_name}).into_response();
    }

    (
        s_web::StatusCode::BAD_REQUEST,
        serde_json::json!({"ok": false, "error": "No file content in body"}),
    )
        .into_response()
}

async fn serve_uploads(ctx: RequestCtx) -> Response {
    if let Some(filepath) = ctx.get_param("filepath") {
        let path = Path::new("uploads").join(filepath);
        if path.exists() && path.is_file() {
            match fs::read(&path).await {
                Ok(content) => {
                    let ct = mime_guess::from_path(&path).first_or_octet_stream();
                    ResponseBuilder::new()
                        .status(s_web::StatusCode::OK)
                        .header("Content-Type", ct.essence_str())
                        .body(content)
                }
                Err(e) => (
                    s_web::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read file: {e}"),
                )
                    .into_response(),
            }
        } else {
            (s_web::StatusCode::NOT_FOUND, "File not found").into_response()
        }
    } else {
        (s_web::StatusCode::BAD_REQUEST, "Invalid path").into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Engine::new();

    // POST raw bytes to /upload?name=filename.ext
    app.post("/upload", upload_handler);

    // GET /uploads/*filepath to download previously uploaded files
    app.get("/uploads/*filepath", serve_uploads);

    // Minimal index page to try with curl
    app.get("/", |_| async {
        (
            s_web::StatusCode::OK,
            "text/html; charset=utf-8",
            r#"
            <html>
            <body>
                <h1>Upload Example</h1>
                <p>Upload with curl:</p>
                <pre>
                curl -X POST http://127.0.0.1:8080/upload?name=test.txt \
                     --data-binary @Cargo.toml
                </pre>
                <p>Then fetch at <code>/uploads/test.txt</code></p>
            </body>
            </html>
        "#,
        )
            .into_response()
    });

    app.run("127.0.0.1:8080").await?;
    Ok(())
}
