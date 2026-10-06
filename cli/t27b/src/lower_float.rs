//! Single precision in the lowering: comptime values coerced to f32,
//! `@floatCast`, and the f32 <-> f64 conversion node; and the
//! comptime_float itself.
//!
//! A child module of `lower`, so it sees that module's private items; the
//! hooks in `lower.rs` stay one line each.
//!
//! Zig keeps a comptime_float as an IEEE binary128 value: a decimal literal
//! is parsed to the nearest binary128, each compile-time `+ - * /` of two
//! of them is rounded to binary128 (ties to even), two of them compare as
//! binary128 values, and the result is rounded once to f64 or f32 where a
//! typed float is needed. `Q` is that value, computed exactly in integers
//! (`Big`) and rounded the same way, so `0.1 * 3.0` folds to the f64 0.3
//! while the same product at run time is 0.30000000000000004.

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
}

// ------------------------------------------------------------- binary128

/// An unsigned integer of any size: little-endian 64-bit limbs, no zero
/// limb on top (zero is no limbs). Just enough to round decimal literals
/// and the four operations exactly.
#[derive(Clone, Debug)]
struct Big(Vec<u64>);

impl Big {
    fn from_u128(v: u128) -> Big {
        let mut b = Big(vec![v as u64, (v >> 64) as u64]);
        b.trim();
        b
    }

