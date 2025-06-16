// =============================================================================
// Internal Module Declarations
// =============================================================================

mod context;
mod engine;
mod handler;
mod middleware;
mod response;
mod router;
mod trie;

// =============================================================================
// Internal System Imports (not exposed to users)
// =============================================================================

// These are used internally by the framework
use handler::Handler;
use middleware::{Middleware, execute_chain};
use router::Router;
// =============================================================================
// Public API Exports
// =============================================================================

pub use context::RequestCtx;
/// Core framework components
pub use engine::Engine;

/// Response handling
pub use response::{IntoResponse, Response, ResponseBuilder};

/// Middleware system
pub use middleware::Next;

/// Macro support
pub use ree_macros::middleware;

// =============================================================================
// Advanced/Internal API Exports
// =============================================================================

// =============================================================================
// Convenient Re-exports
// =============================================================================

/// HTTP status codes for convenience
pub use hyper::StatusCode;
