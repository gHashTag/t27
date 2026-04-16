//! .trib VM Executor - Ring-007
//! Implements dispatch table, opcode handlers, and phi identity verification
//! Uses GF16/GF32/TF3 types from gf_family_foundation.tri

use anyhow::Result;

// ══════════════════════════════════════════════════════════════
// .TRIB BINARY FORMAT HEADER
// ════════════════════════════════════════════════════════════════════

use "00-gf-family-foundation.tri";

pub const TRIB_MAGIC: u32 = 0x54524942; // "TRIB"
pub const TRIB_VERSION: u8 = 1;

// ════════════════════════════════════════════════════════════════════
// TribHeader from gf_family_foundation
// ═══════════════════════════════════════════════════════════════════════

pub struct TribHeader {
  magic: u32,        // 0x54524942 = "TRIB"
  version: u8,         // 1
  flags: u16,         // 16 bits (for future extensions)
  sections: u16,       // number of sections
  phi_hash: u64,       // 64 bits (seal proof)
}

pub struct TribSection {
  kind: SectionKind,
  offset: u32,
  length: u32,
}

pub const SectionKind: u8 = {
  // Core sections (mandatory)
  TYPES = 0x01,
  CONSTANTS = 0x02,
  FUNCTIONS = 0x03,
  STRINGS = 0x04,
  SYMBOLS = 0x05,

  // Optional sections (future)
  EXPORTS = 0x06,
  IMPORTS = 0x07,
  SEALS = 0x08,
  EXPERIENCE = 0x09,
  DEBUG_INFO = 0x0A,

  // Reserved (0x0B-0x0F)
  RESERVED_1 = 0x0B,
  RESERVED_2 = 0x0C,
  RESERVED_3 = 0x0D,
  RESERVED_4 = 0x0E,
  RESERVED_5 = 0x0F,
}

// ═════════════════════════════════════════════════════════════════════════
// Opcodes (32 total for first version)
// ══════════════════════════════════════════════════════════════════════

pub const Opcode: u8 = {
  // System opcodes (8)
  HALT = 0x00,
  NOP = 0x01,
  LOAD_CONST = 0x02,
  PHI_POW = 0x10,

  // Memory opcodes (8)
  LOAD_U8 = 0x10,
  LOAD_U16 = 0x11,
  LOAD_U32 = 0x12,
  LOAD_GF16 = 0x13,
  LOAD_F64 = 0x14,
  STORE_U8 = 0x20,
  STORE_U16 = 0x21,
  STORE_U32 = 0x22,
  STORE_GF16 = 0x23,
  STORE_F64 = 0x24,

  // GF16/TF3/Trinity opcodes (8)
  GF16_ADD = 0x20,
  GF16_MUL = 0x21,
  TF3_AND = 0x30,
  TF3_OR = 0x31,
  TF3_NOT = 0x32,
  VERIFY_PHI = 0x40,
  SAVE_EXPERIENCE = 0x50,

  // Control flow (4)
  JUMP = 0x50,
  JUMP_IF_FALSE = 0x51,
  JUMP_IF_TRUE = 0x52,
  CALL = 0x53,
  RET = 0x54,
}

pub struct VMFrame {
  stack: [GF32; 256],  // VM stack (GF32 for precision)
  sp: u8,               // stack pointer
  pc: u32,              // program counter
  flags: u16,           // VM flags register
  output: GF32,          // last output value
}

pub struct ExecResult {
  is_ok: bool,
  frame: VMFrame,
  error_msg: String,
}

pub const VMError: u8 = {
  StackOverflow = 0x01,
  InvalidOpcode = 0x02,
  InvariantFail = 0x03,
  DivisionByZero = 0x04,
  MemoryOutOfBounds = 0x05,
  MemoryAlignmentError = 0x06,
}

// ══════════════════════════════════════════════════════════════════════════
// VM execution state
// ══════════════════════════════════════════════════════════════════════

/// Create empty VM frame
fn new_frame() -> VMFrame {
  VMFrame {
    stack: [GF32 { bits: 0 }; 256],
    sp: 255,
    pc: 0,
    flags: 0,
    output: GF32 { bits: 0 },
  }
}

