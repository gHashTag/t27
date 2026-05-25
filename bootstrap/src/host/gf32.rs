use std::cell::Cell;

const POLY: u8 = 0x25;
const FIELD_SIZE: usize = 32;

fn make_tables() -> ([u8; 32], [u8; 32]) {
    let mut log = [0u8; 32]; let mut antilog = [0u8; 32];
    let mut x = 1u8;
    for i in 0..31 {
        antilog[i] = x; log[x as usize] = i as u8;
        x <<= 1;
        if x >= 32 { x ^= POLY; x &= 0x1F; }
    }
    antilog[31] = 1;
    (log, antilog)
}

pub struct Gf32 {
    log: [u8; 32],
    antilog: [u8; 32],
    total_ops: Cell<u64>,
}

impl Gf32 {
    pub fn new() -> Self { let (log, antilog) = make_tables(); Self { log, antilog, total_ops: Cell::new(0) } }

    pub fn add(&self, a: u8, b: u8) -> u8 { self.total_ops.set(self.total_ops.get() + 1); (a ^ b) & 0x1F }
    pub fn sub(&self, a: u8, b: u8) -> u8 { self.add(a, b) }

    pub fn mul(&self, a: u8, b: u8) -> u8 {
        self.total_ops.set(self.total_ops.get() + 1);
        if a == 0 || b == 0 { return 0; }
        self.antilog[((self.log[a as usize] as u16 + self.log[b as usize] as u16) % 31) as usize]
    }

    pub fn div(&self, a: u8, b: u8) -> Option<u8> {
        self.total_ops.set(self.total_ops.get() + 1);
        if b == 0 { return None; }
        if a == 0 { return Some(0); }
        let la = self.log[a as usize] as i16;
        let lb = self.log[b as usize] as i16;
        Some(self.antilog[((la - lb + 31) % 31) as usize])
    }

    pub fn pow(&self, a: u8, mut exp: u32) -> u8 {
        self.total_ops.set(self.total_ops.get() + 1);
        if exp == 0 { return 1; }
        let mut result = 1u8;
        let mut base = a;
        while exp > 0 {
            if exp & 1 == 1 { result = self.mul(result, base); }
            base = self.mul(base, base);
            exp >>= 1;
        }
        result
    }

    pub fn inv(&self, a: u8) -> Option<u8> { if a == 0 { None } else { self.div(1, a) } }

    pub fn poly_eval(&self, coeffs: &[u8], x: u8) -> u8 {
        self.total_ops.set(self.total_ops.get() + 1);
        let mut result = 0u8;
        for &c in coeffs.iter().rev() { result = self.add(self.mul(result, x), c); }
        result
    }

    pub fn field_size(&self) -> usize { FIELD_SIZE }
    pub fn total_ops(&self) -> u64 { self.total_ops.get() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let gf = Gf32::new();
        let s = gf.add(5, 17);
        assert_eq!(gf.sub(s, 17), 5);
    }

    #[test]
    fn mul_commute() {
        let gf = Gf32::new();
        assert_eq!(gf.mul(3, 7), gf.mul(7, 3));
    }

    #[test]
    fn mul_identity() {
        let gf = Gf32::new();
        assert_eq!(gf.mul(5, 1), 5);
    }

    #[test]
    fn mul_zero() { assert_eq!(Gf32::new().mul(5, 0), 0); }

    #[test]
    fn div_roundtrip() {
        let gf = Gf32::new();
        let p = gf.mul(3, 7);
        assert_eq!(gf.div(p, 7), Some(3));
    }

    #[test]
    fn div_zero() { assert!(Gf32::new().div(1, 0).is_none()); }

    #[test]
    fn inv() {
        let gf = Gf32::new();
        let inv = gf.inv(3).unwrap();
        assert_eq!(gf.mul(3, inv), 1);
    }

    #[test]
    fn pow() {
        let gf = Gf32::new();
        assert_eq!(gf.pow(2, 0), 1);
        let p2_5 = gf.pow(2, 5);
        let manual = gf.mul(gf.mul(gf.mul(gf.mul(2, 2), 2), 2), 2);
        assert_eq!(p2_5, manual);
    }

    #[test]
    fn poly_eval() {
        let gf = Gf32::new();
        let v = gf.poly_eval(&[1, 0, 1], 2);
        assert_eq!(v, gf.add(1, gf.mul(4, 1)));
    }

    #[test]
    fn stats() {
        let gf = Gf32::new();
        gf.add(1, 2); gf.mul(3, 4);
        assert!(gf.total_ops() >= 2);
    }
}
