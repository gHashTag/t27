//! Randomized differential test: generated AArch64 code (JIT) against the
//! reference interpreter (`eval`), over seeded random IR programs.
//!
//! Every comparison checks the returned value (and that it is in canonical
//! register form) or, for a trap, the exact trap site and, for `assert_eq`,
//! both compared values. Programs are built directly as IR so they reach shapes
//! the front-end rarely produces: expression trees deep enough to spill the
//! temp stack, calls with live temps, eight-argument calls, narrow types with
//! garbage in the upper half of argument registers, every division and shift
//! corner, and loops with `break` / `continue`. Memory: frame slots of every
//! size up to a multi-page aggregate area, read-only data blobs, loads and
//! stores of every scalar type at constant, masked and bounds-checked
//! indices, copies short and long, and pointer parameters into the caller's
//! frame (the hidden-pointer ABI for aggregates).
//!
//! Knobs (environment variables):
//!   T27B_DIFF_CASES  programs per overflow mode (default 2500)
//!   T27B_DIFF_SEED   base seed (default 0x7427)
//!   T27B_DIFF_TRACE  print each case seed before running it (to find a crash)
//!
//! The JIT runs only on arm64 macOS and arm64 Linux, so this file is empty
//! elsewhere.
#![cfg(all(target_arch = "aarch64", any(target_os = "macos", target_os = "linux")))]

use t27b::codegen::{self, canon, TrapStyle};
use t27b::eval::{Interp, Stop};
use t27b::ir::*;
use t27b::jit::Jit;
use t27b::{front, lower};

// ------------------------------------------------------------------ rng

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03);
        if r.0 == 0 {
            r.0 = 1;
        }
        r.next();
        r
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, pct: usize) -> bool {
        self.below(100) < pct
    }
    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len())]
    }
}

const ALL: [Ty; 9] = [
    Ty::Bool,
    Ty::U8,
    Ty::U16,
    Ty::U32,
    Ty::U64,
    Ty::I8,
    Ty::I16,
    Ty::I32,
    Ty::I64,
];

/// Odd widths (`u1`, `i21`): canonical like u8/u16 in a W register, with
/// bitfield extracts where those have byte and halfword extends. The edge-
/// value tests run them next to `Ty::INTS`; the random generator does not.
const ODD: [Ty; 12] = [
    Ty::UN(1),
    Ty::IN(1),
    Ty::IN(2),
    Ty::UN(4),
    Ty::IN(5),
    Ty::UN(7),
    Ty::UN(17),
    Ty::IN(17),
    Ty::UN(21),
    Ty::IN(21),
    Ty::UN(31),
    Ty::IN(31),
];

/// A value of `ty`, biased towards the edges of its range.
fn value(rng: &mut Rng, ty: Ty) -> i128 {
    if ty == Ty::Bool {
        return rng.below(2) as i128;
    }
    let bits = ty.bits() as i128;
    match rng.below(16) {
        0 => 0,
        1 => 1,
        2 => ty.max(),
        3 => ty.min(),
        4 => ty.wrap(-1),
        5 | 6 | 7 => rng.below(16) as i128,
        8 => ty.wrap(bits - 2 + rng.below(3) as i128),
        9 => ty.wrap(ty.min() + 1),
        10 => ty.wrap(-(rng.below(16) as i128)),
        11 => ty.wrap(ty.max() - 1),
        _ => {
            let sh = rng.below(64) as u32;
            ty.wrap((rng.next() >> sh) as i128)
        }
    }
}

fn konst(ty: Ty, v: i128) -> Expr {
    debug_assert!(ty.fits(v));
    Expr { ty, kind: ExprKind::Const(v) }
}

fn var(ty: Ty, id: usize) -> Expr {
    Expr { ty, kind: ExprKind::Var(id as VarId) }
}

fn arith(ty: Ty, op: ArithOp, lhs: Expr, rhs: Expr, site: SiteId) -> Expr {
    Expr {
        ty,
        kind: ExprKind::Arith { op, lhs: Box::new(lhs), rhs: Box::new(rhs), site },
    }
}

fn cmp(op: CmpOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr {
        ty: Ty::Bool,
        kind: ExprKind::Cmp { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
    }
}

const CMPS: [CmpOp; 6] = [CmpOp::Eq, CmpOp::Ne, CmpOp::Lt, CmpOp::Le, CmpOp::Gt, CmpOp::Ge];

fn offset(base: Expr, idx: Expr, scale: u32) -> Expr {
    Expr {
        ty: Ty::Ptr,
        kind: ExprKind::Offset { base: Box::new(base), idx: Box::new(idx), scale },
    }
}

/// `base + by` bytes (no node at all for 0).
fn ptr_add(base: Expr, by: u32) -> Expr {
    if by == 0 {
        base
    } else {
        offset(base, konst(Ty::U64, by as i128), 1)
    }
}

// ------------------------------------------------------------ generator

#[derive(Clone)]
struct Sig {
    params: Vec<Ty>,
    ret: Option<Ty>,
    /// Bytes behind the `Ty::Ptr` parameter, if there is one (at most one, so
    /// the callee never sees two names for the same memory).
    need: u32,
}

/// Memory the generated code may address: a frame slot, the caller's memory
/// behind a pointer parameter, a pointer local into either, or a data blob.
#[derive(Clone)]
struct Region {
    /// Side-effect-free Ptr expression for the first byte of the region.
    base: Expr,
    /// The underlying object: slot k, `OBJ_PARAM`, or `OBJ_DATA + k`.
    obj: usize,
    /// Offset of `base` within that object.
    start: u32,
    size: u32,
    writable: bool,
    /// Holds only 0/1 bytes: loaded and stored as Bool, never copied, so a
    /// Bool load never meets another byte value.
    boolean: bool,
}

const OBJ_PARAM: usize = 1000;
const OBJ_DATA: usize = 2000;
const SCALES: [u32; 9] = [1, 2, 3, 4, 5, 8, 12, 16, 24];

struct Gen<'r> {
    rng: &'r mut Rng,
    mode: OverflowMode,
    sites: Vec<Site>,
    sigs: Vec<Sig>,
    /// Calls may target functions 0..cur only (no recursion).
    cur: usize,
    vars: Vec<Var>,
    visible: Vec<bool>,
    locked: Vec<bool>,
    loops: usize,
    nodes: usize,
    calls: usize,
    line: u32,
    /// Sizes of the program's data blobs.
    data_sizes: Vec<u32>,
    slots: Vec<SlotInfo>,
    regions: Vec<Region>,
    /// Nesting of loads inside index expressions, kept shallow.
    mem_depth: u32,
    /// Inside a `Seq` (whose statements are not nested again).
    in_seq: bool,
}

impl<'r> Gen<'r> {
    fn site(&mut self, kind: TrapKind, ty: Ty) -> SiteId {
        self.line += 1;
        self.sites.push(Site { kind, line: self.line, what: String::new(), ty });
        (self.sites.len() - 1) as SiteId
    }

    fn new_var(&mut self, ty: Ty) -> usize {
        self.vars.push(Var { name: format!("v{}", self.vars.len()), ty });
        self.visible.push(false);
        self.locked.push(false);
        self.vars.len() - 1
    }

    fn leaf(&mut self, ty: Ty) -> Expr {
        if self.rng.chance(10) {
            if let Some(e) = self.load(ty) {
                return e;
            }
        }
        let same: Vec<usize> =
            (0..self.vars.len()).filter(|&i| self.visible[i] && self.vars[i].ty == ty).collect();
        if !same.is_empty() && self.rng.chance(65) {
            let v = self.rng.pick(&same);
            return var(ty, v);
        }
        if ty.is_int() && self.rng.chance(30) {
            let narrow: Vec<usize> = (0..self.vars.len())
                .filter(|&i| {
                    self.visible[i] && self.vars[i].ty != ty && ty.can_widen_from(self.vars[i].ty)
                })
                .collect();
            if !narrow.is_empty() {
                let v = self.rng.pick(&narrow);
                let from = self.vars[v].ty;
                return Expr { ty, kind: ExprKind::Widen(Box::new(var(from, v))) };
            }
        }
        let v = value(self.rng, ty);
        konst(ty, v)
    }

    fn expr(&mut self, ty: Ty, depth: u32) -> Expr {
        self.nodes += 1;
        if depth == 0 || self.nodes > 500 || self.rng.chance(18) {
            return self.leaf(ty);
        }
        if ty == Ty::Bool {
            return self.bool_expr(depth);
        }
        let r = self.rng.below(100);
        if r < 58 {
            return self.arith(ty, depth);
        }
        if r < 68 {
            if let Some(e) = self.call_expr(ty, depth) {
                return e;
            }
            return self.arith(ty, depth);
        }
        if r < 76 {
            let from: Vec<Ty> =
                Ty::INTS.iter().copied().filter(|&t| t != ty && ty.can_widen_from(t)).collect();
            if from.is_empty() {
                return self.arith(ty, depth);
            }
            let t = self.rng.pick(&from);
            let inner = self.expr(t, depth - 1);
            return Expr { ty, kind: ExprKind::Widen(Box::new(inner)) };
        }
        if r < 81 {
            // `x as T` from any other type: a bool widens to 0 / 1; an
            // integer either truncates (site 0) or traps when out of range.
            if self.rng.chance(15) {
                let inner = self.expr(Ty::Bool, depth - 1);
                return Expr { ty, kind: ExprKind::Widen(Box::new(inner)) };
            }
            let from: Vec<Ty> = Ty::INTS.iter().copied().filter(|&t| t != ty).collect();
            let t = self.rng.pick(&from);
            let inner = self.expr(t, depth - 1);
            let site = if self.rng.chance(50) { 0 } else { self.site(TrapKind::Cast, ty) };
            return Expr { ty, kind: ExprKind::Cast { arg: Box::new(inner), site } };
        }
        if r < 86 {
            let inner = self.expr(ty, depth - 1);
            return Expr { ty, kind: ExprKind::BitNot(Box::new(inner)) };
        }
        if r < 91 && ty.signed() {
            // Unary minus lowers to `0 - x`.
            let inner = self.expr(ty, depth - 1);
            let (op, site) = if self.mode == OverflowMode::Trap {
                (ArithOp::Sub, self.site(TrapKind::Overflow, ty))
            } else {
                (ArithOp::SubW, 0)
            };
            return arith(ty, op, konst(ty, 0), inner, site);
        }
        if r < 94 {
            if let Some(e) = self.seq(ty, depth) {
                return e;
            }
        }
        if r < 96 && self.rng.chance(50) {
            // `if (c) a else b` as a value: only the taken arm may trap.
            let cond = self.expr(Ty::Bool, depth - 1);
            let then = self.expr(ty, depth - 1);
            let els = self.expr(ty, depth - 1);
            return Expr {
                ty,
                kind: ExprKind::Select { cond: Box::new(cond), then: Box::new(then), els: Box::new(els) },
            };
        }
        if r < 96 {
            return self.chain(ty);
        }
        self.leaf(ty)
    }

    /// Stores and copies run in the middle of an expression, as lowering
    /// builds a struct temporary: often with temps live around them.
    fn seq(&mut self, ty: Ty, depth: u32) -> Option<Expr> {
        if self.in_seq || self.mem_depth > 0 {
            return None;
        }
        self.in_seq = true;
        let mut stmts = Vec::new();
        for _ in 0..1 + self.rng.below(2) {
            let s = if self.rng.chance(60) { self.store() } else { self.copy() };
            stmts.extend(s);
        }
        let value = self.expr(ty, depth - 1);
        self.in_seq = false;
        if stmts.is_empty() {
            return None;
        }
        Some(Expr { ty, kind: ExprKind::Seq { stmts, value: Box::new(value) } })
    }

    /// A right-leaning chain whose left operands are all computed values, so
    /// evaluating it needs one temp per level: pushes past the seven temp
    /// registers into frame slots. The innermost operand is often a call, so
    /// every live temp must survive it.
    fn chain(&mut self, ty: Ty) -> Expr {
        let n = 6 + self.rng.below(9);
        let mut e = if self.rng.chance(50) {
            match self.call_expr(ty, 2) {
                Some(c) => c,
                None => self.leaf(ty),
            }
        } else {
            self.leaf(ty)
        };
        for _ in 0..n {
            let a = self.leaf(ty);
            let b = self.leaf(ty);
            let inner_op = self.rng.pick(&[ArithOp::AddW, ArithOp::Xor, ArithOp::MulW, ArithOp::SubW]);
            let left = arith(ty, inner_op, a, b, 0);
            let op = self.rng.pick(&[
                ArithOp::AddW,
                ArithOp::SubW,
                ArithOp::Xor,
                ArithOp::Or,
                ArithOp::And,
                ArithOp::MulW,
            ]);
            e = arith(ty, op, left, e, 0);
            self.nodes += 3;
        }
        e
    }

    fn arith(&mut self, ty: Ty, depth: u32) -> Expr {
        // In trap mode about a third of the operators are the checked ones;
        // the rest are the explicit wrapping and bitwise ones, so that most
        // runs get far enough to exercise the code behind the first check.
        let trap = self.mode == OverflowMode::Trap;
        let op = if trap && self.rng.chance(35) {
            self.rng.pick(&[
                ArithOp::Add,
                ArithOp::Sub,
                ArithOp::Mul,
                ArithOp::Div,
                ArithOp::Shl,
                ArithOp::Shr,
            ])
        } else if trap {
            self.rng.pick(&[
                ArithOp::AddW,
                ArithOp::SubW,
                ArithOp::MulW,
                ArithOp::Rem,
                ArithOp::And,
                ArithOp::Or,
                ArithOp::Xor,
            ])
        } else {
            self.rng.pick(&[
                ArithOp::AddW,
                ArithOp::SubW,
                ArithOp::MulW,
                ArithOp::DivW,
                ArithOp::Rem,
                ArithOp::And,
                ArithOp::Or,
                ArithOp::Xor,
                ArithOp::ShlW,
                ArithOp::ShrW,
            ])
        };
        let bits = ty.bits() as i128;
        if op.is_shift() {
            let lhs = self.expr(ty, depth - 1);
            let left = matches!(op, ArithOp::Shl | ArithOp::ShlW);
            if self.rng.chance(35) {
                // Constant amounts are always lowered to the wrapping form.
                let c = self.rng.below(bits as usize) as i128;
                let wop = if left { ArithOp::ShlW } else { ArithOp::ShrW };
                return arith(ty, wop, lhs, konst(Ty::U32, c), 0);
            }
            let at = self.rng.pick(&Ty::INTS);
            let mut amt = self.expr(at, depth - 1);
            if self.rng.chance(80) {
                amt = arith(at, ArithOp::And, amt, konst(at, bits - 1), 0);
            }
            let site = if matches!(op, ArithOp::Shl | ArithOp::Shr) {
                self.site(TrapKind::ShiftRange, ty)
            } else {
                0
            };
            return arith(ty, op, lhs, amt, site);
        }
        let mut lhs = self.expr(ty, depth - 1);
        let mut rhs = self.expr(ty, depth - 1);
        if matches!(op, ArithOp::Add | ArithOp::Sub | ArithOp::Mul) && self.rng.chance(60) {
            // Tame operands: (e & (max >> 3)) op c with c < 8 rarely overflows.
            lhs = arith(ty, ArithOp::And, lhs, konst(ty, ty.max() >> 3), 0);
            rhs = konst(ty, self.rng.below(8) as i128);
        }
        if matches!(op, ArithOp::Div | ArithOp::DivW | ArithOp::Rem) && self.rng.chance(85) {
            rhs = arith(ty, ArithOp::Or, rhs, konst(ty, 1), 0);
        }
        let site = match op {
            ArithOp::Add | ArithOp::Sub | ArithOp::Mul => self.site(TrapKind::Overflow, ty),
            ArithOp::Div => {
                let s = self.site(TrapKind::DivZero, ty);
                self.site(TrapKind::Overflow, ty);
                s
            }
            ArithOp::DivW | ArithOp::Rem => self.site(TrapKind::DivZero, ty),
            _ => 0,
        };
        arith(ty, op, lhs, rhs, site)
    }

