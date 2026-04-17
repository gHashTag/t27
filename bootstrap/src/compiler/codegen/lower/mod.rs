//! Lowering module
//!
//! Aggregates all lowering operations: .tri IR → .trib bytecode.
//! Part of modular compiler architecture (Ring-019 - R001).
//!
//! Re-exports codegen components with stable interface.

pub mod tri_ir;
pub mod trib;

// Re-export lowering functions
pub use tri_ir::{lower_to_tri_ir};
pub use trib::emit_trib;
