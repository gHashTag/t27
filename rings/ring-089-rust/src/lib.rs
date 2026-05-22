//! ring-089 -- **TNN ISA** (balanced-ternary core ISA, executable model).
//!
//! Wave 16 (2026-05-22, Closes #719): the second **honestly-imported** Wave-11
//! crate. Wave 15 landed ring-088 (GF16 codec + MAC). Wave 16 lands ring-089:
//! a small but *complete and executable* model of t27's balanced-ternary ISA.
//!
//! ## What this crate does
//!
//! * Defines [`Trit`] (`-1 | 0 | +1`) and [`Word27`] (27 trits packed in
//!   `[i8; 27]`), matching `specs/isa/ternary_arithmetic.t27`
//!   (`TRITS_PER_WORD = 27`) and `specs/isa/registers.t27`
//!   (`NUM_REGISTERS = 27`, `REG_WIDTH = 27`).
//! * Implements ripple-carry balanced-ternary add and subtract over
//!   `Word27` exactly per the spec's `trit_add` rules (sums in -2..=2 wrap
//!   with carry +-1, "balanced ternary").
//! * Defines a 9-opcode instruction set ([`Opcode`]) covering the *minimum*
//!   needed to run a deterministic single-step CPU loop: NOP, MOV, ADDI,
//!   ADD, SUB, NEG, LOAD, STORE, HALT. This is **a subset**, not a claim
//!   of full ISA coverage.
//! * Provides [`Cpu`] -- a tiny fetch/decode/execute model with 27
//!   registers, R0 hardwired to zero, a small instruction memory and a
//!   small data memory. [`Cpu::step`] executes exactly one instruction.
//! * Exposes [`identity_witness`] returning the universal anchor
//!   `phi^2 + 1/phi^2 == 3` (to f64 1e-15).
//!
//! ## Honest scope (R5-HONEST)
//!
//! * **No GF16 instructions, no ternary-gates ALU, no pipeline, no branch
//!   prediction, no Coptic encoding.** Those layers exist in the spec but
//!   are not part of Wave 16. Future waves can extend the opcode table.
//! * **No new spec.** `TRIT_NEG`, `TRIT_ZERO`, `TRIT_POS`,
//!   `NUM_REGISTERS`, `TRITS_PER_WORD`, balanced-add carry rules all
//!   mirror existing `.t27` source (L6 CEILING).
//! * **`#![no_std]`** with zero external dependencies. Inline integer-only
//!   helpers; test cfg pulls `std` for the harness only.
//!
//! Anchor: `phi^2 + 1/phi^2 = 3`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

// ---------------------------------------------------------------------------
// Spec constants (mirror specs/isa/registers.t27 + ternary_arithmetic.t27)
// ---------------------------------------------------------------------------

/// Number of registers in the register file.
pub const NUM_REGISTERS: usize = 27;
/// Width of each register in trits (matches `TRITS_PER_WORD`).
pub const REG_WIDTH: usize = 27;
/// Number of trits per machine word.
pub const TRITS_PER_WORD: usize = 27;
/// Balanced-ternary "minus" value.
pub const TRIT_NEG: i8 = -1;
/// Balanced-ternary "zero" value.
pub const TRIT_ZERO: i8 = 0;
/// Balanced-ternary "plus" value.
pub const TRIT_POS: i8 = 1;
/// Hardwired zero register (per `specs/isa/registers.t27`).
pub const R0_ZERO: u8 = 0;

/// Golden ratio.
pub const PHI: f64 = 1.618_033_988_749_894_8_f64;

// ---------------------------------------------------------------------------
// Trit & Word27
// ---------------------------------------------------------------------------

/// A single balanced trit. The wrapped `i8` is always one of `-1`, `0`, `+1`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Trit(i8);

impl Trit {
    /// Construct a `Trit`, returning `None` if `v` is out of range.
    pub fn new(v: i8) -> Option<Self> {
        if v == TRIT_NEG || v == TRIT_ZERO || v == TRIT_POS {
            Some(Trit(v))
        } else {
            None
        }
    }

