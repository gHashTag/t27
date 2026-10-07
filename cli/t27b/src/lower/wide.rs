//! Integer types wider than 64 bits (`u128`, `i128`, `u256` ... `u65535`), as
//! compile-time values only.
//!
//! t27c's Zig backend prints `const X: u128 = <literal>;` unchanged, so `X`
//! is comptime-known and Zig's compiler does every operation on it, at any
//! width. t27b does the same with exact big integers (`Big`), following
//! Zig's comptime rules:
//!
//! - `+ - *` of a typed operand must stay inside the type (a compile error
//!   otherwise); an untyped literal beside a typed operand must fit in it;
//! - `<<` drops the bits shifted out of a typed operand (Zig's comptime shift
//!   truncates), and its amount must be below the width; `>>` rounds toward
//!   minus infinity;
//! - `/` and `%` need a non-zero divisor and, as Zig asks of `/` and `%`
//!   without `@divTrunc` / `@rem`, operands that are not negative;
//! - `& | ^` are taken on operands that are not negative;
//! - two typed operands of an arithmetic operator must be of one type (a
//!   comparison takes any two).
//!
//! The value leaves the wide world only through a comparison (a constant
//! bool) or an `as` to a standard integer type (t27c prints `@intCast`, so the
//! value must fit). Anything else that reads a wide value -- a run-time
//! operand, a local, a parameter, a result -- stays refused as `type u128`
//! (and so on): t27b has no run-time integers wider than 64 bits.
//!
//! A fn nothing analyzed reaches may name a wide type in its signature: Zig
//! never resolves it, so the refusal is withdrawn (`unresolved_sig`), as for
//! a struct whose layout fails.

use super::{Lower, Val, R};
use crate::compiler::{Node, NodeKind};
use crate::ir::*;
use std::cmp::Ordering;

/// An integer type wider than 64 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Wide {
    signed: bool,
    bits: u32,
}

impl Wide {
    /// `u65` .. `u65535`, `i65` .. `i65535`: the Zig integer types wider
    /// than the 64-bit registers t27b holds integers in.
    pub(super) fn from_name(t: &str) -> Option<Wide> {
        let t = t.trim();
        let signed = match t.as_bytes().first()? {
            b'u' => false,
            b'i' => true,
            _ => return None,
        };
        let digits = &t[1..];
        if digits.is_empty() || digits.starts_with('0') || !digits.bytes().all(|b| b.is_ascii_digit()) || digits.len() > 5 {
            return None;
        }
        let bits: u32 = digits.parse().ok()?;
        (65..=65535).contains(&bits).then_some(Wide { signed, bits })
    }

    fn name(self) -> String {
        format!("{}{}", if self.signed { 'i' } else { 'u' }, self.bits)
    }

    fn fits(self, v: &Big) -> bool {
        if !self.signed {
            return !v.neg && v.bit_len() <= self.bits as u64;
        }
        if !v.neg {
            return v.bit_len() < self.bits as u64;
        }
        v.sub(&Big::from_i128(-1)).neg() // -(v + 1) = |v| - 1
            .bit_len()
            < self.bits as u64
    }

    /// `v` modulo 2^bits, read back as this type (two's complement).
    fn wrap(self, v: &Big) -> Big {
        let m = Big::pow2(self.bits as u64);
        let mut r = v.low_bits(self.bits as u64);
        if v.neg && !r.is_zero() {
            r = m.sub(&r);
        }
        if self.signed && r.bit_len() == self.bits as u64 {
            r = r.sub(&m);
        }
        r
    }
}

/// An exact integer: sign and magnitude, 32-bit limbs, least significant
/// first, no high zero limbs; zero is never negative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Big {
    neg: bool,
    mag: Vec<u32>,
}

impl Big {
    fn zero() -> Big {
        Big { neg: false, mag: Vec::new() }
    }

    fn norm(mut self) -> Big {
        while self.mag.last() == Some(&0) {
            self.mag.pop();
        }
        if self.mag.is_empty() {
            self.neg = false;
        }
        self
    }