    fn bool_expr(&mut self, depth: u32) -> Expr {
        if self.rng.chance(6) {
            if let Some(e) = self.load(Ty::Bool) {
                return e;
            }
        }
        let r = self.rng.below(100);
        if r < 45 {
            let t = self.rng.pick(&Ty::INTS);
            let op = self.rng.pick(&CMPS);
            let a = self.expr(t, depth - 1);
            let b = self.expr(t, depth - 1);
            return cmp(op, a, b);
        }
        if r < 52 {
            let op = self.rng.pick(&[CmpOp::Eq, CmpOp::Ne]);
            let a = self.expr(Ty::Bool, depth - 1);
            let b = self.expr(Ty::Bool, depth - 1);
            return cmp(op, a, b);
        }
        if r < 74 {
            let a = Box::new(self.expr(Ty::Bool, depth - 1));
            let b = Box::new(self.expr(Ty::Bool, depth - 1));
            let kind = if r < 63 { ExprKind::And(a, b) } else { ExprKind::Or(a, b) };
            return Expr { ty: Ty::Bool, kind };
        }
        if r < 82 {
            let a = self.expr(Ty::Bool, depth - 1);
            return Expr { ty: Ty::Bool, kind: ExprKind::Not(Box::new(a)) };
        }
        if r < 89 {
            let op = self.rng.pick(&[ArithOp::And, ArithOp::Or, ArithOp::Xor]);
            let a = self.expr(Ty::Bool, depth - 1);
            let b = self.expr(Ty::Bool, depth - 1);
            return arith(Ty::Bool, op, a, b, 0);
        }
        if r < 95 {
            if let Some(e) = self.call_expr(Ty::Bool, depth) {
                return e;
            }
        }
        self.leaf(Ty::Bool)
    }

    /// Writable regions of at least `need` bytes that may be lent to a callee.
    fn lendable(&self, need: u32) -> Vec<usize> {
        (0..self.regions.len())
            .filter(|&i| {
                let r = &self.regions[i];
                r.writable && !r.boolean && r.size >= need
            })
            .collect()
    }

    /// Arguments for a call to `f`; None when it takes a pointer and no
    /// region here is large enough to lend it.
    fn args(&mut self, f: usize, depth: u32) -> Option<Vec<Expr>> {
        let params = self.sigs[f].params.clone();
        let need = self.sigs[f].need;
        let cands = if params.contains(&Ty::Ptr) { self.lendable(need) } else { Vec::new() };
        if params.contains(&Ty::Ptr) && cands.is_empty() {
            return None;
        }
        let d = depth.saturating_sub(1).min(2);
        let mut out = Vec::new();
        for &p in &params {
            if p == Ty::Ptr {
                let r = self.regions[self.rng.pick(&cands)].clone();
                let o = self.rng.below((r.size - need) as usize + 1) as u32;
                out.push(ptr_add(r.base, o));
            } else {
                out.push(self.expr(p, d));
            }
        }
        Some(out)
    }

    // --------------------------------------------------------- memory

    /// An index expression for a computed address (any U64 value).
    fn idx_expr(&mut self) -> Expr {
        self.mem_depth += 1;
        let e = if self.rng.chance(60) {
            self.expr(Ty::U64, 2)
        } else {
            let t = self.rng.pick(&[Ty::U8, Ty::U16, Ty::U32]);
            let inner = self.expr(t, 2);
            Expr { ty: Ty::U64, kind: ExprKind::Widen(Box::new(inner)) }
        };
        self.mem_depth -= 1;
        e
    }

    /// An in-bounds address for a `bytes`-wide access into region `r`, as
    /// (address expression, immediate offset). The checked-index form may
    /// trap instead, at its own Bounds site.
    fn access(&mut self, r: usize, bytes: u32) -> (Expr, u32) {
        let reg = self.regions[r].clone();
        let room = reg.size - bytes;
        let s = self.rng.pick(&SCALES);
        let rest = self.rng.below(room.min(7) as usize + 1) as u32;
        let avail = room - rest;
        match self.rng.below(10) {
            0..=3 => {
                let o = self.rng.below(room as usize + 1) as u32;
                (reg.base, o)
            }
            4 | 5 => {
                let o = self.rng.below(room as usize + 1) as u32;
                let i = self.rng.below((o / s) as usize + 1) as u32;
                (offset(reg.base, konst(Ty::U64, i as i128), s), o - i * s)
            }
            6 | 7 => {
                // Masked index (largest 2^k - 1 that fits): never out of range.
                let mut m: u32 = 0;
                while (2 * m + 1) * s <= avail {
                    m = 2 * m + 1;
                }
                let e = self.idx_expr();
                let idx = arith(Ty::U64, ArithOp::And, e, konst(Ty::U64, m as i128), 0);
                (offset(reg.base, idx, s), rest)
            }
            _ => {
                // Checked index: n valid elements of `s` bytes.
                let n = (avail / s + 1) as u64;
                // Mostly in range, so that most runs get past the check.
                let i = match self.rng.below(20) {
                    0 => self.idx_expr(),
                    1 => konst(Ty::U64, self.rng.below(2 * n as usize) as i128),
                    2..=4 => konst(Ty::U64, self.rng.below(n as usize) as i128),
                    r => {
                        let mask = if r <= 10 {
                            // Traps for an index in n..next_power_of_two(n).
                            n.next_power_of_two() - 1
                        } else {
                            // The largest 2^k - 1 below n: never traps.
                            (n + 1).next_power_of_two() / 2 - 1
                        };
                        let e = self.idx_expr();
                        arith(Ty::U64, ArithOp::And, e, konst(Ty::U64, mask as i128), 0)
                    }
                };
                let len = if n > 1 && self.rng.chance(30) {
                    // A length in a register, not an immediate.
                    let k = self.rng.below(n as usize) as i128;
                    arith(Ty::U64, ArithOp::AddW, konst(Ty::U64, n as i128 - k), konst(Ty::U64, k), 0)
                } else {
                    konst(Ty::U64, n as i128)
                };
                let site = self.site(TrapKind::Bounds, Ty::U64);
                let idx = Expr {
                    ty: Ty::U64,
                    kind: ExprKind::Bounds { idx: Box::new(i), len: Box::new(len), site },
                };
                (offset(reg.base, idx, s), rest)
            }
        }
    }

    fn load(&mut self, ty: Ty) -> Option<Expr> {
        if self.mem_depth >= 2 {
            return None;
        }
        let want_bool = ty == Ty::Bool;
        let cands: Vec<usize> = (0..self.regions.len())
            .filter(|&i| self.regions[i].boolean == want_bool && self.regions[i].size >= ty.bytes())
            .collect();
        if cands.is_empty() {
            return None;
        }
        let r = self.rng.pick(&cands);
        self.mem_depth += 1;
        let (addr, off) = self.access(r, ty.bytes());
        self.mem_depth -= 1;
        Some(Expr { ty, kind: ExprKind::Load { addr: Box::new(addr), off } })
    }

    fn store(&mut self) -> Option<Stmt> {
        let cands: Vec<usize> = (0..self.regions.len()).filter(|&i| self.regions[i].writable).collect();
        if cands.is_empty() {
            return None;
        }
        let r = self.rng.pick(&cands);
        let (size, boolean) = (self.regions[r].size, self.regions[r].boolean);
        let ty = if boolean {
            Ty::Bool
        } else {
            let fit: Vec<Ty> = Ty::INTS.iter().copied().filter(|t| t.bytes() <= size).collect();
            self.rng.pick(&fit)
        };
        let (addr, off) = self.access(r, ty.bytes());
        let value = self.expr(ty, 3);
        Some(Stmt::Store { addr, off, value })
    }

    /// A copy between two regions, disjoint or identical (never partially
    /// overlapping, which lowering never makes either).
    fn copy(&mut self) -> Option<Stmt> {
        let dsts = self.lendable(1);
        let srcs: Vec<usize> = (0..self.regions.len()).filter(|&i| !self.regions[i].boolean).collect();
        if dsts.is_empty() {
            return None;
        }
        let d = self.regions[self.rng.pick(&dsts)].clone();
        let s = self.regions[self.rng.pick(&srcs)].clone();
        let max = d.size.min(s.size);
        let n = if max > 64 && self.rng.chance(50) {
            65 + self.rng.below((max - 64) as usize) as u32
        } else {
            1 + self.rng.below(max.min(64) as usize) as u32
        };
        let dofs = self.rng.below((d.size - n) as usize + 1) as u32;
        let mut sofs = self.rng.below((s.size - n) as usize + 1) as u32;
        if d.obj == s.obj {
            let a = d.start + dofs;
            let b = s.start + sofs;
            if a != b && a < b + n && b < a + n {
                if a >= s.start && a - s.start + n <= s.size {
                    sofs = a - s.start;
                } else {
                    return None;
                }
            }
        }
        Some(Stmt::Copy { dst: ptr_add(d.base, dofs), src: ptr_add(s.base, sofs), size: n })
    }

    /// Statements writing every byte of the slot behind `reg`, so no
    /// later load reads uninitialised memory.
    fn init_slot(&mut self, reg: &Region, out: &mut Vec<Stmt>) {
        let reg = reg.clone();
        let size = reg.size;
        if reg.boolean {
            for o in 0..size {
                let v = self.expr(Ty::Bool, 1);
                out.push(Stmt::Store { addr: reg.base.clone(), off: o, value: v });
            }
            return;
        }
        let blobs: Vec<usize> = (0..self.data_sizes.len()).filter(|&b| self.data_sizes[b] >= size).collect();
        if !blobs.is_empty() && self.rng.chance(35) {
            let b = self.rng.pick(&blobs);
            let o = self.rng.below((self.data_sizes[b] - size) as usize + 1) as u32;
            let src = ptr_add(Expr { ty: Ty::Ptr, kind: ExprKind::Data(b as u32) }, o);
            out.push(Stmt::Copy { dst: reg.base, src, size });
            return;
        }
        let mut o = 0;
        if size >= 256 {
            // Fill the 8-byte words in a loop: i < size / 8, store at slot + 8i.
            let n8 = size / 8;
            let i = self.new_var(Ty::U64);
            self.locked[i] = true;
            out.push(Stmt::Assign { var: i as VarId, value: konst(Ty::U64, 0) });
            let k1 = (self.rng.next() | 1) as i128;
            let k2 = self.rng.next() as i128;
            let value = arith(
                Ty::U64,
                ArithOp::Xor,
                arith(Ty::U64, ArithOp::MulW, var(Ty::U64, i), konst(Ty::U64, k1), 0),
                konst(Ty::U64, k2),
                0,
            );
            out.push(Stmt::While {
                cond: cmp(CmpOp::Lt, var(Ty::U64, i), konst(Ty::U64, n8 as i128)),
                body: vec![Stmt::Store { addr: offset(reg.base.clone(), var(Ty::U64, i), 8), off: 0, value }],
                step: vec![Stmt::Assign {
                    var: i as VarId,
                    value: arith(Ty::U64, ArithOp::AddW, var(Ty::U64, i), konst(Ty::U64, 1), 0),
                }],
            });
            o = n8 * 8;
        }
        while o < size {
            let left = size - o;
            let fit: Vec<Ty> = Ty::INTS.iter().copied().filter(|t| t.bytes() <= left).collect();
            let t = if self.rng.chance(70) {
                *fit.iter().max_by_key(|t| t.bytes()).unwrap()
            } else {
                self.rng.pick(&fit)
            };
            let v = self.expr(t, 1);
            out.push(Stmt::Store { addr: reg.base.clone(), off: o, value: v });
            o += t.bytes();
        }
    }

    /// Frame slots (initialised), pointer locals and the data regions.
    fn memory(&mut self, sig: &Sig, out: &mut Vec<Stmt>) {
        self.slots.clear();
        self.regions.clear();
        for b in 0..self.data_sizes.len() {
            self.regions.push(Region {
                base: Expr { ty: Ty::Ptr, kind: ExprKind::Data(b as u32) },
                obj: OBJ_DATA + b,
                start: 0,
                size: self.data_sizes[b],
                writable: false,
                boolean: false,
            });
        }
        if let Some(pi) = sig.params.iter().position(|&t| t == Ty::Ptr) {
            self.regions.push(Region {
                base: var(Ty::Ptr, pi),
                obj: OBJ_PARAM,
                start: 0,
                size: sig.need,
                writable: true,
                boolean: false,
            });
        }
        let nslots = if self.rng.chance(40) { 0 } else { 1 + self.rng.below(4) };
        let mut big = false;
        for k in 0..nslots {
            let boolean = self.rng.chance(15);
            let size = if boolean {
                1 + self.rng.below(8)
            } else if !big && self.rng.chance(8) {
                // Past 4 KiB below the frame pointer: the two-instruction
                // slot address and the register-offset load/store forms.
                big = true;
                4000 + self.rng.below(5000)
            } else if self.rng.chance(25) {
                25 + self.rng.below(136)
            } else {
                1 + self.rng.below(24)
            } as u32;
            let align = self.rng.pick(&[1, 2, 4, 8]);
            self.slots.push(SlotInfo { size, align });
            let reg = Region {
                base: Expr { ty: Ty::Ptr, kind: ExprKind::Slot(k as u32) },
                obj: k,
                start: 0,
                size,
                writable: true,
                boolean,
            };
            // The initialising expressions may load, but only from regions
            // already written: this one is published once it is.
            self.init_slot(&reg, out);
            self.regions.push(reg);
        }
        // A pointer local into a slot or into the caller's memory.
        let cands = self.lendable(1);
        if !cands.is_empty() && self.rng.chance(30) {
            let reg = self.regions[self.rng.pick(&cands)].clone();
            let o = self.rng.below(reg.size as usize) as u32;
            let p = self.new_var(Ty::Ptr);
            self.visible[p] = true;
            self.locked[p] = true;
            out.push(Stmt::Assign { var: p as VarId, value: ptr_add(reg.base, o) });
            self.regions.push(Region {
                base: var(Ty::Ptr, p),
                obj: reg.obj,
                start: reg.start + o,
                size: reg.size - o,
                writable: true,
                boolean: false,
            });
        }
    }

