#[derive(Debug, Clone, PartialEq)]
pub enum ImError {
    NoInverse { a: u64, n: u64 },
    ZeroModulus,
}

impl std::fmt::Display for ImError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImError::NoInverse { a, n } => write!(f, "no inverse: {a} mod {n}"),
            ImError::ZeroModulus => write!(f, "zero modulus"),
        }
    }
}

impl std::error::Error for ImError {}

pub struct IntMod {
    n: u64,
    total_ops: u64,
}

impl IntMod {
    pub fn new(n: u64) -> Result<Self, ImError> {
        if n == 0 { return Err(ImError::ZeroModulus); }
        Ok(Self { n, total_ops: 0 })
    }

    pub fn add(&mut self, a: u64, b: u64) -> u64 { self.total_ops += 1; ((a % self.n) + (b % self.n)) % self.n }
    pub fn sub(&mut self, a: u64, b: u64) -> u64 { self.total_ops += 1; (self.n + a % self.n - b % self.n) % self.n }
    pub fn mul(&mut self, a: u64, b: u64) -> u64 { self.total_ops += 1; ((a % self.n) as u128 * (b % self.n) as u128 % self.n as u128) as u64 }

    pub fn pow(&mut self, mut base: u64, mut exp: u64) -> u64 {
        self.total_ops += 1;
        base %= self.n;
        let mut result = 1u64;
        let n = self.n as u128;
        while exp > 0 {
            if exp & 1 == 1 { result = (result as u128 * base as u128 % n) as u64; }
            exp >>= 1;
            base = (base as u128 * base as u128 % n) as u64;
        }
        result
    }

    pub fn inverse(&mut self, a: u64) -> Result<u64, ImError> {
        self.total_ops += 1;
        let a = a % self.n;
        if a == 0 { return Err(ImError::NoInverse { a, n: self.n }); }
        let (g, x, _) = self.extended_gcd(a as i64, self.n as i64);
        if g != 1 { return Err(ImError::NoInverse { a, n: self.n }); }
        Ok((x.rem_euclid(self.n as i64)) as u64)
    }

    fn extended_gcd(&self, a: i64, b: i64) -> (i64, i64, i64) {
        if a == 0 { return (b, 0, 1); }
        let (g, x, y) = self.extended_gcd(b % a, a);
        (g, y - (b / a) * x, x)
    }

    pub fn div(&mut self, a: u64, b: u64) -> Result<u64, ImError> {
        let inv = self.inverse(b)?;
        Ok(self.mul(a, inv))
    }

    pub fn is_quadratic_residue(&mut self, a: u64) -> bool {
        if self.n == 2 { return true; }
        let e = (self.n - 1) / 2;
        self.pow(a, e) == 1
    }

    pub fn modulus(&self) -> u64 { self.n }
    pub fn total_ops(&self) -> u64 { self.total_ops }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_im() { let im = IntMod::new(7).unwrap(); assert_eq!(im.modulus(), 7); }

    #[test]
    fn add_sub() {
        let mut im = IntMod::new(7).unwrap();
        assert_eq!(im.add(5, 6), 4);
        assert_eq!(im.sub(2, 5), 4);
    }

    #[test]
    fn mul() {
        let mut im = IntMod::new(7).unwrap();
        assert_eq!(im.mul(3, 5), 1);
        assert_eq!(im.mul(6, 6), 1);
    }

    #[test]
    fn pow() {
        let mut im = IntMod::new(7).unwrap();
        assert_eq!(im.pow(2, 3), 1);
        assert_eq!(im.pow(3, 6), 1);
    }

    #[test]
    fn inverse() {
        let mut im = IntMod::new(7).unwrap();
        let inv = im.inverse(3).unwrap();
        assert_eq!(im.mul(3, inv), 1);
    }

    #[test]
    fn no_inverse() {
        let mut im = IntMod::new(12).unwrap();
        assert!(im.inverse(4).is_err());
    }

    #[test]
    fn div() {
        let mut im = IntMod::new(7).unwrap();
        let q = im.div(6, 2).unwrap();
        assert_eq!(im.mul(q, 2), 6);
    }

    #[test]
    fn quadratic_residue() {
        let mut im = IntMod::new(7).unwrap();
        assert!(im.is_quadratic_residue(2));
        assert!(!im.is_quadratic_residue(3));
    }

    #[test]
    fn zero_modulus() { assert!(IntMod::new(0).is_err()); }

    #[test]
    fn stats() {
        let mut im = IntMod::new(7).unwrap();
        im.add(1, 2); im.mul(3, 4);
        assert_eq!(im.total_ops(), 2);
    }

    #[test]
    fn error_display() { assert!(ImError::NoInverse { a: 4, n: 12 }.to_string().contains("inverse")); }
}
