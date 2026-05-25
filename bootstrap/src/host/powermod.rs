pub struct PowerMod;

impl PowerMod {
    pub fn pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
        if modulus == 1 { return 0; }
        let m = modulus as u128;
        let mut result = 1u128;
        base %= modulus;
        let mut b = base as u128;
        while exp > 0 {
            if exp & 1 == 1 { result = result * b % m; }
            exp >>= 1;
            b = b * b % m;
        }
        result as u64
    }

    pub fn is_prime(n: u64) -> bool {
        if n < 2 { return false; }
        if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let witnesses: &[u64] = if n < 2_047 { &[2] }
            else if n < 1_373_653 { &[2, 3] }
            else if n < 9_080_191 { &[31, 73] }
            else if n < 25_326_001 { &[2, 3, 5] }
            else if n < 3_215_031_751 { &[2, 3, 5, 7] }
            else { &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] };
        let mut d = n - 1;
        let mut r = 0u32;
        while d % 2 == 0 { d /= 2; r += 1; }
        'next: for &a in witnesses {
            let a = a.min(n - 2);
            let mut x = Self::pow(a, d, n);
            if x == 1 || x == n - 1 { continue; }
            for _ in 0..r - 1 {
                x = (x as u128 * x as u128 % n as u128) as u64;
                if x == n - 1 { continue 'next; }
            }
            return false;
        }
        true
    }

    pub fn mod_inverse(a: u64, modulus: u64) -> Option<u64> {
        if a == 0 { return None; }
        let result = Self::pow(a, modulus - 2, modulus);
        if (a as u128 * result as u128 % modulus as u128) == 1 { Some(result) } else { None }
    }

    pub fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 { let t = b; b = a % b; a = t; }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pow_basic() { assert_eq!(PowerMod::pow(2, 10, 1000), 24); }

    #[test]
    fn pow_identity() { assert_eq!(PowerMod::pow(5, 0, 100), 1); }

    #[test]
    fn prime_small() {
        assert!(PowerMod::is_prime(7));
        assert!(!PowerMod::is_prime(9));
        assert!(PowerMod::is_prime(997));
    }

    #[test]
    fn inverse() {
        let inv = PowerMod::mod_inverse(3, 7).unwrap();
        assert_eq!(3u128 * inv as u128 % 7, 1);
    }

    #[test]
    fn gcd() {
        assert_eq!(PowerMod::gcd(12, 8), 4);
        assert_eq!(PowerMod::gcd(17, 13), 1);
    }

    #[test]
    fn fermat_little() {
        let p = 1_000_000_007u64;
        assert_eq!(PowerMod::pow(2, p - 1, p), 1);
    }
}