    fn call_expr(&mut self, ty: Ty, depth: u32) -> Option<Expr> {
        if self.calls >= 8 {
            return None;
        }
        let cands: Vec<usize> = (0..self.cur)
            .filter(|&j| match self.sigs[j].ret {
                Some(r) => r == ty || (ty.is_int() && r.is_int() && ty.can_widen_from(r)),
                None => false,
            })
            .collect();
        if cands.is_empty() {
            return None;
        }
        self.calls += 1;
        let j = self.rng.pick(&cands);
        let rt = self.sigs[j].ret.unwrap();
        let args = self.args(j, depth)?;
        let call = Expr { ty: rt, kind: ExprKind::Call { func: j as FuncId, args } };
        Some(if rt == ty { call } else { Expr { ty, kind: ExprKind::Widen(Box::new(call)) } })
    }

    fn block(&mut self, len: usize, depth: u32, ret: Option<Ty>) -> Vec<Stmt> {
        let mut out = Vec::new();
        for _ in 0..len {
            if self.nodes > 500 {
                break;
            }
            if !self.regions.is_empty() && self.rng.chance(16) {
                let s = if self.rng.chance(70) { self.store() } else { self.copy() };
                out.extend(s);
                continue;
            }
            let r = self.rng.below(100);
            if r < 38 {
                let cands: Vec<usize> = (0..self.vars.len())
                    .filter(|&i| self.visible[i] && !self.locked[i])
                    .collect();
                if cands.is_empty() {
                    continue;
                }
                let v = self.rng.pick(&cands);
                let ty = self.vars[v].ty;
                let value = self.expr(ty, 3);
                out.push(Stmt::Assign { var: v as VarId, value });
            } else if r < 52 && depth > 0 {
                let cond = self.expr(Ty::Bool, 3);
                let n1 = 1 + self.rng.below(3);
                let then = self.block(n1, depth - 1, ret);
                let n2 = 1 + self.rng.below(3);
                let els = if self.rng.chance(50) {
                    self.block(n2, depth - 1, ret)
                } else {
                    Vec::new()
                };
                out.push(Stmt::If { cond, then, els });
            } else if r < 63 && depth > 0 && self.loops < 2 {
                let ct = self.rng.pick(&Ty::INTS);
                let c = self.new_var(ct);
                out.push(Stmt::Assign { var: c as VarId, value: konst(ct, 0) });
                self.visible[c] = true;
                self.locked[c] = true;
                let n = self.rng.below(9) as i128;
                let mut cond = cmp(CmpOp::Lt, var(ct, c), konst(ct, n));
                if self.rng.chance(30) {
                    let extra = self.expr(Ty::Bool, 2);
                    cond = Expr { ty: Ty::Bool, kind: ExprKind::And(Box::new(cond), Box::new(extra)) };
                }
                let (op, site) = if self.mode == OverflowMode::Trap && self.rng.chance(50) {
                    (ArithOp::Add, self.site(TrapKind::Overflow, ct))
                } else {
                    (ArithOp::AddW, 0)
                };
                let mut step = vec![Stmt::Assign {
                    var: c as VarId,
                    value: arith(ct, op, var(ct, c), konst(ct, 1), site),
                }];
                // A step is an assignment in the language (`: (i += 1)`):
                // never break, continue or return.
                if self.rng.chance(20) {
                    let cands: Vec<usize> = (0..self.vars.len())
                        .filter(|&i| self.visible[i] && !self.locked[i])
                        .collect();
                    if !cands.is_empty() {
                        let v = self.rng.pick(&cands);
                        let ty = self.vars[v].ty;
                        let value = self.expr(ty, 2);
                        step.push(Stmt::Assign { var: v as VarId, value });
                    }
                }
                self.loops += 1;
                let nb = 2 + self.rng.below(4);
                let body = self.block(nb, depth - 1, ret);
                self.loops -= 1;
                self.visible[c] = false;
                out.push(Stmt::While { cond, body, step });
            } else if r < 72 && self.loops > 0 {
                let cond = self.expr(Ty::Bool, 2);
                let s = if r < 68 { Stmt::Break } else { Stmt::Continue };
                out.push(Stmt::If { cond, then: vec![s], els: Vec::new() });
            } else if r < 74 && self.loops > 0 {
                // Unconditional: the rest of this block is dead code.
                out.push(if self.rng.chance(50) { Stmt::Break } else { Stmt::Continue });
                break;
            } else if r < 79 {
                let value = ret.map(|t| self.expr(t, 3));
                let cond = self.expr(Ty::Bool, 2);
                out.push(Stmt::If { cond, then: vec![Stmt::Return(value)], els: Vec::new() });
            } else if r < 86 {
                if self.cur > 0 && self.calls < 8 {
                    self.calls += 1;
                    let j = self.rng.below(self.cur);
                    if let Some(args) = self.args(j, 3) {
                        let ty = self.sigs[j].ret.unwrap_or(Ty::U8);
                        out.push(Stmt::Eval(Expr { ty, kind: ExprKind::Call { func: j as FuncId, args } }));
                    }
                }
            } else if r < 92 {
                let cond = if self.rng.chance(70) {
                    // Usually true: `e == e` evaluates e twice.
                    let t = self.rng.pick(&ALL);
                    let e = self.expr(t, 3);
                    cmp(CmpOp::Eq, e.clone(), e)
                } else {
                    self.expr(Ty::Bool, 3)
                };
                let site = self.site(TrapKind::Assert, Ty::Bool);
                out.push(Stmt::Assert { cond, site });
            } else {
                let t = self.rng.pick(&ALL);
                let lhs = self.expr(t, 3);
                let rhs = if self.rng.chance(75) { lhs.clone() } else { self.expr(t, 2) };
                let site = self.site(TrapKind::AssertEq, t);
                out.push(Stmt::AssertEq { lhs, rhs, site });
            }
        }
        out
    }

    fn func(&mut self, idx: usize, is_test: bool) -> Func {
        let sig = self.sigs[idx].clone();
        // A test may call every helper; a helper only the ones before it.
        self.cur = idx;
        self.vars.clear();
        self.visible.clear();
        self.locked.clear();
        self.loops = 0;
        self.nodes = 0;
        self.calls = 0;
        for &p in &sig.params {
            let v = self.new_var(p);
            self.visible[v] = true;
            // Parameters are immutable in the language; keep most of them so.
            // A pointer is never reassigned (it names its region).
            self.locked[v] = p == Ty::Ptr || !self.rng.chance(15);
        }
        let mut body = Vec::new();
        self.memory(&sig, &mut body);
        for _ in 0..self.rng.below(7) {
            let t = self.rng.pick(&ALL);
            let value = self.expr(t, 2);
            let v = self.new_var(t);
            body.push(Stmt::Assign { var: v as VarId, value });
            self.visible[v] = true;
        }
        let n = 3 + self.rng.below(6);
        body.extend(self.block(n, 2, sig.ret));
        if is_test {
            // Pin a few helper results.
            for _ in 0..1 + self.rng.below(3) {
                if self.cur == 0 {
                    break;
                }
                let j = self.rng.below(self.cur);
                if let Some(rt) = self.sigs[j].ret {
                    let Some(args) = self.args(j, 2) else { continue };
                    let lhs = Expr { ty: rt, kind: ExprKind::Call { func: j as FuncId, args } };
                    let rhs = self.leaf(rt);
                    let site = self.site(TrapKind::AssertEq, rt);
                    body.push(Stmt::AssertEq { lhs, rhs, site });
                }
            }
        }
        match sig.ret {
            Some(t) if self.rng.chance(93) => {
                let e = self.expr(t, 3);
                body.push(Stmt::Return(Some(e)));
            }
            None if self.rng.chance(30) => body.push(Stmt::Return(None)),
            _ => {}
        }
        let noreturn_site = match sig.ret {
            Some(t) => self.site(TrapKind::NoReturn, t),
            None => 0,
        };
        Func {
            name: if is_test { format!("test{}", idx) } else { format!("f{}", idx) },
            nparams: sig.params.len(),
            ret: sig.ret,
            vars: self.vars.clone(),
            body,
            line: self.line,
            is_test,
            is_invariant: false,
            noreturn_site,
            slots: self.slots.clone(),
        }
    }
}

fn program(rng: &mut Rng, mode: OverflowMode) -> Program {
    let nhelpers = 1 + rng.below(6);
    let ntests = rng.below(3);
    let mut sigs = Vec::new();
    for _ in 0..nhelpers {
        let np = if rng.chance(15) { 8 } else { rng.below(9) };
        let mut params: Vec<Ty> = (0..np).map(|_| rng.pick(&ALL)).collect();
        let mut need = 0;
        if np > 0 && rng.chance(25) {
            // An aggregate passed (or returned) by hidden pointer.
            let i = rng.below(np);
            params[i] = Ty::Ptr;
            let max = if rng.chance(20) { 160 } else { 32 };
            need = 1 + rng.below(max) as u32;
        }
        let ret = if rng.chance(88) { Some(rng.pick(&ALL)) } else { None };
        sigs.push(Sig { params, ret, need });
    }
    let ndata = rng.below(4);
    let data: Vec<Vec<u8>> = (0..ndata)
        .map(|_| {
            let n = if rng.chance(20) { 65 + rng.below(200) } else { 1 + rng.below(64) };
            (0..n).map(|_| rng.next() as u8).collect()
        })
        .collect();
    let placeholder = Site { kind: TrapKind::Overflow, line: 0, what: String::new(), ty: Ty::U8 };
    let mut g = Gen {
        rng,
        mode,
        sites: vec![placeholder],
        sigs,
        cur: 0,
        vars: Vec::new(),
        visible: Vec::new(),
        locked: Vec::new(),
        loops: 0,
        nodes: 0,
        calls: 0,
        line: 0,
        data_sizes: data.iter().map(|d| d.len() as u32).collect(),
        slots: Vec::new(),
        regions: Vec::new(),
        mem_depth: 0,
        in_seq: false,
    };
    let mut funcs = Vec::new();
    for i in 0..nhelpers {
        funcs.push(g.func(i, false));
    }
    for t in 0..ntests {
        g.sigs.push(Sig { params: Vec::new(), ret: None, need: 0 });
        let mut f = g.func(nhelpers, true);
        f.name = format!("test{}", t);
        // An invariant block is lowered exactly like a test.
        f.is_invariant = g.rng.chance(30);
        funcs.push(f);
        g.sigs.pop();
    }
    Program { module: "rnd".into(), funcs, sites: g.sites, mode, unchecked: Vec::new(), data, globals: Vec::new(), internal_abi: Vec::new() }
}

// ------------------------------------------------------- pretty printer

fn show_expr(p: &Program, f: &Func, e: &Expr) -> String {
    match &e.kind {
        ExprKind::Const(c) => format!("{}:{}", c, e.ty.name()),
        ExprKind::Var(v) => f.vars[*v as usize].name.clone(),
        ExprKind::Arith { op, lhs, rhs, site } => {
            let o = match op {
                ArithOp::AddW => "+%",
                ArithOp::SubW => "-%",
                ArithOp::MulW => "*%",
                ArithOp::DivW => "/%",
                ArithOp::ShlW => "<<%",
                ArithOp::ShrW => ">>%",
                _ => op.symbol(),
            };
            let s = if *site != 0 { format!("@{}", site) } else { String::new() };
            format!("({} {}{} {})", show_expr(p, f, lhs), o, s, show_expr(p, f, rhs))
        }
        ExprKind::Cmp { op, lhs, rhs } => {
            format!("({} {} {})", show_expr(p, f, lhs), op.symbol(), show_expr(p, f, rhs))
        }
        ExprKind::And(a, b) => format!("({} and {})", show_expr(p, f, a), show_expr(p, f, b)),
        ExprKind::Or(a, b) => format!("({} or {})", show_expr(p, f, a), show_expr(p, f, b)),
        ExprKind::Not(a) => format!("!{}", show_expr(p, f, a)),
        ExprKind::BitNot(a) => format!("~{}", show_expr(p, f, a)),
        ExprKind::Call { func, args } => {
            let a: Vec<String> = args.iter().map(|x| show_expr(p, f, x)).collect();
            format!("{}({})", p.funcs[*func as usize].name, a.join(", "))
        }
        ExprKind::Widen(a) => format!("{}({})", e.ty.name(), show_expr(p, f, a)),
        ExprKind::Cast { arg, site } => {
            let how = if *site != 0 { format!("@{}", site) } else { "%".into() };
            format!("({} as{} {})", show_expr(p, f, arg), how, e.ty.name())
        }
        ExprKind::FArith { op, lhs, rhs } => {
            format!("({} f{} {})", show_expr(p, f, lhs), op.symbol(), show_expr(p, f, rhs))
        }
        ExprKind::FNeg(a) => format!("-f{}", show_expr(p, f, a)),
        ExprKind::FSqrt(a) => format!("@sqrt({})", show_expr(p, f, a)),
        ExprKind::IntToFloat(a) => format!("@floatFromInt({})", show_expr(p, f, a)),
        ExprKind::FloatCast(a) => format!("@floatCast({}):{}", show_expr(p, f, a), e.ty.name()),
        ExprKind::FloatToInt { arg, site } => {
            format!("@intFromFloat@{}({}):{}", site, show_expr(p, f, arg), e.ty.name())
        }
        ExprKind::Slot(k) => format!("&s{}", k),
        ExprKind::Data(k) => format!("&d{}", k),
        ExprKind::Global(k) => format!("&g{}", k),
        ExprKind::Load { addr, off } => format!("{}[{} + {}]", e.ty.name(), show_expr(p, f, addr), off),
        ExprKind::Offset { base, idx, scale } => {
            format!("({} + {}*{})", show_expr(p, f, base), show_expr(p, f, idx), scale)
        }
        ExprKind::Bounds { idx, len, site } => {
            format!("chk@{}({} < {})", site, show_expr(p, f, idx), show_expr(p, f, len))
        }
        ExprKind::Seq { stmts, value } => {
            let mut b = String::new();
            show_block(p, f, stmts, 0, &mut b);
            format!("seq{{ {}; {} }}", b.trim().replace('\n', " "), show_expr(p, f, value))
        }
        ExprKind::Select { cond, then, els } => {
            format!("(if {} {} else {})", show_expr(p, f, cond), show_expr(p, f, then), show_expr(p, f, els))
        }
    }
}

