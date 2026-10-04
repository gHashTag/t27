//! AArch64 (A64) instruction encoder for the instruction subset t27b emits.
//!
//! Each function returns one 32-bit instruction word. `sf` selects the 64-bit
//! (X) form when true and the 32-bit (W) form when false. Register 31 means the
//! zero register (WZR/XZR) or SP depending on the instruction class, exactly as
//! in the architecture; the doc comment of each function says which.
//!
//! The words are checked against the system assembler in tests/encoder.rs.

pub type Reg = u8;

pub const ZR: Reg = 31;
pub const SP: Reg = 31;
pub const FP: Reg = 29;
pub const LR: Reg = 30;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cond {
    Eq = 0,
    Ne = 1,
    Hs = 2,
    Lo = 3,
    Mi = 4,
    Pl = 5,
    Vs = 6,
    Vc = 7,
    Hi = 8,
    Ls = 9,
    Ge = 10,
    Lt = 11,
    Gt = 12,
    Le = 13,
    Al = 14,
}

impl Cond {
    pub fn invert(self) -> Cond {
        use Cond::*;
        match self {
            Eq => Ne,
            Ne => Eq,
            Hs => Lo,
            Lo => Hs,
            Mi => Pl,
            Pl => Mi,
            Vs => Vc,
            Vc => Vs,
            Hi => Ls,
            Ls => Hi,
            Ge => Lt,
            Lt => Ge,
            Gt => Le,
            Le => Gt,
            Al => Al,
        }
    }

    pub fn name(self) -> &'static str {
        use Cond::*;
        match self {
            Eq => "eq",
            Ne => "ne",
            Hs => "hs",
            Lo => "lo",
            Mi => "mi",
            Pl => "pl",
            Vs => "vs",
            Vc => "vc",
            Hi => "hi",
            Ls => "ls",
            Ge => "ge",
            Lt => "lt",
            Gt => "gt",
            Le => "le",
            Al => "al",
        }
    }

    pub fn from_bits(b: u32) -> Cond {
        use Cond::*;
        [Eq, Ne, Hs, Lo, Mi, Pl, Vs, Vc, Hi, Ls, Ge, Lt, Gt, Le, Al, Al][(b & 15) as usize]
    }
}

/// Extend option of the add/sub (extended register) form.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ext {
    Uxtb = 0,
    Uxth = 1,
    Uxtw = 2,
    Uxtx = 3,
    Sxtb = 4,
    Sxth = 5,
    Sxtw = 6,
    Sxtx = 7,
}

/// Shift type of the shifted-register forms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shift {
    Lsl = 0,
    Lsr = 1,
    Asr = 2,
}

#[inline]
fn sfb(sf: bool) -> u32 {
    (sf as u32) << 31
}

#[inline]
fn r(x: Reg) -> u32 {
    (x as u32) & 31
}

// ------------------------------------------------------------ add / subtract

/// ADD/ADDS/SUB/SUBS (shifted register). `op`: 0 add, 1 sub. Reg 31 is ZR.
pub fn addsub_reg(sf: bool, sub: bool, setflags: bool, rd: Reg, rn: Reg, rm: Reg, sh: Shift, amt: u32) -> u32 {
    sfb(sf)
        | (sub as u32) << 30
        | (setflags as u32) << 29
        | 0x0B00_0000
        | (sh as u32) << 22
        | r(rm) << 16
        | (amt & 63) << 10
        | r(rn) << 5
        | r(rd)
}

pub fn add(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    addsub_reg(sf, false, false, rd, rn, rm, Shift::Lsl, 0)
}
pub fn adds(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    addsub_reg(sf, false, true, rd, rn, rm, Shift::Lsl, 0)
}
pub fn sub(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    addsub_reg(sf, true, false, rd, rn, rm, Shift::Lsl, 0)
}
pub fn subs(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    addsub_reg(sf, true, true, rd, rn, rm, Shift::Lsl, 0)
}
/// CMP rn, rm (SUBS ZR).
pub fn cmp(sf: bool, rn: Reg, rm: Reg) -> u32 {
    subs(sf, ZR, rn, rm)
}
/// CMP rn, rm, <sh> #amt.
pub fn cmp_shifted(sf: bool, rn: Reg, rm: Reg, sh: Shift, amt: u32) -> u32 {
    addsub_reg(sf, true, true, ZR, rn, rm, sh, amt)
}

