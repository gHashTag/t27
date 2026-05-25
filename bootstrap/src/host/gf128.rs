use std::cell::Cell;

pub struct Gf128 {
    log: [u8; 128],
    antilog: [u8; 127],
    ops: Cell<u64>,
}

impl Gf128 {
    const PRIM: u8 = 0x89;
    const ORDER: usize = 127;

    pub fn new() -> Self {
        let mut log = [0u8; 128];
        let mut antilog = [0u8; 127];
        let mut x: u8 = 1;
        for i in 0..127 {
            antilog[i] = x;
            log[x as usize] = i as u8;
            x = Self::raw_mul(x, 2);
        }
        Self { log, antilog, ops: Cell::new(0) }
    }

    fn raw_mul(a: u8, b: u8) -> u8 {
        let mut r = 0u8;
        let mut aa = a;
        let mut bb = b;
        while bb > 0 {
            if bb & 1 != 0 { r ^= aa; }
            bb >>= 1;
            let carry = aa & 0x40;
            aa <<= 1;
            if carry != 0 { aa ^= Self::PRIM; }
            aa &= 0x7F;
        }
        r & 0x7F
    }

    pub fn add(&self, a: u8, b: u8) -> u8 { self.ops.set(self.ops.get() + 1); (a ^ b) & 0x7F }
    pub fn sub(&self, a: u8, b: u8) -> u8 { self.add(a, b) }

    pub fn mul(&self, a: u8, b: u8) -> u8 {
        self.ops.set(self.ops.get() + 1);
        if a == 0 || b == 0 { return 0; }
        let la = self.log[a as usize] as usize;
        let lb = self.log[b as usize] as usize;
        self.antilog[(la + lb) % Self::ORDER]
    }

    pub fn div(&self, a: u8, b: u8) -> u8 {
        self.ops.set(self.ops.get() + 1);
        assert_ne!(b, 0, "GF(128) div by zero");
        if a == 0 { return 0; }
        let la = self.log[a as usize] as usize;
        let lb = self.log[b as usize] as usize;
        self.antilog[(la + Self::ORDER - lb) % Self::ORDER]
    }

    pub fn inv(&self, a: u8) -> u8 {
        assert_ne!(a, 0, "GF(128) inv(0)");
        let la = self.log[a as usize] as usize;
        self.antilog[(Self::ORDER - la) % Self::ORDER]
    }

    pub fn pow(&self, a: u8, e: u32) -> u8 {
        if a == 0 { return if e == 0 { 1 } else { 0 }; }
        if e == 0 { return 1; }
        let la = self.log[a as usize] as u64;
        self.antilog[((la * e as u64) % Self::ORDER as u64) as usize]
    }

    pub fn poly_eval(&self, coeffs: &[u8], x: u8) -> u8 {
        let mut result = 0u8;
        for &c in coeffs.iter().rev() { result = self.add(self.mul(result, x), c); }
        result
    }

    pub fn ops(&self) -> u64 { self.ops.get() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() { let gf = Gf128::new(); for a in 0..128u8 { assert_eq!(gf.add(a, a), 0); } }

    #[test]
    fn mul_identity() { let gf = Gf128::new(); for a in 1..128u8 { assert_eq!(gf.mul(a, 1), a); } }

    #[test]
    fn mul_zero() { let gf = Gf128::new(); for a in 0..128u8 { assert_eq!(gf.mul(a, 0), 0); } }

    #[test]
    fn div_roundtrip() {
        let gf = Gf128::new();
        for a in 1..128u8 { for b in 1..128u8 { assert_eq!(gf.mul(gf.div(a, b), b), a); } }
    }

    #[test]
    fn inv() { let gf = Gf128::new(); for a in 1..128u8 { assert_eq!(gf.mul(a, gf.inv(a)), 1); } }

    #[test]
    fn pow() { let gf = Gf128::new(); assert_eq!(gf.pow(2, 0), 1); assert_eq!(gf.pow(2, 1), 2); }

    #[test]
    fn poly_eval() {
        let gf = Gf128::new();
        let c = vec![3, 1, 2];
        let at0 = gf.poly_eval(&c, 0);
        assert_eq!(at0, 3);
    }

    #[test]
    fn ops_count() { let gf = Gf128::new(); gf.add(1, 2); gf.mul(3, 4); assert!(gf.ops() >= 2); }
}
