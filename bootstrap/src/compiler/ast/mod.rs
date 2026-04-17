//! AST module
//!
//! Aggregates all AST node types for the compiler.
//! Part of modular compiler architecture (Ring-018).

pub mod expr;
pub mod decl;
pub mod pattern;
pub mod types;

// Re-export common types for convenience
pub use expr::{Expr, Literal, TritValue, Stmt, BinOp, UnaryOp, MatchArm, Pattern};
pub use decl::{Decl, Export, Param, ImportItem};
pub use types::Type;
pub use pattern::Pattern;