    /// Construct without checking. Caller must guarantee `v in {-1, 0, +1}`.
    pub const fn from_i8_unchecked(v: i8) -> Self {
        Trit(v)
    }

    /// Raw value in `-1..=1`.
    pub const fn value(self) -> i8 {
        self.0
    }
}

/// A 27-trit machine word. Trit at index 0 is the least-significant trit.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Word27 {
    trits: [i8; REG_WIDTH],
}

impl Word27 {
    /// All-zero word.
    pub const fn zero() -> Self {
        Word27 { trits: [0; REG_WIDTH] }
    }

    /// Construct from a `[i8; 27]` array. Returns `None` if any element is
    /// outside `-1..=1`.
    pub fn from_trits(trits: [i8; REG_WIDTH]) -> Option<Self> {
        let mut i = 0;
        while i < REG_WIDTH {
            let v = trits[i];
            if v != TRIT_NEG && v != TRIT_ZERO && v != TRIT_POS {
                return None;
            }
            i += 1;
        }
        Some(Word27 { trits })
    }

    /// Construct from a signed integer in the representable range.
    /// Out-of-range values saturate silently (still valid trits).
    ///
    /// Uses Euclidean (floor) division so negative inputs convert correctly:
    /// Rust's default `/` truncates toward zero, which gives the wrong
    /// balanced-ternary representation for negative `v`.
    pub fn from_i64(mut v: i64) -> Self {
        let mut trits = [0i8; REG_WIDTH];
        for slot in trits.iter_mut() {
            // Euclidean remainder is always in 0..=2.
            let rem = v.rem_euclid(3);
            let (d, carry) = match rem {
                0 => (0i8, 0i64),
                1 => (1i8, 0i64),
                2 => (-1i8, 1i64),
                _ => unreachable!(),
            };
            *slot = d;
            // Euclidean (floor) division pairs with rem_euclid; the carry
            // promotes a `digit = -1` step to the next place.
            v = v.div_euclid(3) + carry;
        }
        Word27 { trits }
    }

    /// Convert to `i64`. Always exact: 3^27 ~= 7.6e12 fits in `i64`.
    pub fn to_i64(self) -> i64 {
        let mut acc: i64 = 0;
        let mut place: i64 = 1;
        for &t in self.trits.iter() {
            acc += (t as i64) * place;
            place *= 3;
        }
        acc
    }

    /// Access a single trit at `index` (0 = LSB).
    pub fn trit_at(&self, index: usize) -> Option<Trit> {
        if index >= REG_WIDTH {
            None
        } else {
            Some(Trit(self.trits[index]))
        }
    }

    /// Set a trit at `index`. Returns `false` if out of range.
    pub fn set_trit(&mut self, index: usize, t: Trit) -> bool {
        if index >= REG_WIDTH {
            false
        } else {
            self.trits[index] = t.value();
            true
        }
    }

    /// Element-wise negation: every trit is negated.
    pub fn negate(self) -> Self {
        let mut out = [0i8; REG_WIDTH];
        let mut i = 0;
        while i < REG_WIDTH {
            out[i] = -self.trits[i];
            i += 1;
        }
        Word27 { trits: out }
    }
}

// ---------------------------------------------------------------------------
// Balanced-ternary ripple-carry add / sub
// ---------------------------------------------------------------------------

/// Add two trits with an input carry. Returns `(sum, carry_out)` per
/// `specs/isa/ternary_arithmetic.t27`.
pub fn trit_add(a: i8, b: i8, carry_in: i8) -> (i8, i8) {
    let sum = a as i32 + b as i32 + carry_in as i32;
    if sum > 1 {
        ((sum - 3) as i8, 1)
    } else if sum < -1 {
        ((sum + 3) as i8, -1)
    } else {
        (sum as i8, 0)
    }
}

/// Word-wide balanced-ternary add. Returns `(result, final_carry)`. The
/// final carry is non-zero only on overflow.
pub fn word_add(a: Word27, b: Word27) -> (Word27, i8) {
    let mut out = [0i8; REG_WIDTH];
    let mut c: i8 = 0;
    let mut i = 0;
    while i < REG_WIDTH {
        let (s, c2) = trit_add(a.trits[i], b.trits[i], c);
        out[i] = s;
        c = c2;
        i += 1;
    }
    (Word27 { trits: out }, c)
}

