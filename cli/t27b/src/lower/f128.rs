//! Zig's comptime_float: an IEEE binary128 value (#7021).
//!
//! The reference keeps an untyped float literal, and a `const` set from
//! one, as a comptime_float. Zig parses the literal to binary128 (round to
//! nearest, ties to even), folds `+ - * /` on two of them in binary128,
//! rounding each result the same way, compares two of them exactly, and
//! rounds to f64 or f32 only when the value meets one. This module does
//! that arithmetic in software: every operation forms the exact result as
//! a ratio of integers times a power of two and rounds it once.

use std::cmp::Ordering;

/// A finite binary128 value: `(-1)^neg * m * 2^e` with `m < 2^113`; zero
/// when `m == 0` (`neg` is then the sign of the zero).
#[derive(Clone, Copy, Debug)]
pub(crate) struct F128 {
    neg: bool,
    m: u128,
    e: i64,
}

/// An IEEE binary format: `p` significant bits (the hidden one included),
/// the exponent of the least subnormal bit, and `top`: every finite value
/// lies below `2^top`.
struct Fmt {
    p: u32,
    emin_lsb: i64,
    top: i64,
}

const B128: Fmt = Fmt { p: 113, emin_lsb: -16494, top: 16384 };
const B64: Fmt = Fmt { p: 53, emin_lsb: -1074, top: 1024 };
const B32: Fmt = Fmt { p: 24, emin_lsb: -149, top: 128 };

/// A natural number, little-endian 64-bit limbs, no high zero limb.
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

    /// The number of significant bits (0 for zero).
    fn bits(&self) -> i64 {
        match self.0.last() {
            None => 0,
            Some(&t) => (self.0.len() as i64 - 1) * 64 + (64 - t.leading_zeros() as i64),
        }
    }

    fn shl(&self, k: u64) -> Big {
        if self.is_zero() {
            return Big(vec![]);
        }
        let (w, b) = ((k / 64) as usize, (k % 64) as u32);
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
        let mut r = Big(out);
        r.trim();
        r
    }

    fn shr1(&mut self) {
        let n = self.0.len();
        for i in 0..n {
            let hi = if i + 1 < n { self.0[i + 1] << 63 } else { 0 };
            self.0[i] = (self.0[i] >> 1) | hi;
        }
        self.trim();
    }

    fn cmp(&self, o: &Big) -> Ordering {
        if self.0.len() != o.0.len() {
            return self.0.len().cmp(&o.0.len());
        }
        for i in (0..self.0.len()).rev() {
            if self.0[i] != o.0[i] {
                return self.0[i].cmp(&o.0[i]);
            }
        }
        Ordering::Equal
    }

    /// `self -= o`, for `self >= o`.
    fn sub_assign(&mut self, o: &Big) {
        let mut borrow = false;
        for i in 0..self.0.len() {
            let y = o.0.get(i).copied().unwrap_or(0);
            let (d1, b1) = self.0[i].overflowing_sub(y);
            let (d2, b2) = d1.overflowing_sub(borrow as u64);
            self.0[i] = d2;
            borrow = b1 || b2;
        }
        self.trim();
    }

    fn add(&self, o: &Big) -> Big {
        let n = self.0.len().max(o.0.len());
        let mut out = Vec::with_capacity(n + 1);
        let mut carry = 0u128;
        for i in 0..n {
            let s = self.0.get(i).copied().unwrap_or(0) as u128 + o.0.get(i).copied().unwrap_or(0) as u128 + carry;
            out.push(s as u64);
            carry = s >> 64;
        }
        if carry != 0 {
            out.push(carry as u64);
        }
        let mut r = Big(out);
        r.trim();
        r
    }

    fn mul(&self, o: &Big) -> Big {
        if self.is_zero() || o.is_zero() {
            return Big(vec![]);
        }
        let mut out = vec![0u64; self.0.len() + o.0.len()];
        for (i, &x) in self.0.iter().enumerate() {
            let mut carry = 0u128;
            for (j, &y) in o.0.iter().enumerate() {
                let t = out[i + j] as u128 + x as u128 * y as u128 + carry;
                out[i + j] = t as u64;
                carry = t >> 64;
            }
            let mut k = i + o.0.len();
            while carry != 0 {
                let t = out[k] as u128 + carry;
                out[k] = t as u64;
                carry = t >> 64;
                k += 1;
            }
        }
        let mut r = Big(out);
        r.trim();
        r
    }

    fn pow(base: u64, mut n: u64) -> Big {
        let mut r = Big::from_u128(1);
        let mut b = Big::from_u128(base as u128);
        while n > 0 {
            if n & 1 == 1 {
                r = r.mul(&b);
            }
            n >>= 1;
            if n > 0 {
                b = b.mul(&b);
            }
        }
        r
    }
}

