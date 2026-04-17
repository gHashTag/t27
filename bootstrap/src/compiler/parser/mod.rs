//! Parser module
//!
//! Entry point for parsing .t27 specifications into AST.
//! Part of modular compiler architecture (Ring-018).

pub mod expr_parser;
pub mod decl_parser;
pub mod type_parser;

// Re-export parser functions
pub use expr_parser::parse_expr;
pub use decl_parser::parse_module;
pub use type_parser::parse_type;
