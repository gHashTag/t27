//! Single precision in the lowering: comptime values coerced to f32,
//! `@floatCast`, and the f32 <-> f64 conversion node.
//!
//! A child module of `lower`, so it sees that module's private items; the
//! hooks in `lower.rs` stay one line each.
//!
//! A float literal is a comptime_float, a binary128 value (`f128`); Zig
//! rounds that value to f32 once, and so does `comptime_to_f32`.

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
    /// integer that is not exactly an f32; a float literal rounds its
    /// binary128 value to nearest even, out-of-range ones to infinity.
    pub(super) fn comptime_to_f32(&mut self, v: Val) -> R<Expr> {
        let f = match v {
            Val::Ct(c) => {
                let f = c as f32;
                if !f.is_finite() || f as i128 != c {
                    return self.reject("literal out of range", format!("{} is not exactly an f32", c));
                }
                f
            }
            Val::Cf(q) => q.to_f32(),
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
