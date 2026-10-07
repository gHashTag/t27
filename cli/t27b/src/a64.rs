//! AArch64 (A64) instruction encoder for the instruction subset t27b emits.
//!
//! Each function returns one 32-bit instruction word. `sf` selects the 64-bit
//! (X) form when true and the 32-bit (W) form when false. Register 31 means the
//! zero register (WZR/XZR) or SP depending on the instruction class, exactly as
//! in the architecture; the doc comment of each function says which.
//!
//! The words are checked against the system assembler in tests/encoder.rs.
//! The encoders are generated: specs/tri/t27b/a64.t27 -> `t27c gen-rust` ->
//! gen/rust/tri/t27b/a64.rs (#7531, #7549). This file keeps what gen-rust
//! cannot express yet: Cond's methods, the tuple and Vec adapters
//! `bitmask_imm` and `mov_imm`, and the disassembler.

#[path = "../../../gen/rust/tri/t27b/a64.rs"]
#[allow(dead_code, unused_parens, unexpected_cfgs)]
mod enc;
pub use enc::branch as b;
pub use enc::*;

pub type Reg = u8;

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

/// Encode a logical (bitmask) immediate: (N, immr, imms), split from
/// enc::bitmask_field's packed field.
pub fn bitmask_imm(value: u64, sf: bool) -> Option<(u32, u32, u32)> {
    enc::bitmask_field(value, sf).map(|f| (f >> 12, (f >> 6) & 63, f & 63))
}

/// The words of enc::mov_imm_at, pushed into `out`.
pub fn mov_imm(sf: bool, rd: Reg, value: u64, out: &mut Vec<u32>) {
    out.extend((0..enc::mov_imm_len(sf, value)).map(|k| enc::mov_imm_at(sf, rd, value, k)));
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
    if let Some(t) = disasm_fp(w) {
        return t;
    }
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
    if w & 0x9F00_0000 == 0x1000_0000 || w & 0x9F00_0000 == 0x9000_0000 {
        let imm = sext(((w >> 5) & 0x7ffff) << 2 | (w >> 29) & 3, 21);
        if w >> 31 == 1 {
            return format!("adrp x{}, {:#x}", rd, (pc as i64 & !0xfff) + imm * 4096);
        }
        return format!("adr x{}, {:#x}", rd, pc as i64 + imm);
    }
    // sized load/store: unsigned offset, unscaled, post-index, register
    if w & 0x3B00_0000 == 0x3900_0000 || w & 0x3B00_0000 == 0x3800_0000 {
        let size = w >> 30;
        let opc = (w >> 22) & 3;
        let uoff = w & 0x0100_0000 != 0;
        if size == 3 && opc >= 2 || size == 2 && opc == 3 {
            return format!(".word {:#010x}", w);
        }
        let mode = (w >> 10) & 3;
        let regoff = !uoff && (w >> 21) & 1 == 1 && mode == 2;
        if !uoff && !regoff && ((w >> 21) & 1 == 1 || mode == 2) {
            return format!(".word {:#010x}", w);
        }
        let unscaled = !uoff && !regoff && mode == 0;
        let name = format!(
            "{}{}{}{}",
            if opc == ST { "st" } else { "ld" },
            if unscaled { "ur" } else { "r" },
            if opc >= LDS64 { "s" } else { "" },
            match (size, opc) {
                (0, _) => "b",
                (1, _) => "h",
                (2, LDS64) => "w",
                _ => "",
            }
        );
        let t = xr(size == 3 || opc == LDS64, rd, false);
        let base = xr(true, rn, true);
        if uoff {
            return format!("{} {}, [{}, #{}]", name, t, base, ((w >> 10) & 0xfff) << size);
        }
        if regoff {
            let sh = if (w >> 12) & 1 == 1 && size > 0 { format!(", lsl #{}", size) } else { String::new() };
            return format!("{} {}, [{}, x{}{}]", name, t, base, rm, sh);
        }
        let imm = sext((w >> 12) & 0x1ff, 9);
        return match mode {
            0 => format!("{} {}, [{}, #{}]", name, t, base, imm),
            1 => format!("{} {}, [{}], #{}", name, t, base, imm),
            _ => format!("{} {}, [{}, #{}]!", name, t, base, imm),
        };
    }
    format!(".word {:#010x}", w)
}

