//! Single precision in the lowering: comptime values coerced to f32,
//! `@floatCast`, and the f32 <-> f64 conversion node; and the
//! comptime_float itself.
//!
//! A child module of `lower`, so it sees that module's private items; the
//! hooks in `lower.rs` stay one line each.
//!
//! Zig keeps a comptime_float as an IEEE binary128 value; `Q` is that value,
//! and specs/tri/t27b/comptime_float.t27 is its arithmetic (gen-rust).

use super::*;

/// `e` (f32 or f64) converted to the float type `to`: f32 to f64 is exact,
/// f64 to f32 rounds to nearest even and overflows to infinity, as Zig's
/// `@floatCast` of a runtime value does. Constants fold.
pub(super) fn float_cast(e: Expr, to: Ty) -> Expr {
    if e.ty == to {
        return e;
    }
    if let ExprKind::Const(c) = e.kind {
        let bits = if to == Ty::F32 { f32_bits(f64_of(c) as f32) } else { f64_bits(f32_of(c) as f64) };
        return Expr { ty: to, kind: ExprKind::Const(bits) };
    }
    Expr { ty: to, kind: ExprKind::FloatCast(Box::new(e)) }
}

impl<'a> Lower<'a> {
    /// A comptime_int or comptime_float coerced to f32. Zig refuses an
    /// integer that is not exactly an f32; a comptime_float rounds once from
    /// binary128, one past the f32 range to infinity (one past the f64 range
    /// is refused: not checked against the reference).
    pub(super) fn comptime_to_f32(&mut self, v: Val) -> R<Expr> {
        let f = match v {
            Val::Ct(c) => {
                let f = c as f32;
                if !f.is_finite() || f as i128 != c {
                    return self.reject("literal out of range", format!("{} is not exactly an f32", c));
                }
                f
            }
            Val::Cf(q) => {
                if q.to_f64().is_none() {
                    return self.reject("literal out of range", format!("{:e} is outside the f64 range", q.approx()));
                }
                q.to_f32()
            }
            Val::Poison => return Err(()),
            _ => unreachable!(),
        };
        Ok(Expr { ty: Ty::F32, kind: ExprKind::Const(f32_bits(f)) })
    }

    /// `@floatCast(x)` with result type `ty`: a typed f32 / f64 operand.
    /// A comptime operand is refused (it is a plain coercion in Zig, not
    /// worth a second rounding path).
    pub(super) fn float_cast_call(&mut self, n: &Node, ty: Ty) -> R<Val> {
        self.see(n);
        let what = "ExprCall(@floatCast)";
        if n.children.len() != 1 {
            return self.reject(what, format!("{} arguments", n.children.len()));
        }
        if !ty.is_float() {
            return self.reject(what, format!("result type {}", ty.name()));
        }
        match self.expr(&n.children[0])? {
            Val::Poison => Err(()),
            Val::E(e) if e.ty.is_float() => Ok(Val::E(float_cast(e, ty))),
            v => {
                let d = match &v { Val::E(e) => e.ty.name().to_string(), _ => self.val_desc(&v) };
                self.reject(what, format!("operand is {}, not a typed float", d))
            }
        }
    }

    /// `abs(x)` with no `fn abs` declared: t27c's Zig backend prints the
    /// builtin `@abs(x)` (its `declared_fns` guard). A typed f64 / f32 loses
    /// its sign bit, so `-0.0` gives `+0.0`; written `x <= 0 ? 0 - x : x`,
    /// with `x` evaluated once (`NaN <= 0` is false: a NaN comes back as is).
    /// A comptime operand folds. A typed integer is refused: Zig's `@abs`
    /// of an `iN` is a `uN`. `None`: not this call.
    pub(super) fn bare_abs(&mut self, n: &Node) -> R<Option<Val>> {
        if n.name != "abs" || self.sigs.contains_key("abs") || self.poison_names.contains("abs") {
            return Ok(None);
        }
        self.builtin_plan(n).map(Some)
    }

    /// `@abs(x)`, `@max(a, b)`, `@min(a, b)`, `@divTrunc(a, b)` and a bare `abs(x)`: what they take, the
    /// result type and the select chain or operator each lowers to are specs/tri/t27b/builtin_plan.t27.
    pub(super) fn builtin_plan(&mut self, n: &Node) -> R<Val> {
        let b = match n.name.as_str() { "@abs" => bp::ABS, "@max" => bp::MAX, "@min" => bp::MIN, "@divTrunc" => bp::DIV_TRUNC, _ => bp::BARE_ABS };
        self.see(n);
        if n.children.len() != bp::arity(b) {
            return self.reject(bp::what_arity(b), format!("`{}` with {} operands", n.name, n.children.len()));
        }
        let vals = n.children.iter().map(|c| self.expr(c)).collect::<R<Vec<Val>>>()?;
        if vals.iter().any(Val::is_poison) {
            return Err(());
        }
        let ty_of = |v: &Val| if let Val::E(e) = v { Some(e.ty) } else { None };
        let same = vals.len() == 2 && ty_of(&vals[0]) == ty_of(&vals[1]);
        let ty = match (bp::type_from(b, operand_kind(vals.first()), operand_kind(vals.get(1)), same), &vals[0]) {
            (bp::OPERATOR, _) => return self.binary(bp::op_text(bp::operator(b)), vals[0].clone(), vals[1].clone()),
            (bp::FOLD, Val::Cf(q)) => return Ok(Val::Cf(q.abs())),
            (bp::FOLD, Val::Ct(c)) => return c.checked_abs().map(Val::Ct).map_or_else(|| self.reject("literal out of range", format!("abs({})", c)), Ok),
            (bp::REFUSE, _) => {
                let d: Vec<String> = vals.iter().map(|v| ty_of(v).map_or_else(|| self.val_desc(v), |t| t.name().into())).collect();
                return self.reject(bp::what_operands(b), format!("`{}` of {}", n.name, d.join(" and ")));
            }
            (from, _) => ty_of(&vals[from as usize - 1]).unwrap(), // FROM_FIRST, FROM_SECOND: the operand's slot
        };
        self.plan_rows(b, ty, Val::Cf(Q::zero()), vals)
    }