/// ADD/ADDS/SUB/SUBS (extended register). Rn and (non-flag-setting) Rd 31 are
/// SP; Rd 31 with flags is ZR.
pub fn addsub_ext(sf: bool, sub: bool, setflags: bool, rd: Reg, rn: Reg, rm: Reg, ext: Ext, lsl: u32) -> u32 {
    sfb(sf)
        | (sub as u32) << 30
        | (setflags as u32) << 29
        | 0x0B20_0000
        | r(rm) << 16
        | (ext as u32) << 13
        | (lsl & 7) << 10
        | r(rn) << 5
        | r(rd)
}

/// CMP rn, rm, <ext>: compares rn with the extended low part of rm.
pub fn cmp_ext(sf: bool, rn: Reg, rm: Reg, ext: Ext) -> u32 {
    addsub_ext(sf, true, true, ZR, rn, rm, ext, 0)
}

/// ADD/ADDS/SUB/SUBS (immediate): imm12, optionally shifted left by 12.
/// Rn 31 is SP; Rd 31 is SP without flags, ZR with flags.
pub fn addsub_imm(sf: bool, sub: bool, setflags: bool, rd: Reg, rn: Reg, imm12: u32, lsl12: bool) -> u32 {
    debug_assert!(imm12 < 4096);
    sfb(sf)
        | (sub as u32) << 30
        | (setflags as u32) << 29
        | 0x1100_0000
        | (lsl12 as u32) << 22
        | (imm12 & 0xfff) << 10
        | r(rn) << 5
        | r(rd)
}

pub fn add_imm(sf: bool, rd: Reg, rn: Reg, imm12: u32) -> u32 {
    addsub_imm(sf, false, false, rd, rn, imm12, false)
}
pub fn sub_imm(sf: bool, rd: Reg, rn: Reg, imm12: u32) -> u32 {
    addsub_imm(sf, true, false, rd, rn, imm12, false)
}
pub fn cmp_imm(sf: bool, rn: Reg, imm12: u32) -> u32 {
    addsub_imm(sf, true, true, ZR, rn, imm12, false)
}
pub fn cmn_imm(sf: bool, rn: Reg, imm12: u32) -> u32 {
    addsub_imm(sf, false, true, ZR, rn, imm12, false)
}

/// MOV between a general register and SP (ADD #0).
pub fn mov_sp(rd: Reg, rn: Reg) -> u32 {
    add_imm(true, rd, rn, 0)
}

// ------------------------------------------------------------------- logical

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogOp {
    And = 0,
    Orr = 1,
    Eor = 2,
    Ands = 3,
}

/// AND/ORR/EOR/ANDS (shifted register); `not` gives BIC/ORN/EON/BICS. Reg 31 is ZR.
pub fn logic_reg(sf: bool, op: LogOp, not: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    sfb(sf) | (op as u32) << 29 | 0x0A00_0000 | (not as u32) << 21 | r(rm) << 16 | r(rn) << 5 | r(rd)
}

/// MOV rd, rm (ORR rd, ZR, rm). The W form zero-extends into the X register.
pub fn mov(sf: bool, rd: Reg, rm: Reg) -> u32 {
    logic_reg(sf, LogOp::Orr, false, rd, ZR, rm)
}

/// MVN rd, rm (ORN rd, ZR, rm).
pub fn mvn(sf: bool, rd: Reg, rm: Reg) -> u32 {
    logic_reg(sf, LogOp::Orr, true, rd, ZR, rm)
}