    pub(super) fn from_i128(v: i128) -> Big {
        let mut m = v.unsigned_abs();
        let mut mag = Vec::new();
        while m != 0 {
            mag.push(m as u32);
            m >>= 32;
        }
        Big { neg: v < 0, mag }.norm()
    }

    pub(super) fn to_i128(&self) -> Option<i128> {
        if self.bit_len() > 127 {
            // -2^127 is the one 128-bit magnitude that fits.
            return (self.neg && *self == Big::pow2(127).neg()).then_some(i128::MIN);
        }
        let mut m: i128 = 0;
        for &l in self.mag.iter().rev() {
            m = (m << 32) | l as i128;
        }
        Some(if self.neg { -m } else { m })
    }

    fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }

    fn bit_len(&self) -> u64 {
        match self.mag.last() {
            None => 0,
            Some(&top) => (self.mag.len() as u64 - 1) * 32 + (32 - top.leading_zeros()) as u64,
        }
    }

    fn pow2(k: u64) -> Big {
        let mut mag = vec![0u32; (k / 32) as usize + 1];
        mag[(k / 32) as usize] = 1 << (k % 32);
        Big { neg: false, mag }
    }

    /// An integer literal as the t27c lexer keeps it: an optional `-`, then
    /// decimal, `0x`, `0o` or `0b` digits, `_` allowed between them.
    pub(super) fn parse(s: &str) -> Option<Big> {
        let (neg, s) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s),
        };
        let t: String = s.chars().filter(|c| *c != '_').collect();
        let (digits, radix) = if let Some(r) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
            (r, 16)
        } else if let Some(r) = t.strip_prefix("0o") {
            (r, 8)
        } else if let Some(r) = t.strip_prefix("0b") {
            (r, 2)
        } else {
            (t.as_str(), 10)
        };
        if digits.is_empty() {
            return None;
        }
        let mut v = Big::zero();
        let r = Big::from_i128(radix);
        for c in digits.chars() {
            let d = c.to_digit(radix as u32)?;
            v = v.mul(&r).add(&Big::from_i128(d as i128));
        }
        Some(if neg { v.neg() } else { v })
    }

    fn neg(&self) -> Big {
        Big { neg: !self.neg, mag: self.mag.clone() }.norm()
    }

    fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
        a.len().cmp(&b.len()).then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }

    pub(super) fn cmp(&self, o: &Big) -> Ordering {
        match (self.neg, o.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Big::cmp_mag(&self.mag, &o.mag),
            (true, true) => Big::cmp_mag(&o.mag, &self.mag),
        }
    }

    fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for i in 0..a.len().max(b.len()) {
            let s = *a.get(i).unwrap_or(&0) as u64 + *b.get(i).unwrap_or(&0) as u64 + carry;
            out.push(s as u32);
            carry = s >> 32;
        }
        out.push(carry as u32);
        out
    }

    /// `a - b` for `|a| >= |b|`.
    fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for (i, &x) in a.iter().enumerate() {
            let mut d = x as i64 - *b.get(i).unwrap_or(&0) as i64 - borrow;
            borrow = (d < 0) as i64;
            if d < 0 {
                d += 1 << 32;
            }
            out.push(d as u32);
        }
        out
    }

    pub(super) fn add(&self, o: &Big) -> Big {
        if self.neg == o.neg {
            return Big { neg: self.neg, mag: Big::add_mag(&self.mag, &o.mag) }.norm();
        }
        match Big::cmp_mag(&self.mag, &o.mag) {
            Ordering::Less => Big { neg: o.neg, mag: Big::sub_mag(&o.mag, &self.mag) }.norm(),
            _ => Big { neg: self.neg, mag: Big::sub_mag(&self.mag, &o.mag) }.norm(),
        }
    }

    pub(super) fn sub(&self, o: &Big) -> Big {
        self.add(&o.neg())
    }

    pub(super) fn mul(&self, o: &Big) -> Big {
        let mut out = vec![0u32; self.mag.len() + o.mag.len() + 1];
        for (i, &x) in self.mag.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &y) in o.mag.iter().enumerate() {
                let t = out[i + j] as u64 + x as u64 * y as u64 + carry;
                out[i + j] = t as u32;
                carry = t >> 32;
            }
            let mut k = i + o.mag.len();
            while carry != 0 {
                let t = out[k] as u64 + carry;
                out[k] = t as u32;
                carry = t >> 32;
                k += 1;
            }
        }
        Big { neg: self.neg != o.neg, mag: out }.norm()
    }

    fn bit(&self, k: u64) -> bool {
        self.mag.get((k / 32) as usize).is_some_and(|l| (l >> (k % 32)) & 1 == 1)
    }

    /// Quotient and remainder of the magnitudes (`|o|` not zero), by shift
    /// and subtract.
    fn divrem_mag(&self, o: &Big) -> (Big, Big) {
        let d = Big { neg: false, mag: o.mag.clone() };
        let mut q = Big::zero();
        let mut r = Big::zero();
        for k in (0..self.bit_len()).rev() {
            r = r.shl(1);
            if self.bit(k) {
                r = r.add(&Big::from_i128(1));
            }
            if Big::cmp_mag(&r.mag, &d.mag) != Ordering::Less {
                r = r.sub(&d);
                q = q.add(&Big::pow2(k));
            }
        }
        (q, r)
    }

    /// Truncating division and its remainder, the divisor not zero.
    pub(super) fn divrem(&self, o: &Big) -> (Big, Big) {
        let (q, r) = self.divrem_mag(o);
        let q = if self.neg != o.neg { q.neg() } else { q };
        let r = if self.neg { r.neg() } else { r };
        (q, r)
    }

    pub(super) fn shl(&self, k: u64) -> Big {
        let (limbs, bits) = ((k / 32) as usize, (k % 32) as u32);
        let mut out = vec![0u32; limbs];
        let mut carry = 0u32;
        for &l in &self.mag {
            out.push(if bits == 0 { l } else { (l << bits) | carry });
            carry = if bits == 0 { 0 } else { l >> (32 - bits) };
        }
        out.push(carry);
        Big { neg: self.neg, mag: out }.norm()
    }

    /// `>>`: rounds toward minus infinity, like an arithmetic shift.
    pub(super) fn shr(&self, k: u64) -> Big {
        if self.neg {
            // -((|v| - 1) >> k) - 1
            let m = self.neg().sub(&Big::from_i128(1));
            return m.shr(k).add(&Big::from_i128(1)).neg();
        }
        let (limbs, bits) = ((k / 32) as usize, (k % 32) as u32);
        if limbs >= self.mag.len() {
            return Big::zero();
        }
        let src = &self.mag[limbs..];
        let mut out = Vec::with_capacity(src.len());
        for i in 0..src.len() {
            let hi = if bits == 0 { 0 } else { src.get(i + 1).map_or(0, |h| h << (32 - bits)) };
            out.push((src[i] >> bits) | hi);
        }
        Big { neg: false, mag: out }.norm()
    }

    /// The low `k` bits of the magnitude.
    fn low_bits(&self, k: u64) -> Big {
        let mut mag: Vec<u32> = self.mag.iter().take(((k + 31) / 32) as usize).copied().collect();
        if k % 32 != 0 && mag.len() as u64 == (k + 31) / 32 {
            let last = mag.len() - 1;
            mag[last] &= (1u32 << (k % 32)) - 1;
        }
        Big { neg: false, mag }.norm()
    }

    /// `& | ^` of two operands that are not negative.
    fn bitwise(&self, o: &Big, f: fn(u32, u32) -> u32) -> Big {
        let n = self.mag.len().max(o.mag.len());
        let mag = (0..n).map(|i| f(*self.mag.get(i).unwrap_or(&0), *o.mag.get(i).unwrap_or(&0))).collect();
        Big { neg: false, mag }.norm()
    }

    /// Decimal digits, for refusals.
    fn decimal(&self) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let mut parts = Vec::new();
        let mut m = Big { neg: false, mag: self.mag.clone() };
        let billion = Big::from_i128(1_000_000_000);
        while !m.is_zero() {
            let (q, r) = m.divrem(&billion);
            parts.push(r.to_i128().unwrap_or(0));
            m = q;
        }
        let mut s = String::new();
        if self.neg {
            s.push('-');
        }
        s += &parts.pop().unwrap_or(0).to_string();
        for p in parts.iter().rev() {
            s += &format!("{:09}", p);
        }
        s
    }
}

