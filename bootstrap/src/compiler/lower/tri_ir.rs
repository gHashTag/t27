//! Lower to .tri IR
//!
//! Lowers Trinity AST to .tri intermediate representation.
//! Part of modular compiler architecture (Ring-018).

use anyhow::Result;
use super::Expr;
use super::Literal;
use super::TritValue;

/// .tri IR instruction
#[derive(Debug, Clone)]
pub enum TriIr {
    /// Load variable
    Load(String),

    /// Store to variable
    Store(String, Box<TriIr>),

    /// Binary operation
    BinOp {
        op: String,
        left: Box<TriIr>,
        right: Box<TriIr>,
    },

    /// Unary operation
    UnaryOp {
        op: String,
        operand: Box<TriIr>,
    },

    /// Call function
    Call {
        func: String,
        args: Vec<TriIr>,
    },

    /// Constant literal
    Const(i64),

    /// Trit literal (-1, 0, 1)
    Trit(i8),

    /// GF16 literal
    GF16(u16),

    /// GF32 literal
    GF32(u32),

    /// No-op
    Nop,
}

/// Lower expression to .tri IR (R-L01: Const support)
pub fn lower_to_tri_ir(expr: &Expr) -> Result<Vec<TriIr>> {
    match expr {
        // R-L01: Integer literals
        Expr::Literal(Literal::Integer(n)) => {
            Ok(vec![TriIr::Const(*n)])
        }

        // R-L01: Trit literals
        Expr::Literal(Literal::Trit(trit)) => {
            let val = match trit {
                super::ast::TritValue::Neg => -1i8,
                super::ast::TritValue::Zero => 0i8,
                super::ast::TritValue::Pos => 1i8,
            };
            Ok(vec![TriIr::Trit(val)])
        }

        // R-L01: Boolean literals (as integers)
        Expr::Literal(Literal::Bool(b)) => {
            Ok(vec![TriIr::Const(if *b { 1 } else { 0 })])
        }

        // R-L01: Float literals (as GF32 placeholder)
        Expr::Literal(Literal::Float(f)) => {
            // For now: convert to u32 bits
            Ok(vec![TriIr::GF32(f.to_bits())])
        }

        // Variable reference
        Expr::Var(name) => {
            Ok(vec![TriIr::Load(name.clone())])
        }

        // Not yet implemented in R-L01
        _ => Err(anyhow!("R-L01: not implemented: {:?}", expr)),
    }
}

/// Lower declaration to .tri IR (stub)
pub fn lower_decl(_decl: &Decl) -> Result<Vec<TriIr>> {
    // Placeholder
    Ok(vec![TriIr::Nop])
}