/// Encode a logical (bitmask) immediate. Returns (N, immr, imms).
pub fn bitmask_imm(value: u64, sf: bool) -> Option<(u32, u32, u32)> {
    let width = if sf { 64 } else { 32 };
    let v = if sf { value } else { value & 0xffff_ffff };
    if v == 0 || (sf && v == u64::MAX) || (!sf && v == 0xffff_ffff) {
        return None;
    }
    // Replicate a 32-bit value so the 64-bit search covers both forms.
    let v64 = if sf { v } else { v | (v << 32) };
    // Find the smallest element size whose replication gives v64.
    let mut size = 64u32;
    while size > 2 {
        let half = size / 2;
        let mask = (1u64 << half) - 1;
        if (v64 & mask) != ((v64 >> half) & mask) {
            break;
        }
        size = half;
    }
    if !sf && size == 64 {
        return None;
    }
    let mask = if size == 64 { u64::MAX } else { (1u64 << size) - 1 };
    let elem = v64 & mask;
    // elem must be a rotated run of ones.
    let ones = elem.count_ones();
    if ones == 0 || ones == size {
        return None;
    }
    // Find rotation r such that rotating elem right by r gives 0..01..1.
    let run = if ones == 64 { u64::MAX } else { (1u64 << ones) - 1 };
    let mut rot = None;
    for rr in 0..size {
        // rotate right by rr within `size` bits
        let rotated = if rr == 0 {
            elem
        } else {
            ((elem >> rr) | (elem << (size - rr))) & mask
        };
        if rotated == run {
            rot = Some(rr);
            break;
        }
    }
    let rr = rot?;
    // The instruction rotates the run RIGHT by immr; we found the right
    // rotation that normalises elem, so immr is the inverse rotation.
    let immr = (size - rr) % size;
    let n = (size == 64) as u32;
    let imms = ((!(size * 2 - 1)) & 0x3f) | (ones - 1);
    let _ = width;
    Some((n, immr, imms & 0x3f))
}

/// AND/ORR/EOR/ANDS (immediate). Rd 31 is SP for AND/ORR/EOR, ZR for ANDS.
pub fn logic_imm(sf: bool, op: LogOp, rd: Reg, rn: Reg, value: u64) -> Option<u32> {
    let (n, immr, imms) = bitmask_imm(value, sf)?;
    Some(sfb(sf) | (op as u32) << 29 | 0x1200_0000 | n << 22 | immr << 16 | imms << 10 | r(rn) << 5 | r(rd))
}

// ---------------------------------------------------------------- move wide

/// MOVZ rd, #imm16, LSL #(16*hw).
pub fn movz(sf: bool, rd: Reg, imm16: u32, hw: u32) -> u32 {
    sfb(sf) | 0x5280_0000 | (hw & 3) << 21 | (imm16 & 0xffff) << 5 | r(rd)
}
/// MOVN rd, #imm16, LSL #(16*hw).
pub fn movn(sf: bool, rd: Reg, imm16: u32, hw: u32) -> u32 {
    sfb(sf) | 0x1280_0000 | (hw & 3) << 21 | (imm16 & 0xffff) << 5 | r(rd)
}
/// MOVK rd, #imm16, LSL #(16*hw).
pub fn movk(sf: bool, rd: Reg, imm16: u32, hw: u32) -> u32 {
    sfb(sf) | 0x7280_0000 | (hw & 3) << 21 | (imm16 & 0xffff) << 5 | r(rd)
}

/// The shortest MOVZ/MOVN/MOVK (or ORR bitmask) sequence putting `value` in
/// rd. For W registers only the low 32 bits of `value` are used.
pub fn mov_imm(sf: bool, rd: Reg, value: u64, out: &mut Vec<u32>) {
    let v = if sf { value } else { value & 0xffff_ffff };
    let nhw = if sf { 4 } else { 2 };
    let chunk = |x: u64, i: u32| ((x >> (16 * i)) & 0xffff) as u32;
    let zeros = (0..nhw).filter(|&i| chunk(v, i) == 0).count();
    let ones = (0..nhw).filter(|&i| chunk(v, i) == 0xffff).count();
    if zeros == nhw as usize {
        out.push(movz(sf, rd, 0, 0));
        return;
    }
    let need_z = nhw as usize - zeros;
    let need_n = nhw as usize - ones;
    if need_z > 1 && need_n > 1 {
        if let Some(w) = logic_imm(sf, LogOp::Orr, rd, ZR, v) {
            out.push(w);
            return;
        }
    }
    if need_n < need_z {
        // MOVN: start from all ones.
        let mut first = true;
        for i in 0..nhw {
            let c = chunk(v, i);
            if c != 0xffff {
                if first {
                    out.push(movn(sf, rd, !c & 0xffff, i));
                    first = false;
                } else {
                    out.push(movk(sf, rd, c, i));
                }
            }
        }
        if first {
            out.push(movn(sf, rd, 0, 0));
        }
    } else {
        let mut first = true;
        for i in 0..nhw {
            let c = chunk(v, i);
            if c != 0 {
                if first {
                    out.push(movz(sf, rd, c, i));
                    first = false;
                } else {
                    out.push(movk(sf, rd, c, i));
                }
            }
        }
    }
}

// ------------------------------------------------- data processing, 2 and 3

