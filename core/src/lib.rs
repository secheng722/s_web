// =============================================================================
// Internal Module Declarations
// =============================================================================

mod context;
mod engine;
mod handler;
mod middleware;
mod response;
mod router;
mod swagger;
mod trie;

// =============================================================================
// Internal System Imports (not exposed to users)
// =============================================================================

// These are used internally by the framework
use middleware::{execute_chain, Middleware};
use router::Router;
// =============================================================================
// Public API Exports
// =============================================================================

pub use context::RequestCtx;
/// Core framework components
pub use engine::Engine;

/// Handler trait for request processing
pub use handler::Handler;

/// Response handling
pub use response::{IntoResponse, Response, ResponseBuilder};

/// Middleware system
pub use middleware::{IntoNext, Next};

/// Swagger API documentation
pub use swagger::{SwaggerInfo, SwaggerBuilder, swagger};

// =============================================================================
// Advanced/Internal API Exports
// =============================================================================

// =============================================================================
// Convenient Re-exports
// =============================================================================

/// HTTP status codes for convenience
pub use hyper::StatusCode;