fn show_block(p: &Program, f: &Func, ss: &[Stmt], ind: usize, out: &mut String) {
    let pad = "    ".repeat(ind);
    for s in ss {
        match s {
            Stmt::Assign { var, value } => {
                out.push_str(&format!("{}{} = {};\n", pad, f.vars[*var as usize].name, show_expr(p, f, value)))
            }
            Stmt::If { cond, then, els } => {
                out.push_str(&format!("{}if {} {{\n", pad, show_expr(p, f, cond)));
                show_block(p, f, then, ind + 1, out);
                if !els.is_empty() {
                    out.push_str(&format!("{}}} else {{\n", pad));
                    show_block(p, f, els, ind + 1, out);
                }
                out.push_str(&format!("{}}}\n", pad));
            }
            Stmt::While { cond, body, step } => {
                out.push_str(&format!("{}while {} : {{\n", pad, show_expr(p, f, cond)));
                show_block(p, f, step, ind + 2, out);
                out.push_str(&format!("{}}} {{\n", pad));
                show_block(p, f, body, ind + 1, out);
                out.push_str(&format!("{}}}\n", pad));
            }
            Stmt::Break => out.push_str(&format!("{}break;\n", pad)),
            Stmt::Continue => out.push_str(&format!("{}continue;\n", pad)),
            Stmt::Return(None) => out.push_str(&format!("{}return;\n", pad)),
            Stmt::Return(Some(e)) => out.push_str(&format!("{}return {};\n", pad, show_expr(p, f, e))),
            Stmt::Eval(e) => out.push_str(&format!("{}{};\n", pad, show_expr(p, f, e))),
            Stmt::Assert { cond, site } => {
                out.push_str(&format!("{}assert@{}({});\n", pad, site, show_expr(p, f, cond)))
            }
            Stmt::AssertEq { lhs, rhs, site } => out.push_str(&format!(
                "{}assert_eq@{}({}, {});\n",
                pad,
                site,
                show_expr(p, f, lhs),
                show_expr(p, f, rhs)
            )),
            Stmt::Store { addr, off, value } => out.push_str(&format!(
                "{}{}[{} + {}] = {};\n",
                pad,
                value.ty.name(),
                show_expr(p, f, addr),
                off,
                show_expr(p, f, value)
            )),
            Stmt::Copy { dst, src, size } => out.push_str(&format!(
                "{}copy {} bytes {} <- {};\n",
                pad,
                size,
                show_expr(p, f, dst),
                show_expr(p, f, src)
            )),
        }
    }
}

/// Source-like rendering of a generated program, for failure reports.
fn show_program(p: &Program) -> String {
    let mut out = String::new();
    for (k, d) in p.data.iter().enumerate() {
        out.push_str(&format!("data d{} = {:02x?}\n", k, d));
    }
    for f in &p.funcs {
        let params: Vec<String> =
            f.vars[..f.nparams].iter().map(|v| format!("{}: {}", v.name, v.ty.name())).collect();
        let locals: Vec<String> =
            f.vars[f.nparams..].iter().map(|v| format!("{}: {}", v.name, v.ty.name())).collect();
        let slots: Vec<String> = f.slots.iter().map(|s| format!("{}/{}", s.size, s.align)).collect();
        out.push_str(&format!(
            "fn {}({}) -> {} {{  // locals {}; slots {}\n",
            f.name,
            params.join(", "),
            f.ret.map_or("void", |t| t.name()),
            locals.join(", "),
            slots.join(", ")
        ));
        show_block(p, f, &f.body, 1, &mut out);
        out.push_str("}\n");
    }
    out
}

// --------------------------------------------------------------- runner

#[derive(Default, Debug)]
struct Stats {
    programs: usize,
    functions: usize,
    calls: usize,
    returns: usize,
    /// Indexed by `TrapKind as usize`.
    traps: [usize; 17],
    skipped: usize,
    /// Functions with frame slots, and functions only called from others
    /// (they take a pointer, which only a caller can supply).
    with_slots: usize,
    ptr_only: usize,
}

/// Raw register image of an argument: canonical form, plus random garbage in
/// the upper half for types of 32 bits or less (the ABI leaves it undefined).
fn raw_arg(rng: &mut Rng, v: i128, ty: Ty) -> u64 {
    let c = canon(v, ty);
    if ty.is64() {
        c
    } else {
        c | (rng.next() << 32)
    }
}

fn check_value(ty: Ty, want: i128, raw: u64) -> bool {
    if ty.is64() {
        raw == canon(want, ty)
    } else {
        (raw & 0xffff_ffff) == canon(want, ty)
    }
}

/// Run every function of `prog` on random arguments in both engines.
fn compare(prog: &Program, rng: &mut Rng, calls_per_fn: usize, stats: &mut Stats) -> Result<(), String> {
    let code = codegen::compile(prog, TrapStyle::Jit, true)
        .map_err(|e| format!("codegen error in {}: {} {}", e.func, e.construct, e.detail))?;
    let mut jit = Jit::load(&code, prog.funcs.len(), &prog.data, &prog.globals)?;
    stats.programs += 1;
    for (fi, f) in prog.funcs.iter().enumerate() {
        stats.functions += 1;
        if !f.slots.is_empty() {
            stats.with_slots += 1;
        }
        let ptys: Vec<Ty> = f.vars[..f.nparams].iter().map(|v| v.ty).collect();
        if ptys.contains(&Ty::Ptr) {
            stats.ptr_only += 1;
            continue;
        }
        let n = if f.is_test || ptys.is_empty() { 1 } else { calls_per_fn };
        for _ in 0..n {
            let args: Vec<i128> = ptys.iter().map(|&t| value(rng, t)).collect();
            let mut it = Interp::new(prog);
            it.fuel = 200_000;
            let want = it.call(fi, &args);
            if matches!(want, Err(Stop::Fuel) | Err(Stop::Depth)) {
                stats.skipped += 1;
                continue;
            }
            let raw: Vec<u64> = args.iter().zip(&ptys).map(|(&v, &t)| raw_arg(rng, v, t)).collect();
            if std::env::var("T27B_DIFF_TRACE").is_ok() {
                eprintln!("  f{} {:?}: interpreter {:?} ({} fuel left)", fi, args, want, it.fuel);
            }
            let got = jit.call(fi as FuncId, &raw);
            stats.calls += 1;
            let ok = match (&want, &got) {
                (Ok(v), Ok(r)) => {
                    stats.returns += 1;
                    match (f.ret, v) {
                        (Some(t), Some(v)) => check_value(t, *v, *r),
                        (None, None) => true,
                        _ => false,
                    }
                }
                (Err(Stop::Trap { site, a, b }), Err(t)) => {
                    let s = &prog.sites[*site as usize];
                    stats.traps[s.kind as usize] += 1;
                    t.site == *site
                        && (s.kind != TrapKind::AssertEq
                            || (s.ty.from_raw(t.a) == *a && s.ty.from_raw(t.b) == *b))
                }
                _ => false,
            };
            if !ok {
                let got_s = match &got {
                    Ok(r) => format!("ret {:#x}", r),
                    Err(t) => format!(
                        "trap site {} ({:?} line {}) a={:#x} b={:#x}",
                        t.site,
                        prog.sites.get(t.site as usize).map(|s| s.kind),
                        prog.sites.get(t.site as usize).map_or(0, |s| s.line),
                        t.a,
                        t.b
                    ),
                };
                let want_s = match &want {
                    Ok(v) => format!("ret {:?}", v),
                    Err(Stop::Trap { site, a, b }) => format!(
                        "trap site {} ({:?} line {}) a={} b={}",
                        site, prog.sites[*site as usize].kind, prog.sites[*site as usize].line, a, b
                    ),
                    Err(e) => format!("{:?}", e),
                };
                return Err(format!(
                    "function {} ({}) args {:?} (raw {:x?}): interpreter {}, jit {}",
                    fi, f.name, args, raw, want_s, got_s
                ));
            }
        }
    }
    Ok(())
}

fn env_u64(name: &str, default: u64) -> u64 {
    match std::env::var(name) {
        Ok(s) => {
            let s = s.trim();
            if let Some(h) = s.strip_prefix("0x") {
                u64::from_str_radix(h, 16).unwrap_or(default)
            } else {
                s.parse().unwrap_or(default)
            }
        }
        Err(_) => default,
    }
}

#[test]
fn random_programs_jit_matches_interpreter() {
    let cases = env_u64("T27B_DIFF_CASES", 2500);
    let base = env_u64("T27B_DIFF_SEED", 0x7427);
    let trace = std::env::var("T27B_DIFF_TRACE").is_ok();
    // T27B_DIFF_ONE=<seed> replays a single case (bit 40 selects wrap mode).
    let one = std::env::var("T27B_DIFF_ONE").ok().map(|_| env_u64("T27B_DIFF_ONE", 0));
    let mut failures = Vec::new();
    for mode in [OverflowMode::Trap, OverflowMode::Wrap] {
        let mut stats = Stats::default();
        for i in 0..cases {
            let mut seed = base
                .wrapping_mul(0x1_0000_0001)
                .wrapping_add(i)
                .wrapping_add(if mode == OverflowMode::Wrap { 1 << 40 } else { 0 });
            if let Some(s) = one {
                if i > 0 || (s >> 40 & 1 == 1) != (mode == OverflowMode::Wrap) {
                    break;
                }
                seed = s;
            }
            if trace {
                eprintln!("case seed {:#x} mode {:?}", seed, mode);
            }
            let mut rng = Rng::new(seed);
            let prog = program(&mut rng, mode);
            if one.is_some() {
                eprintln!("{}", show_program(&prog));
            }
            if let Err(e) = compare(&prog, &mut rng, 6, &mut stats) {
                if failures.len() < 3 {
                    eprintln!("MISMATCH seed {:#x} mode {:?}: {}\n{}", seed, mode, e, show_program(&prog));
                }
                failures.push(format!("seed {:#x} {:?}: {}", seed, mode, e));
            }
        }
        eprintln!(
            "differential {:?}: {} programs, {} functions, {} calls compared ({} returns, traps: \
             overflow {}, div-zero {}, shift {}, assert {}, assert_eq {}, no-return {}, cast {}, bounds {}), {} skipped (fuel); \
             {} functions with frame slots, {} reached only through a pointer-passing call",
            mode,
            stats.programs,
            stats.functions,
            stats.calls,
            stats.returns,
            stats.traps[1],
            stats.traps[2],
            stats.traps[3],
            stats.traps[4],
            stats.traps[5],
            stats.traps[6],
            stats.traps[TrapKind::Cast as usize],
            stats.traps[TrapKind::Bounds as usize],
            stats.skipped,
            stats.with_slots,
            stats.ptr_only
        );
    }
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}

// ------------------------------------------------- exhaustive edge values

fn edge_values(ty: Ty) -> Vec<i128> {
    if ty == Ty::Bool {
        return vec![0, 1];
    }
    let bits = ty.bits() as i128;
    let mut v = vec![
        ty.min(),
        ty.min() + 1,
        -2,
        -1,
        0,
        1,
        2,
        3,
        7,
        bits - 1,
        bits,
        bits + 1,
        ty.max() - 1,
        ty.max(),
        ty.max() / 3,
        ty.min() / 2,
    ];
    v.retain(|&x| ty.fits(x));
    v.sort();
    v.dedup();
    v
}

fn one_func(name: &str, params: &[Ty], ret: Ty, body: Vec<Stmt>, nsite: SiteId) -> Func {
    Func {
        name: name.into(),
        nparams: params.len(),
        ret: Some(ret),
        vars: params.iter().enumerate().map(|(i, &t)| Var { name: format!("p{}", i), ty: t }).collect(),
        body,
        line: 1,
        is_test: false,
        is_invariant: false,
        noreturn_site: nsite,
        slots: Vec::new(),
    }
}

fn sites_for(op: ArithOp, ty: Ty) -> (Vec<Site>, SiteId) {
    let mk = |kind| Site { kind, line: 1, what: String::new(), ty };
    let mut sites = vec![mk(TrapKind::Overflow), mk(TrapKind::NoReturn)];
    let site = match op {
        ArithOp::Add | ArithOp::Sub | ArithOp::Mul => {
            sites.push(mk(TrapKind::Overflow));
            2
        }
        ArithOp::Div => {
            sites.push(mk(TrapKind::DivZero));
            sites.push(mk(TrapKind::Overflow));
            2
        }
        ArithOp::DivW | ArithOp::Rem => {
            sites.push(mk(TrapKind::DivZero));
            2
        }
        ArithOp::Shl | ArithOp::Shr => {
            sites.push(mk(TrapKind::ShiftRange));
            2
        }
        _ => 0,
    };
    (sites, site)
}

const OPS: [ArithOp; 16] = [
    ArithOp::Add,
    ArithOp::Sub,
    ArithOp::Mul,
    ArithOp::AddW,
    ArithOp::SubW,
    ArithOp::MulW,
    ArithOp::Div,
    ArithOp::DivW,
    ArithOp::Rem,
    ArithOp::And,
    ArithOp::Or,
    ArithOp::Xor,
    ArithOp::Shl,
    ArithOp::Shr,
    ArithOp::ShlW,
    ArithOp::ShrW,
];

