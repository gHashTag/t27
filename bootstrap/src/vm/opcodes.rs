//! Opcodes — Trinity VM Opcode Set
//!
//! Defines all opcodes for Trinity VM as specified in `specs/vm_core.t27`.
//! Part of modular compiler architecture (Ring-019 - Parity Target 1).
//!
//! This module provides opcode definitions and encoding utilities
//! that conform to the .trib canonical format.

use super::super::super::types;

/// Core opcodes (subset of full ISA)
///
/// These are the fundamental operations that any Trinity VM must implement.
pub const OP_NOP: u8 = 0x00;
pub const OP_HALT: u8 = 0xFF;

// Load/Store operations
pub const OP_LOAD_CONST: u8 = 0x01;
pub const OP_LOAD_VAR: u8 = 0x02;
pub const OP_STORE_VAR: u8 = 0x03;

// Arithmetic operations
pub const OP_ADD: u8 = 0x10;
pub const OP_SUB: u8 = 0x11;
pub const OP_MUL: u8 = 0x12;
pub const OP_DIV: u8 = 0x13;

// Logical operations
pub const OP_AND: u8 = 0x20;
pub const OP_OR: u8 = 0x21;
pub const OP_XOR: u8 = 0x22;

// Trit/Kleene logic operations (K3)
pub const OP_TRIT_AND: u8 = 0x30;
pub const OP_TRIT_OR: u8 = 0x31;
pub const OP_TRIT_XOR: u8 = 0x32;
pub const OP_TRIT_NEG: u8 = 0x33;

// VSA operations (from VSA/FHRR)
pub const OP_VSA_BIND: u8 = 0x40;
pub const OP_VSA_BUNDLE: u8 = 0x41;
pub const OP_VSA_UNBUNDLE: u8 = 0x42;

// Control flow operations
pub const OP_JUMP: u8 = 0x60;
pub const OP_JUMP_IF: u8 = 0x61;
pub const OP_CALL: u8 = 0x62;
pub const OP_RET: u8 = 0x63;

// Memory operations
pub const OP_MALLOC: u8 = 0x70;
pub const OP_FREE: u8 = 0x71;
pub const OP_MEMCPY: u8 = 0x72;

/// Opcode categories
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpcodeCategory {
  LoadStore,
  Arithmetic,
  Logical,
  TritKleene,
  VSA,
  ControlFlow,
  Memory,
  Model,
}

/// Get opcode category
pub fn opcode_category(opcode: u8) -> OpcodeCategory {
  match opcode {
    OP_NOP | OP_HALT => OpcodeCategory::ControlFlow,
    OP_LOAD_CONST | OP_LOAD_VAR => OpcodeCategory::LoadStore,
    OP_STORE_VAR => OpcodeCategory::LoadStore,
    OP_ADD | OP_SUB | OP_MUL | OP_DIV => OpcodeCategory::Arithmetic,
    OP_AND | OP_OR | OP_XOR => OpcodeCategory::Logical,
    OP_TRIT_AND | OP_TRIT_OR | OP_TRIT_XOR | OP_TRIT_NEG => OpcodeCategory::TritKleene,
    OP_VSA_BIND | OP_VSA_BUNDLE | OP_VSA_UNBUNDLE => OpcodeCategory::VSA,
    OP_JUMP | OP_JUMP_IF | OP_CALL | OP_RET => OpcodeCategory::ControlFlow,
    OP_MALLOC | OP_FREE | OP_MEMCPY => OpcodeCategory::Memory,
    _ => OpcodeCategory::Model,
  }
}

/// Check if opcode is a control flow operation
pub fn is_control_flow(opcode: u8) -> bool {
  matches opcode_category(opcode) {
    OpcodeCategory::ControlFlow => true,
    _ => false,
  }
}

/// Check if opcode terminates execution
pub fn is_terminating(opcode: u8) -> bool {
  opcode == OP_HALT || opcode == OP_RET
}

/// Encode instruction to bytes
///
/// Simple encoding for single-byte opcodes.
/// Complex instructions (with operands) will be handled in encoder module.
pub fn encode_opcode(opcode: u8) -> Vec<u8> {
  vec![opcode]
}

/// Opcode name for debugging
pub fn opcode_name(opcode: u8) -> &'static str {
  match opcode {
    OP_NOP => "nop",
    OP_HALT => "halt",
    OP_LOAD_CONST => "load_const",
    OP_LOAD_VAR => "load_var",
    OP_STORE_VAR => "store_var",
    OP_ADD => "add",
    OP_SUB => "sub",
    OP_MUL => "mul",
    OP_DIV => "div",
    OP_AND => "and",
    OP_OR => "or",
    OP_XOR => "xor",
    OP_TRIT_AND => "trit_and",
    OP_TRIT_OR => "trit_or",
    OP_TRIT_XOR => "trit_xor",
    OP_TRIT_NEG => "trit_neg",
    OP_VSA_BIND => "vsa_bind",
    OP_VSA_BUNDLE => "vsa_bundle",
    OP_VSA_UNBUNDLE => "vsa_unbundle",
    OP_JUMP => "jump",
    OP_JUMP_IF => "jump_if",
    OP_CALL => "call",
    OP_RET => "ret",
    OP_MALLOC => "malloc",
    OP_FREE => "free",
    OP_MEMCPY => "memcpy",
    _ => "unknown",
  }
}
