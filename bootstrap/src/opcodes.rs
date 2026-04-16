//! Opcode Definitions for .trib VM Executor
//! Part of Ring-007: .trib VM Executor

use "00-gf-family-foundation.tri";

/// Full opcode set for first VM version
pub const OPCODE: u8 {
    // System (8)
    HALT    = 0x00,
    NOP     = 0x01,

    // Memory Load (8)
    LOAD_CONST     = 0x02,
    PHI_POW       = 0x10,

    // Store (3)
    STORE_U8       = 0x20,
    STORE_U16      = 0x21,
    STORE_U32      = 0x22,
    STORE_GF16    = 0x23,
    STORE_GF32    = 0x24,

    // GF16/TF3/Trinity (7)
    GF16_ADD     = 0x30,
    GF16_MUL     = 0x31,
    TF3_AND      = 0x32,
    TF3_OR       = 0x33,
    TF3_NOT      = 0x34,

    // Trinity Verification (4)
    VERIFY_PHI    = 0x40,
    SAVE_EXPERIENCE = 0x50,

    // Control Flow (4)
    JUMP          = 0x50,
    JUMP_IF_FALSE = 0x51,
    JUMP_IF_TRUE  = 0x52,
    CALL          = 0x53,
    RET           = 0x54,
}

/// Opcode name for error messages
pub fn opcode_name(op: OPCODE) -> String {
  match op {
    OPCODE::HALT => "HALT",
    OPCODE::NOP => "NOP",
    OPCODE::LOAD_CONST => "LOAD_CONST",
    OPCODE::PHI_POW => "PHI_POW",
    OPCODE::STORE_U8 => "STORE_U8",
    OPCODE::STORE_U16 => "STORE_U16",
    OPCODE::STORE_U32 => "STORE_U32",
    OPCODE::STORE_GF16 => "STORE_GF16",
    OPCODE::STORE_GF32 => "STORE_GF32",
    OPCODE::GF16_ADD => "GF16_ADD",
    OPCODE::GF16_MUL => "GF16_MUL",
    OPCODE::TF3_AND => "TF3_AND",
    OPCODE::TF3_OR => "TF3_OR",
    OPCODE::TF3_NOT => "TF3_NOT",
    OPCODE::VERIFY_PHI => "VERIFY_PHI",
    OPCODE::SAVE_EXPERIENCE => "SAVE_EXPERIENCE",
    OPCODE::JUMP => "JUMP",
    OPCODE::JUMP_IF_FALSE => "JUMP_IF_FALSE",
    OPCODE::JUMP_IF_TRUE => "JUMP_IF_TRUE",
    OPCODE::CALL => "CALL",
    OPCODE::RET => "RET",
    _ => format!("UNKNOWN(0x{:02x})", op),
  }
}
