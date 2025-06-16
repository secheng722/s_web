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
// Macro Exports
// =============================================================================

/// 🚀 中间件宏支持 - 让参数化中间件可以写成简洁的 async fn 形式
pub use ree_macros::{middleware, middleware_fn};

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
