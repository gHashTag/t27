// bootstrap/src/compiler/t27c/mod.rs
// t27c codegen module (Ring-003 VM Core)
//
// This module compiles .t27 AST to .tri bytecode format.
// Follows Trinity architecture: spec-first (.t27) → .tri (canonical IR) → VM.

pub mod parser;
pub mod codegen;
pub mod emitter;

// Re-export commonly used types
pub use super::parser::{parse_module, ModuleNode};
pub use super::ast::{ExprNode, StmtNode, Expr};
pub use super::codegen::{TriBytecode};
pub use super::emitter::{write_trib_file};