    fn trim(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    fn bitlen(&self) -> u64 {
        match self.0.last() {
            None => 0,
            Some(&top) => (self.0.len() as u64 - 1) * 64 + (64 - top.leading_zeros() as u64),
        }
    }

    fn bit(&self, i: u64) -> bool {
        let w = (i / 64) as usize;
        w < self.0.len() && (self.0[w] >> (i % 64)) & 1 == 1
    }

    /// Whether any of bits `0..i` is set.
    fn any_below(&self, i: u64) -> bool {
        let (w, b) = ((i / 64) as usize, i % 64);
        self.0.iter().take(w).any(|&x| x != 0) || (w < self.0.len() && b > 0 && self.0[w] & ((1u64 << b) - 1) != 0)
    }

    fn shl(&self, n: u64) -> Big {
        if self.is_zero() {
            return Big(Vec::new());
        }
        let (w, b) = ((n / 64) as usize, (n % 64) as u32);
        let mut out = vec![0u64; w];
        let mut carry = 0u64;
        for &x in &self.0 {
            if b == 0 {
                out.push(x);
            } else {
                out.push((x << b) | carry);
                carry = x >> (64 - b);
            }
        }
        if carry != 0 {
            out.push(carry);
        }
        Big(out)
    }

    /// The low 128 bits of `self >> n`.
    fn shr_u128(&self, n: u64) -> u128 {
        (0..128u64).filter(|&i| self.bit(n + i)).fold(0u128, |r, i| r | (1u128 << i))
    }

    fn add(&self, o: &Big) -> Big {
        let n = self.0.len().max(o.0.len());
        let mut out = Vec::with_capacity(n + 1);
        let mut c = 0u128;
        for i in 0..n {
            let s = *self.0.get(i).unwrap_or(&0) as u128 + *o.0.get(i).unwrap_or(&0) as u128 + c;
            out.push(s as u64);
            c = s >> 64;
        }
        out.push(c as u64);
        let mut b = Big(out);
        b.trim();
        b
    }

    /// `self - o`, for `self >= o`.
    fn sub(&self, o: &Big) -> Big {
        let mut out = Vec::with_capacity(self.0.len());
        let mut borrow = false;
        for (i, &x) in self.0.iter().enumerate() {
            let (d1, b1) = x.overflowing_sub(*o.0.get(i).unwrap_or(&0));
            let (d2, b2) = d1.overflowing_sub(borrow as u64);
            out.push(d2);
            borrow = b1 || b2;
        }
        let mut b = Big(out);
        b.trim();
        b
    }

    fn cmp(&self, o: &Big) -> std::cmp::Ordering {
        self.0.len().cmp(&o.0.len()).then_with(|| self.0.iter().rev().cmp(o.0.iter().rev()))
    }

    fn mul(&self, o: &Big) -> Big {
        let mut out = vec![0u64; self.0.len() + o.0.len()];
        for (i, &x) in self.0.iter().enumerate() {
            let mut c = 0u128;
            for (j, &y) in o.0.iter().enumerate() {
                let t = out[i + j] as u128 + x as u128 * y as u128 + c;
                out[i + j] = t as u64;
                c = t >> 64;
            }
            out[i + o.0.len()] = c as u64;
        }
        let mut b = Big(out);
        b.trim();
        b
    }

    /// `floor(self / d)` and whether a remainder is left, `d` not zero.
    fn div(&self, d: &Big) -> (Big, bool) {
        let n = self.bitlen();
        let mut q = vec![0u64; n.div_ceil(64) as usize];
        let mut r = Big(Vec::new());
        for i in (0..n).rev() {
            r = r.shl(1);
            if self.bit(i) {
                if r.is_zero() {
                    r.0.push(1);
                } else {
                    r.0[0] |= 1;
                }
            }
            if r.cmp(d) != std::cmp::Ordering::Less {
                r = r.sub(d);
                q[(i / 64) as usize] |= 1u64 << (i % 64);
            }
        }
        let mut q = Big(q);
        q.trim();
        (q, !r.is_zero())
    }

    fn pow10(mut k: u64) -> Big {
        let mut b = Big::from_u128(1);
        while k >= 19 {
            b = b.mul(&Big::from_u128(10u128.pow(19)));
            k -= 19;
        }
        b.mul(&Big::from_u128(10u128.pow(k as u32)))
    }
}

/// `n * 2^e`, plus a positive amount below `2^e` when `sticky`, rounded to
/// `p` significant bits, ties to even, with no bit below `2^lsb_min` (the
/// subnormal range): `(m, lsb)` for the value `m * 2^lsb`. None when the
/// top bit would land above `2^top_max`, or when `sticky` sits at or above
/// the last kept bit (the caller gave too few bits to decide).
fn round_bits(n: &Big, e: i64, sticky: bool, p: u32, lsb_min: i64, top_max: i64) -> Option<(u128, i64)> {
    if n.is_zero() {
        return if sticky { None } else { Some((0, lsb_min)) };
    }
    let top = e + n.bitlen() as i64 - 1;
    let mut lsb = (top - (p as i64 - 1)).max(lsb_min);
    let shift = lsb - e;
    let mut m;
    if shift <= 0 {
        if sticky {
            return None;
        }
        m = n.shl((-shift) as u64).shr_u128(0);
    } else {
        let s = shift as u64;
        m = n.shr_u128(s);
        let half = n.bit(s - 1);
        let low = sticky || n.any_below(s - 1);
        if half && (low || m & 1 == 1) {
            m += 1;
            if m == 1u128 << p {
                m >>= 1;
                lsb += 1;
            }
        }
    }
    if m != 0 && lsb + (127 - m.leading_zeros() as i64) > top_max {
        return None;
    }
    Some((m, lsb))
}

const Q_P: u32 = 113;
const Q_LSB_MIN: i64 = -16494;
const Q_TOP_MAX: i64 = 16383;

/// A comptime_float as Zig holds it: an IEEE binary128 value,
/// `(-1)^neg * m * 2^e` with `m` below `2^113`. Never infinite or NaN: an
/// operation that would make one is refused instead.
#[derive(Clone, Copy, Debug)]
pub(super) struct Q {
    neg: bool,
    m: u128,
    e: i64,
}

impl Q {
    fn zero() -> Q {
        Q { neg: false, m: 0, e: Q_LSB_MIN }
    }

    fn round(neg: bool, n: &Big, e: i64, sticky: bool) -> Option<Q> {
        let (m, e) = round_bits(n, e, sticky, Q_P, Q_LSB_MIN, Q_TOP_MAX)?;
        Some(Q { neg, m, e })
    }