/// The floating-point forms `disasm` knows, or None.
fn disasm_fp(w: u32) -> Option<String> {
    let rd = w & 31;
    let rn = (w >> 5) & 31;
    let rm = (w >> 16) & 31;
    let g = |sf: bool, n: u32| xr(sf, n, false);
    let sf = w >> 31 == 1;
    let opcode = (w >> 10) & 0x3f;
    // Bit 22 picks double (1) or single (0) in the forms matched through `v`.
    let p = if (w >> 22) & 1 == 1 { 'd' } else { 's' };
    let v = w | 0x0040_0000;
    Some(if w & 0xFFFF_FC00 == 0x1E22_C000 {
        format!("fcvt d{}, s{}", rd, rn)
    } else if w & 0xFFFF_FC00 == 0x1E62_4000 {
        format!("fcvt s{}, d{}", rd, rn)
    } else if w & 0xFFFF_FC00 == 0x1E27_0000 {
        format!("fmov s{}, {}", rd, g(false, rn))
    } else if w & 0xFFFF_FC00 == 0x1E26_0000 {
        format!("fmov w{}, s{}", rd, rn)
    } else if v & 0xFFE0_0C00 == 0x1E60_0800 && matches!(opcode, 0x02 | 0x06 | 0x0A | 0x0E) {
        // FP data processing, 2 source
        let name = match opcode {
            0x02 => "fmul",
            0x06 => "fdiv",
            0x0A => "fadd",
            _ => "fsub",
        };
        format!("{} {p}{}, {p}{}, {p}{}", name, rd, rn, rm)
    } else if v & 0xFFE0_FC1F == 0x1E60_2000 {
        format!("fcmp {p}{}, {p}{}", rn, rm)
    } else if v & 0xFFFF_FC00 == 0x1E61_4000 {
        format!("fneg {p}{}, {p}{}", rd, rn)
    } else if v & 0xFFFF_FC00 == 0x1E61_C000 {
        format!("fsqrt {p}{}, {p}{}", rd, rn)
    } else if w & 0xFFFF_FC00 == 0x9E67_0000 {
        format!("fmov d{}, {}", rd, g(true, rn))
    } else if w & 0xFFFF_FC00 == 0x9E66_0000 {
        format!("fmov x{}, d{}", rd, rn)
    } else if v & 0x7FFF_FC00 == 0x1E62_0000 {
        format!("scvtf {p}{}, {}", rd, g(sf, rn))
    } else if v & 0x7FFF_FC00 == 0x1E63_0000 {
        format!("ucvtf {p}{}, {}", rd, g(sf, rn))
    } else if v & 0x7FFF_FC00 == 0x1E78_0000 {
        format!("fcvtzs {}, {p}{}", g(sf, rd), rn)
    } else if v & 0x7FFF_FC00 == 0x1E79_0000 {
        format!("fcvtzu {}, {p}{}", g(sf, rd), rn)
    } else if w & 0xFF80_0000 == 0xFD00_0000 {
        let ld = (w >> 22) & 1 == 1;
        let imm = ((w >> 10) & 0xfff) * 8;
        format!("{} d{}, [{}, #{}]", if ld { "ldr" } else { "str" }, rd, xr(true, rn, true), imm)
    } else if w & 0xFFC0_0000 == 0x6D40_0000 {
        let imm = ((((w >> 15) & 0x7f) << 25) as i32) >> 25;
        format!("ldp d{}, d{}, [{}, #{}]", rd, (w >> 10) & 31, xr(true, rn, true), imm * 8)
    } else {
        return None;
    })
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