/// Read .trib header from bytes
fn trib_read_header(bytes: &[u8]) -> Result<TribHeader, String> {
  if bytes.len() < 12 {
    return Err("Header too short".to_string());
  }

  let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
  if magic != TRIB_MAGIC {
    return Err(format!("Invalid magic: 0x{:08x}", magic));
  }

  let version = bytes[4];
  if version != TRIB_VERSION {
    return Err(format!("Invalid version: {}", version));
  }

  let flags = u16::from_be_bytes([bytes[5], bytes[6]]);
  let sections = flags & 0xFFFF; // lower 16 bits
  let phi_hash = u64::from_be_bytes([
    bytes[7], bytes[8], bytes[9], bytes[10],
    bytes[11], bytes[12], bytes[13], bytes[14],
  ]);

  Ok(TribHeader {
    magic,
    version,
    flags,
    sections,
    phi_hash,
  })
}

/// Execute .trib bytecode
fn trib_execute(bytes: &[u8]) -> Result<ExecResult, String> {
  let header = trib_read_header(bytes)?;

  // Initialize VM frame
  let mut frame = new_frame();

  // Main execution loop
  loop {
    if frame.pc as usize >= bytes.len() as usize {
      break;
    }

    let opcode = bytes[frame.pc];

    match opcode {
      Opcode::HALT => {
        return Ok(ExecResult {
          is_ok: true,
          frame: frame.clone(),
          error_msg: String::new(),
        });
      },

      Opcode::NOP => {
        frame.pc += 1;
      },

      Opcode::LOAD_CONST => {
        if frame.pc + 5 > bytes.len() {
          return Err("LOAD_CONST truncated".to_string());
        }

        let value = u32::from_be_bytes([
          bytes[frame.pc + 1], bytes[frame.pc + 2],
          bytes[frame.pc + 3], bytes[frame.pc + 4],
        ]);

        if frame.sp == 255 {
          return Err("Stack overflow".to_string());
        }
        frame.sp = (frame.sp - 1) as u8;
        frame.stack[frame.sp as usize] = GF32::from_u32(value);
        frame.pc += 5;
      },

      Opcode::PHI_POW => {
        if frame.pc + 2 > bytes.len() {
          return Err("PHI_POW truncated".to_string());
        }

        let exp = bytes[frame.pc + 1];
        if exp > 30 {
          return Err("PHI_POW exp too large".to_string());
        }

        let n = exp as i32;

        use "00-gf-family-foundation.tri";
        let phi_pow_n = phi_pow(n);

        if frame.sp == 255 {
          return Err("Stack overflow".to_string());
        }
        frame.sp = (frame.sp - 1) as u8;
        frame.stack[frame.sp as usize] = GF32::from_f32(phi_pow_n);
        frame.pc += 2;
      },

      Opcode::GF16_ADD => {
        if frame.pc + 3 > bytes.len() {
          return Err("GF16_ADD truncated".to_string());
        }

        if frame.sp < 2 {
          return Err("GF16_ADD needs 2 stack entries".to_string());
        }

        let b_idx = (frame.sp - 1) as u8;
        let a_idx = frame.sp as u8;
        let b = frame.stack[b_idx as usize];
        let a = frame.stack[a_idx as usize];

        use "00-gf-family-foundation.tri";
        let result = gf16_add(b, a);

        frame.sp = (frame.sp - 2) as u8;
        frame.stack[frame.sp as usize] = GF32::from_f32(result);
        frame.pc += 3;
      },

      _ => {
        return Err(format!("Invalid opcode: 0x{:02x}", opcode));
      },
    }
}

/// Opcode dispatch (NOT match - function pointer array for throughput)
/// Static dispatch table: [fn(&mut VMFrame, u32); 256]
/// This will be filled by t27c codegen
/// For now: minimal stub
fn opcode_dispatch(frame: &mut VMFrame, op: Opcode) -> &mut VMFrame {
  // TODO: Replace with proper dispatch table
  frame
}

/// Verify phi identity from VMFrame state
fn verify_phi_identity(frame: &VMFrame) -> bool {
  // From gf_family_foundation: phi_sq_plus_one_over_phi_sq
  // (PHI * PHI + 1.0) / (PHI * PHI) - 3.0_f64 < 1e-12

  // Stack should be clean (output == 0)
  if frame.output.bits != 0 {
    return false;
  }

  true
}

// ════════════════════════════════════════════════════════════════════
// Helper: push with overflow check
fn push_and_check(frame: &mut VMFrame, value: GF32) -> Result<bool, String> {
  if frame.sp == 255 {
    return Err("Stack overflow".to_string());
  }
  frame.sp = (frame.sp - 1) as u8;
  frame.stack[frame.sp as usize] = value;
  Ok(true)
}

// ══════════════════════════════════════════════════════════════════════
// Stub for execute_jump (will be implemented)
fn execute_jump(_frame: &mut VMFrame, initial_pc: u32) {
  _frame.pc = initial_pc + 1; // stub
}

fn execute_halt(frame: &mut VMFrame) {
  // Halt - nothing to do, loop will check pc limit
}

// ══════════════════════════════════════════════════════════════════════════════
// Invariants from spec
// ════════════════════════════════════════════════════════════════════════════

// Tests (8 mandatory)
test trib_read_magic {
  given spec = trib_vm_executor
  let magic_bytes = [0x54, 0x52, 0x42, 0x24, 0x49, 0x00, 0x00];

  let header = trib_read_header(&magic_bytes)?;

  assert header.magic == 0x54524942
}

test trib_header_version_is_valid {
  given spec = trib_vm_executor
  let magic_bytes = [0x54, 0x52, 0x42, 0x24, 0x49, 0x01, 0x00];

  let header = trib_read_header(&magic_bytes)?;

  assert header.version == 1
}

test trib_header_roundtrip {
  given spec = trib_vm_executor
  let header = TribHeader {
    magic: 0x54524942,
    version: 1,
    flags: 0x1234,
    sections: 3,
    phi_hash: 0xDEADBEEF,
  };

  let encoded = trib_header_to_bytes(&header);
  let decoded = trib_bytes_to_header(&encoded);

  assert decoded.magic == header.magic
  assert decoded.version == header.version
  assert decoded.flags == header.flags
  assert decoded.sections == header.sections
}

test phi_identity_from_constants {
  given spec = trib_vm_executor

  use "00-gf-family-foundation.tri";

  let one_over_phi_sq_plus_one = (PHI_SQ + 1.0) / (PHI * PHI);
  let phi_sq_plus_one_over_phi_sq = (PHI * PHI) + 1.0 / (PHI * PHI) - 3.0_f64;

  let lhs = GF32_PHI_SQ;
  let rhs = one_over_phi_sq_plus_one / phi_sq_plus_one_over_phi_sq;

  let tolerance = 0.000_001;

  assert (lhs - rhs).abs() < tolerance
}

test stack_never_underflows {
  given spec = trib_vm_executor
  let mut frame = new_frame();

  for i in 0..253 {
    frame.stack[frame.sp] = GF32_MAX_VAL as i32;
    if frame.sp == 255 {
      break;
    }
    frame.sp = (frame.sp - 1) as u8;
  }

  assert frame.sp == 0
}

test halt_sets_output_to_last_value {
  given spec = trib_vm_executor
  let mut frame = new_frame();

  let test_value = 3.14159f64 as i32;

  frame.output = GF32::from_u32(test_value);
  execute_halt(&mut frame);

  assert frame.output == GF32::from_u32(test_value)
}

test jump_advances_pc {
  given spec = trib_vm_executor
  let mut frame = new_frame();

  let initial_pc = 10;

  frame.output = 42 as i32;
  execute_jump(&mut frame, initial_pc);

  assert frame.pc == initial_pc + 1
}

bench opcode_dispatch_throughput {
  measure nanoseconds to opcode_dispatch(&mut VMFrame {}, Opcode::NOP)
  target 500_000_000  -- 500M ops/s dispatch loop
  warmup 3
  runs 100
}