/// Every arithmetic operator on every integer type, at every pair of edge
/// values, with the operands as registers, as a constant on the right and
/// as a constant on the left; shifts also with amounts of other types.
#[test]
fn every_operator_at_edge_values() {
    let mut rng = Rng::new(99);
    let mut stats = Stats::default();
    let mut failures = Vec::new();
    for ty in Ty::INTS.into_iter().chain(ODD) {
        let vals = edge_values(ty);
        for op in OPS {
            // A runtime wrap-mode shift of an odd width is refused by
            // lowering (`bits - 1` is no mask); its constant form is below.
            if ty.is_odd() && matches!(op, ArithOp::ShlW | ArithOp::ShrW) {
                continue;
            }
            let amt_tys: Vec<Ty> = if op.is_shift() {
                vec![ty, Ty::U8, Ty::I8, Ty::U32, Ty::I64]
            } else {
                vec![ty]
            };
            for at in amt_tys {
                let (sites, site) = sites_for(op, ty);
                let rvals = edge_values(at);
                // f0(a, b) = a op b
                let mut funcs = vec![one_func(
                    "rr",
                    &[ty, at],
                    ty,
                    vec![Stmt::Return(Some(arith(ty, op, var(ty, 0), var(at, 1), site)))],
                    1,
                )];
                // Constant right operand (shift: only the wrapping in-range form).
                let mut rconst = Vec::new();
                for &c in &rvals {
                    if op.is_shift() {
                        if matches!(op, ArithOp::Shl | ArithOp::Shr) || !(0..ty.bits() as i128).contains(&c) {
                            continue;
                        }
                        let e = arith(ty, op, var(ty, 0), konst(Ty::U32, c), 0);
                        funcs.push(one_func("rc", &[ty], ty, vec![Stmt::Return(Some(e))], 1));
                    } else {
                        let e = arith(ty, op, var(ty, 0), konst(ty, c), site);
                        funcs.push(one_func("rc", &[ty], ty, vec![Stmt::Return(Some(e))], 1));
                    }
                    rconst.push((funcs.len() - 1, c));
                }
                // Constant left operand.
                let mut lconst = Vec::new();
                for &c in &vals {
                    let e = arith(ty, op, konst(ty, c), var(at, 0), site);
                    funcs.push(one_func("cr", &[at], ty, vec![Stmt::Return(Some(e))], 1));
                    lconst.push((funcs.len() - 1, c));
                }
                let prog = Program { module: "edge".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), globals: Vec::new(), internal_abi: Vec::new() };
                let mut calls: Vec<(usize, Vec<i128>)> = Vec::new();
                for &a in &vals {
                    for &b in &rvals {
                        calls.push((0, vec![a, b]));
                    }
                }
                for &(f, _) in &rconst {
                    for &a in &vals {
                        calls.push((f, vec![a]));
                    }
                }
                for &(f, _) in &lconst {
                    for &b in &rvals {
                        calls.push((f, vec![b]));
                    }
                }
                if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
                    failures.push(format!("{:?} on {} (amount {}): {}", op, ty.name(), at.name(), e));
                }
            }
        }
    }
    // Constant shifts of the odd widths: every in-range amount, the form
    // lowering gives a comptime-known amount.
    for ty in ODD {
        let vals = edge_values(ty);
        let (sites, _) = sites_for(ArithOp::ShlW, ty);
        let mut funcs = Vec::new();
        for c in 0..ty.bits() as i128 {
            for op in [ArithOp::ShlW, ArithOp::ShrW] {
                let e = arith(ty, op, var(ty, 0), konst(Ty::U32, c), 0);
                funcs.push(one_func("sc", &[ty], ty, vec![Stmt::Return(Some(e))], 1));
            }
        }
        let n = funcs.len();
        let prog = Program { module: "oddshift".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), globals: Vec::new(), internal_abi: Vec::new() };
        let calls: Vec<(usize, Vec<i128>)> = (0..n).flat_map(|f| vals.iter().map(move |&a| (f, vec![a]))).collect();
        if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
            failures.push(format!("constant shifts on {}: {}", ty.name(), e));
        }
    }
    eprintln!(
        "edge values: {} programs, {} calls compared ({} returns, {} traps)",
        stats.programs,
        stats.calls,
        stats.returns,
        stats.traps.iter().sum::<usize>()
    );
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// Comparisons as values and as branch conditions, unary operators and
/// widening, at every edge value of every type.
#[test]
fn compare_unary_widen_at_edge_values() {
    let mut rng = Rng::new(7);
    let mut stats = Stats::default();
    let mut failures = Vec::new();
    let mk = |kind, ty| Site { kind, line: 1, what: String::new(), ty };
    for ty in ALL.into_iter().chain(ODD) {
        let vals = edge_values(ty);
        let ops: Vec<CmpOp> = if ty == Ty::Bool { vec![CmpOp::Eq, CmpOp::Ne] } else { CMPS.to_vec() };
        for op in ops {
            let sites = vec![mk(TrapKind::Overflow, ty), mk(TrapKind::NoReturn, Ty::U32)];
            let as_branch = |c: Expr| {
                vec![
                    Stmt::If {
                        cond: c,
                        then: vec![Stmt::Return(Some(konst(Ty::U32, 7)))],
                        els: vec![],
                    },
                    Stmt::Return(Some(konst(Ty::U32, 3))),
                ]
            };
            let mut funcs = vec![
                one_func("v", &[ty, ty], Ty::Bool, vec![Stmt::Return(Some(cmp(op, var(ty, 0), var(ty, 1))))], 1),
                one_func("b", &[ty, ty], Ty::U32, as_branch(cmp(op, var(ty, 0), var(ty, 1))), 1),
            ];
            let mut single = Vec::new();
            for &c in &vals {
                funcs.push(one_func("vc", &[ty], Ty::Bool, vec![Stmt::Return(Some(cmp(op, var(ty, 0), konst(ty, c))))], 1));
                single.push(funcs.len() - 1);
                funcs.push(one_func("bc", &[ty], Ty::U32, as_branch(cmp(op, var(ty, 0), konst(ty, c))), 1));
                single.push(funcs.len() - 1);
                funcs.push(one_func("cb", &[ty], Ty::U32, as_branch(cmp(op, konst(ty, c), var(ty, 0))), 1));
                single.push(funcs.len() - 1);
            }
            let prog = Program { module: "cmp".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), globals: Vec::new(), internal_abi: Vec::new() };
            let mut calls = Vec::new();
            for &a in &vals {
                for &b in &vals {
                    calls.push((0, vec![a, b]));
                    calls.push((1, vec![a, b]));
                }
                for &f in &single {
                    calls.push((f, vec![a]));
                }
            }
            if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
                failures.push(format!("{:?} on {}: {}", op, ty.name(), e));
            }
        }
        // Unary operators.
        let sites = vec![mk(TrapKind::Overflow, ty), mk(TrapKind::NoReturn, ty), mk(TrapKind::Overflow, ty)];
        let mut funcs = Vec::new();
        if ty == Ty::Bool {
            funcs.push(one_func("not", &[ty], ty, vec![Stmt::Return(Some(Expr { ty, kind: ExprKind::Not(Box::new(var(ty, 0))) }))], 1));
        } else {
            funcs.push(one_func("bitnot", &[ty], ty, vec![Stmt::Return(Some(Expr { ty, kind: ExprKind::BitNot(Box::new(var(ty, 0))) }))], 1));
            if ty.signed() {
                funcs.push(one_func("neg", &[ty], ty, vec![Stmt::Return(Some(arith(ty, ArithOp::Sub, konst(ty, 0), var(ty, 0), 2)))], 1));
                funcs.push(one_func("negw", &[ty], ty, vec![Stmt::Return(Some(arith(ty, ArithOp::SubW, konst(ty, 0), var(ty, 0), 0)))], 1));
            }
        }
        let n = funcs.len();
        let prog = Program { module: "unary".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), globals: Vec::new(), internal_abi: Vec::new() };
        let calls: Vec<(usize, Vec<i128>)> =
            (0..n).flat_map(|f| vals.iter().map(move |&a| (f, vec![a]))).collect();
        if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
            failures.push(format!("unary on {}: {}", ty.name(), e));
        }
        // Widening into every wider type.
        for to in Ty::INTS.into_iter().chain(ODD) {
            if to == ty || !to.can_widen_from(ty) {
                continue;
            }
            let sites = vec![mk(TrapKind::Overflow, to), mk(TrapKind::NoReturn, to)];
            let w = Expr { ty: to, kind: ExprKind::Widen(Box::new(var(ty, 0))) };
            // Widen and then use it in 64-bit-sensitive arithmetic.
            let use_ = arith(to, ArithOp::AddW, w.clone(), konst(to, 0), 0);
            let funcs = vec![
                one_func("w", &[ty], to, vec![Stmt::Return(Some(w.clone()))], 1),
                one_func("wa", &[ty], to, vec![Stmt::Return(Some(arith(to, ArithOp::ShrW, use_, konst(Ty::U32, 1), 0)))], 1),
                one_func("wc", &[ty], Ty::Bool, vec![Stmt::Return(Some(cmp(CmpOp::Lt, w, konst(to, 0))))], 1),
            ];
            let prog = Program { module: "widen".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), globals: Vec::new(), internal_abi: Vec::new() };
            let calls: Vec<(usize, Vec<i128>)> =
                (0..3).flat_map(|f| vals.iter().map(move |&a| (f, vec![a]))).collect();
            if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
                failures.push(format!("widen {} -> {}: {}", ty.name(), to.name(), e));
            }
        }
    }
    eprintln!(
        "compare/unary/widen: {} programs, {} calls compared",
        stats.programs, stats.calls
    );
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// `x as T` between every pair of types, truncating and checked, at every
/// edge value of the source: the result returned, fed to 64-bit-sensitive
/// arithmetic and compared, and the operand as a constant.
#[test]
fn casts_at_edge_values() {
    let mut rng = Rng::new(27);
    let mut stats = Stats::default();
    let mut failures = Vec::new();
    let mk = |kind, ty| Site { kind, line: 1, what: String::new(), ty };
    for from in ALL.into_iter().chain(ODD) {
        let vals = edge_values(from);
        for to in Ty::INTS.into_iter().chain(ODD) {
            if to == from {
                continue;
            }
            for checked in [false, true] {
                let sites = vec![mk(TrapKind::Overflow, to), mk(TrapKind::NoReturn, to), mk(TrapKind::Cast, to)];
                let site = if checked { 2 } else { 0 };
                let cast = |arg: Expr| {
                    if from == Ty::Bool {
                        Expr { ty: to, kind: ExprKind::Widen(Box::new(arg)) }
                    } else {
                        Expr { ty: to, kind: ExprKind::Cast { arg: Box::new(arg), site } }
                    }
                };
                let c = cast(var(from, 0));
                let use_ = arith(to, ArithOp::AddW, c.clone(), konst(to, 0), 0);
                let mut funcs = vec![
                    one_func("c", &[from], to, vec![Stmt::Return(Some(c.clone()))], 1),
                    one_func("ca", &[from], to, vec![Stmt::Return(Some(arith(to, ArithOp::ShrW, use_, konst(Ty::U32, 1), 0)))], 1),
                    one_func("cc", &[from], Ty::Bool, vec![Stmt::Return(Some(cmp(CmpOp::Lt, c, konst(to, 0))))], 1),
                ];
                // The operand as a constant (the generator never folds it).
                for &k in &vals {
                    funcs.push(one_func("k", &[from], to, vec![Stmt::Return(Some(cast(konst(from, k))))], 1));
                }
                let n = funcs.len();
                let prog = Program { module: "cast".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), globals: Vec::new(), internal_abi: Vec::new() };
                let mut calls: Vec<(usize, Vec<i128>)> =
                    (0..3).flat_map(|f| vals.iter().map(move |&a| (f, vec![a]))).collect();
                for f in 3..n {
                    calls.push((f, vec![0]));
                }
                if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
                    failures.push(format!("{} as {} ({}): {}", from.name(), to.name(), if checked { "checked" } else { "truncating" }, e));
                }
            }
        }
    }
    eprintln!(
        "casts: {} programs, {} calls compared ({} returns, {} cast traps)",
        stats.programs, stats.calls, stats.returns, stats.traps[TrapKind::Cast as usize]
    );
    assert!(stats.traps[TrapKind::Cast as usize] > 0);
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// Like `compare`, for an explicit list of (function, arguments).
fn compare_calls(
    prog: &Program,
    calls: &[(usize, Vec<i128>)],
    rng: &mut Rng,
    stats: &mut Stats,
) -> Result<(), String> {
    let code = codegen::compile(prog, TrapStyle::Jit, true)
        .map_err(|e| format!("codegen error in {}: {} {}", e.func, e.construct, e.detail))?;
    let mut jit = Jit::load(&code, prog.funcs.len(), &prog.data, &prog.globals)?;
    stats.programs += 1;
    let mut bad = Vec::new();
    for (fi, args) in calls {
        let f = &prog.funcs[*fi];
        let ptys: Vec<Ty> = f.vars[..f.nparams].iter().map(|v| v.ty).collect();
        let want = Interp::new(prog).call(*fi, args);
        // AAPCS64: integer arguments in x0-x7, float ones in d0-d7 (s0-s7), each
        // class numbered on its own; a float result comes back in d0 (s0).
        let (mut xs, mut ds) = (Vec::new(), Vec::new());
        for (&v, &t) in args.iter().zip(&ptys) {
            if t.is_float() {
                ds.push(v as u64);
            } else {
                xs.push(raw_arg(rng, v, t));
            }
        }
        let fp_ret = f.ret.is_some_and(|t| t.is_float());
        let got = jit
            .call_fp(*fi as FuncId, &xs, &ds)
            .map(|(x0, d0)| if fp_ret { d0 } else { x0 });
        stats.calls += 1;
        let ok = match (&want, &got) {
            (Ok(Some(v)), Ok(r)) => {
                stats.returns += 1;
                check_value(f.ret.unwrap(), *v, *r)
            }
            (Err(Stop::Trap { site, .. }), Err(t)) => {
                stats.traps[prog.sites[*site as usize].kind as usize] += 1;
                t.site == *site
            }
            _ => false,
        };
        if !ok && bad.len() < 6 {
            bad.push(format!(
                "{}{:?}: interpreter {:?}, jit {:?}",
                f.name,
                args,
                want,
                got.map(|r| format!("{:#x}", r)).map_err(|t| t.site)
            ));
        }
    }
    if bad.is_empty() {
        Ok(())
    } else {
        Err(bad.join("; "))
    }
}

// ------------------------------------------------------------------ f64

/// Binary64 edge values as IR values (bit patterns): signed zeros, the
/// infinities, quiet and signalling NaNs with payloads, subnormals, the
/// integer-exactness edge 2^53, and the bounds of every `@intFromFloat`.
fn f64_edges() -> Vec<i128> {
    let mut v: Vec<i128> = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.5,
        -0.5,
        1.5,
        2.0,
        3.0,
        0.1,
        1.0 / 3.0,
        -2.5,
        127.99,
        128.0,
        -128.5,
        -129.0,
        255.5,
        256.0,
        65535.75,
        2147483647.5,
        -2147483648.75,
        -2147483649.0,
        4294967295.9,
        4294967296.0,
        9007199254740992.0,  // 2^53
        9007199254740994.0,  // 2^53 + 2
        -9007199254740992.0,
        9223372036854775807.0, // rounds to 2^63
        -9223372036854775808.0,
        -9223372036854777856.0, // the next f64 below -2^63
        18446744073709551615.0, // rounds to 2^64
        18446744073709549568.0, // the largest f64 below 2^64
        1.0e300,
        -1.0e300,
        1.0e-300,
        f64::MAX,
        f64::MIN,
        f64::MIN_POSITIVE,
        f64::EPSILON,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ]
    .iter()
    .map(|&x: &f64| f64_bits(x))
    .collect();
    v.extend([
        1,                       // smallest subnormal
        0x8000_0000_0000_0001,   // its negation
        0x000f_ffff_ffff_ffff,   // largest subnormal
        0x7ff8_0000_0000_0000,   // the default quiet NaN
        0xfff8_0000_0000_0000,   // negative quiet NaN
        0x7ff8_0000_0000_beef,   // quiet NaN with a payload
        0x7ff0_0000_0000_0001,   // signalling NaN
    ]);
    v
}