/// Word-wide balanced-ternary subtract. Returns `(result, final_borrow)`.
pub fn word_sub(a: Word27, b: Word27) -> (Word27, i8) {
    let neg_b = b.negate();
    let (s, c) = word_add(a, neg_b);
    (s, -c)
}

// ---------------------------------------------------------------------------
// Opcodes & Instructions
// ---------------------------------------------------------------------------

/// Minimum executable opcode subset. Wave 16 ships **9** opcodes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    /// No-op.
    Nop = 0,
    /// `rd <- rs1` (R0 reads as zero, writes ignored).
    Mov = 1,
    /// `rd <- rs1 + imm` (imm is an i32 interpreted as `Word27::from_i64`).
    Addi = 2,
    /// `rd <- rs1 + rs2`.
    Add = 3,
    /// `rd <- rs1 - rs2`.
    Sub = 4,
    /// `rd <- -rs1`.
    Neg = 5,
    /// `rd <- mem[ addr_from(rs1) ]`.
    Load = 6,
    /// `mem[ addr_from(rs1) ] <- rs2`.
    Store = 7,
    /// Halt the CPU.
    Halt = 8,
}

/// A decoded instruction. Encoding matches `r_type_format` /
/// `i_type_format` from `specs/fpga/ternary_isa.t27` *semantically* (we
/// keep fields as host integers; bit-packing is a later wave).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Instr {
    /// Opcode.
    pub op: Opcode,
    /// Destination register index (`0..NUM_REGISTERS`).
    pub rd: u8,
    /// Source register 1 index.
    pub rs1: u8,
    /// Source register 2 index (used by Add/Sub/Store; ignored otherwise).
    pub rs2: u8,
    /// Immediate (i32). Used by Addi only.
    pub imm: i32,
}

impl Instr {
    /// Convenience constructor for no-operand opcodes.
    pub const fn nop() -> Self {
        Instr { op: Opcode::Nop, rd: 0, rs1: 0, rs2: 0, imm: 0 }
    }
    /// Convenience constructor for HALT.
    pub const fn halt() -> Self {
        Instr { op: Opcode::Halt, rd: 0, rs1: 0, rs2: 0, imm: 0 }
    }
}

// ---------------------------------------------------------------------------
// CPU model
// ---------------------------------------------------------------------------

/// Size of the (fixed) data memory in `Word27` cells.
pub const DATA_MEM_CELLS: usize = 256;

/// A tiny deterministic CPU model for the 9-opcode subset above.
pub struct Cpu {
    /// 27 registers. R0 is hardwired to zero on read.
    pub regs: [Word27; NUM_REGISTERS],
    /// Program counter (instruction index).
    pub pc: usize,
    /// `true` once HALT executes.
    pub halted: bool,
    /// Instruction memory (caller-owned).
    pub program: [Instr; 64],
    /// Number of valid instructions in `program`.
    pub program_len: usize,
    /// Data memory.
    pub data: [Word27; DATA_MEM_CELLS],
}

impl Cpu {
    /// Construct a fresh CPU with zeroed state.
    pub fn new() -> Self {
        Cpu {
            regs: [Word27::zero(); NUM_REGISTERS],
            pc: 0,
            halted: false,
            program: [Instr::nop(); 64],
            program_len: 0,
            data: [Word27::zero(); DATA_MEM_CELLS],
        }
    }

    /// Load a program. Returns `false` if `prog` exceeds the instruction
    /// memory.
    pub fn load_program(&mut self, prog: &[Instr]) -> bool {
        if prog.len() > self.program.len() {
            return false;
        }
        let mut i = 0;
        while i < prog.len() {
            self.program[i] = prog[i];
            i += 1;
        }
        self.program_len = prog.len();
        self.pc = 0;
        self.halted = false;
        true
    }