fn dp2(sf: bool, opcode: u32, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    sfb(sf) | 0x1AC0_0000 | r(rm) << 16 | opcode << 10 | r(rn) << 5 | r(rd)
}
pub fn udiv(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    dp2(sf, 0b000010, rd, rn, rm)
}
pub fn sdiv(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    dp2(sf, 0b000011, rd, rn, rm)
}
pub fn lslv(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    dp2(sf, 0b001000, rd, rn, rm)
}
pub fn lsrv(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    dp2(sf, 0b001001, rd, rn, rm)
}
pub fn asrv(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    dp2(sf, 0b001010, rd, rn, rm)
}

/// MADD rd, rn, rm, ra (rd = ra + rn*rm).
pub fn madd(sf: bool, rd: Reg, rn: Reg, rm: Reg, ra: Reg) -> u32 {
    sfb(sf) | 0x1B00_0000 | r(rm) << 16 | r(ra) << 10 | r(rn) << 5 | r(rd)
}
/// MSUB rd, rn, rm, ra (rd = ra - rn*rm).
pub fn msub(sf: bool, rd: Reg, rn: Reg, rm: Reg, ra: Reg) -> u32 {
    sfb(sf) | 0x1B00_8000 | r(rm) << 16 | r(ra) << 10 | r(rn) << 5 | r(rd)
}
pub fn mul(sf: bool, rd: Reg, rn: Reg, rm: Reg) -> u32 {
    madd(sf, rd, rn, rm, ZR)
}
/// SMULL xd, wn, wm (64-bit product of signed 32-bit operands).
pub fn smull(rd: Reg, rn: Reg, rm: Reg) -> u32 {
    0x9B20_0000 | r(rm) << 16 | r(ZR) << 10 | r(rn) << 5 | r(rd)
}
/// UMULL xd, wn, wm.
pub fn umull(rd: Reg, rn: Reg, rm: Reg) -> u32 {
    0x9BA0_0000 | r(rm) << 16 | r(ZR) << 10 | r(rn) << 5 | r(rd)
}
/// SMULH xd, xn, xm (high 64 bits of the signed 128-bit product).
pub fn smulh(rd: Reg, rn: Reg, rm: Reg) -> u32 {
    0x9B40_0000 | r(rm) << 16 | r(ZR) << 10 | r(rn) << 5 | r(rd)
}
/// UMULH xd, xn, xm.
pub fn umulh(rd: Reg, rn: Reg, rm: Reg) -> u32 {
    0x9BC0_0000 | r(rm) << 16 | r(ZR) << 10 | r(rn) << 5 | r(rd)
}

// ----------------------------------------------------------------- bitfield

pub fn sbfm(sf: bool, rd: Reg, rn: Reg, immr: u32, imms: u32) -> u32 {
    sfb(sf) | 0x1300_0000 | (sf as u32) << 22 | (immr & 63) << 16 | (imms & 63) << 10 | r(rn) << 5 | r(rd)
}
pub fn ubfm(sf: bool, rd: Reg, rn: Reg, immr: u32, imms: u32) -> u32 {
    sfb(sf) | 0x5300_0000 | (sf as u32) << 22 | (immr & 63) << 16 | (imms & 63) << 10 | r(rn) << 5 | r(rd)
}
pub fn lsl_imm(sf: bool, rd: Reg, rn: Reg, sh: u32) -> u32 {
    let size = if sf { 64 } else { 32 };
    ubfm(sf, rd, rn, (size - sh) % size, size - 1 - sh)
}
pub fn lsr_imm(sf: bool, rd: Reg, rn: Reg, sh: u32) -> u32 {
    let size = if sf { 64 } else { 32 };
    ubfm(sf, rd, rn, sh, size - 1)
}
pub fn asr_imm(sf: bool, rd: Reg, rn: Reg, sh: u32) -> u32 {
    let size = if sf { 64 } else { 32 };
    sbfm(sf, rd, rn, sh, size - 1)
}
pub fn uxtb(rd: Reg, rn: Reg) -> u32 {
    ubfm(false, rd, rn, 0, 7)
}
pub fn uxth(rd: Reg, rn: Reg) -> u32 {
    ubfm(false, rd, rn, 0, 15)
}
pub fn sxtb(rd: Reg, rn: Reg) -> u32 {
    sbfm(false, rd, rn, 0, 7)
}
pub fn sxth(rd: Reg, rn: Reg) -> u32 {
    sbfm(false, rd, rn, 0, 15)
}
/// SXTW xd, wn.
pub fn sxtw(rd: Reg, rn: Reg) -> u32 {
    sbfm(true, rd, rn, 0, 31)
}

