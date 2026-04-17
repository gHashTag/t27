//! Diagnostics module
//!
//! Error reporting, source locations, and rendering.
//! Part of modular compiler architecture (Ring-018).

pub mod error;
pub mod span;
pub mod render;

// Re-export diagnostic types
pub use error::{CompilerError, Diagnostic};
pub use span::{Span, Position, Location};
pub use render::render_error;