/// A wide operand: its type (None: an untyped integer, Zig's
/// comptime_int) and its value.
type WVal = (Option<Wide>, Big);

impl<'a> Lower<'a> {
    /// The type of `name` when it names a module constant declared with a
    /// wide integer type, and no local hides it.
    fn wide_const(&self, name: &str) -> Option<Wide> {
        if self.lookup(name).is_some() {
            return None;
        }
        Wide::from_name(self.const_nodes.get(name)?.extra_type.trim())
    }

    /// `name` is a module constant declared with a wide integer type. Zig
    /// analyzes a module constant only where it is used, so such a constant
    /// is folded at its uses (`wide_global`), not up front: an unused one
    /// refuses nothing, whatever its value.
    pub(super) fn wide_decl(&self, name: &str) -> bool {
        self.const_nodes.get(name).is_some_and(|n| Wide::from_name(n.extra_type.trim()).is_some())
    }

    /// `n` computes with a wide constant: through operators down to a name,
    /// not into a call or a cast (each has a result type of its own).
    pub(super) fn holds_wide(&self, n: &Node) -> Option<Wide> {
        match n.kind {
            NodeKind::ExprIdentifier => self.wide_const(&n.name),
            NodeKind::ExprBinary if !matches!(n.extra_op.as_str(), "and" | "or" | "&&" | "||") => {
                n.children.iter().find_map(|c| self.holds_wide(c))
            }
            NodeKind::ExprUnary if matches!(n.extra_op.trim(), "-" | "~") => {
                n.children.iter().find_map(|c| self.holds_wide(c))
            }
            _ => None,
        }
    }

