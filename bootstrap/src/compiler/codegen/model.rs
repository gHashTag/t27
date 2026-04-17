//! Model lowering to .tri IR
//!
//! Lowers Model DSL AST to .tri IR format.
//! Part of modular compiler architecture (Ring-019 - R001).

use anyhow::Result;
use super::ast::{Expr, Type, Literal};

/// Model operation types
#[derive(Debug, Clone)]
pub enum ModelOp {
    /// Dense layer: matmul
    Dense {
        input_size: u32,
        output_size: u32,
        weight_rows: u32,
        weight_cols: u32,
    },

    /// Embedding layer
    Embedding {
        vocab_size: u32,
        embed_dim: u32,
    },
}

/// Lower a model operation to .tri IR (stub)
///
/// TODO: Implement full model lowering for Parameter Golf.
/// For now, returns simple opcodes.
pub fn lower_model(ast: &super::ast::Expr) -> Vec<String> {
    // Placeholder: Returns simple opcodes
    // This will be replaced with full model lowering in future rings

    match ast {
        // Model declaration → emit opcodes
        Expr::Literal(Literal::Integer(0)) => {
            vec![
                "dense".to_string(),
                "dense_input_size: 0".to_string(),
            ]
        }

        _ => vec!["nop".to_string()],
    }
}