    /// `std.math.pi` / `std.math.e` (no local or constant named `std`): builtin_plan.t27's digits. `std.testing.allocator`:
    /// opaque_plan.t27's carried value, a fresh one of zero bytes.
    pub(super) fn std_const(&mut self, n: &Node) -> Option<Val> {
        let [m] = &n.children[..] else { return None };
        let [s] = &m.children[..] else { return None };
        let shape = m.kind == NodeKind::ExprFieldAccess && s.kind == NodeKind::ExprIdentifier && s.name == "std";
        if !shape || self.lookup("std").is_some() || self.const_nodes.contains_key("std") {
            return None;
        }
        if oq::std_value(m.name.as_bytes(), n.name.as_bytes()) == oq::CARRY {
            self.see(n);
            let (t, k) = self.lty(oq::carried_type()).and_then(|t| Ok((t.clone(), self.new_slot(&t)?))).ok()?;
            let zero = |off| Stmt::Store { addr: slot_expr(k), off, value: Expr { ty: Ty::U64, kind: ExprKind::Const(0) } };
            let addr = Expr { ty: Ty::Ptr, kind: ExprKind::Seq { stmts: vec![zero(0), zero(8)], value: Box::new(slot_expr(k)) } };
            return Some(Val::M(Place { addr, off: 0, ty: t, mutable: false, temp: Some(k) }));
        }
        let d = match (m.name.as_str(), n.name.as_str()) { ("math", "pi") => bp::PI_DIGITS, ("math", "e") => bp::E_DIGITS, _ => return None };
        self.see(n);
        Some(Val::Cf(Q::parse(d).ok()?))
    }
}

// ------------------------------------------------------------- binary128

#[path = "../../../gen/rust/tri/t27b/comptime_float.rs"]
#[allow(dead_code, unused_parens, unexpected_cfgs)]
mod cf; // t27c gen-rust of specs/tri/t27b/comptime_float.t27: literals, `+ - * /`, order and rounding of a comptime_float

/// A comptime_float as Zig holds it: an IEEE binary128 value, never infinite or NaN.
#[derive(Clone, Copy, Debug)]
pub(super) struct Q(cf::Q128);

impl Q {
    fn zero() -> Q {
        Q(cf::q_zero())
    }

    pub(super) fn is_zero(self) -> bool {
        cf::q_is_zero(self.0)
    }

    pub(super) fn neg(self) -> Q {
        Q(cf::q_neg(self.0))
    }

    pub(super) fn abs(self) -> Q {
        Q(cf::q_abs(self.0))
    }

    pub(super) fn from_int(c: i128) -> Option<Q> {
        let a = c.unsigned_abs();
        let r = cf::q_from_int(c < 0, (a >> 64) as u64, a as u64);
        r.ok.then_some(Q(r.q))
    }

    pub(super) fn parse(s: &str) -> Result<Q, String> {
        let r = cf::q_parse(s.as_bytes());
        if r.err == cf::OK { Ok(Q(r.q)) } else { Err(cf::why(r.err).into()) }
    }

    pub(super) fn op(op: FOp, a: Q, b: Q) -> Option<Q> {
        let r = cf::q_op(op as u8, a.0, b.0);
        r.ok.then_some(Q(r.q))
    }

    pub(super) fn cmp(a: Q, b: Q) -> std::cmp::Ordering {
        cf::q_cmp(a.0, b.0).cmp(&0)
    }

    pub(super) fn to_f64(self) -> Option<f64> {
        let r = cf::q_to_f64(self.0);
        r.ok.then(|| f64::from_bits(r.bits))
    }

    pub(super) fn to_f32(self) -> f32 {
        f32::from_bits(cf::q_to_f32(self.0))
    }

    pub(super) fn exact_f64(self) -> Option<f64> {
        let r = cf::q_exact_f64(self.0);
        r.ok.then(|| f64::from_bits(r.bits))
    }

    /// The nearest f64, or an infinity, for messages.
    pub(super) fn approx(self) -> f64 {
        self.to_f64().unwrap_or(if self.0.neg { f64::NEG_INFINITY } else { f64::INFINITY })
    }
}
