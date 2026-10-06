//! `for` over memory, one iterable or several: `for (xs) |x|` and the
//! multi-object `for (xs, ys) |x, y|`.
//!
//! Zig evaluates the iterables once, in order, then requires their lengths
//! to be equal: two different comptime lengths are a compile error (refused
//! here, the reference cannot compile it), and any other difference is a
//! runtime safety panic before the first iteration ("for loop over objects
//! with non-equal lengths"), lowered as a trap. One hidden index walks every
//! iterable; each capture binds the element of its own iterable.

use super::*;

impl<'a> Lower<'a> {
    /// `for (a, b, ...) |x, y, ...|`: every iterable must be memory (an
    /// array, a slice or a string); a range is refused (Zig's `0..` open
    /// range has no t27c spelling this lane has seen).
    pub(super) fn for_multi(
        &mut self,
        iters: &[Node],
        params: &[(String, String)],
        body_node: &Node,
        out: &mut Vec<Stmt>,
    ) -> R<()> {
        if params.len() != iters.len() {
            return self.reject(
                "StmtFor(multi-object)",
                format!("{} iterables but {} captures", iters.len(), params.len()),
            );
        }
        if let Some(r) = iters.iter().find(|it| it.kind == NodeKind::ExprBinary && it.extra_op == "..") {
            self.see(r);
            return self.reject("StmtFor(multi-object)", "a range among several iterables".into());
        }
        let mut objs = Vec::new();
        let mut lens: Vec<Expr> = Vec::new();
        for (it, p) in iters.iter().zip(params) {
            let (elem, base, len) = self.for_iterable(it, out)?;
            objs.push((p.0.trim().to_string(), elem, base));
            lens.push(len);
        }
        let first = lens[0].clone();
        for len in &lens[1..] {
            match (&first.kind, &len.kind) {
                (ExprKind::Const(a), ExprKind::Const(b)) if a != b => {
                    return self.reject(
                        "StmtFor(multi-object)",
                        format!("lengths {} and {} differ at compile time", a, b),
                    );
                }
                (ExprKind::Const(_), ExprKind::Const(_)) => {}
                _ => {
                    let site = self.site(
                        TrapKind::ForLength,
                        "for loop over objects with non-equal lengths".into(),
                        Ty::Bool,
                    );
                    let cond = Expr {
                        ty: Ty::Bool,
                        kind: ExprKind::Cmp { op: CmpOp::Eq, lhs: Box::new(first.clone()), rhs: Box::new(len.clone()) },
                    };
                    out.push(Stmt::Assert { cond, site });
                }
            }
        }
        // Zig takes the loop length from the first iterable whose length is
        // comptime-known; all are equal by now, so any one will do.
        let len = lens.iter().find(|l| matches!(l.kind, ExprKind::Const(_))).cloned().unwrap_or(first);
        self.for_objects(objs, len, body_node, out)
    }

    /// The loop itself: one hidden index from 0 to `len`, each capture
    /// `(name, element type, base address)` bound to its element.
    pub(super) fn for_objects(
        &mut self,
        objs: Vec<(String, LTy, Expr)>,
        len: Expr,
        body_node: &Node,
        out: &mut Vec<Stmt>,
    ) -> R<()> {
        let mut seen: Vec<&str> = Vec::new();
        for (capture, _, _) in &objs {
            self.no_var_shadow(capture)?;
            if capture.starts_with('*') || capture.contains(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                return self.reject("StmtFor", format!("capture `{}`", capture));
            }
            if capture != "_" && seen.contains(&capture.as_str()) {
                return self.reject("StmtFor(multi-object)", format!("capture `{}` twice", capture));
            }
            seen.push(capture);
        }
        let mut esizes = Vec::new();
        for (_, elem, _) in &objs {
            esizes.push(self.size_align(elem)?.0);
        }
        let i = self.hidden_var("%for_i", LTy::S(Ty::U64));
        let var_i = Expr { ty: Ty::U64, kind: ExprKind::Var(i) };
        out.push(Stmt::Assign { var: i, value: Expr { ty: Ty::U64, kind: ExprKind::Const(0) } });
        let mut body = Vec::new();
        self.scopes.push(HashMap::new());
        for ((capture, elem, base), esize) in objs.into_iter().zip(esizes) {
            if capture == "_" {
                continue;
            }
            let at = Expr {
                ty: Ty::Ptr,
                kind: ExprKind::Offset { base: Box::new(base), idx: Box::new(var_i.clone()), scale: esize },
            };
            match reg_ty(&elem) {
                Some(ty) => {
                    let value = Expr { ty, kind: ExprKind::Load { addr: Box::new(at), off: 0 } };
                    let x = self.new_lvar(&capture, elem.clone(), false);
                    body.push(Stmt::Assign { var: x, value });
                }
                None => {
                    let place = Place { addr: at, off: 0, ty: elem.clone(), mutable: false, temp: None };
                    self.bind(&capture, Binding::Mem(place));
                }
            }
        }
        self.loop_depth += 1;
        let r = self.stmts(&body_node.children);
        self.loop_depth -= 1;
        self.scopes.pop();
        body.extend(r?);
        let cond = Expr {
            ty: Ty::Bool,
            kind: ExprKind::Cmp { op: CmpOp::Lt, lhs: Box::new(var_i.clone()), rhs: Box::new(len) },
        };
        // i < len < 2^64, so i + 1 cannot wrap.
        let next = Expr {
            ty: Ty::U64,
            kind: ExprKind::Arith {
                op: ArithOp::AddW,
                lhs: Box::new(var_i),
                rhs: Box::new(Expr { ty: Ty::U64, kind: ExprKind::Const(1) }),
                site: 0,
            },
        };
        out.push(Stmt::While { cond, body, step: vec![Stmt::Assign { var: i, value: next }] });
        Ok(())
    }
}