/// A value rounded to some format: `(-1)^neg * m * 2^e`, and whether the
/// rounding was exact.
struct Rounded {
    neg: bool,
    m: u128,
    e: i64,
    exact: bool,
}

/// `num / den * 2^e2` (`den > 0`) rounded to nearest, ties to even, in
/// `f`, subnormals included. None when the result reaches `2^f.top`.
fn round(neg: bool, num: &Big, den: &Big, e2: i64, f: &Fmt) -> Option<Rounded> {
    if num.is_zero() {
        return Some(Rounded { neg, m: 0, e: 0, exact: true });
    }
    let p = f.p as i64;
    // Scale so that q = floor(num * 2^s / den) lies in [2^(p+1), 2^(p+3)).
    let s = p + 2 - (num.bits() - den.bits());
    let (mut r, d) = if s >= 0 { (num.shl(s as u64), den.clone()) } else { (num.clone(), den.shl((-s) as u64)) };
    let qb = (p + 3) as u64;
    let mut dd = d.shl(qb);
    let mut q: u128 = 0;
    for i in (0..=qb).rev() {
        if r.cmp(&dd) != Ordering::Less {
            r.sub_assign(&dd);
            q |= 1u128 << i;
        }
        dd.shr1();
    }
    let sticky = !r.is_zero();
    // q's unit bit has weight 2^lsb; drop `shift` (>= 2) bits of it.
    let lsb = e2 - s;
    let bl = 128 - q.leading_zeros() as i64;
    let shift = (bl - p).max(f.emin_lsb - lsb);
    let (kept, up, exact) = if shift >= 128 {
        // Everything is below half of the least bit: rounds to zero.
        (0u128, false, false)
    } else {
        let kept = q >> shift;
        let rest = q & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        let up = rest > half || (rest == half && (sticky || kept & 1 == 1));
        (kept, up, rest == 0 && !sticky)
    };
    let mut m = kept + up as u128;
    let mut e = lsb + shift;
    if m >> f.p != 0 {
        m >>= 1;
        e += 1;
    }
    if m != 0 && e + (128 - m.leading_zeros() as i64) > f.top {
        return None;
    }
    Some(Rounded { neg, m, e, exact })
}

/// The IEEE bits of `r` in a format of `width` bits and `p` significant
/// bits whose least subnormal bit is `2^emin_lsb` (`top`: overflow bound).
fn ieee_bits(r: Option<Rounded>, neg: bool, width: u32, f: &Fmt) -> u64 {
    let p = f.p;
    let ebits = width - p;
    let sign = (neg as u64) << (width - 1);
    let Some(r) = r else {
        return sign | (((1u64 << ebits) - 1) << (p - 1));
    };
    if r.m == 0 {
        return sign;
    }
    let bl = 128 - r.m.leading_zeros() as i64;
    let emin_normal = f.emin_lsb + p as i64 - 1;
    let top = r.e + bl - 1;
    if top >= emin_normal {
        let frac = ((r.m << (p as i64 - bl)) as u64) & ((1u64 << (p - 1)) - 1);
        let biased = (top - emin_normal + 1) as u64;
        sign | (biased << (p - 1)) | frac
    } else {
        sign | ((r.m << (r.e - f.emin_lsb)) as u64)
    }
}

impl F128 {
    fn from_rounded(r: Rounded) -> F128 {
        F128 { neg: r.neg, m: r.m, e: r.e }
    }

    fn zero(neg: bool) -> F128 {
        F128 { neg, m: 0, e: 0 }
    }

    /// The decimal `digits * 10^e10` (`digits`: ASCII decimal digits)
    /// rounded to binary128. None when it overflows binary128. The caller
    /// bounds `e10`: the work grows with it.
    pub(crate) fn from_decimal(digits: &str, e10: i64) -> Option<F128> {
        let ten = Big::from_u128(10);
        let mut m = Big(vec![]);
        for b in digits.bytes() {
            m = m.mul(&ten).add(&Big::from_u128((b - b'0') as u128));
        }
        let one = Big::from_u128(1);
        let r = if e10 >= 0 {
            round(false, &m.mul(&Big::pow(5, e10 as u64)), &one, e10, &B128)
        } else {
            round(false, &m, &Big::pow(5, (-e10) as u64), e10, &B128)
        };
        r.map(F128::from_rounded)
    }