    /// A type refused by name. A wide integer type is a type Zig knows,
    /// refused only when something analyzed uses it: it counts as a layout
    /// failure for `unresolved_sig`.
    pub(super) fn reject_type<T>(&mut self, t: &str) -> R<T> {
        if Wide::from_name(t).is_some() {
            self.layout_err = true;
        }
        let (construct, detail) = self.type_construct(t);
        self.reject(&construct, detail)
    }

    fn wide_refuse<T>(&mut self, w: Wide, detail: String) -> R<T> {
        self.reject(&format!("type {}", w.name()), detail)
    }

    /// A comparison with a wide operand (`w` is its type): a constant bool.
    pub(super) fn wide_compare(&mut self, n: &Node, w: Wide) -> R<Val> {
        let op = match n.extra_op.as_str() {
            "==" => CmpOp::Eq,
            "!=" => CmpOp::Ne,
            "<" => CmpOp::Lt,
            "<=" => CmpOp::Le,
            ">" => CmpOp::Gt,
            ">=" => CmpOp::Ge,
            o => return self.wide_refuse(w, format!("`{}` yields a compile-time {}, which t27b cannot hold", o, w.name())),
        };
        let (_, a) = self.wide_eval(&n.children[0], w)?;
        let (_, b) = self.wide_eval(&n.children[1], w)?;
        // Zig settles a comparison of two comptime-known integers exactly,
        // whatever their types (a u128 with a u256, or with a comptime_int
        // outside its range).
        let ord = a.cmp(&b) as i8 as i128;
        Ok(Val::E(Expr { ty: Ty::Bool, kind: ExprKind::Const(op.holds(ord, 0) as i128) }))
    }

    /// `x as T` with a wide `x` (`w` its type): t27c prints
    /// `@as(T, @intCast(x))`, so the value must fit in the integer type `T`.
    pub(super) fn wide_cast(&mut self, n: &Node, w: Wide) -> R<Val> {
        let (_, v) = self.wide_eval(&n.children[0], w)?;
        let to = self.ty(n.extra_type.trim())?;
        if !to.is_int() {
            return self.reject(&format!("ExprCast({})", to.name()), format!("a compile-time {} as {}", w.name(), to.name()));
        }
        match v.to_i128() {
            Some(c) if to.fits(c) => Ok(Val::E(Expr { ty: to, kind: ExprKind::Const(c) })),
            _ => self.reject("literal out of range", format!("{} does not fit in {}", v.decimal(), to.name())),
        }
    }

