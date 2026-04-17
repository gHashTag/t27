//! Compiler façade
//!
//! Public API for the Trinity compiler.
//! Re-exports all compiler subsystems with clean, stable interface.
//!
//! This is the ONLY public entry point for compiler codegen.
//! Part of modular compiler architecture (Ring-019 - R001).
//!
//! This file is the ONLY public entry point for compiler codegen.
//! Re-exports codegen components with stable, versioned interface.

// Re-export all compiler modules
pub mod ast;
pub mod parser;
pub mod semantic;
pub mod lower;
pub mod diagnostics;
pub mod render;
pub mod codegen;

// Re-export key types and functions
pub use ast::{Expr, Decl, Type, Literal, Stmt, Pattern, BinOp, UnaryOp};
pub use parser::{parse_expr, parse_module, parse_type};
pub use semantic::{typecheck, promotion, name_resolution};
pub use lower::{tri_ir::lower_to_tri_ir, trib::lower_to_trib};
pub use diagnostics::{error::CompilerError, Diagnostic, render_error};
pub use render::{render_tri_ir, render_trib, emit_decl};

/// Parser facade
///
/// Orchestrates .t27 file parsing into AST.
///
/// Usage:
/// ```rust
/// use compiler::*;
///
/// let ast = compiler::parse_module("specs/core_trinity.t27")?;
/// println!("Parsed: {:#?}", ast);
/// ```
pub fn parse_module(_spec: &str) -> Result<Vec<super::ast::Decl>> {
    use super::super::parser::parse_module;
    super::super::parser::parse_module(_spec)
}

/// Type checker facade
///
/// Performs type checking on AST expressions and declarations.
///
/// Usage:
/// ```rust
/// use compiler::*;
///
/// let decls = compiler::parse_module("specs/core_trinity.t27")?;
/// let () = compiler::typecheck_program(&decls)?;
/// println!("Type checked: {:?}", ());
/// ```
pub fn typecheck_program(_decls: &[super::ast::Decl]) -> Result<()> {
    use super::super::semantic::typecheck;
    super::super::semantic::typecheck::typecheck_program(_decls)
}

/// Lowerer facade
///
/// Lowers AST to .tri IR format.
///
/// Usage:
/// ```rust
/// use compiler::*;
///
/// let decls = compiler::parse_module("specs/core_trinity.t27")?;
/// let ir = compiler::lower_program(&decls)?;
/// println!("Lowered: {:#?}", ir);
/// ```
pub fn lower_program(_decls: &[super::ast::Decl]) -> Result<Vec<super::lower::tri_ir::TriIr>> {
    use super::super::lower::tri_ir::lower_program;
    super::super::lower::tri_ir::lower_program(_decls)
}

/// Codegen facade
///
/// Orchestrates .tri IR generation to .trib bytecode.
///
/// Usage:
/// ```rust
/// use compiler::*;
///
/// let decls = compiler::parse_module("specs/trity_model.t27")?;
/// let ir = compiler::lower_program(&decls)?;
/// let code = compiler::compile_model(&ir)?;
/// println!("Generated:\n{}", code);
/// ```
pub fn compile_model(_spec: &str) -> Result<String> {
    use super::codegen::compile_model;
    super::codegen::compile_model(_spec)
}
