//! # 示例 4：Todo App —— 内存 CRUD（中高级）
//!
//! 综合运用框架的主要特性：
//!   - 共享状态（`Arc<Mutex<…>>`）跨处理器传递
//!   - 完整 REST CRUD（GET / POST / PUT / PATCH / DELETE）
//!   - 路由分组 + 组级中间件（日志）
//!   - 启动 / 关闭钩子
//!   - 查询参数过滤
//!   - 统一 JSON 错误格式
//!
//! 运行：
//!   cargo run -p 04_todo_app
//!
//! 接口：
//!   GET    /todos              → 获取全部（支持 ?done=true/false 过滤）
//!   POST   /todos              → 新建（body: {"title":"…"}）
//!   GET    /todos/:id          → 获取单条
//!   PUT    /todos/:id          → 整体替换（body: {"title":"…","done":bool}）
//!   PATCH  /todos/:id/done     → 标记完成
//!   DELETE /todos/:id          → 删除

use s_web::{Engine, IntoResponse, Next, RequestCtx, Response, ResponseBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};

// ──────────────────────────────────────────
// 数据模型
// ──────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

/// 共享内存存储
#[derive(Clone, Default)]
struct Store {
    todos: Arc<Mutex<Vec<Todo>>>,
    next_id: Arc<Mutex<u32>>,
}

impl Store {
    fn new() -> Self {
        Self {
            todos: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    fn insert(&self, title: String) -> Todo {
        let mut id_guard = self.next_id.lock().unwrap();
        let id = *id_guard;
        *id_guard += 1;

        let todo = Todo { id, title, done: false };
        self.todos.lock().unwrap().push(todo.clone());
        todo
    }

    fn list(&self) -> Vec<Todo> {
        self.todos.lock().unwrap().clone()
    }

    fn get(&self, id: u32) -> Option<Todo> {
        self.todos.lock().unwrap().iter().find(|t| t.id == id).cloned()
    }

    fn update(&self, id: u32, title: String, done: bool) -> Option<Todo> {
        let mut todos = self.todos.lock().unwrap();
        if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
            t.title = title;
            t.done = done;
            Some(t.clone())
        } else {
            None
        }
    }

    fn mark_done(&self, id: u32) -> Option<Todo> {
        let mut todos = self.todos.lock().unwrap();
        if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
            t.done = true;
            Some(t.clone())
        } else {
            None
        }
    }

    fn delete(&self, id: u32) -> bool {
        let mut todos = self.todos.lock().unwrap();
        let before = todos.len();
        todos.retain(|t| t.id != id);
        todos.len() < before
    }

    fn count(&self) -> usize {
        self.todos.lock().unwrap().len()
    }
}

// ──────────────────────────────────────────
// 辅助
// ──────────────────────────────────────────

fn json_err(status: StatusCode, msg: &str) -> Response {
    ResponseBuilder::new()
        .status(status)
        .content_type("application/json; charset=utf-8")
        .body(json!({ "error": msg }).to_string())
}

fn parse_id(ctx: &RequestCtx) -> Option<u32> {
    ctx.get_param("id")?.parse().ok()
}

/// 路由组日志中间件
async fn log_middleware(ctx: RequestCtx, next: Next) -> Response {
    let method = ctx.request.method().to_string();
    let path   = ctx.request.uri().path().to_string();
    let resp   = next(ctx).await;
    println!("[todos] {} {} → {}", method, path, resp.status());
    resp
}

