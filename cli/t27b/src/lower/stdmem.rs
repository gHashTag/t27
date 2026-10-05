//! Two `std.mem` calls on strings, as t27c's Zig backend prints them
//! (verbatim) and Zig 0.16 computes them:
//!
//! - `std.mem.eql(u8, a, b)`: equal lengths and equal bytes, exactly `a == b`
//!   on two `str` values (two literals fold; otherwise `__t27b_str_eql`);
//! - `std.mem.indexOf(u8, hay, needle) == null` / `!= null`: whether
//!   `needle` occurs in `hay` (`std.mem.find`: a longer needle is never
//!   found, an empty one always is, at 0), through the synthesized
//!   `__t27b_str_contains`. The index itself (a `?usize` used any other
//!   way) stays refused as `ExprCall(std.*)`.
//!
//! The element type must be the identifier `u8`, and both slices `str`
//! (`[]const u8`); anything else stays refused.

use super::{LTy, Lower, Val, R, STR_EQL};
use crate::compiler::{Node, NodeKind};
use crate::ir::*;

pub const STR_CONTAINS: &str = "__t27b_str_contains";

impl<'a> Lower<'a> {
    /// `std.mem.eql(u8, a, b)` as a value; None for any other call.
    pub(super) fn std_mem_call(&mut self, c: &Node) -> R<Option<Val>> {
        if c.name != "std.mem.eql" {
            return Ok(None);
        }
        let (a, b) = self.u8_slices(c)?;
        self.str_eq(false, a, b).map(Some)
    }

    /// `std.mem.indexOf(u8, hay, needle)` compared with `null` (`op` is
    /// `==` or `!=`); None when `c` is not that call.
    pub(super) fn index_of_null(&mut self, op: &str, c: &Node) -> R<Option<Val>> {
        if c.kind != NodeKind::ExprCall || c.name != "std.mem.indexOf" {
            return Ok(None);
        }
        self.see(c);
        let (hay, needle) = self.u8_slices(c)?;
        let mut args = Vec::new();
        for v in [hay, needle] {
            match self.coerce_to(v, &LTy::Str)? {
                Val::M(p) => args.push(super::addr_of(&p)),
                _ => return Err(()),
            }
        }
        self.contains_used = true;
        let found = Expr { ty: Ty::Bool, kind: ExprKind::Call { func: self.nfuncs + 1, args } };
        Ok(Some(Val::E(if op == "==" { Expr { ty: Ty::Bool, kind: ExprKind::Not(Box::new(found)) } } else { found })))
    }

    /// The two `str` operands of `std.mem.<f>(u8, x, y)`.
    fn u8_slices(&mut self, c: &Node) -> R<(Val, Val)> {
        self.see(c);
        let a = &c.children;
        if a.len() != 3 || a[0].kind != NodeKind::ExprIdentifier || a[0].name != "u8" {
            return self.reject("ExprCall(std.*)", format!("call to `{}` other than (u8, str, str)", c.name));
        }
        let (x, y) = (self.expr(&a[1])?, self.expr(&a[2])?);
        for v in [&x, &y] {
            if !self.is_str(v) {
                let d = self.val_desc(v);
                return self.reject("ExprCall(std.*)", format!("`{}` on {}, not a string", c.name, d));
            }
        }
        Ok((x, y))
    }

    /// The fns lowering synthesizes, after the source fns (ids `nfuncs`
    /// and `nfuncs + 1`): `__t27b_str_eql` when either is used (the second
    /// id needs the first taken), `__t27b_str_contains` when it is.
    pub(super) fn synthesized(&mut self, funcs: &mut Vec<Func>) {
        if self.eql_used || self.contains_used {
            // Source fns are ids 0..nfuncs and come first in `funcs`, in order.
            let site = self.site(TrapKind::NoReturn, format!("end of fn {}", STR_EQL), Ty::Bool);
            funcs.insert(self.nfuncs as usize, super::str_eql_func(site));
            self.internal_abi.push(self.nfuncs);
        }
        if self.contains_used {
            let site = self.site(TrapKind::NoReturn, format!("end of fn {}", STR_CONTAINS), Ty::Bool);
            funcs.insert(self.nfuncs as usize + 1, str_contains_func(site));
            self.internal_abi.push(self.nfuncs + 1);
        }
    }
}

