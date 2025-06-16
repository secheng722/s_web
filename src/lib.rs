//! # Ree HTTP Framework
//!
//! A simple, fast, and type-safe HTTP framework for Rust built on top of Hyper.
//!
//! ## Design Philosophy
//!
//! - **Simple First**: Use `handler()` for automatic type conversion in 99% of cases
//! - **Flexible Control**: Return `Response` directly when you need precise control
//! - **Type Safety**: Compile-time guarantees for request/response handling
//!
//! ## Quick Start
//!
//! ```rust
//! use ree::{Engine, handler};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = Engine::new();
//!     
//!     // Simple text response
//!     app.get("/hello", handler(|_| async { "Hello, World!" }));
//!     
//!     // JSON response
//!     app.get("/json", handler(|_| async {
//!         json!({"message": "Hello", "status": "ok"})
//!     }));
//!     
//!     // Path parameters
//!     app.get("/greet/:name", handler(|ctx| async move {
//!         let name = ctx.get_param("name").map_or("Guest", |v| v);
//!         format!("Hello, {}!", name)
//!     }));
//!     
//!     app.run("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! For precise control over responses:
//!
//! ```rust
//! use ree::{Engine, ResponseBuilder};
//!
//! let mut app = Engine::new();
//!
//! app.get("/custom", |_| async {
//!     let mut response = ResponseBuilder::with_json(r#"{"custom": true}"#);
//!     response.headers_mut().insert("X-Framework", "Ree".parse().unwrap());
//!     response
//! });
//! ```

// =============================================================================
// Module Declarations
// =============================================================================

mod context;
mod engine;
mod handler;
mod middleware;
mod response;
mod router;
mod trie;

// =============================================================================
// Core Exports
// =============================================================================

/// Request context and parameter handling
pub use context::RequestCtx;

/// Main HTTP engine for building applications
pub use engine::Engine;

/// Handler trait and helper functions
pub use handler::Handler;

/// Middleware system
pub use middleware::{Middleware, Next, execute_chain};

/// Response types and builders
pub use response::{IntoResponse, Response, ResponseBuilder};

/// Internal router (typically not needed for end users)
pub use router::Router;

// =============================================================================
// Re-exports from Dependencies
// =============================================================================

/// Re-export common HTTP status codes for convenience
pub use hyper::StatusCode;

// =============================================================================
// Advanced Exports
// =============================================================================

/// Trie data structure (for advanced routing customization)
pub use trie::Node;
