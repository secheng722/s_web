//! # Ree HTTP Framework
//! 
//! A simple and fast HTTP framework for Rust, inspired by Gin and Gee.
//! 
//! ## Quick Start
//! 
//! ```rust
//! use ree::{Engine, ResponseBuilder};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = Engine::new();
//!     
//!     app.get("/", |_| async {
//!         ResponseBuilder::with_text("Hello, World!")
//!     });
//!     
//!     app.run("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//! ```

mod context;
mod engine;
mod handler;
mod middleware;
mod response;
mod router;
mod trie;

// Core exports
pub use context::RequestCtx;
pub use engine::Engine;
pub use handler::Handler;
pub use middleware::{Middleware, Next, AccessLog, Cors};
pub use response::{Response, ResponseBuilder};
pub use router::Router;

// Advanced exports for custom usage
pub use trie::Node;

// Re-export common hyper types for convenience
pub use hyper::StatusCode;