    /// The value of wide constant `name`, its initializer folded once and
    /// checked against its type.
    fn wide_global(&mut self, name: &str, w: Wide) -> R<Big> {
        if let Some(v) = self.wide_vals.get(name) {
            return v.clone().ok_or(());
        }
        let node = *self.const_nodes.get(name).expect("wide_const read it");
        let Some(init) = node.children.first() else {
            return self.reject("ConstDecl", format!("`{}` has no value", name));
        };
        if !self.resolving.insert(name.to_string()) {
            return self.reject("ConstDecl", format!("`{}` refers to itself", name));
        }
        let saved_line = self.line;
        let saved_scopes = std::mem::take(&mut self.scopes);
        let saved_ct = std::mem::replace(&mut self.comptime, true);
        self.see(node);
        let r = self.wide_eval(init, w).and_then(|(t, v)| match t {
            Some(t) if t != w => self.wide_refuse(w, format!("`{}` is a {} set to a {}", name, w.name(), t.name())),
            _ if !w.fits(&v) => self.reject("literal out of range", format!("{} does not fit in {}", v.decimal(), w.name())),
            _ => Ok(v),
        });
        self.comptime = saved_ct;
        self.scopes = saved_scopes;
        self.line = saved_line;
        self.resolving.remove(name);
        self.wide_vals.insert(name.to_string(), r.clone().ok());
        r
    }

    /// Fold `n`, an operand beside a wide value of type `w`.
    fn wide_eval(&mut self, n: &Node, w: Wide) -> R<WVal> {
        self.see(n);
        match n.kind {
            NodeKind::ExprLiteral if n.extra_kind != "string" && n.extra_type.trim().is_empty() => {
                match Big::parse(n.value.trim()) {
                    Some(v) => Ok((None, v)),
                    None => self.wide_refuse(w, format!("`{}` beside a compile-time {}", n.value.trim(), w.name())),
                }
            }
            NodeKind::ExprIdentifier => match self.wide_const(&n.name) {
                Some(t) => Ok((Some(t), self.wide_global(&n.name, t)?)),
                None => self.wide_refuse(
                    w,
                    format!("`{}` beside a compile-time {}: only wide constants and integer literals fold", n.name, w.name()),
                ),
            },
            NodeKind::ExprUnary if n.children.len() == 1 => {
                let op = n.extra_op.trim().to_string();
                let (t, v) = self.wide_eval(&n.children[0], w)?;
                match (op.as_str(), t) {
                    ("-", None) => Ok((None, v.neg())),
                    ("-", Some(t)) if t.signed => self.wide_fit(t, v.neg(), "-"),
                    ("~", Some(t)) => {
                        // ~v = -v - 1, read back as the type.
                        Ok((Some(t), t.wrap(&v.neg().sub(&Big::from_i128(1)))))
                    }
                    _ => self.wide_refuse(w, format!("unary `{}` on a compile-time {}", op, t.map_or("integer".into(), |t| t.name()))),
                }
            }
            NodeKind::ExprBinary if n.children.len() == 2 => {
                let op = n.extra_op.clone();
                let a = self.wide_eval(&n.children[0], w)?;
                let b = self.wide_eval(&n.children[1], w)?;
                self.wide_arith(&op, a, b, w)
            }
            _ => self.wide_refuse(w, format!("{:?} beside a compile-time {}", n.kind, w.name())),
        }
    }

    /// `v` as type `t`, refused when it does not fit: Zig's comptime
    /// arithmetic is checked.
    fn wide_fit(&mut self, t: Wide, v: Big, op: &str) -> R<WVal> {
        if t.fits(&v) {
            return Ok((Some(t), v));
        }
        self.reject("ExprBinary", format!("compile-time `{}` overflows {}", op, t.name()))
    }