// --------------------------------------------------------- conditional select

/// CSINC rd, rn, rm, cond.
pub fn csinc(sf: bool, rd: Reg, rn: Reg, rm: Reg, cond: Cond) -> u32 {
    sfb(sf) | 0x1A80_0400 | r(rm) << 16 | (cond as u32) << 12 | r(rn) << 5 | r(rd)
}
/// CSET rd, cond (rd = cond ? 1 : 0).
pub fn cset(sf: bool, rd: Reg, cond: Cond) -> u32 {
    csinc(sf, rd, ZR, ZR, cond.invert())
}

// ----------------------------------------------------------------- branches

/// B with a word offset (target - pc) / 4.
pub fn b(off: i32) -> u32 {
    0x1400_0000 | (off as u32 & 0x03ff_ffff)
}
pub fn bl(off: i32) -> u32 {
    0x9400_0000 | (off as u32 & 0x03ff_ffff)
}
pub fn b_cond(cond: Cond, off: i32) -> u32 {
    0x5400_0000 | (off as u32 & 0x7ffff) << 5 | cond as u32
}
pub fn cbz(sf: bool, rt: Reg, off: i32) -> u32 {
    sfb(sf) | 0x3400_0000 | (off as u32 & 0x7ffff) << 5 | r(rt)
}
pub fn cbnz(sf: bool, rt: Reg, off: i32) -> u32 {
    sfb(sf) | 0x3500_0000 | (off as u32 & 0x7ffff) << 5 | r(rt)
}
pub fn ret() -> u32 {
    0xD65F_03C0
}
pub fn blr(rn: Reg) -> u32 {
    0xD63F_0000 | r(rn) << 5
}
pub fn brk(imm16: u32) -> u32 {
    0xD420_0000 | (imm16 & 0xffff) << 5
}
pub fn nop() -> u32 {
    0xD503_201F
}

// -------------------------------------------------------------- load/store

/// STR xt, [xn, #imm] with imm a multiple of 8 in 0..32760. Xn 31 is SP.
pub fn str_x(rt: Reg, rn: Reg, imm: u32) -> u32 {
    debug_assert!(imm % 8 == 0 && imm / 8 < 4096);
    0xF900_0000 | (imm / 8) << 10 | r(rn) << 5 | r(rt)
}
pub fn ldr_x(rt: Reg, rn: Reg, imm: u32) -> u32 {
    debug_assert!(imm % 8 == 0 && imm / 8 < 4096);
    0xF940_0000 | (imm / 8) << 10 | r(rn) << 5 | r(rt)
}
/// STUR xt, [xn, #simm9].
pub fn stur_x(rt: Reg, rn: Reg, simm: i32) -> u32 {
    debug_assert!((-256..256).contains(&simm));
    0xF800_0000 | (simm as u32 & 0x1ff) << 12 | r(rn) << 5 | r(rt)
}
pub fn ldur_x(rt: Reg, rn: Reg, simm: i32) -> u32 {
    debug_assert!((-256..256).contains(&simm));
    0xF840_0000 | (simm as u32 & 0x1ff) << 12 | r(rn) << 5 | r(rt)
}
/// STP xt1, xt2, [xn, #simm] (signed offset, multiple of 8 in -512..504).
pub fn stp_x(rt1: Reg, rt2: Reg, rn: Reg, simm: i32) -> u32 {
    debug_assert!(simm % 8 == 0 && (-512..=504).contains(&simm));
    0xA900_0000 | ((simm / 8) as u32 & 0x7f) << 15 | r(rt2) << 10 | r(rn) << 5 | r(rt1)
}
pub fn ldp_x(rt1: Reg, rt2: Reg, rn: Reg, simm: i32) -> u32 {
    debug_assert!(simm % 8 == 0 && (-512..=504).contains(&simm));
    0xA940_0000 | ((simm / 8) as u32 & 0x7f) << 15 | r(rt2) << 10 | r(rn) << 5 | r(rt1)
}
/// STP xt1, xt2, [xn, #simm]! (pre-index).
pub fn stp_x_pre(rt1: Reg, rt2: Reg, rn: Reg, simm: i32) -> u32 {
    0xA980_0000 | ((simm / 8) as u32 & 0x7f) << 15 | r(rt2) << 10 | r(rn) << 5 | r(rt1)
}
/// LDP xt1, xt2, [xn], #simm (post-index).
pub fn ldp_x_post(rt1: Reg, rt2: Reg, rn: Reg, simm: i32) -> u32 {
    0xA8C0_0000 | ((simm / 8) as u32 & 0x7f) << 15 | r(rt2) << 10 | r(rn) << 5 | r(rt1)
}