fn f64_konst(x: f64) -> Expr {
    konst(Ty::F64, f64_bits(x))
}

fn farith(op: FOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr { ty: Ty::F64, kind: ExprKind::FArith { op, lhs: Box::new(lhs), rhs: Box::new(rhs) } }
}

fn f64_program(name: &str, funcs: Vec<Func>, sites: Vec<Site>) -> Program {
    Program {
        module: name.into(),
        funcs,
        sites,
        mode: OverflowMode::Trap,
        unchecked: Vec::new(),
        data: Vec::new(),
        globals: Vec::new(),
        internal_abi: Vec::new(),
    }
}

/// `+ - * /`, negation and the six comparisons on every pair of edge
/// values, operands in d registers and as constants: the JIT's d0 (or x0
/// for a bool) must be the interpreter's bit pattern exactly.
#[test]
fn f64_arith_and_compare_match_interpreter() {
    let mut rng = Rng::new(64);
    let mut stats = Stats::default();
    let mut failures = Vec::new();
    let vals = f64_edges();
    let nsite = |ty| vec![Site { kind: TrapKind::Overflow, line: 1, what: String::new(), ty }, Site { kind: TrapKind::NoReturn, line: 1, what: String::new(), ty }];
    for op in [FOp::Add, FOp::Sub, FOp::Mul, FOp::Div] {
        let (a, b) = (var(Ty::F64, 0), var(Ty::F64, 1));
        let mut funcs = vec![
            one_func("f", &[Ty::F64, Ty::F64], Ty::F64, vec![Stmt::Return(Some(farith(op, a.clone(), b.clone())))], 1),
            // Both orders with a constant operand, and a nested tree.
            one_func("fk", &[Ty::F64, Ty::F64], Ty::F64, vec![Stmt::Return(Some(farith(op, a.clone(), f64_konst(0.1))))], 1),
            one_func("kf", &[Ty::F64, Ty::F64], Ty::F64, vec![Stmt::Return(Some(farith(op, f64_konst(-3.0), b.clone())))], 1),
            one_func(
                "tree",
                &[Ty::F64, Ty::F64],
                Ty::F64,
                vec![Stmt::Return(Some(farith(
                    op,
                    farith(FOp::Mul, a.clone(), b.clone()),
                    Expr { ty: Ty::F64, kind: ExprKind::FNeg(Box::new(farith(op, b.clone(), a.clone()))) },
                )))],
                1,
            ),
        ];
        if op == FOp::Add {
            funcs.push(one_func("neg", &[Ty::F64, Ty::F64], Ty::F64, vec![Stmt::Return(Some(Expr { ty: Ty::F64, kind: ExprKind::FNeg(Box::new(a.clone())) }))], 1));
            for c in CMPS {
                funcs.push(one_func("cmp", &[Ty::F64, Ty::F64], Ty::Bool, vec![Stmt::Return(Some(cmp(c, a.clone(), b.clone())))], 1));
                // A comparison as a branch condition, not only as a value.
                funcs.push(one_func(
                    "br",
                    &[Ty::F64, Ty::F64],
                    Ty::I32,
                    vec![
                        Stmt::If { cond: cmp(c, a.clone(), b.clone()), then: vec![Stmt::Return(Some(konst(Ty::I32, 1)))], els: vec![] },
                        Stmt::Return(Some(konst(Ty::I32, 2))),
                    ],
                    1,
                ));
            }
        }
        let n = funcs.len();
        let prog = f64_program("f64", funcs, nsite(Ty::F64));
        let mut calls = Vec::new();
        for fi in 0..n {
            for &x in &vals {
                for &y in &vals {
                    calls.push((fi, vec![x, y]));
                }
            }
        }
        if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
            failures.push(format!("f{}: {}", op.symbol(), e));
        }
    }
    eprintln!("f64 arith: {} programs, {} calls compared ({} returns)", stats.programs, stats.calls, stats.returns);
    assert!(stats.returns > 0);
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// `@floatFromInt` from every integer type at its edge values, and
/// `@intFromFloat` to every integer type at every f64 edge value: in range
/// the same integer (0 for a NaN, as in Zig 0.16 Debug), out of range (the
/// infinities included) the same trap site.
#[test]
fn f64_conversions_match_interpreter() {
    let mut rng = Rng::new(65);
    let mut stats = Stats::default();
    let mut failures = Vec::new();
    let mk = |kind, ty| Site { kind, line: 1, what: String::new(), ty };
    for ty in Ty::INTS {
        let sites = vec![mk(TrapKind::Overflow, ty), mk(TrapKind::NoReturn, ty), mk(TrapKind::FloatToInt, ty)];
        let to_f = Expr { ty: Ty::F64, kind: ExprKind::IntToFloat(Box::new(var(ty, 0))) };
        let to_i = Expr { ty, kind: ExprKind::FloatToInt { arg: Box::new(var(Ty::F64, 0)), site: 2 } };
        // Round trip through f64 and back, mixed with integer arithmetic.
        let back = Expr {
            ty,
            kind: ExprKind::FloatToInt { arg: Box::new(farith(FOp::Mul, to_f.clone(), f64_konst(0.5))), site: 2 },
        };
        let funcs = vec![
            one_func("ff", &[ty], Ty::F64, vec![Stmt::Return(Some(to_f))], 1),
            one_func("fi", &[Ty::F64], ty, vec![Stmt::Return(Some(to_i))], 1),
            one_func("rt", &[ty], ty, vec![Stmt::Return(Some(back))], 1),
        ];
        let prog = f64_program("conv", funcs, sites);
        let mut calls: Vec<(usize, Vec<i128>)> = Vec::new();
        for &v in &edge_values(ty) {
            calls.push((0, vec![v]));
            calls.push((2, vec![v]));
        }
        for &x in &f64_edges() {
            calls.push((1, vec![x]));
        }
        if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
            failures.push(format!("{}: {}", ty.name(), e));
        }
    }
    eprintln!(
        "f64 conversions: {} programs, {} calls compared ({} returns, {} @intFromFloat traps)",
        stats.programs, stats.calls, stats.returns, stats.traps[TrapKind::FloatToInt as usize]
    );
    assert!(stats.traps[TrapKind::FloatToInt as usize] > 0);
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// AAPCS64: F64 and integer parameters interleaved (the d and x registers
/// are numbered per class), all eight d registers in use, and a non-leaf
/// caller passing its arguments on in another order, with the F64 result
/// crossing the call in d0.
#[test]
fn f64_mixed_arguments_and_calls_match_interpreter() {
    let mut rng = Rng::new(66);
    let mut stats = Stats::default();
    let params = [Ty::F64, Ty::I32, Ty::F64, Ty::U8, Ty::F64, Ty::F64, Ty::I64, Ty::F64];
    let v = |i: usize| var(params[i], i);
    let fl = |i: usize| Expr { ty: Ty::F64, kind: ExprKind::IntToFloat(Box::new(v(i))) };
    // callee: ((p0 - p2) * p4 + p5 / p7) + float(p1) - float(p3) * float(p6)
    let body = farith(
        FOp::Sub,
        farith(FOp::Add, farith(FOp::Add, farith(FOp::Mul, farith(FOp::Sub, v(0), v(2)), v(4)), farith(FOp::Div, v(5), v(7))), fl(1)),
        farith(FOp::Mul, fl(3), fl(6)),
    );
    let callee = one_func("callee", &params, Ty::F64, vec![Stmt::Return(Some(body))], 1);
    // caller: callee with its F64 arguments rotated, then the result used
    // after the call alongside a parameter that lives across it.
    let call = Expr {
        ty: Ty::F64,
        kind: ExprKind::Call { func: 0, args: vec![v(7), v(1), v(0), v(3), v(2), v(4), v(6), v(5)] },
    };
    let caller = one_func("caller", &params, Ty::F64, vec![Stmt::Return(Some(farith(FOp::Add, call, v(5))))], 1);
    // Only F64 parameters: all eight d registers.
    let eight = [Ty::F64; 8];
    let w = |i: usize| var(Ty::F64, i);
    let mut sum = w(0);
    for i in 1..8 {
        sum = farith(if i % 2 == 1 { FOp::Sub } else { FOp::Div }, sum, w(i));
    }
    let all_d = one_func("all_d", &eight, Ty::F64, vec![Stmt::Return(Some(sum))], 1);
    let sites = vec![
        Site { kind: TrapKind::Overflow, line: 1, what: String::new(), ty: Ty::F64 },
        Site { kind: TrapKind::NoReturn, line: 1, what: String::new(), ty: Ty::F64 },
    ];
    let prog = f64_program("mixed", vec![callee, caller, all_d], sites);
    let edges = f64_edges();
    let ints = |ty: Ty| edge_values(ty);
    let mut calls = Vec::new();
    for k in 0..400 {
        let mut args = Vec::new();
        for &t in &params {
            args.push(if t == Ty::F64 { edges[rng.below(edges.len())] } else { rng.pick(&ints(t)) });
        }
        calls.push((k % 2, args));
        let all: Vec<i128> = (0..8).map(|_| edges[rng.below(edges.len())]).collect();
        calls.push((2, all));
    }
    let r = compare_calls(&prog, &calls, &mut rng, &mut stats);
    eprintln!("f64 calls: {} calls compared ({} returns)", stats.calls, stats.returns);
    assert!(r.is_ok(), "{}", r.unwrap_err());
    assert_eq!(stats.returns, calls.len());
}

// ------------------------------------------------------------ f64 source

fn f64_lower(src: &str) -> Result<Program, Vec<String>> {
    let parsed = front::parse(std::path::Path::new("/nonexistent/f.t27"), src).map_err(|e| vec![format!("parse: {}", e)])?;
    lower::lower_src(&parsed.ast, OverflowMode::Trap, Some(src)).map_err(|rs| rs.iter().map(|r| r.message()).collect())
}

/// Every test block of `src` in both engines: (name, passed). A block on
/// which they disagree fails the test outright.
fn f64_run(src: &str) -> Vec<(String, Result<(), TrapKind>)> {
    let prog = f64_lower(src).unwrap_or_else(|e| panic!("lowering failed:\n{}", e.join("\n")));
    let code = codegen::compile(&prog, TrapStyle::Jit, true).expect("codegen");
    let mut jit = Jit::load(&code, prog.funcs.len(), &prog.data, &prog.globals).expect("jit load");
    let mut out = Vec::new();
    for (id, f) in prog.tests() {
        let want = Interp::new(&prog).call(id, &[]);
        let got = jit.call(id as FuncId, &[]);
        let o = match (&want, &got) {
            (Ok(_), Ok(_)) => Ok(()),
            (Err(Stop::Trap { site, .. }), Err(t)) if *site == t.site => Err(prog.sites[*site as usize].kind),
            _ => panic!("{}: interpreter {:?}, jit {:?}", f.name, want, got),
        };
        out.push((f.name.clone(), o));
    }
    out
}

// ------------------------------------------------------------ bench blocks

const BENCH_SRC: &str = r#"module b;

fn sq(x: u32) u32 {
    return x * x;
}

test sq_small {
    assert(sq(3) == 9);
}

bench "sq_loop" {
    var acc: u32 = 0;
    var i: u32 = 0;
    while (i < 10) : (i += 1) {
        acc = acc +% sq(i);
    }
    assert(acc == 0);
}

bench sq_colon: sq(2) == 5
"#;

fn lower_text(src: &str) -> Result<Program, Vec<String>> {
    let parsed = t27b::front::parse(std::path::Path::new("/nonexistent/b.t27"), src).map_err(|e| vec![format!("parse: {}", e)])?;
    t27b::lower::lower_src(&parsed.ast, OverflowMode::Trap, Some(src)).map_err(|rs| rs.iter().map(|r| r.message()).collect())
}

/// Every test of `prog` in the JIT and the interpreter: (name, trapped).
fn run_both(prog: &Program) -> Vec<(String, bool)> {
    let code = codegen::compile(prog, TrapStyle::Jit, true).expect("codegen");
    let mut jit = Jit::load(&code, prog.funcs.len(), &prog.data, &prog.globals).expect("jit load");
    let mut out = Vec::new();
    for (id, f) in prog.tests() {
        let want = Interp::new(prog).call(id, &[]);
        let got = jit.call(id as FuncId, &[]);
        let trapped = match (&want, &got) {
            (Ok(_), Ok(_)) => false,
            (Err(Stop::Trap { site, .. }), Err(t)) if *site == t.site => true,
            _ => panic!("{}: interpreter {:?}, jit {:?}", f.name, want, got),
        };
        out.push((f.name.clone(), trapped));
    }
    out
}

/// f64 from source: literals, `+ - * /`, negation, comparisons, f64
/// parameters, results, locals and constants, and the two conversions in
/// Zig's spelling with the result type from context. Every block's verdict
/// is the one `t27c gen` + `zig test` gives for the same text.
#[test]
fn f64_source_programs_run_in_both_engines() {
    let src = "module f64src;

const PHI: f64 = 1.618033988749895;
const THIRD: f64 = 1.0 / 3.0;
const QUARTER = 0.25;

fn half(x: f64) f64 {
    return x / 2.0;
}

fn mix(n: i32, x: f64) i32 {
    const y: f64 = @as(f64, @floatFromInt(n)) * x;
    var z: f64 = -y + 0.5;
    z = z * 2.0;
    return @intFromFloat(z);
}

fn many(a: f64, b: i64, c: f64, d: u8, e: f64) f64 {
    return a - c * e + @as(f64, @floatFromInt(b)) / @as(f64, @floatFromInt(d));
}

fn to_i8(x: f64) i8 {
    return @intFromFloat(x);
}

fn nan() f64 {
    const z: f64 = 0.0;
    return z / z;
}

test half_works {
    assert(half(3.0) == 1.5);
    assert(mix(3, 1.5) == -8);
    assert(PHI * PHI - PHI > 0.99);
    assert(THIRD < 0.34);
    assert(many(2.0, 3, 2.0, 4, 0.5) + QUARTER == 2.0);
    const q: f64 = 0.5 * 3.0;
    assert(q == 1.5);
    assert(QUARTER * 4.0 == 1.0);
    assert_eq(half(-0.0), 0.0);
    assert(nan() != nan());
    assert(to_i8(-128.9) == -128);
}

test out_of_range {
    assert(to_i8(128.0) == 0);
}

test nan_to_int {
    assert(to_i8(nan()) == 0);
}
";
    let r = f64_run(src);
    let got: Vec<(&str, Result<(), TrapKind>)> = r.iter().map(|(n, o)| (n.as_str(), *o)).collect();
    assert_eq!(
        got,
        vec![
            ("half_works", Ok(())),
            ("out_of_range", Err(TrapKind::FloatToInt)),
            ("nan_to_int", Ok(())),
        ]
    );
}

