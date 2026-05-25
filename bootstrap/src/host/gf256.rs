use std::cell::Cell;

#[derive(Debug, Clone, PartialEq)]
pub enum GfError {
    ZeroInverse,
}

impl std::fmt::Display for GfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GfError::ZeroInverse => write!(f, "cannot invert zero"),
        }
    }
}

impl std::error::Error for GfError {}

const PRIM: u8 = 0x03;

pub struct Gf256 {
    exp_table: [u8; 512],
    log_table: [u8; 256],
    total_multiplies: Cell<u64>,
    total_inverses: Cell<u64>,
    total_divides: Cell<u64>,
}

impl Gf256 {
    pub fn new() -> Self {
        let mut exp_table = [0u8; 512];
        let mut log_table = [0u8; 256];
        let mut x: u8 = 1;
        for i in 0..255 {
            exp_table[i] = x;
            exp_table[i + 255] = x;
            log_table[x as usize] = i as u8;
            x = x.wrapping_mul(PRIM);
            if x >= 128 { x ^= 0x11B; }
        }
        Self { exp_table, log_table, total_multiplies: Cell::new(0), total_inverses: Cell::new(0), total_divides: Cell::new(0) }
    }

    pub fn add(&self, a: u8, b: u8) -> u8 { a ^ b }
    pub fn sub(&self, a: u8, b: u8) -> u8 { a ^ b }

    pub fn mul(&self, a: u8, b: u8) -> u8 {
        self.total_multiplies.set(self.total_multiplies.get() + 1);
        if a == 0 || b == 0 { return 0; }
        self.exp_table[(self.log_table[a as usize] as usize + self.log_table[b as usize] as usize) % 255]
    }

    pub fn div(&self, a: u8, b: u8) -> u8 {
        self.total_divides.set(self.total_divides.get() + 1);
        if b == 0 { panic!("GF(256) div by zero"); }
        if a == 0 { return 0; }
        let log_a = self.log_table[a as usize] as usize;
        let log_b = self.log_table[b as usize] as usize;
        self.exp_table[(log_a + 255 - log_b) % 255]
    }

    pub fn inv(&self, a: u8) -> Result<u8, GfError> {
        self.total_inverses.set(self.total_inverses.get() + 1);
        if a == 0 { return Err(GfError::ZeroInverse); }
        Ok(self.exp_table[255 - self.log_table[a as usize] as usize])
    }

    pub fn pow(&self, a: u8, exp: u32) -> u8 {
        if exp == 0 { return 1; }
        if a == 0 { return 0; }
        let log_a = self.log_table[a as usize] as u64;
        self.exp_table[((log_a * exp as u64) % 255) as usize]
    }

    pub fn poly_eval(&self, coeffs: &[u8], x: u8) -> u8 {
        let mut result = 0u8;
        for &c in coeffs.iter().rev() {
            result = self.add(self.mul(result, x), c);
        }
        result
    }

    pub fn poly_mul(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; a.len() + b.len() - 1];
        for i in 0..a.len() {
            for j in 0..b.len() {
                result[i + j] = self.add(result[i + j], self.mul(a[i], b[j]));
            }
        }
        result
    }

    pub fn total_multiplies(&self) -> u64 { self.total_multiplies.get() }
    pub fn total_inverses(&self) -> u64 { self.total_inverses.get() }
    pub fn total_divides(&self) -> u64 { self.total_divides.get() }
}

impl Default for Gf256 {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gf() { let gf = Gf256::new(); assert_eq!(gf.add(1, 1), 0); }

    #[test]
    fn add_sub_same() {
        let gf = Gf256::new();
        let a = 0x57; let b = 0x83;
        assert_eq!(gf.add(a, b), gf.sub(a, b));
    }

    #[test]
    fn mul_identity() {
        let gf = Gf256::new();
        assert_eq!(gf.mul(1, 0x53), 0x53);
        assert_eq!(gf.mul(0x53, 1), 0x53);
    }

    #[test]
    fn mul_zero() {
        let gf = Gf256::new();
        assert_eq!(gf.mul(0, 0xFF), 0);
        assert_eq!(gf.mul(0xFF, 0), 0);
    }

    #[test]
    fn inv() {
        let gf = Gf256::new();
        let a = 0x53;
        let inv_a = gf.inv(a).unwrap();
        assert_eq!(gf.mul(a, inv_a), 1);
    }

    #[test]
    fn inv_zero() { assert!(Gf256::new().inv(0).is_err()); }

    #[test]
    fn div() {
        let gf = Gf256::new();
        let a = 0x53; let b = 0xCA;
        let product = gf.mul(a, b);
        let q = gf.div(product, b);
        assert_eq!(q, a);
    }

    #[test]
    fn pow() {
        let gf = Gf256::new();
        assert_eq!(gf.pow(2, 0), 1);
        assert_eq!(gf.pow(2, 1), 2);
        let r = gf.pow(2, 8);
        let mut expected = 1u8;
        for _ in 0..8 { expected = gf.mul(expected, 2); }
        assert_eq!(r, expected);
    }

    #[test]
    fn poly_eval() {
        let gf = Gf256::new();
        let x2 = gf.mul(2, 2);
        let expected = gf.add(1, x2);
        let result = gf.poly_eval(&[1, 0, 1], 2);
        assert_eq!(result, expected);
    }

    #[test]
    fn poly_mul() {
        let gf = Gf256::new();
        let result = gf.poly_mul(&[1, 1], &[1, 1]);
        assert_eq!(result, vec![1, 0, 1]);
    }

    #[test]
    fn stats() {
        let gf = Gf256::new();
        gf.mul(1, 2); gf.inv(3).unwrap(); gf.div(6, 3);
        assert_eq!(gf.total_multiplies(), 1);
        assert_eq!(gf.total_inverses(), 1);
        assert_eq!(gf.total_divides(), 1);
    }
}