// -------------------------------------------------------------- disassembly

fn xr(sf: bool, n: u32, sp_ctx: bool) -> String {
    if n == 31 {
        if sp_ctx {
            return if sf { "sp".into() } else { "wsp".into() };
        }
        return if sf { "xzr".into() } else { "wzr".into() };
    }
    format!("{}{}", if sf { 'x' } else { 'w' }, n)
}

/// Disassemble one word of the subset this module encodes (for `t27b asm`).
/// Words outside the subset print as `.word`.
pub fn disasm(w: u32, pc: usize) -> String {
    let sf = w >> 31 == 1;
    let rd = w & 31;
    let rn = (w >> 5) & 31;
    let rm = (w >> 16) & 31;
    let tgt = |off: i64| format!("{:#x}", pc as i64 + off * 4);
    let sext = |v: u32, bits: u32| ((v << (32 - bits)) as i32 >> (32 - bits)) as i64;
    match w {
        0xD65F_03C0 => return "ret".into(),
        0xD503_201F => return "nop".into(),
        _ => {}
    }
    if w & 0xFFFF_FC1F == 0xD63F_0000 {
        return format!("blr x{}", rn);
    }
    if w & 0xFFE0_001F == 0xD420_0000 {
        return format!("brk #{:#x}", (w >> 5) & 0xffff);
    }
    if w & 0x7C00_0000 == 0x1400_0000 {
        let off = sext(w & 0x03ff_ffff, 26);
        return format!("{} {}", if w >> 31 == 1 { "bl" } else { "b" }, tgt(off));
    }
    if w & 0xFF00_0010 == 0x5400_0000 {
        let off = sext((w >> 5) & 0x7ffff, 19);
        return format!("b.{} {}", Cond::from_bits(w).name(), tgt(off));
    }
    if w & 0x7E00_0000 == 0x3400_0000 {
        let off = sext((w >> 5) & 0x7ffff, 19);
        let nz = (w >> 24) & 1 == 1;
        return format!("{} {}, {}", if nz { "cbnz" } else { "cbz" }, xr(sf, rd, false), tgt(off));
    }
    // add/sub immediate
    if w & 0x1F80_0000 == 0x1100_0000 {
        let sub = (w >> 30) & 1 == 1;
        let s = (w >> 29) & 1 == 1;
        let imm = (w >> 10) & 0xfff;
        let sh = if (w >> 22) & 1 == 1 { ", lsl #12" } else { "" };
        let name = match (sub, s) {
            (false, false) => "add",
            (false, true) => "adds",
            (true, false) => "sub",
            (true, true) => "subs",
        };
        if s && rd == 31 {
            return format!("{} {}, #{}{}", if sub { "cmp" } else { "cmn" }, xr(sf, rn, true), imm, sh);
        }
        if !s && !sub && imm == 0 && (rd == 31 || rn == 31) {
            return format!("mov {}, {}", xr(sf, rd, true), xr(sf, rn, true));
        }
        return format!("{} {}, {}, #{}{}", name, xr(sf, rd, !s), xr(sf, rn, true), imm, sh);
    }
    // add/sub shifted / extended register
    if w & 0x1F00_0000 == 0x0B00_0000 {
        let sub = (w >> 30) & 1 == 1;
        let s = (w >> 29) & 1 == 1;
        let name = match (sub, s) {
            (false, false) => "add",
            (false, true) => "adds",
            (true, false) => "sub",
            (true, true) => "subs",
        };
        if (w >> 21) & 1 == 1 {
            let opt = (w >> 13) & 7;
            let ext = ["uxtb", "uxth", "uxtw", "uxtx", "sxtb", "sxth", "sxtw", "sxtx"][opt as usize];
            let msf = sf && (opt & 3) == 3;
            if s && rd == 31 {
                return format!("{} {}, {}, {}", if sub { "cmp" } else { "cmn" }, xr(sf, rn, true), xr(msf, rm, false), ext);
            }
            return format!("{} {}, {}, {}, {}", name, xr(sf, rd, !s), xr(sf, rn, true), xr(msf, rm, false), ext);
        }
        let sht = ["lsl", "lsr", "asr", "ror"][((w >> 22) & 3) as usize];
        let amt = (w >> 10) & 63;
        let shs = if amt != 0 { format!(", {} #{}", sht, amt) } else { String::new() };
        if s && rd == 31 {
            return format!("{} {}, {}{}", if sub { "cmp" } else { "cmn" }, xr(sf, rn, false), xr(sf, rm, false), shs);
        }
        if sub && rn == 31 {
            return format!("{} {}, {}{}", if s { "negs" } else { "neg" }, xr(sf, rd, false), xr(sf, rm, false), shs);
        }
        return format!("{} {}, {}, {}{}", name, xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false), shs);
    }
    // logical shifted register
    if w & 0x1F00_0000 == 0x0A00_0000 {
        let opc = (w >> 29) & 3;
        let not = (w >> 21) & 1 == 1;
        let names = if not { ["bic", "orn", "eon", "bics"] } else { ["and", "orr", "eor", "ands"] };
        if opc == 1 && rn == 31 {
            return format!("{} {}, {}", if not { "mvn" } else { "mov" }, xr(sf, rd, false), xr(sf, rm, false));
        }
        if opc == 3 && rd == 31 && !not {
            return format!("tst {}, {}", xr(sf, rn, false), xr(sf, rm, false));
        }
        return format!("{} {}, {}, {}", names[opc as usize], xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false));
    }
    // logical immediate
    if w & 0x1F80_0000 == 0x1200_0000 {
        let opc = (w >> 29) & 3;
        let n = (w >> 22) & 1;
        let immr = (w >> 16) & 63;
        let imms = (w >> 10) & 63;
        let v = decode_bitmask(n, immr, imms, sf);
        let names = ["and", "orr", "eor", "ands"];
        if opc == 1 && rn == 31 {
            return format!("mov {}, #{:#x}", xr(sf, rd, true), v);
        }
        if opc == 3 && rd == 31 {
            return format!("tst {}, #{:#x}", xr(sf, rn, false), v);
        }
        return format!("{} {}, {}, #{:#x}", names[opc as usize], xr(sf, rd, opc != 3), xr(sf, rn, false), v);
    }
    // move wide
    if w & 0x1F80_0000 == 0x1280_0000 {
        let opc = (w >> 29) & 3;
        let hw = (w >> 21) & 3;
        let imm = (w >> 5) & 0xffff;
        let name = match opc {
            0 => "movn",
            2 => "movz",
            3 => "movk",
            _ => return format!(".word {:#010x}", w),
        };
        let sh = if hw != 0 { format!(", lsl #{}", hw * 16) } else { String::new() };
        return format!("{} {}, #{:#x}{}", name, xr(sf, rd, false), imm, sh);
    }
    // data processing 2 source
    if w & 0x7FE0_0000 == 0x1AC0_0000 {
        let op = (w >> 10) & 63;
        let name = match op {
            2 => "udiv",
            3 => "sdiv",
            8 => "lsl",
            9 => "lsr",
            10 => "asr",
            _ => return format!(".word {:#010x}", w),
        };
        return format!("{} {}, {}, {}", name, xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false));
    }
    // data processing 3 source
    if w & 0x1F00_0000 == 0x1B00_0000 {
        let op31 = (w >> 21) & 7;
        let o0 = (w >> 15) & 1;
        let ra = (w >> 10) & 31;
        return match (op31, o0) {
            (0, 0) if ra == 31 => format!("mul {}, {}, {}", xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false)),
            (0, 0) => format!("madd {}, {}, {}, {}", xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false), xr(sf, ra, false)),
            (0, 1) if ra == 31 => format!("mneg {}, {}, {}", xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false)),
            (0, 1) => format!("msub {}, {}, {}, {}", xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false), xr(sf, ra, false)),
            (1, 0) => format!("smull x{}, w{}, w{}", rd, rn, rm),
            (5, 0) => format!("umull x{}, w{}, w{}", rd, rn, rm),
            (2, 0) => format!("smulh x{}, x{}, x{}", rd, rn, rm),
            (6, 0) => format!("umulh x{}, x{}, x{}", rd, rn, rm),
            _ => format!(".word {:#010x}", w),
        };
    }
    // bitfield
    if w & 0x1F80_0000 == 0x1300_0000 {
        let opc = (w >> 29) & 3;
        let immr = (w >> 16) & 63;
        let imms = (w >> 10) & 63;
        let size = if sf { 64 } else { 32 };
        let d = xr(sf, rd, false);
        return match opc {
            0 if immr == 0 && imms == 7 => format!("sxtb {}, w{}", d, rn),
            0 if immr == 0 && imms == 15 => format!("sxth {}, w{}", d, rn),
            0 if immr == 0 && imms == 31 && sf => format!("sxtw {}, w{}", d, rn),
            0 if imms == size - 1 => format!("asr {}, {}, #{}", d, xr(sf, rn, false), immr),
            2 if immr == 0 && imms == 7 && !sf => format!("uxtb {}, w{}", d, rn),
            2 if immr == 0 && imms == 15 && !sf => format!("uxth {}, w{}", d, rn),
            2 if imms == size - 1 => format!("lsr {}, {}, #{}", d, xr(sf, rn, false), immr),
            2 if imms + 1 == immr => format!("lsl {}, {}, #{}", d, xr(sf, rn, false), size - 1 - imms),
            0 => format!("sbfm {}, {}, #{}, #{}", d, xr(sf, rn, false), immr, imms),
            2 => format!("ubfm {}, {}, #{}, #{}", d, xr(sf, rn, false), immr, imms),
            _ => format!(".word {:#010x}", w),
        };
    }
    // conditional select
    if w & 0x1FE0_0C00 == 0x1A80_0400 {
        let cond = Cond::from_bits(w >> 12);
        if rn == 31 && rm == 31 {
            return format!("cset {}, {}", xr(sf, rd, false), cond.invert().name());
        }
        return format!("csinc {}, {}, {}, {}", xr(sf, rd, false), xr(sf, rn, false), xr(sf, rm, false), cond.name());
    }
    // load/store
    if w & 0xFFC0_0000 == 0xF900_0000 || w & 0xFFC0_0000 == 0xF940_0000 {
        let ld = (w >> 22) & 1 == 1;
        let imm = ((w >> 10) & 0xfff) * 8;
        return format!("{} x{}, [{}, #{}]", if ld { "ldr" } else { "str" }, rd, xr(true, rn, true), imm);
    }
    if w & 0xFFE0_0C00 == 0xF800_0000 || w & 0xFFE0_0C00 == 0xF840_0000 {
        let ld = (w >> 22) & 1 == 1;
        let imm = sext((w >> 12) & 0x1ff, 9);
        return format!("{} x{}, [{}, #{}]", if ld { "ldur" } else { "stur" }, rd, xr(true, rn, true), imm);
    }
    if w & 0xFE00_0000 == 0xA800_0000 {
        let ld = (w >> 22) & 1 == 1;
        let mode = (w >> 23) & 3;
        let imm = sext((w >> 15) & 0x7f, 7) * 8;
        let rt2 = (w >> 10) & 31;
        let base = xr(true, rn, true);
        let name = if ld { "ldp" } else { "stp" };
        return match mode {
            1 => format!("{} x{}, x{}, [{}], #{}", name, rd, rt2, base, imm),
            2 => format!("{} x{}, x{}, [{}, #{}]", name, rd, rt2, base, imm),
            3 => format!("{} x{}, x{}, [{}, #{}]!", name, rd, rt2, base, imm),
            _ => format!(".word {:#010x}", w),
        };
    }
    format!(".word {:#010x}", w)
}

/// Decode (N, immr, imms) back to the bitmask value (for disassembly/tests).
pub fn decode_bitmask(n: u32, immr: u32, imms: u32, sf: bool) -> u64 {
    let combined = (n << 6) | (!imms & 0x3f);
    let len = 31 - combined.leading_zeros();
    if len == 0 || len > 6 {
        return 0;
    }
    let size = 1u32 << len;
    let levels = size - 1;
    let s = imms & levels;
    let rr = immr & levels;
    let ones = s + 1;
    let mask = if size == 64 { u64::MAX } else { (1u64 << size) - 1 };
    let mut elem = if ones == 64 { u64::MAX } else { (1u64 << ones) - 1 };
    if rr != 0 {
        elem = ((elem >> rr) | (elem << (size - rr))) & mask;
    }
    let mut v = 0u64;
    let mut i = 0;
    while i < 64 {
        v |= elem << i;
        i += size;
    }
    if sf {
        v
    } else {
        v & 0xffff_ffff
    }
}