    /// A comptime_int as a comptime_float: None unless it is exactly one.
    pub(crate) fn from_int(c: i128) -> Option<F128> {
        let a = c.unsigned_abs();
        if a >> B128.p != 0 {
            return None;
        }
        Some(F128 { neg: c < 0, m: a, e: 0 })
    }

    pub(crate) fn is_zero(self) -> bool {
        self.m == 0
    }

    pub(crate) fn neg(self) -> F128 {
        F128 { neg: !self.neg, ..self }
    }

    pub(crate) fn add(self, o: F128) -> Option<F128> {
        if self.m == 0 && o.m == 0 {
            return Some(F128::zero(self.neg && o.neg));
        }
        if self.m == 0 {
            return Some(o);
        }
        if o.m == 0 {
            return Some(self);
        }
        let e = self.e.min(o.e);
        let mut a = Big::from_u128(self.m).shl((self.e - e) as u64);
        let mut b = Big::from_u128(o.m).shl((o.e - e) as u64);
        let one = Big::from_u128(1);
        let r = if self.neg == o.neg {
            round(self.neg, &a.add(&b), &one, e, &B128)
        } else {
            match a.cmp(&b) {
                Ordering::Equal => return Some(F128::zero(false)),
                Ordering::Greater => {
                    a.sub_assign(&b);
                    round(self.neg, &a, &one, e, &B128)
                }
                Ordering::Less => {
                    b.sub_assign(&a);
                    round(o.neg, &b, &one, e, &B128)
                }
            }
        };
        r.map(F128::from_rounded)
    }

    pub(crate) fn sub(self, o: F128) -> Option<F128> {
        self.add(o.neg())
    }

    pub(crate) fn mul(self, o: F128) -> Option<F128> {
        let neg = self.neg != o.neg;
        let num = Big::from_u128(self.m).mul(&Big::from_u128(o.m));
        round(neg, &num, &Big::from_u128(1), self.e + o.e, &B128).map(F128::from_rounded)
    }

    /// None for a zero divisor or an overflow.
    pub(crate) fn div(self, o: F128) -> Option<F128> {
        if o.m == 0 {
            return None;
        }
        let neg = self.neg != o.neg;
        round(neg, &Big::from_u128(self.m), &Big::from_u128(o.m), self.e - o.e, &B128).map(F128::from_rounded)
    }

    /// The exact order of two values (-0 equals +0).
    pub(crate) fn cmp(self, o: F128) -> Ordering {
        let sign = |x: F128| if x.m == 0 { 0 } else if x.neg { -1 } else { 1 };
        let (sa, sb) = (sign(self), sign(o));
        if sa != sb || sa == 0 {
            return sa.cmp(&sb);
        }
        let bl = |m: u128| 128 - m.leading_zeros() as i64;
        let (ta, tb) = (self.e + bl(self.m), o.e + bl(o.m));
        let mag = if ta != tb {
            ta.cmp(&tb)
        } else if self.e >= o.e {
            (self.m << (self.e - o.e)).cmp(&o.m)
        } else {
            self.m.cmp(&(o.m << (o.e - self.e)))
        };
        if sa < 0 {
            mag.reverse()
        } else {
            mag
        }
    }

    fn to_fmt(self, f: &Fmt) -> Option<Rounded> {
        round(self.neg, &Big::from_u128(self.m), &Big::from_u128(1), self.e, f)
    }

    /// Rounded to f64 (to nearest even); out of range to an infinity.
    pub(crate) fn to_f64(self) -> f64 {
        f64::from_bits(ieee_bits(self.to_fmt(&B64), self.neg, 64, &B64))
    }

    /// Whether the value is an f64 exactly.
    pub(crate) fn is_f64(self) -> bool {
        self.to_fmt(&B64).is_some_and(|r| r.exact)
    }

    /// Rounded to f32 (to nearest even); out of range to an infinity.
    pub(crate) fn to_f32(self) -> f32 {
        f32::from_bits(ieee_bits(self.to_fmt(&B32), self.neg, 32, &B32) as u32)
    }
}