/// What stays refused, each named: `@sqrt` of a literal, `std.math.*`, a
/// conversion with no result type, f16, `as` from an f64 t27c gen does not
/// spell as a float (here a call; a spelled one is lowered, see
/// `float_as.t27`) or from a bool to f64 (an integer `as f64` is
/// `@floatFromInt`, see `source.rs`), a folded value
/// past the f64 range, and `x * 2^k` on f64 (t27c gen rewrites it into a
/// shift that cannot compile).
#[test]
fn f64_refusals_name_the_construct() {
    let first = |body: &str| -> String {
        let src = format!("module f64rej;\nfn f(x: f64, n: i32) f64 {{\n{}\n}}\n", body);
        match f64_lower(&src) {
            Ok(_) => panic!("expected a rejection for {}", body),
            Err(e) => e[0].clone(),
        }
    };
    for (body, want) in [
        ("return @sqrt(2.0) + x;", "ExprCall(@sqrt)"),
        ("return std.math.sqrt(x);", "ExprCall(std.*)"),
        ("return @floatFromInt(n) + x;", "ExprCall(@floatFromInt)"),
        ("const y: f16 = 1.0;\nreturn x;", "type f16"),
        ("return f(x, n) as f64;", "ExprCast(f64)"),
        ("const k: i32 = f(x, n) as i32;\nreturn x;", "ExprCast(f64)"),
        ("return (n > 0) as f64;", "ExprCast(f64)"),
        ("return x + 1e308 * 10.0;", "literal out of range"),
        ("return x + 1.0 / 0.0;", "ExprBinary"),
        ("return x * 2;", "ExprBinary(f64 * 2^k)"),
        ("return x % 2.0;", "ExprBinary(%)"),
        ("return x + n;", "type mismatch"),
    ] {
        let msg = first(body);
        assert!(msg.contains(&format!("unsupported construct {} ", want)), "{}: {}", body, msg);
    }
    // Literals fold whether or not they are an f64 exactly (in binary128,
    // see `comptime_floats_fold_in_binary128`): 1e22 is one, 1e23 is not.
    assert!(f64_lower("module ok;\nfn f() f64 {\nreturn 1e22 * 0.5 + 0.5 * 3.0;\n}\n").is_ok());
    assert!(f64_lower("module ok;\nfn f(x: f64) f64 {\nreturn x + 1e23 * 1.0 + 0.1 * 3.0;\n}\n").is_ok());
}

// ------------------------------------------------------------------ f32

/// Binary32 edge values as IR values (zero-extended bit patterns): signed
/// zeros, infinities, NaNs with payloads, subnormals, 2^24 (the integer
/// exactness edge) and the bounds of every `@intFromFloat`.
fn f32_edges() -> Vec<i128> {
    let mut v: Vec<i128> = [
        0.0f32, -0.0, 1.0, -1.0, 0.5, 1.5, 2.0, 3.0, 0.1, 1.0 / 3.0, -2.5, 127.99, 128.0, -128.5, -129.0, 255.5,
        256.0, 65535.75, 16777216.0, 16777218.0, -16777216.0, 2147483520.0, 2147483648.0, -2147483648.0,
        -2147483904.0, 4294967040.0, 4294967296.0, 9223371487098961920.0, 9223372036854775808.0,
        18446742974197923840.0, 18446744073709551616.0, 1.0e30, -1.0e30, 1.0e-30, f32::MAX, f32::MIN,
        f32::MIN_POSITIVE, f32::EPSILON, f32::INFINITY, f32::NEG_INFINITY,
    ]
    .iter()
    .map(|&x: &f32| f32_bits(x))
    .collect();
    v.extend([
        1,           // smallest subnormal
        0x8000_0001, // its negation
        0x007f_ffff, // largest subnormal
        0x7fc0_0000, // the default quiet NaN
        0xffc0_0000, // negative quiet NaN
        0x7fc0_beef, // quiet NaN with a payload
        0x7f80_0001, // signalling NaN
    ]);
    v
}

fn f32_konst(x: f32) -> Expr {
    konst(Ty::F32, f32_bits(x))
}

fn farith32(op: FOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr { ty: Ty::F32, kind: ExprKind::FArith { op, lhs: Box::new(lhs), rhs: Box::new(rhs) } }
}

fn fcast(to: Ty, e: Expr) -> Expr {
    Expr { ty: to, kind: ExprKind::FloatCast(Box::new(e)) }
}

/// `+ - * /`, negation and the six comparisons on every pair of f32 edge
/// values, in s registers and as constants: the JIT's s0 (or x0 for a
/// bool) must be the interpreter's bit pattern exactly.
#[test]
fn f32_arith_and_compare_match_interpreter() {
    let mut rng = Rng::new(32);
    let mut stats = Stats::default();
    let mut failures = Vec::new();
    let vals = f32_edges();
    let nsite = |ty| vec![Site { kind: TrapKind::Overflow, line: 1, what: String::new(), ty }, Site { kind: TrapKind::NoReturn, line: 1, what: String::new(), ty }];
    for op in [FOp::Add, FOp::Sub, FOp::Mul, FOp::Div] {
        let (a, b) = (var(Ty::F32, 0), var(Ty::F32, 1));
        let mut funcs = vec![
            one_func("f", &[Ty::F32, Ty::F32], Ty::F32, vec![Stmt::Return(Some(farith32(op, a.clone(), b.clone())))], 1),
            one_func("fk", &[Ty::F32, Ty::F32], Ty::F32, vec![Stmt::Return(Some(farith32(op, a.clone(), f32_konst(0.1))))], 1),
            one_func("kf", &[Ty::F32, Ty::F32], Ty::F32, vec![Stmt::Return(Some(farith32(op, f32_konst(-3.0), b.clone())))], 1),
            one_func(
                "tree",
                &[Ty::F32, Ty::F32],
                Ty::F32,
                vec![Stmt::Return(Some(farith32(
                    op,
                    farith32(FOp::Mul, a.clone(), b.clone()),
                    Expr { ty: Ty::F32, kind: ExprKind::FNeg(Box::new(farith32(op, b.clone(), a.clone()))) },
                )))],
                1,
            ),
            // The same op in f64 on the widened operands, narrowed back.
            one_func(
                "wide",
                &[Ty::F32, Ty::F32],
                Ty::F32,
                vec![Stmt::Return(Some(fcast(Ty::F32, farith(op, fcast(Ty::F64, a.clone()), fcast(Ty::F64, b.clone())))))],
                1,
            ),
        ];
        if op == FOp::Add {
            funcs.push(one_func("neg", &[Ty::F32, Ty::F32], Ty::F32, vec![Stmt::Return(Some(Expr { ty: Ty::F32, kind: ExprKind::FNeg(Box::new(a.clone())) }))], 1));
            for c in CMPS {
                funcs.push(one_func("cmp", &[Ty::F32, Ty::F32], Ty::Bool, vec![Stmt::Return(Some(cmp(c, a.clone(), b.clone())))], 1));
                funcs.push(one_func(
                    "br",
                    &[Ty::F32, Ty::F32],
                    Ty::I32,
                    vec![
                        Stmt::If { cond: cmp(c, a.clone(), b.clone()), then: vec![Stmt::Return(Some(konst(Ty::I32, 1)))], els: vec![] },
                        Stmt::Return(Some(konst(Ty::I32, 2))),
                    ],
                    1,
                ));
            }
        }
        let n = funcs.len();
        let prog = f64_program("f32", funcs, nsite(Ty::F32));
        let mut calls = Vec::new();
        for fi in 0..n {
            for &x in &vals {
                for &y in &vals {
                    calls.push((fi, vec![x, y]));
                }
            }
        }
        if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
            failures.push(format!("f{}: {}", op.symbol(), e));
        }
    }
    eprintln!("f32 arith: {} programs, {} calls compared ({} returns)", stats.programs, stats.calls, stats.returns);
    assert!(stats.returns > 0);
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// `@floatFromInt` to f32 from every integer type at its edge values (one
/// rounding, straight from the integer), `@intFromFloat` from every f32
/// edge value, and `@floatCast` both ways over the f32 and f64 edges.
#[test]
fn f32_conversions_match_interpreter() {
    let mut rng = Rng::new(33);
    let mut stats = Stats::default();
    let mut failures = Vec::new();
    let mk = |kind, ty| Site { kind, line: 1, what: String::new(), ty };
    for ty in Ty::INTS {
        let sites = vec![mk(TrapKind::Overflow, ty), mk(TrapKind::NoReturn, ty), mk(TrapKind::FloatToInt, ty)];
        let to_f = Expr { ty: Ty::F32, kind: ExprKind::IntToFloat(Box::new(var(ty, 0))) };
        let to_i = Expr { ty, kind: ExprKind::FloatToInt { arg: Box::new(var(Ty::F32, 0)), site: 2 } };
        let back = Expr {
            ty,
            kind: ExprKind::FloatToInt { arg: Box::new(farith32(FOp::Mul, to_f.clone(), f32_konst(0.5))), site: 2 },
        };
        let funcs = vec![
            one_func("ff", &[ty], Ty::F32, vec![Stmt::Return(Some(to_f))], 1),
            one_func("fi", &[Ty::F32], ty, vec![Stmt::Return(Some(to_i))], 1),
            one_func("rt", &[ty], ty, vec![Stmt::Return(Some(back))], 1),
        ];
        let prog = f64_program("conv32", funcs, sites);
        let mut calls: Vec<(usize, Vec<i128>)> = Vec::new();
        for &v in &edge_values(ty) {
            calls.push((0, vec![v]));
            calls.push((2, vec![v]));
        }
        for &x in &f32_edges() {
            calls.push((1, vec![x]));
        }
        if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
            failures.push(format!("{}: {}", ty.name(), e));
        }
    }
    let sites = vec![mk(TrapKind::Overflow, Ty::F32), mk(TrapKind::NoReturn, Ty::F32)];
    let funcs = vec![
        one_func("narrow", &[Ty::F64], Ty::F32, vec![Stmt::Return(Some(fcast(Ty::F32, var(Ty::F64, 0))))], 1),
        one_func("widen", &[Ty::F32], Ty::F64, vec![Stmt::Return(Some(fcast(Ty::F64, var(Ty::F32, 0))))], 1),
    ];
    let prog = f64_program("cast32", funcs, sites);
    let mut calls: Vec<(usize, Vec<i128>)> = Vec::new();
    let mut narrow_in = f64_edges();
    // Every f32 edge widened, and the f32 midpoints next to some of them.
    for &x in &f32_edges() {
        let w = f32_of(x) as f64;
        narrow_in.push(f64_bits(w));
        if w.is_finite() {
            let up = f32_of(x).next_up() as f64;
            narrow_in.push(f64_bits((w + up) / 2.0));
        }
    }
    for &x in &narrow_in {
        calls.push((0, vec![x]));
    }
    for &x in &f32_edges() {
        calls.push((1, vec![x]));
    }
    if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
        failures.push(format!("floatCast: {}", e));
    }
    eprintln!(
        "f32 conversions: {} programs, {} calls compared ({} returns, {} @intFromFloat traps)",
        stats.programs, stats.calls, stats.returns, stats.traps[TrapKind::FloatToInt as usize]
    );
    assert!(stats.traps[TrapKind::FloatToInt as usize] > 0);
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
}

/// AAPCS64 with f32 and f64 parameters interleaved with integers (s_k and
/// d_k are the same register), and a non-leaf caller passing them on
/// rotated, the f32 result crossing the call in s0.
#[test]
fn f32_mixed_arguments_and_calls_match_interpreter() {
    let mut rng = Rng::new(34);
    let mut stats = Stats::default();
    let params = [Ty::F32, Ty::I32, Ty::F64, Ty::U8, Ty::F32, Ty::F32, Ty::I64, Ty::F64];
    let v = |i: usize| var(params[i], i);
    let w = |i: usize| if params[i] == Ty::F32 { fcast(Ty::F64, v(i)) } else { v(i) };
    let fl = |i: usize| Expr { ty: Ty::F32, kind: ExprKind::IntToFloat(Box::new(v(i))) };
    // callee: f32(((p0 - p2) * p4 + p5 / p7)) + float32(p1) - float32(p3) * float32(p6)
    let wide = farith(FOp::Add, farith(FOp::Mul, farith(FOp::Sub, w(0), w(2)), w(4)), farith(FOp::Div, w(5), w(7)));
    let body = farith32(FOp::Sub, farith32(FOp::Add, fcast(Ty::F32, wide), fl(1)), farith32(FOp::Mul, fl(3), fl(6)));
    let callee = one_func("callee", &params, Ty::F32, vec![Stmt::Return(Some(body))], 1);
    let call = Expr {
        ty: Ty::F32,
        kind: ExprKind::Call { func: 0, args: vec![v(5), v(1), v(7), v(3), v(0), v(4), v(6), v(2)] },
    };
    let caller = one_func("caller", &params, Ty::F32, vec![Stmt::Return(Some(farith32(FOp::Add, call, v(5))))], 1);
    let sites = vec![
        Site { kind: TrapKind::Overflow, line: 1, what: String::new(), ty: Ty::F32 },
        Site { kind: TrapKind::NoReturn, line: 1, what: String::new(), ty: Ty::F32 },
    ];
    let prog = f64_program("mixed32", vec![callee, caller], sites);
    let (e32, e64) = (f32_edges(), f64_edges());
    let mut calls = Vec::new();
    for k in 0..400 {
        let mut args = Vec::new();
        for &t in &params {
            args.push(match t {
                Ty::F32 => e32[rng.below(e32.len())],
                Ty::F64 => e64[rng.below(e64.len())],
                t => rng.pick(&edge_values(t)),
            });
        }
        calls.push((k % 2, args));
    }
    let r = compare_calls(&prog, &calls, &mut rng, &mut stats);
    eprintln!("f32 calls: {} calls compared ({} returns)", stats.calls, stats.returns);
    assert!(r.is_ok(), "{}", r.unwrap_err());
    assert_eq!(stats.returns, calls.len());
}

