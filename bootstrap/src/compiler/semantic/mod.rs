//! Semantic analysis module
//!
//! Entry point for type checking, name resolution, and semantic validation.
//! Part of modular compiler architecture (Ring-018).

pub mod typecheck;
pub mod promotion;
pub mod name_resolution;

// Re-export semantic analysis functions
pub use typecheck::typecheck;
pub use promotion::promote;
pub use name_resolution::resolve;
