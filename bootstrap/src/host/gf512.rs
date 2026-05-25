use std::cell::Cell;

pub struct Gf512 {
    log: [u16; 512],
    antilog: [u16; 511],
    ops: Cell<u64>,
}

impl Gf512 {
    const PRIM: u16 = 0x113;
    const ORDER: usize = 511;

    pub fn new() -> Self {
        let mut log = [0u16; 512];
        let mut antilog = [0u16; 511];
        let mut x: u16 = 1;
        let gen: u16 = 2;
        for i in 0..511 {
            antilog[i] = x;
            log[x as usize] = i as u16;
            let mut new_x = 0u16;
            let mut aa = x;
            let mut bb = gen;
            while bb > 0 {
                if bb & 1 != 0 { new_x ^= aa; }
                bb >>= 1;
                let carry = aa & 0x100;
                aa <<= 1;
                if carry != 0 { aa ^= Self::PRIM; }
                aa &= 0x1FF;
            }
            x = new_x;
        }
        Self { log, antilog, ops: Cell::new(0) }
    }

    pub fn add(&self, a: u16, b: u16) -> u16 { self.ops.set(self.ops.get() + 1); (a ^ b) & 0x1FF }
    pub fn sub(&self, a: u16, b: u16) -> u16 { self.add(a, b) }

    pub fn mul(&self, a: u16, b: u16) -> u16 {
        self.ops.set(self.ops.get() + 1);
        if a == 0 || b == 0 { return 0; }
        let la = self.log[a as usize] as usize;
        let lb = self.log[b as usize] as usize;
        self.antilog[(la + lb) % Self::ORDER]
    }

    pub fn div(&self, a: u16, b: u16) -> u16 {
        self.ops.set(self.ops.get() + 1);
        assert_ne!(b, 0, "GF(512) div by zero");
        if a == 0 { return 0; }
        let la = self.log[a as usize] as usize;
        let lb = self.log[b as usize] as usize;
        self.antilog[(la + Self::ORDER - lb) % Self::ORDER]
    }

    pub fn inv(&self, a: u16) -> u16 {
        assert_ne!(a, 0, "GF(512) inv(0)");
        let la = self.log[a as usize] as usize;
        self.antilog[(Self::ORDER - la) % Self::ORDER]
    }

    pub fn poly_eval(&self, coeffs: &[u16], x: u16) -> u16 {
        let mut result = 0u16;
        for &c in coeffs.iter().rev() { result = self.add(self.mul(result, x), c); }
        result
    }

    pub fn ops(&self) -> u64 { self.ops.get() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() { let gf = Gf512::new(); for a in 0..512u16 { assert_eq!(gf.add(a, a), 0); } }

    #[test]
    fn mul_identity() { let gf = Gf512::new(); for a in 1..512u16 { assert_eq!(gf.mul(a, 1), a); } }

    #[test]
    fn mul_zero() { let gf = Gf512::new(); for a in 0..512u16 { assert_eq!(gf.mul(a, 0), 0); } }

    #[test]
    fn div_roundtrip() {
        let gf = Gf512::new();
        for a in 1..512u16 { for b in 1..16u16 { assert_eq!(gf.mul(gf.div(a, b), b), a); } }
    }

    #[test]
    fn inv() { let gf = Gf512::new(); for a in 1..512u16 { assert_eq!(gf.mul(a, gf.inv(a)), 1); } }

    #[test]
    fn poly_eval() { let gf = Gf512::new(); assert_eq!(gf.poly_eval(&[3, 1, 2], 0), 3); }

    #[test]
    fn ops_count() { let gf = Gf512::new(); gf.add(1, 2); gf.mul(3, 4); assert!(gf.ops() >= 2); }
}
