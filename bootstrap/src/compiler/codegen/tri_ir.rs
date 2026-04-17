//! Lowering to .tri IR
//!
//! Lowers Model DSL AST to .tri IR intermediate format.
//! Part of modular compiler architecture (Ring-019 - R001).
//!
//! .tri IR is the canonical intermediate representation between
//! source `.tri` specs and `.trib` bytecode.
//!
//! This is NOT the final emit phase — that's handled by `emit/trib`.

use anyhow::Result;
use super::ast::{Expr, Type, Literal};

/// .tri IR instruction
#[derive(Debug, Clone)]
pub enum TriIr {
    /// Constant value
    Const(i64),

    /// Load variable
    Var(String),

    /// Model operation (dense, embedding, etc.)
    ModelOp {
        op_type: u8,
        params: Vec<String>,
    },
}

/// Lower a model declaration to .tri IR (stub)
///
/// TODO: Implement full model lowering for Parameter Golf.
/// For now, returns simple IR with placeholder model body.
pub fn lower_model(ast: &Expr) -> Vec<TriIr> {
    // Placeholder: Returns simple .tri IR
    // This will be replaced with full model lowering in future rings

    match ast {
        // Model declaration → emit model opcodes
        Expr::Literal(Literal::Integer(0)) => {
            vec![
                TriIr::Const(42),  // Magic constant
            ]
        }

        _ => vec![TriIr::Nop],
    }
}