// ──────────────────────────────────────────
// main
// ──────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::new();

    // 预置几条数据
    store.insert("Buy groceries".into());
    store.insert("Read the Rust book".into());
    store.insert("Write more examples".into());

    // ── 启动 / 关闭钩子 ───────────────────────────────
    let store_for_hooks = store.clone();
    let app = Engine::new()
        .on_startup(|| async {
            println!("✅ Todo server started");
        })
        .on_shutdown(move || {
            let count = store_for_hooks.count();
            async move {
                println!("🛑 Shutting down. {} todo(s) in memory.", count);
            }
        });

    let mut app = app;

    // ── /todos 路由分组 ───────────────────────────────
    {
        let g = app.group("/todos");
        g.use_middleware(log_middleware);

        // GET /todos?done=true|false
        let s = store.clone();
        g.get("/", move |ctx: RequestCtx| {
            let s = s.clone();
            async move {
                let filter = ctx.query_param("done");
                let todos: Vec<Todo> = match filter.as_deref() {
                    Some("true")  => s.list().into_iter().filter(|t|  t.done).collect(),
                    Some("false") => s.list().into_iter().filter(|t| !t.done).collect(),
                    _             => s.list(),
                };
                json!({ "count": todos.len(), "todos": todos }).into_response()
            }
        });

        // POST /todos  — 创建
        let s = store.clone();
        g.post("/", move |mut ctx: RequestCtx| {
            let s = s.clone();
            async move {
                #[derive(Deserialize)]
                struct Payload { title: String }

                let payload: Payload = match ctx.json().await {
                    Ok(v) => v,
                    Err(_) => return json_err(StatusCode::BAD_REQUEST, "field `title` is required"),
                };

                if payload.title.trim().is_empty() {
                    return json_err(StatusCode::BAD_REQUEST, "title must not be empty");
                }

                let todo = s.insert(payload.title);
                ResponseBuilder::new()
                    .status(StatusCode::CREATED)
                    .content_type("application/json; charset=utf-8")
                    .body(json!(todo).to_string())
            }
        });

        // GET /todos/:id
        let s = store.clone();
        g.get("/:id", move |ctx: RequestCtx| {
            let s = s.clone();
            async move {
                let id = match parse_id(&ctx) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };
                match s.get(id) {
                    Some(t) => json!(t).into_response(),
                    None    => json_err(StatusCode::NOT_FOUND, "todo not found"),
                }
            }
        });

        // PUT /todos/:id — 整体更新
        let s = store.clone();
        g.put("/:id", move |mut ctx: RequestCtx| {
            let s = s.clone();
            async move {
                let id = match parse_id(&ctx) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };

                #[derive(Deserialize)]
                struct Payload { title: String, done: bool }

                let p: Payload = match ctx.json().await {
                    Ok(v) => v,
                    Err(_) => return json_err(StatusCode::BAD_REQUEST, "invalid JSON body"),
                };

                match s.update(id, p.title, p.done) {
                    Some(t) => json!(t).into_response(),
                    None    => json_err(StatusCode::NOT_FOUND, "todo not found"),
                }
            }
        });

        // PATCH /todos/:id/done — 标记完成
        let s = store.clone();
        g.add_route("PATCH", "/:id/done", move |ctx: RequestCtx| {
            let s = s.clone();
            async move {
                let id = match parse_id(&ctx) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };
                match s.mark_done(id) {
                    Some(t) => json!(t).into_response(),
                    None    => json_err(StatusCode::NOT_FOUND, "todo not found"),
                }
            }
        });

        // DELETE /todos/:id
        let s = store.clone();
        g.delete("/:id", move |ctx: RequestCtx| {
            let s = s.clone();
            async move {
                let id = match parse_id(&ctx) {
                    Some(v) => v,
                    None    => return json_err(StatusCode::BAD_REQUEST, "id must be a positive integer"),
                };
                if s.delete(id) {
                    ResponseBuilder::new()
                        .status(StatusCode::NO_CONTENT)
                        .empty_body()
                } else {
                    json_err(StatusCode::NOT_FOUND, "todo not found")
                }
            }
        });
    }

    println!("🚀 Example 4 · Todo App  →  http://127.0.0.1:3000");
    println!();
    println!("  curl http://127.0.0.1:3000/todos/");
    println!("  curl http://127.0.0.1:3000/todos/1");
    println!("  curl -X POST http://127.0.0.1:3000/todos/ \\");
    println!("       -H 'Content-Type: application/json' -d '{{\"title\":\"New task\"}}'");
    println!("  curl -X PATCH http://127.0.0.1:3000/todos/1/done");
    println!("  curl -X DELETE http://127.0.0.1:3000/todos/2");
    println!("  curl 'http://127.0.0.1:3000/todos/?done=false'");

    app.run("127.0.0.1:3000").await
}