    pub(super) fn is_zero(self) -> bool {
        self.m == 0
    }

    pub(super) fn neg(self) -> Q {
        Q { neg: !self.neg, ..self }
    }

    /// A finite f64, exactly.
    fn from_f64(x: f64) -> Q {
        let bits = x.to_bits();
        let (be, bf) = ((bits >> 52) & 0x7ff, bits & ((1u64 << 52) - 1));
        let (m, e) = if be == 0 { (bf, -1074) } else { (bf | (1u64 << 52), be as i64 - 1075) };
        Q { neg: bits >> 63 == 1, m: m as u128, e }
    }

    /// A comptime_int as a comptime_float, when that is exact.
    pub(super) fn from_int(c: i128) -> Option<Q> {
        let a = c.unsigned_abs();
        if a != 0 && 128 - (a >> a.trailing_zeros()).leading_zeros() > Q_P {
            return None;
        }
        Q::round(c < 0, &Big::from_u128(a), 0, false)
    }

    /// A decimal float literal (digits, an optional fraction, an optional
    /// exponent, `_` separators) rounded once to binary128, ties to even.
    pub(super) fn parse(s: &str) -> Result<Q, String> {
        let t: String = s.chars().filter(|c| *c != '_').collect();
        if t.starts_with("0x") || t.starts_with("0X") {
            return Err("hexadecimal float literal".into());
        }
        let (mant, exp) = match t.find(['e', 'E']) {
            Some(i) => (&t[..i], &t[i + 1..]),
            None => (t.as_str(), "0"),
        };
        let (ip, fp) = mant.split_once('.').unwrap_or((mant, ""));
        let digits_ok = |d: &str| d.bytes().all(|b| b.is_ascii_digit());
        let exp_digits = exp.strip_prefix(['+', '-']).unwrap_or(exp);
        if ip.is_empty() || !digits_ok(ip) || !digits_ok(fp) || exp_digits.is_empty() || !digits_ok(exp_digits) {
            return Err("malformed float literal".into());
        }
        let v: f64 = t.parse().map_err(|_| "malformed float literal".to_string())?;
        if !v.is_finite() {
            return Err("float literal overflows f64".into());
        }
        let all = format!("{}{}", ip, fp);
        let digits = all.trim_start_matches('0');
        if digits.is_empty() {
            return Ok(Q::zero());
        }
        // A finite f64 bounds a positive exponent; an exponent too negative
        // for i64 is a value far below the smallest binary128.
        let Ok(x) = exp.parse::<i64>() else { return Ok(Q::zero()) };
        let e10 = x.saturating_sub(fp.len() as i64);
        let mut d = Big(Vec::new());
        for b in digits.bytes() {
            d = d.mul(&Big::from_u128(10)).add(&Big::from_u128((b - b'0') as u128));
        }
        let too_big = || "float literal overflows f128".to_string();
        if e10 >= 0 {
            return Q::round(false, &d.mul(&Big::pow10(e10 as u64)), 0, false).ok_or_else(too_big);
        }
        // Below 10^-5000, under half the smallest binary128 subnormal.
        if e10.saturating_add(digits.len() as i64) < -5000 {
            return Ok(Q::zero());
        }
        let den = Big::pow10(e10.unsigned_abs());
        // At least 116 quotient bits: 113 kept, a half bit, and more.
        let s = (den.bitlen() as i64 - d.bitlen() as i64 + 116).max(0) as u64;
        let (q, rem) = d.shl(s).div(&den);
        Q::round(false, &q, -(s as i64), rem).ok_or_else(too_big)
    }