/// `__t27b_str_contains(hay: *const str, needle: *const str) bool`:
/// `std.mem.indexOf(u8, hay, needle) != null`.
fn str_contains_func(noreturn_site: SiteId) -> Func {
    let var = |id: VarId| Expr { ty: Ty::U64, kind: ExprKind::Var(id) };
    let cnst = |ty: Ty, v: i128| Expr { ty, kind: ExprKind::Const(v) };
    let load = |addr: Expr, off: u32, ty: Ty| Expr { ty, kind: ExprKind::Load { addr: Box::new(addr), off } };
    let cmp = |op: CmpOp, a: Expr, b: Expr| Expr { ty: Ty::Bool, kind: ExprKind::Cmp { op, lhs: Box::new(a), rhs: Box::new(b) } };
    // Indices never wrap: i <= hn - nn and j < nn, so i + j < hn.
    let add = |a: Expr, b: Expr| Expr {
        ty: Ty::U64,
        kind: ExprKind::Arith { op: ArithOp::AddW, lhs: Box::new(a), rhs: Box::new(b), site: 0 },
    };
    let sub = |a: Expr, b: Expr| Expr {
        ty: Ty::U64,
        kind: ExprKind::Arith { op: ArithOp::SubW, lhs: Box::new(a), rhs: Box::new(b), site: 0 },
    };
    let ptr = |id: VarId| Expr { ty: Ty::Ptr, kind: ExprKind::Var(id) };
    let byte = |s: VarId, idx: Expr| {
        let base = load(ptr(s), 0, Ty::Ptr);
        load(Expr { ty: Ty::Ptr, kind: ExprKind::Offset { base: Box::new(base), idx: Box::new(idx), scale: 1 } }, 0, Ty::U8)
    };
    let (h, nd, hn, nn, i, j, end) = (0, 1, 2, 3, 4, 5, 6);
    let inner = Stmt::While {
        cond: cmp(CmpOp::Lt, var(j), var(nn)),
        body: vec![Stmt::If {
            cond: cmp(CmpOp::Ne, byte(h, add(var(i), var(j))), byte(nd, var(j))),
            then: vec![Stmt::Break],
            els: vec![],
        }],
        step: vec![Stmt::Assign { var: j, value: add(var(j), cnst(Ty::U64, 1)) }],
    };
    let body = vec![
        Stmt::Assign { var: hn, value: load(ptr(h), 8, Ty::U64) },
        Stmt::Assign { var: nn, value: load(ptr(nd), 8, Ty::U64) },
        Stmt::If {
            cond: cmp(CmpOp::Gt, var(nn), var(hn)),
            then: vec![Stmt::Return(Some(cnst(Ty::Bool, 0)))],
            els: vec![],
        },
        Stmt::Assign { var: end, value: sub(var(hn), var(nn)) },
        Stmt::Assign { var: i, value: cnst(Ty::U64, 0) },
        Stmt::While {
            cond: cmp(CmpOp::Le, var(i), var(end)),
            body: vec![
                Stmt::Assign { var: j, value: cnst(Ty::U64, 0) },
                inner,
                Stmt::If {
                    cond: cmp(CmpOp::Eq, var(j), var(nn)),
                    then: vec![Stmt::Return(Some(cnst(Ty::Bool, 1)))],
                    els: vec![],
                },
            ],
            // A string's length is far below 2^64 - 1, so i + 1 cannot wrap.
            step: vec![Stmt::Assign { var: i, value: add(var(i), cnst(Ty::U64, 1)) }],
        },
        Stmt::Return(Some(cnst(Ty::Bool, 0))),
    ];
    let v = |name: &str, ty: Ty| Var { name: name.to_string(), ty };
    Func {
        name: STR_CONTAINS.to_string(),
        nparams: 2,
        ret: Some(Ty::Bool),
        vars: vec![
            v("hay", Ty::Ptr),
            v("needle", Ty::Ptr),
            v("hn", Ty::U64),
            v("nn", Ty::U64),
            v("i", Ty::U64),
            v("j", Ty::U64),
            v("end", Ty::U64),
        ],
        body,
        line: 0,
        is_test: false,
        is_invariant: false,
        noreturn_site,
        slots: Vec::new(),
    }
}