/// f32 from source: literals rounded like Zig's comptime_float -> f32,
/// arithmetic, comparisons with a literal (coerced to f32), widening to f64
/// as a peer, `@floatCast`, and the conversions. The verdicts are the ones
/// `t27c gen` + `zig test` (Zig 0.16) gives for the same text.
#[test]
fn f32_source_programs_run_in_both_engines() {
    let src = "module f32src;

const K: f32 = 0.1;
const BIG: f32 = 1e39;
const WIDE: f64 = 0.1;

fn half(x: f32) f32 {
    return x / 2.0;
}

fn widen(x: f32) f64 {
    return x;
}

fn mix(a: f32, n: i32, b: f64, m: u8, c: f32) f64 {
    const s: f32 = a * @as(f32, @floatFromInt(n)) - c;
    return s + b * @as(f64, @floatFromInt(m));
}

fn narrow(x: f64) f32 {
    return @floatCast(x);
}

fn to_i8(x: f32) i8 {
    return @intFromFloat(x);
}

fn big_int(n: i64) f32 {
    return @floatFromInt(n);
}

test f32_works {
    assert(half(3.0) == 1.5);
    const w: f64 = widen(K);
    assert(w == 0.100000001490116119384765625);
    assert(w != WIDE);
    assert(K == 0.1);
    assert(widen(BIG) > 1e308);
    assert(mix(1.5, 3, 0.25, 4, 0.5) == 5.0);
    assert(narrow(0.1) == K);
    assert(widen(narrow(1e300)) > 1e308);
    assert(-K < 0.0);
    assert(to_i8(-128.9) == -128);
    const big: f64 = widen(big_int(9007199254740993));
    assert(big == 9007199254740992.0);
    const t: f32 = 16777216.0;
    assert(widen(t + 1.0) == 16777216.0);
    assert(widen(@as(f32, 0.1) + 0.2) == 0.300000011920928955078125);
}

test f32_out_of_range {
    assert(to_i8(128.0) == 0);
}
";
    let r = f64_run(src);
    let got: Vec<(&str, Result<(), TrapKind>)> = r.iter().map(|(n, o)| (n.as_str(), *o)).collect();
    assert_eq!(got, vec![("f32_works", Ok(())), ("f32_out_of_range", Err(TrapKind::FloatToInt))]);
}

/// What stays refused for f32, each named: an integer literal that is not
/// exactly an f32 (a Zig compile error), `as` from a float t27c gen does
/// not spell as one (a call), and
/// `@floatCast` of a literal or with no result type. (A literal one f64
/// apart from an f32 midpoint rounds from its binary128 value, see
/// `comptime_floats_fold_in_binary128`.)
#[test]
fn f32_refusals_name_the_construct() {
    let first = |body: &str| -> String {
        let src = format!("module f32rej;\nfn f(x: f32, n: i32) f32 {{\n{}\n}}\n", body);
        match f64_lower(&src) {
            Ok(_) => panic!("expected a rejection for {}", body),
            Err(e) => e[0].clone(),
        }
    };
    for (body, want) in [
        ("return 16777217;", "literal out of range"),
        ("return f(x, n) as f32;", "ExprCast(f32)"),
        ("const k: i32 = f(x, n) as i32;\nreturn x;", "ExprCast(f32)"),
        ("return @floatCast(0.5);", "ExprCall(@floatCast)"),
        ("return @floatCast(x) + x;", "ExprCall(@floatCast)"),
        ("return x * 4;", "ExprBinary(f64 * 2^k)"),
        ("return x + n;", "type mismatch"),
    ] {
        let msg = first(body);
        assert!(msg.contains(&format!("unsupported construct {} ", want)), "{}: {}", body, msg);
    }
    // Exact or plainly rounded literals are fine: 16777216, 0.1, and the
    // midpoints 2^24 + 1 and 2^24 + 3 written as exact float literals
    // (ties to even: 0x4b800000 and 0x4b800002 in Zig too).
    for body in ["return 16777216;", "return 0.1 + x;", "return 16777217.0;", "return 16777219.0 - x;", "return n as f32;"] {
        let src = format!("module f32ok;\nfn f(x: f32, n: i32) f32 {{\n{}\n}}\n", body);
        assert!(f64_lower(&src).is_ok(), "{}", body);
    }
}

/// Zig keeps a comptime_float in binary128: `+ - * /` of two of them round
/// to binary128, they compare as binary128 values, and a typed f64 or f32
/// takes one rounding of the result. Every value below was checked with
/// exact rational arithmetic, and the same text passes `t27c gen` + `zig
/// test` 6/6 with every assert run at run time (the conformance spec
/// `specs/tri/t27b/conformance/comptime_float.t27` holds the first cases).
#[test]
fn comptime_floats_fold_in_binary128() {
    let src = "module cfwide;

fn add(x: f64, y: f64) f64 {
    return x + y;
}

fn sub(x: f64, y: f64) f64 {
    return x - y;
}

fn mul(x: f64, y: f64) f64 {
    return x * y;
}

fn same(b: bool) bool {
    return b;
}

fn widen(x: f32) f64 {
    return x;
}

fn to_i32(x: f64) i32 {
    return @intFromFloat(x);
}

test past_f64_and_back {
    const big: f64 = 1e308 * 10.0 / 10.0;
    assert(add(big, 0.0) == 1e308);
    assert(mul(mul(1e308, 10.0), 0.1) > 1e308);
}

test subnormal_results_round_once {
    const half: f64 = 5e-324 * 0.5;
    const quarter: f64 = 5e-324 * 0.25;
    assert(add(half, 0.0) == 5e-324);
    assert(add(quarter, 0.0) == 0.0);
}

test comptime_ints_take_part {
    assert(same(0.1 * 10.0 == 1));
    assert(same(0.1 * 3.0 != 0.3));
    const one: f64 = (1.0 / 3.0) * 3.0;
    assert(add(one, 0.0) == 1.0);
}

test differences_and_squares_fold {
    const d: f64 = 0.3 - 0.1;
    assert(add(d, 0.0) == 0.2);
    assert(sub(0.3, 0.1) < 0.2);
    const sq: f64 = 1.1 * 1.1;
    assert(add(sq, 0.0) == 1.21);
    assert(mul(1.1, 1.1) > 1.21);
}

test f32_rounds_once_from_binary128 {
    const f: f32 = 0.1 + 0.2;
    assert(widen(f) == 0.300000011920928955078125);
    const up: f32 = 1.0000000596046447753906250001;
    const down: f32 = 1.00000005960464477539062499;
    assert(widen(up) == 1.00000011920928955078125);
    assert(widen(down) == 1.0);
}

test exact_folds_convert {
    assert(to_i32(0.5 * 4.0) == 2);
    const k: i32 = @intFromFloat(0.5 * 4.0);
    assert(same(k == 2));
}
";
    let r = f64_run(src);
    assert_eq!(r.len(), 6, "{:?}", r);
    for (name, o) in &r {
        assert_eq!(*o, Ok(()), "{}", name);
    }
    // The same asserts with the folded and the run-time values swapped fail.
    for (from, to) in [
        ("assert(add(big, 0.0) == 1e308);", "assert(add(big, 0.0) != 1e308);"),
        ("assert(add(quarter, 0.0) == 0.0);", "assert(add(quarter, 0.0) == 5e-324);"),
        ("assert(same(0.1 * 3.0 != 0.3));", "assert(same(0.1 * 3.0 == 0.3));"),
        ("assert(add(d, 0.0) == 0.2);", "assert(add(d, 0.0) < 0.2);"),
        ("assert(widen(up) == 1.00000011920928955078125);", "assert(widen(up) == 1.0);"),
    ] {
        assert!(src.contains(from), "{}", from);
        let r = f64_run(&src.replace(from, to));
        assert_eq!(r.iter().filter(|(_, o)| o.is_err()).count(), 1, "{}: {:?}", to, r);
    }
}

/// t27c emits a bench as `fn bench_<name>() void { ... }` that nothing calls,
/// so `zig test` neither runs nor counts it. t27b compiles the body and leaves
/// it out of the tests: the failing asserts in both benches here do not fail
/// the file. The same bodies, renamed to tests, run and trap identically in
/// the JIT and the interpreter, so what was compiled is the body as written.
#[test]
fn bench_bodies_compile_but_never_run() {
    let prog = lower_text(BENCH_SRC).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    assert_eq!(run_both(&prog), vec![("sq_small".to_string(), false)]);
    assert!(prog.funcs.iter().all(|f| !f.name.starts_with("sq_loop") && !f.name.starts_with("sq_colon")));

    let as_tests = BENCH_SRC.replace("bench \"sq_loop\"", "test \"sq_loop\"").replace("bench sq_colon: sq(2) == 5", "invariant sq_colon: sq(2) == 5");
    let prog = lower_text(&as_tests).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    let r = run_both(&prog);
    assert_eq!(r.len(), 3, "{:?}", r);
    assert!(r.iter().any(|(n, t)| n == "sq_small" && !t), "{:?}", r);
    assert!(r.iter().any(|(n, t)| n == "sq_loop" && *t), "{:?}", r);
    assert!(r.iter().any(|(n, t)| n == "sq_colon" && *t), "{:?}", r);
}

/// A bench body t27b cannot lower rejects the file under the construct it
/// contains, never under `BenchBlock`.
#[test]
fn bench_unlowerable_body_names_its_construct() {
    let src = BENCH_SRC.replace(
        "    var i: u32 = 0;\n    while (i < 10) : (i += 1) {\n        acc = acc +% sq(i);\n    }",
        "    for (0..10, 0..10) |i, j| {\n        acc = acc +% sq(@as(u32, i + j));\n    }",
    );
    let e = lower_text(&src).err().expect("a rejection");
    assert!(e[0].starts_with("t27b: unsupported construct StmtFor(multi-object) at line "), "{:?}", e);
    assert!(e.iter().all(|m| !m.contains("BenchBlock")), "{:?}", e);
}

/// A bench written as prose clauses (`measure: nanoseconds to f(x)`,
/// `target: < 10ns`) reaches lowering as childless StmtExpr nodes holding the
/// text. t27c writes `// NOT LOWERED: empty statement` for each, so the bench
/// compiles to an empty body and the file's tests run as before.
#[test]
fn bench_prose_clauses_lower_to_nothing() {
    let src = "module bp;

fn sq(x: u32) u32 {
    return x * x;
}

test sq_small {
    assert(sq(3) == 9);
}

bench sq_latency
    measure: nanoseconds to sq(sq(2)) when 2 bytes available
    target: < 10ns
";
    let prog = lower_text(src).unwrap_or_else(|e| panic!("{}", e.join("\n")));
    assert_eq!(run_both(&prog), vec![("sq_small".to_string(), false)]);
}

// ------------------------------------------------------------ shift source

/// `<<` and `>>` from source, in the shapes t27c's Zig backend emits: a
/// non-literal amount goes through `@intCast` to `Log2Int(T)` (a negative or
/// too-wide amount traps), and an untyped literal on the left is pinned with
/// `@as(T, lit)`: T is the declared integer type of the local being
/// initialized, else u32, else u64 when the literal does not fit u32 -- even
/// when the amount is a named constant (`0xFF << K` wraps in u32). `<<`
/// drops the bits that leave the type (no `@shlExact`); `>>` is arithmetic on
/// signed types and logical on unsigned ones. Every block's verdict here is
/// the one `t27c gen` + `zig test` (Zig 0.16) gives for the same text.
const SHIFT_SRC: &str = r#"module shiftsrc;

const K = 28;

fn pow2(d: i32) i32 {
    var hf: i32 = 1 << d;
    return hf;
}

fn mask(width: u32) u32 {
    return (1 << width) - 1;
}

fn wide(d: u32) u64 {
    var m: u64 = 1 << d;
    return m;
}

fn big(d: u32) u64 {
    return 0x100000000 << d;
}

fn shl_i8(x: i8, n: u8) i8 {
    return x << n;
}

fn sar(x: i32, n: i32) i32 {
    return x >> n;
}

fn shr(x: u32, n: u32) u32 {
    return x >> n;
}

fn named() u64 {
    return 0xFF << K;
}

fn folded() u32 {
    const s = 3;
    return 1 << (s + 2);
}

fn folded_right() u32 {
    return 0x100 >> (2 + 2);
}

fn compound(x: u16, n: u16) u16 {
    var y: u16 = x;
    y = y << n;
    y = y >> 1;
    return y;
}

test shift_basics {
    assert(pow2(0) == 1);
    assert(pow2(31) == -2147483648);
    assert(mask(8) == 255);
    assert(mask(0) == 0);
    assert(wide(40) == 1099511627776);
    assert(big(4) == 68719476736);
    assert(shl_i8(3, 6) == -64);
    assert(sar(-8, 1) == -4);
    assert(sar(-1, 31) == -1);
    assert(shr(0x80000000, 31) == 1);
    assert(named() == 0xF0000000);
    assert(compound(3, 15) == 0x4000);
    assert(folded() == 32);
    assert(folded_right() == 16);
}

test shift_amount_too_wide {
    assert(pow2(32) == 0);
}

test shift_negative_amount {
    assert(sar(1, -1) == 0);
}

test shift_literal_pinned_to_u32 {
    assert(mask(32) == 0);
}
"#;

#[test]
fn shift_source_programs_run_in_both_engines() {
    let r = f64_run(SHIFT_SRC);
    let got: Vec<(&str, Result<(), TrapKind>)> = r.iter().map(|(n, o)| (n.as_str(), *o)).collect();
    assert_eq!(
        got,
        vec![
            ("shift_basics", Ok(())),
            ("shift_amount_too_wide", Err(TrapKind::ShiftRange)),
            ("shift_negative_amount", Err(TrapKind::ShiftRange)),
            ("shift_literal_pinned_to_u32", Err(TrapKind::ShiftRange)),
        ]
    );
}

/// Shift shapes refused by name. A constant amount out of range for the left
/// type (a literal or a typed constant) is a Zig compile error. An untyped
/// left operand that is not a literal (`(1 + 1) << n`) has a width only if
/// t27c's optimizer happens to fold it, which depends on where it stands, so
/// it is refused. So is an untyped literal shifted by a literal-only amount
/// whose value differs between the folded (comptime_int) and the pinned
/// (`@as(u32, ..)`) readings.
#[test]
fn shift_refusals_name_the_construct() {
    let first = |body: &str| -> String {
        let src = format!("module shiftrej;\nconst KB: u32 = 40;\nfn f(x: u32, n: u32) u32 {{\n{}\n}}\n", body);
        match f64_lower(&src) {
            Ok(_) => panic!("expected a rejection for {}", body),
            Err(e) => e[0].clone(),
        }
    };
    for (body, want) in [
        ("return x << 32;", "shift amount 32 out of range for u32"),
        ("return x >> KB;", "shift amount 40 out of range for u32"),
        ("return 1 << KB;", "shift amount 40 out of range for u32"),
        ("return (1 + 1) << n;", "untyped literal shifted by a runtime amount"),
        ("return 0xFF << (20 + 8);", "width depends on t27c constant folding"),
    ] {
        let msg = first(body);
        assert!(msg.contains("unsupported construct ExprBinary(<< >>) "), "{}: {}", body, msg);
        assert!(msg.contains(want), "{}: {}", body, msg);
    }
    // In range, a typed constant amount folds like a literal one.
    assert!(f64_lower("module ok;\nconst KS: u32 = 31;\nfn f(x: u32) u32 {\nreturn (x >> KS) + (1 << KS);\n}\n").is_ok());
}
