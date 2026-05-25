pub struct Matmul64;

impl Matmul64 {
    pub fn multiply(a: &[Vec<u64>], b: &[Vec<u64>], modulus: u64) -> Vec<Vec<u64>> {
        let n = a.len();
        if n == 0 { return Vec::new(); }
        let m = b[0].len();
        let k = b.len();
        let mut c = vec![vec![0u64; m]; n];
        for i in 0..n {
            for j in 0..m {
                let mut sum = 0u128;
                for p in 0..k {
                    sum += (a[i][p] as u128) * (b[p][j] as u128);
                }
                c[i][j] = (sum % modulus as u128) as u64;
            }
        }
        c
    }

    pub fn identity(n: usize) -> Vec<Vec<u64>> {
        let mut m = vec![vec![0u64; n]; n];
        for i in 0..n { m[i][i] = 1; }
        m
    }

    pub fn pow(mat: &[Vec<u64>], mut exp: u64, modulus: u64) -> Vec<Vec<u64>> {
        let n = mat.len();
        if n == 0 { return Vec::new(); }
        let mut result = Self::identity(n);
        let mut base = mat.to_vec();
        while exp > 0 {
            if exp & 1 == 1 { result = Self::multiply(&result, &base, modulus); }
            base = Self::multiply(&base, &base, modulus);
            exp >>= 1;
        }
        result
    }

    pub fn trace(mat: &[Vec<u64>], modulus: u64) -> u64 {
        (mat.iter().enumerate().fold(0u128, |acc, (i, row)| acc + row[i] as u128) % modulus as u128) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOD: u64 = 1_000_000_007;

    fn mat2(a: u64, b: u64, c: u64, d: u64) -> Vec<Vec<u64>> {
        vec![vec![a, b], vec![c, d]]
    }

    #[test]
    fn identity_mul() {
        let id = Matmul64::identity(2);
        let a = mat2(1, 2, 3, 4);
        let r = Matmul64::multiply(&id, &a, MOD);
        assert_eq!(r, a);
    }

    #[test]
    fn pow_identity() {
        let a = mat2(1, 2, 3, 4);
        let r = Matmul64::pow(&a, 0, MOD);
        assert_eq!(r, Matmul64::identity(2));
    }

    #[test]
    fn pow_one() {
        let a = mat2(1, 2, 3, 4);
        let r = Matmul64::pow(&a, 1, MOD);
        assert_eq!(r, a);
    }

    #[test]
    fn fib_matrix() {
        let f = mat2(1, 1, 1, 0);
        let r = Matmul64::pow(&f, 10, MOD);
        assert_eq!((r[0][1]) % MOD, 55);
    }

    #[test]
    fn trace_mod() {
        let a = mat2(MOD - 1, 0, 0, MOD - 1);
        assert_eq!(Matmul64::trace(&a, MOD), (MOD as u128 * 2 - 2) as u64 % MOD);
    }

    #[test]
    fn empty() {
        assert!(Matmul64::multiply(&[], &[], MOD).is_empty());
        assert!(Matmul64::pow(&[], 5, MOD).is_empty());
    }
}