    /// `a op b` rounded once to binary128, ties to even. None on division
    /// by zero and when the result leaves the binary128 range.
    pub(super) fn op(op: FOp, a: Q, b: Q) -> Option<Q> {
        let neg = a.neg != b.neg;
        match op {
            FOp::Add => Q::add(a, b),
            FOp::Sub => Q::add(a, b.neg()),
            FOp::Mul => Q::round(neg, &Big::from_u128(a.m).mul(&Big::from_u128(b.m)), a.e + b.e, false),
            FOp::Div => {
                if b.m == 0 {
                    return None;
                }
                if a.m == 0 {
                    return Some(Q { neg, ..Q::zero() });
                }
                let (x, y) = (Big::from_u128(a.m), Big::from_u128(b.m));
                let s = (y.bitlen() as i64 - x.bitlen() as i64 + 116).max(0) as u64;
                let (q, rem) = x.shl(s).div(&y);
                Q::round(neg, &q, a.e - s as i64 - b.e, rem)
            }
        }
    }

    fn add(a: Q, b: Q) -> Option<Q> {
        if a.m == 0 && b.m == 0 {
            return Some(Q { neg: a.neg && b.neg, ..Q::zero() });
        }
        if a.m == 0 {
            return Some(b);
        }
        if b.m == 0 {
            return Some(a);
        }
        let e = a.e.min(b.e);
        let x = Big::from_u128(a.m).shl((a.e - e) as u64);
        let y = Big::from_u128(b.m).shl((b.e - e) as u64);
        if a.neg == b.neg {
            return Q::round(a.neg, &x.add(&y), e, false);
        }
        match x.cmp(&y) {
            std::cmp::Ordering::Equal => Some(Q::zero()),
            std::cmp::Ordering::Greater => Q::round(a.neg, &x.sub(&y), e, false),
            std::cmp::Ordering::Less => Q::round(b.neg, &y.sub(&x), e, false),
        }
    }

    /// The order of the two values; the zeros are equal.
    pub(super) fn cmp(a: Q, b: Q) -> std::cmp::Ordering {
        let sign = |q: Q| if q.m == 0 { 0 } else if q.neg { -1 } else { 1 };
        let (sa, sb) = (sign(a), sign(b));
        if sa != sb || sa == 0 {
            return sa.cmp(&sb);
        }
        let e = a.e.min(b.e);
        let mag = Big::from_u128(a.m).shl((a.e - e) as u64).cmp(&Big::from_u128(b.m).shl((b.e - e) as u64));
        if sa < 0 {
            mag.reverse()
        } else {
            mag
        }
    }

    /// Rounded once to f64, ties to even; None past the f64 range.
    pub(super) fn to_f64(self) -> Option<f64> {
        let (m, lsb) = round_bits(&Big::from_u128(self.m), self.e, false, 53, -1074, 1023)?;
        let bits = if m >> 52 == 0 { m as u64 } else { (((lsb + 52 + 1023) as u64) << 52) | (m as u64 & ((1u64 << 52) - 1)) };
        Some(f64::from_bits(bits | ((self.neg as u64) << 63)))
    }

    /// Rounded once to f32, ties to even; past the f32 range, infinity.
    pub(super) fn to_f32(self) -> f32 {
        let bits = match round_bits(&Big::from_u128(self.m), self.e, false, 24, -149, 127) {
            Some((m, _)) if m >> 23 == 0 => m as u32,
            Some((m, lsb)) => (((lsb + 23 + 127) as u32) << 23) | (m as u32 & ((1u32 << 23) - 1)),
            None => f32::INFINITY.to_bits(),
        };
        f32::from_bits(bits | ((self.neg as u32) << 31))
    }

    /// The value as an f64 when it is one exactly.
    pub(super) fn exact_f64(self) -> Option<f64> {
        let f = self.to_f64()?;
        (Q::cmp(Q::from_f64(f), self) == std::cmp::Ordering::Equal).then_some(f)
    }

    /// The nearest f64, or an infinity, for messages.
    pub(super) fn approx(self) -> f64 {
        self.to_f64().unwrap_or(if self.neg { f64::NEG_INFINITY } else { f64::INFINITY })
    }
}
