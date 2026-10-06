//! Single precision in the lowering: comptime values coerced to f32,
//! `@floatCast`, and the f32 <-> f64 conversion node.
//!
//! A child module of `lower`, so it sees that module's private items; the
//! hooks in `lower.rs` stay one line each.
//!
//! Rounding a float literal to f32 must agree with Zig, which keeps a
//! comptime_float as an f128 and rounds that to f32. The literal reaches us
//! as the nearest f64 `v` plus a flag saying whether `v` is the decimal
//! exactly. When it is, `v as f32` is the one correct rounding. When it is
//! not, the decimal `x`, its f128 rounding and `v` all lie strictly on the
//! same side of every f32 rounding boundary, unless `v` itself sits on one
//! (an f64 cannot fall between `x` and a boundary that is itself an f64
//! without being that boundary). So an inexact literal whose f64 is exactly
//! an f32 midpoint, or exactly the overflow threshold, is refused, and every
//! other literal rounds as `v as f32`.

use super::*;

/// The f32 Zig gives the comptime_float `v` (`exact`: `v` is the literal
/// exactly), or None when the f64 alone cannot decide it.
pub(super) fn cf_to_f32(v: f64, exact: bool) -> Option<f32> {
    let r = v as f32;
    if exact || v.is_nan() || r as f64 == v {
        return Some(r);
    }
    if r.is_infinite() {
        // f32::MAX + half an ulp (2^103): the overflow boundary.
        let m = f32::MAX as f64 + 2f64.powi(103);
        return if v.abs() == m { None } else { Some(r) };
    }
    let s = if v > r as f64 { r.next_up() } else { r.next_down() };
    let mid = (r as f64 + s as f64) / 2.0;
    if mid == v {
        None
    } else {
        Some(r)
    }
}

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
    /// integer that is not exactly an f32; a float literal rounds (see the
    /// module doc), out-of-range ones to infinity.
    pub(super) fn comptime_to_f32(&mut self, v: Val) -> R<Expr> {
        let f = match v {
            Val::Ct(c) => {
                let f = c as f32;
                if !f.is_finite() || f as i128 != c {
                    return self.reject("literal out of range", format!("{} is not exactly an f32", c));
                }
                f
            }
            Val::Cf(x, exact) => match cf_to_f32(x, exact) {
                Some(f) => f,
                None => {
                    return self.reject(
                        "literal out of range",
                        format!("float literal {:e} is an f32 rounding boundary in f64 (Zig rounds the f128 value)", x),
                    )
                }
            },
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
}
