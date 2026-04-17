//! Lower to trib bytecode
//!
//! Lowers .tri IR to trib bytecode format.
//! Part of modular compiler architecture (Ring-018).

use anyhow::Result;
use super::tri_ir::TriIr;

/// Trib opcode (simplified for now)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TribOp {
    /// Load immediate
    LdImm(u8),

    /// Load from register
    LdReg(u8),

    /// Store immediate
    StImm(u8),

    /// Store to register
    StReg(u8),

    /// Add
    Add,

    /// Subtract
    Sub,

    /// Multiply
    Mul,

    /// Halt
    Halt,
}

/// Trib instruction
#[derive(Debug, Clone)]
pub struct TribInstruction {
    pub op: TribOp,
    pub args: Vec<u8>,
}

/// Lower .tri IR to trib bytecode (stub)
///
/// TODO: Implement full lowering in Ring-018/019.
pub fn lower_to_trib(_ir: &[TriIr]) -> Result<Vec<TribInstruction>> {
    // Placeholder: Returns a simple halt instruction
    // This will be replaced with full lowering in future rings

    Ok(vec![TribInstruction {
        op: TribOp::Halt,
        args: Vec::new(),
    }])
}