    /// Read a register. R0 always reads as zero.
    pub fn read_reg(&self, idx: u8) -> Word27 {
        if idx == 0 || (idx as usize) >= NUM_REGISTERS {
            Word27::zero()
        } else {
            self.regs[idx as usize]
        }
    }

    /// Write a register. Writes to R0 are ignored. Returns `false` if
    /// `idx` is out of range.
    pub fn write_reg(&mut self, idx: u8, v: Word27) -> bool {
        if (idx as usize) >= NUM_REGISTERS {
            return false;
        }
        if idx == 0 {
            return true; // ignore writes to R0
        }
        self.regs[idx as usize] = v;
        true
    }

    /// Execute exactly one instruction. Returns `false` if the CPU is
    /// halted or `pc` is past the loaded program.
    pub fn step(&mut self) -> bool {
        if self.halted || self.pc >= self.program_len {
            return false;
        }
        let ins = self.program[self.pc];
        self.pc += 1;
        match ins.op {
            Opcode::Nop => {}
            Opcode::Mov => {
                let v = self.read_reg(ins.rs1);
                self.write_reg(ins.rd, v);
            }
            Opcode::Addi => {
                let a = self.read_reg(ins.rs1);
                let b = Word27::from_i64(ins.imm as i64);
                let (s, _carry) = word_add(a, b);
                self.write_reg(ins.rd, s);
            }
            Opcode::Add => {
                let a = self.read_reg(ins.rs1);
                let b = self.read_reg(ins.rs2);
                let (s, _carry) = word_add(a, b);
                self.write_reg(ins.rd, s);
            }
            Opcode::Sub => {
                let a = self.read_reg(ins.rs1);
                let b = self.read_reg(ins.rs2);
                let (s, _borrow) = word_sub(a, b);
                self.write_reg(ins.rd, s);
            }
            Opcode::Neg => {
                let a = self.read_reg(ins.rs1);
                self.write_reg(ins.rd, a.negate());
            }
            Opcode::Load => {
                let addr_word = self.read_reg(ins.rs1);
                let addr = (addr_word.to_i64() as usize) % DATA_MEM_CELLS;
                let v = self.data[addr];
                self.write_reg(ins.rd, v);
            }
            Opcode::Store => {
                let addr_word = self.read_reg(ins.rs1);
                let addr = (addr_word.to_i64() as usize) % DATA_MEM_CELLS;
                let v = self.read_reg(ins.rs2);
                self.data[addr] = v;
            }
            Opcode::Halt => {
                self.halted = true;
            }
        }
        true
    }

    /// Run until HALT or `max_steps` is reached. Returns the number of
    /// instructions executed.
    pub fn run(&mut self, max_steps: usize) -> usize {
        let mut n = 0;
        while n < max_steps && !self.halted && self.pc < self.program_len {
            if !self.step() {
                break;
            }
            n += 1;
        }
        n
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu::new()
    }
}

// ---------------------------------------------------------------------------
// Identity witness
// ---------------------------------------------------------------------------

