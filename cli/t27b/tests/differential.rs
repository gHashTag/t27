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
    Program { module: "rnd".into(), funcs, sites: g.sites, mode, unchecked: Vec::new(), data, internal_abi: Vec::new() }
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
        ExprKind::Slot(k) => format!("&s{}", k),
        ExprKind::Data(k) => format!("&d{}", k),
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
    let mut jit = Jit::load(&code, prog.funcs.len(), &prog.data)?;
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
    for ty in Ty::INTS {
        let vals = edge_values(ty);
        for op in OPS {
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
                let prog = Program { module: "edge".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), internal_abi: Vec::new() };
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
    for ty in ALL {
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
            let prog = Program { module: "cmp".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), internal_abi: Vec::new() };
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
        let prog = Program { module: "unary".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), internal_abi: Vec::new() };
        let calls: Vec<(usize, Vec<i128>)> =
            (0..n).flat_map(|f| vals.iter().map(move |&a| (f, vec![a]))).collect();
        if let Err(e) = compare_calls(&prog, &calls, &mut rng, &mut stats) {
            failures.push(format!("unary on {}: {}", ty.name(), e));
        }
        // Widening into every wider type.
        for to in Ty::INTS {
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
            let prog = Program { module: "widen".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), internal_abi: Vec::new() };
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
    for from in ALL {
        let vals = edge_values(from);
        for to in Ty::INTS {
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
                let prog = Program { module: "cast".into(), funcs, sites, mode: OverflowMode::Trap, unchecked: Vec::new(), data: Vec::new(), internal_abi: Vec::new() };
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
    let mut jit = Jit::load(&code, prog.funcs.len(), &prog.data)?;
    stats.programs += 1;
    let mut bad = Vec::new();
    for (fi, args) in calls {
        let f = &prog.funcs[*fi];
        let ptys: Vec<Ty> = f.vars[..f.nparams].iter().map(|v| v.ty).collect();
        let want = Interp::new(prog).call(*fi, args);
        let raw: Vec<u64> = args.iter().zip(&ptys).map(|(&v, &t)| raw_arg(rng, v, t)).collect();
        let got = jit.call(*fi as FuncId, &raw);
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