    fn wide_arith(&mut self, op: &str, (ta, a): WVal, (tb, b): WVal, w: Wide) -> R<WVal> {
        if op == "<<" || op == ">>" {
            // The amount is an untyped integer, below the width of a typed
            // left operand (Zig refuses a larger one).
            let limit = ta.map_or(65536, |t| t.bits as u64);
            let k = match (tb, b.to_i128()) {
                (None, Some(k)) if k >= 0 && (k as u64) < limit => k as u64,
                _ => return self.wide_refuse(w, format!("`{}` by {} on a compile-time {}", op, b.decimal(), w.name())),
            };
            return Ok(match (op, ta) {
                ("<<", Some(t)) => (Some(t), t.wrap(&a.shl(k))),
                ("<<", None) => (None, a.shl(k)),
                (_, t) => (t, a.shr(k)),
            });
        }
        // One type for both operands; an untyped one must fit in it.
        let t = match (ta, tb) {
            (Some(x), Some(y)) if x != y => {
                return self.wide_refuse(w, format!("`{}` between {} and {}", op, x.name(), y.name()))
            }
            (Some(x), _) | (_, Some(x)) => Some(x),
            (None, None) => None,
        };
        if let Some(t) = t {
            for v in [&a, &b] {
                if !t.fits(v) {
                    return self.reject("literal out of range", format!("{} does not fit in {}", v.decimal(), t.name()));
                }
            }
        }
        let exact = |r: Big| -> WVal { (t, r) };
        match op {
            "+" | "-" | "*" => {
                let r = match op {
                    "+" => a.add(&b),
                    "-" => a.sub(&b),
                    _ => a.mul(&b),
                };
                match t {
                    Some(t) => self.wide_fit(t, r, op),
                    None => Ok(exact(r)),
                }
            }
            "+%" | "-%" | "*%" if t.is_some() => {
                let t = t.unwrap();
                let r = match op {
                    "+%" => a.add(&b),
                    "-%" => a.sub(&b),
                    _ => a.mul(&b),
                };
                Ok((Some(t), t.wrap(&r)))
            }
            "/" | "%" => {
                if b.is_zero() {
                    return self.reject("ExprBinary", "constant division by zero".into());
                }
                if a.neg || b.neg {
                    return self.wide_refuse(w, format!("`{}` on a negative compile-time integer", op));
                }
                let (q, r) = a.divrem(&b);
                Ok(exact(if op == "/" { q } else { r }))
            }
            "&" | "|" | "^" => {
                if a.neg || b.neg {
                    return self.wide_refuse(w, format!("`{}` on a negative compile-time integer", op));
                }
                let f: fn(u32, u32) -> u32 = match op {
                    "&" => |x, y| x & y,
                    "|" => |x, y| x | y,
                    _ => |x, y| x ^ y,
                };
                Ok(exact(a.bitwise(&b, f)))
            }
            _ => self.wide_refuse(w, format!("`{}` on a compile-time {}", op, w.name())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic values across the i128 range and its edges.
    fn samples() -> Vec<i128> {
        let mut v = vec![0, 1, -1, 2, -2, 31, 32, 33, i64::MAX as i128, i64::MIN as i128, u64::MAX as i128];
        v.extend([i128::MAX / 3, i128::MIN / 3, (1 << 100) + 12345, -(1 << 90) - 7, 1 << 64, (1 << 64) - 1]);
        let mut x: u128 = 0x9E37_79B9_7F4A_7C15_F39C_C060_5CED_C834;
        for _ in 0..200 {
            x = x.wrapping_mul(0x2360_ED05_1FC6_5DA4_4385_DF64_9FCC_F645).wrapping_add(1442695040888963407);
            let width = (x >> 120) as u32 % 126 + 1;
            let m = (x >> 1) as i128 & ((1i128 << width) - 1);
            v.push(if x & 1 == 1 { -m } else { m });
        }
        v
    }

    fn b(v: i128) -> Big {
        Big::from_i128(v)
    }

    #[test]
    fn arithmetic_agrees_with_i128() {
        let s = samples();
        for &x in &s {
            assert_eq!(b(x).to_i128(), Some(x));
            assert_eq!(Big::parse(&x.to_string()), Some(b(x)), "{}", x);
            assert_eq!(b(x).decimal(), x.to_string());
            for &y in &s {
                assert_eq!(b(x).cmp(&b(y)), x.cmp(&y), "{} cmp {}", x, y);
                if let Some(r) = x.checked_add(y) {
                    assert_eq!(b(x).add(&b(y)), b(r), "{} + {}", x, y);
                }
                if let Some(r) = x.checked_sub(y) {
                    assert_eq!(b(x).sub(&b(y)), b(r), "{} - {}", x, y);
                }
                if let Some(r) = x.checked_mul(y) {
                    assert_eq!(b(x).mul(&b(y)), b(r), "{} * {}", x, y);
                }
                if y != 0 && !(x == i128::MIN && y == -1) {
                    assert_eq!(b(x).divrem(&b(y)), (b(x / y), b(x % y)), "{} / {}", x, y);
                }
                if x >= 0 && y >= 0 {
                    assert_eq!(b(x).bitwise(&b(y), |p, q| p & q), b(x & y));
                    assert_eq!(b(x).bitwise(&b(y), |p, q| p | q), b(x | y));
                    assert_eq!(b(x).bitwise(&b(y), |p, q| p ^ q), b(x ^ y));
                }
            }
            for k in [0u64, 1, 5, 31, 32, 33, 63, 64, 65, 100, 126, 127, 200] {
                assert_eq!(b(x).shr(k), b(if k >= 127 { if x < 0 { -1 } else { 0 } } else { x >> k }), "{} >> {}", x, k);
                if x.checked_shl(k as u32).is_some_and(|r| r >> k == x) && k < 127 {
                    assert_eq!(b(x).shl(k), b(x << k), "{} << {}", x, k);
                }
            }
        }
    }

    #[test]
    fn widths_wrap_and_fit_like_twos_complement() {
        let u128t = Wide::from_name("u128").unwrap();
        let i128t = Wide::from_name("i128").unwrap();
        let u65 = Wide::from_name("u65").unwrap();
        assert!(Wide::from_name("u64").is_none() && Wide::from_name("i8").is_none());
        assert!(Wide::from_name("u65536").is_none() && Wide::from_name("u0128").is_none());
        assert!(Wide::from_name("u1024").is_some() && Wide::from_name("i65535").is_some());
        let top = Big::pow2(128).sub(&b(1));
        assert!(u128t.fits(&top) && !u128t.fits(&Big::pow2(128)) && !u128t.fits(&b(-1)));
        assert!(i128t.fits(&b(i128::MIN)) && i128t.fits(&b(i128::MAX)));
        assert!(!i128t.fits(&b(i128::MIN).sub(&b(1))) && !i128t.fits(&Big::pow2(127)));
        assert!(u65.fits(&Big::pow2(64)) && !u65.fits(&Big::pow2(65)));
        assert_eq!(u128t.wrap(&top.shl(4)).shr(124), b(15));
        assert_eq!(u128t.wrap(&b(-1)), top);
        assert_eq!(i128t.wrap(&Big::pow2(127)), b(i128::MIN));
        assert_eq!(i128t.wrap(&top), b(-1));
        assert_eq!(u65.wrap(&Big::pow2(65).add(&b(9))), b(9));
        for &x in &samples() {
            assert_eq!(i128t.wrap(&b(x)), b(x));
            assert_eq!(u128t.wrap(&b(x)), Big::parse(&(x as u128).to_string()).unwrap());
        }
    }

    #[test]
    fn literals_in_every_radix() {
        assert_eq!(Big::parse("0xFFFF_0000_0000_0000_0000_0000_0000_0001").unwrap().shr(112), b(0xFFFF));
        assert_eq!(Big::parse("0b1010").unwrap(), b(10));
        assert_eq!(Big::parse("0o777").unwrap(), b(511));
        assert_eq!(Big::parse("-170141183460469231731687303715884105728").unwrap(), b(i128::MIN));
        let p = Big::parse("340282366920938463463374607431768211456").unwrap();
        assert_eq!(p, Big::pow2(128));
        assert_eq!(p.to_i128(), None);
        assert_eq!(p.decimal(), "340282366920938463463374607431768211456");
        assert!(Big::parse("12a").is_none() && Big::parse("").is_none() && Big::parse("0x").is_none());
    }
}