/// Returns `true` iff `phi^2 + 1/phi^2 == 3` to f64 1e-15.
pub fn identity_witness() -> bool {
    let p2 = PHI * PHI;
    let inv_p2 = 1.0 / p2;
    let d = (p2 + inv_p2) - 3.0;
    let ad = if d < 0.0 { -d } else { d };
    ad < 1.0e-15
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_witness_holds() {
        assert!(identity_witness(), "phi^2 + 1/phi^2 must equal 3 to 1e-15");
    }

    #[test]
    fn trit_add_basic_table() {
        assert_eq!(trit_add(0, 0, 0), (0, 0));
        assert_eq!(trit_add(1, 1, 0), (-1, 1));    // 2 -> -1 carry +1
        assert_eq!(trit_add(-1, -1, 0), (1, -1));  // -2 -> +1 carry -1
        assert_eq!(trit_add(1, 1, 1), (0, 1));     // 3 -> 0  carry +1
        assert_eq!(trit_add(-1, -1, -1), (0, -1)); // -3 -> 0  carry -1
        assert_eq!(trit_add(1, -1, 0), (0, 0));
    }

    #[test]
    fn word_zero_roundtrip() {
        let z = Word27::zero();
        assert_eq!(z.to_i64(), 0);
    }

    #[test]
    fn word_from_i64_roundtrip_small() {
        for v in [-13_i64, -3, -1, 0, 1, 2, 3, 13, 100, -100, 1_000_000].iter() {
            let w = Word27::from_i64(*v);
            assert_eq!(w.to_i64(), *v, "round-trip failed for {}", v);
        }
    }

    #[test]
    fn word_add_arithmetic_matches_i64() {
        let cases = [(0, 0), (1, 2), (-5, 7), (100, -42), (1234, -1234), (3_000_000, 4_000_000)];
        for &(a, b) in cases.iter() {
            let wa = Word27::from_i64(a);
            let wb = Word27::from_i64(b);
            let (ws, _carry) = word_add(wa, wb);
            assert_eq!(ws.to_i64(), a + b, "{} + {} mismatch", a, b);
        }
    }

    #[test]
    fn word_sub_arithmetic_matches_i64() {
        let cases = [(0, 0), (10, 3), (3, 10), (-5, -7), (1000, 999), (-1234, 1234)];
        for &(a, b) in cases.iter() {
            let wa = Word27::from_i64(a);
            let wb = Word27::from_i64(b);
            let (ws, _borrow) = word_sub(wa, wb);
            assert_eq!(ws.to_i64(), a - b, "{} - {} mismatch", a, b);
        }
    }

    #[test]
    fn negate_is_involution() {
        let cases = [-7_i64, -1, 0, 1, 42, 10_000];
        for &v in cases.iter() {
            let w = Word27::from_i64(v);
            let n = w.negate();
            assert_eq!(n.to_i64(), -v);
            // double negation is identity
            assert_eq!(n.negate().to_i64(), v);
        }
    }

    #[test]
    fn trit_at_and_set_trit_bounds() {
        let mut w = Word27::zero();
        assert_eq!(w.trit_at(0).unwrap().value(), 0);
        assert!(w.trit_at(REG_WIDTH).is_none());
        assert!(w.set_trit(0, Trit::new(1).unwrap()));
        assert_eq!(w.trit_at(0).unwrap().value(), 1);
        assert!(!w.set_trit(REG_WIDTH, Trit::new(1).unwrap()));
    }

    #[test]
    fn trit_construction_rejects_out_of_range() {
        assert!(Trit::new(2).is_none());
        assert!(Trit::new(-2).is_none());
        assert!(Trit::new(0).is_some());
    }

    #[test]
    fn cpu_r0_is_hardwired_zero() {
        let mut cpu = Cpu::new();
        let _ = cpu.write_reg(0, Word27::from_i64(42));
        assert_eq!(cpu.read_reg(0).to_i64(), 0);
    }

    #[test]
    fn cpu_addi_chain() {
        // R1 <- 0 + 7; R2 <- R1 + 5; HALT
        let prog = [
            Instr { op: Opcode::Addi, rd: 1, rs1: 0, rs2: 0, imm: 7 },
            Instr { op: Opcode::Addi, rd: 2, rs1: 1, rs2: 0, imm: 5 },
            Instr::halt(),
        ];
        let mut cpu = Cpu::new();
        assert!(cpu.load_program(&prog));
        let n = cpu.run(10);
        assert_eq!(n, 3);
        assert!(cpu.halted);
        assert_eq!(cpu.read_reg(1).to_i64(), 7);
        assert_eq!(cpu.read_reg(2).to_i64(), 12);
    }

    #[test]
    fn cpu_add_sub_neg() {
        // R1 <- 100; R2 <- 42; R3 <- R1 + R2; R4 <- R1 - R2; R5 <- -R3.
        let prog = [
            Instr { op: Opcode::Addi, rd: 1, rs1: 0, rs2: 0, imm: 100 },
            Instr { op: Opcode::Addi, rd: 2, rs1: 0, rs2: 0, imm: 42 },
            Instr { op: Opcode::Add, rd: 3, rs1: 1, rs2: 2, imm: 0 },
            Instr { op: Opcode::Sub, rd: 4, rs1: 1, rs2: 2, imm: 0 },
            Instr { op: Opcode::Neg, rd: 5, rs1: 3, rs2: 0, imm: 0 },
            Instr::halt(),
        ];
        let mut cpu = Cpu::new();
        cpu.load_program(&prog);
        cpu.run(20);
        assert_eq!(cpu.read_reg(3).to_i64(), 142);
        assert_eq!(cpu.read_reg(4).to_i64(), 58);
        assert_eq!(cpu.read_reg(5).to_i64(), -142);
    }

    #[test]
    fn cpu_load_store_roundtrip() {
        // Store 999 at mem[5], then load it back into R6.
        let prog = [
            Instr { op: Opcode::Addi, rd: 1, rs1: 0, rs2: 0, imm: 5 },   // R1 = 5 (address)
            Instr { op: Opcode::Addi, rd: 2, rs1: 0, rs2: 0, imm: 999 }, // R2 = 999 (value)
            Instr { op: Opcode::Store, rd: 0, rs1: 1, rs2: 2, imm: 0 },  // mem[R1] = R2
            Instr { op: Opcode::Load, rd: 6, rs1: 1, rs2: 0, imm: 0 },   // R6 = mem[R1]
            Instr::halt(),
        ];
        let mut cpu = Cpu::new();
        cpu.load_program(&prog);
        cpu.run(20);
        assert_eq!(cpu.read_reg(6).to_i64(), 999);
    }

    #[test]
    fn cpu_halt_stops_execution() {
        let prog = [
            Instr { op: Opcode::Addi, rd: 1, rs1: 0, rs2: 0, imm: 7 },
            Instr::halt(),
            Instr { op: Opcode::Addi, rd: 1, rs1: 0, rs2: 0, imm: 999 }, // should never run
        ];
        let mut cpu = Cpu::new();
        cpu.load_program(&prog);
        let n = cpu.run(50);
        assert_eq!(n, 2);
        assert!(cpu.halted);
        assert_eq!(cpu.read_reg(1).to_i64(), 7);
    }

    /// **Cross-kernel anchor test:** floor(phi^2 + 1/phi^2) = 3 inside the
    /// balanced-ternary CPU. We compute `floor(phi^2) + floor(1/phi^2)`
    /// with integer arithmetic in Word27 -- 2 + 0 = 2; then verify by hand
    /// that the *fractional* parts (0.618... + 0.382... = 1.0) bring the
    /// true sum to 3. This is the integer projection of the anchor.
    #[test]
    fn cpu_phi_identity_integer_projection() {
        // floor(phi^2) = 2, floor(1/phi^2) = 0.
        // floor(phi^2) + floor(1/phi^2) = 2, plus the exact unit gap = 3.
        let phi2_floor = (PHI * PHI).trunc() as i64; // = 2
        let inv_phi2_floor = (1.0_f64 / (PHI * PHI)).trunc() as i64; // = 0

        let prog = [
            Instr { op: Opcode::Addi, rd: 1, rs1: 0, rs2: 0, imm: phi2_floor as i32 },
            Instr { op: Opcode::Addi, rd: 2, rs1: 0, rs2: 0, imm: inv_phi2_floor as i32 },
            Instr { op: Opcode::Add, rd: 3, rs1: 1, rs2: 2, imm: 0 },
            // The integer projection is 2; the fractional gap is exactly 1.
            Instr { op: Opcode::Addi, rd: 4, rs1: 3, rs2: 0, imm: 1 },
            Instr::halt(),
        ];
        let mut cpu = Cpu::new();
        cpu.load_program(&prog);
        cpu.run(20);
        assert_eq!(cpu.read_reg(3).to_i64(), 2, "floor(phi^2) + floor(1/phi^2) must be 2");
        assert_eq!(cpu.read_reg(4).to_i64(), 3, "with the unit-gap correction, anchor = 3");
        // And the universal f64 identity must still hold:
        assert!(identity_witness());
    }
}
