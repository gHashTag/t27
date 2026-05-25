pub struct MatPow {
    total_muls: u64,
    total_pows: u64,
}

impl MatPow {
    pub fn new() -> Self { Self { total_muls: 0, total_pows: 0 } }

    pub fn identity(n: usize) -> Vec<Vec<u64>> {
        let mut m = vec![vec![0u64; n]; n];
        for i in 0..n { m[i][i] = 1; }
        m
    }

    pub fn mul_mod(&mut self, a: &[Vec<u64>], b: &[Vec<u64>], modulus: u64) -> Vec<Vec<u64>> {
        self.total_muls += 1;
        let n = a.len();
        let m = modulus as u128;
        let mut c = vec![vec![0u128; n]; n];
        for i in 0..n {
            for k in 0..n {
                for j in 0..n {
                    c[i][j] = (c[i][j] + a[i][k] as u128 * b[k][j] as u128) % m;
                }
            }
        }
        c.into_iter().map(|row| row.into_iter().map(|v| v as u64).collect()).collect()
    }

    pub fn pow_mod(&mut self, mat: &[Vec<u64>], mut exp: u64, modulus: u64) -> Vec<Vec<u64>> {
        self.total_pows += 1;
        let n = mat.len();
        let mut result = Self::identity(n);
        let mut base = mat.to_vec();
        while exp > 0 {
            if exp & 1 == 1 { result = self.mul_mod(&result, &base, modulus); }
            base = self.mul_mod(&base, &base, modulus);
            exp >>= 1;
        }
        result
    }

    pub fn trace(mat: &[Vec<u64>], modulus: u64) -> u64 {
        (0..mat.len()).map(|i| mat[i][i]).fold(0u64, |a, v| (a + v) % modulus)
    }

    pub fn fib(n: u64, modulus: u64) -> u64 {
        if n == 0 { return 0; }
        let mut mp = MatPow::new();
        let mat = vec![vec![1u64, 1], vec![1, 0]];
        let result = mp.pow_mod(&mat, n, modulus);
        result[0][1]
    }

    pub fn total_muls(&self) -> u64 { self.total_muls }
    pub fn total_pows(&self) -> u64 { self.total_pows }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let id = MatPow::identity(3);
        assert_eq!(id[0][0], 1);
        assert_eq!(id[0][1], 0);
    }

    #[test]
    fn mul_mod() {
        let mut mp = MatPow::new();
        let a = vec![vec![1, 2], vec![3, 4]];
        let b = vec![vec![5, 6], vec![7, 8]];
        let c = mp.mul_mod(&a, &b, 1000);
        assert_eq!(c[0][0], 19);
        assert_eq!(c[1][1], 50);
    }

    #[test]
    fn pow_identity() {
        let mut mp = MatPow::new();
        let m = vec![vec![1, 0], vec![0, 1]];
        let r = mp.pow_mod(&m, 100, 1000);
        assert_eq!(r[0][0], 1);
    }

    #[test]
    fn fib_test() {
        assert_eq!(MatPow::fib(1, 1_000_000_007), 1);
        assert_eq!(MatPow::fib(2, 1_000_000_007), 1);
        assert_eq!(MatPow::fib(10, 1_000_000_007), 55);
    }

    #[test]
    fn trace() {
        let m = vec![vec![3, 0], vec![0, 7]];
        assert_eq!(MatPow::trace(&m, 1000), 10);
    }

    #[test]
    fn pow_mod_large() {
        let mut mp = MatPow::new();
        let m = vec![vec![1, 1], vec![1, 0]];
        let r = mp.pow_mod(&m, 50, 1_000_000_007);
        assert_ne!(r[0][0], 0);
    }

    #[test]
    fn stats() {
        let mut mp = MatPow::new();
        mp.pow_mod(&vec![vec![1, 0], vec![0, 1]], 10, 1000);
        assert_eq!(mp.total_pows(), 1);
        assert!(mp.total_muls() > 0);
    }
}
