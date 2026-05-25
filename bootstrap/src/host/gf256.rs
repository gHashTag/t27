use std::cell::Cell;

pub struct Gf256 {
    log: [u8; 256],
    antilog: [u8; 255],
    ops: Cell<u64>,
}

impl Gf256 {
    const PRIM: u8 = 0x1B;
    const ORDER: usize = 255;

    pub fn new() -> Self {
        let mut log = [0u8; 256];
        let mut antilog = [0u8; 255];
        let mut x: u8 = 1;
        let gen: u8 = 3;
        for i in 0..255 {
            antilog[i] = x;
            log[x as usize] = i as u8;
            let mut new_x = 0u8;
            let mut aa = x;
            let mut bb = gen;
            while bb > 0 {
                if bb & 1 != 0 { new_x ^= aa; }
                bb >>= 1;
                let carry = aa & 0x80;
                aa <<= 1;
                if carry != 0 { aa ^= Self::PRIM; }
            }
            x = new_x;
        }
        Self { log, antilog, ops: Cell::new(0) }
    }

    pub fn add(&self, a: u8, b: u8) -> u8 { self.ops.set(self.ops.get() + 1); a ^ b }
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
        assert_ne!(b, 0, "GF(256) div by zero");
        if a == 0 { return 0; }
        let la = self.log[a as usize] as usize;
        let lb = self.log[b as usize] as usize;
        self.antilog[(la + Self::ORDER - lb) % Self::ORDER]
    }

    pub fn inv(&self, a: u8) -> u8 {
        assert_ne!(a, 0, "GF(256) inv(0)");
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

    pub fn poly_mul(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; a.len() + b.len() - 1];
        for i in 0..a.len() { for j in 0..b.len() { result[i + j] = self.add(result[i + j], self.mul(a[i], b[j])); } }
        result
    }

    pub fn ops(&self) -> u64 { self.ops.get() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() { let gf = Gf256::new(); for a in 0u8..=255 { assert_eq!(gf.add(a, a), 0); } }

    #[test]
    fn mul_identity() { let gf = Gf256::new(); for a in 1u8..=255 { assert_eq!(gf.mul(a, 1), a); } }

    #[test]
    fn mul_zero() { let gf = Gf256::new(); for a in 0u8..=255 { assert_eq!(gf.mul(a, 0), 0); } }

    #[test]
    fn div_roundtrip() {
        let gf = Gf256::new();
        for a in 1u8..=255 { for b in 1u8..=255 { assert_eq!(gf.mul(gf.div(a, b), b), a); } }
    }

    #[test]
    fn inv() { let gf = Gf256::new(); for a in 1u8..=255 { assert_eq!(gf.mul(a, gf.inv(a)), 1); } }

    #[test]
    fn pow() { let gf = Gf256::new(); assert_eq!(gf.pow(2, 0), 1); assert_eq!(gf.pow(2, 8), gf.mul(2, gf.pow(2, 7))); }

    #[test]
    fn poly_eval() { let gf = Gf256::new(); assert_eq!(gf.poly_eval(&[3, 1, 2], 0), 3); }

    #[test]
    fn poly_mul() {
        let gf = Gf256::new();
        let r = gf.poly_mul(&[1, 2], &[1, 3]);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0], gf.mul(1, 1));
        assert_eq!(r[2], gf.mul(2, 3));
    }

    #[test]
    fn ops_count() { let gf = Gf256::new(); gf.add(1, 2); gf.mul(3, 4); assert!(gf.ops() >= 2); }
}
