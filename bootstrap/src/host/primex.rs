pub struct PrimeX {
    total_checks: u64,
    total_sieves: u64,
}

impl PrimeX {
    pub fn new() -> Self { Self { total_checks: 0, total_sieves: 0 } }

    pub fn is_prime(&mut self, n: u64) -> bool {
        self.total_checks += 1;
        if n < 2 { return false; }
        if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5u64;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 { return false; }
            i += 6;
        }
        true
    }

    fn mod_pow(&self, mut base: u64, mut exp: u64, modulus: u64) -> u64 {
        if modulus == 1 { return 0; }
        let mut result: u128 = 1;
        base %= modulus;
        let m = modulus as u128;
        let mut b = base as u128;
        while exp > 0 {
            if exp % 2 == 1 { result = result * b % m; }
            exp >>= 1;
            b = b * b % m;
        }
        result as u64
    }

    pub fn miller_rabin(&mut self, n: u64, witnesses: &[u64]) -> bool {
        self.total_checks += 1;
        if n < 2 { return false; }
        if n < 4 { return true; }
        if n % 2 == 0 { return false; }
        let mut d = n - 1;
        let mut r = 0u32;
        while d % 2 == 0 { d /= 2; r += 1; }
        'witness: for &a in witnesses {
            if a >= n { continue; }
            let mut x = self.mod_pow(a, d, n);
            if x == 1 || x == n - 1 { continue; }
            for _ in 0..r - 1 {
                x = ((x as u128 * x as u128) % (n as u128)) as u64;
                if x == n - 1 { continue 'witness; }
            }
            return false;
        }
        true
    }

    pub fn is_prime_fast(&mut self, n: u64) -> bool {
        if n < 3_474_749_660_383 {
            self.miller_rabin(n, &[2, 3, 5, 7, 11, 13])
        } else {
            self.miller_rabin(n, &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37])
        }
    }

    pub fn sieve(&mut self, limit: u64) -> Vec<u64> {
        self.total_sieves += 1;
        if limit < 2 { return Vec::new(); }
        let size = (limit + 1) as usize;
        let mut is_prime = vec![true; size];
        is_prime[0] = false; is_prime[1] = false;
        let mut i = 2;
        while (i * i) as u64 <= limit {
            if is_prime[i] {
                let mut j = i * i;
                while j < size { is_prime[j] = false; j += i; }
            }
            i += 1;
        }
        (2..=limit).filter(|&i| is_prime[i as usize]).collect()
    }

    pub fn segmented_sieve(&mut self, lo: u64, hi: u64) -> Vec<u64> {
        self.total_sieves += 1;
        if hi < 2 { return Vec::new(); }
        let lo = lo.max(2);
        let small_limit = (hi as f64).sqrt() as u64 + 1;
        let small_primes = {
            let mut tmp = PrimeX::new();
            tmp.sieve(small_limit)
        };
        let range = (hi - lo + 1) as usize;
        let mut is_prime = vec![true; range];
        for &p in &small_primes {
            let start = ((lo + p - 1) / p) * p;
            let mut j = start;
            while j <= hi {
                if j >= lo { is_prime[(j - lo) as usize] = false; }
                j += p;
            }
        }
        for p in lo..=hi {
            if is_prime[(p - lo) as usize] {
                let mut found = false;
                for &sp in &small_primes { if sp == p { found = true; break; } }
                if !found && p > small_limit { is_prime[(p - lo) as usize] = true; }
            }
        }
        (lo..=hi).filter(|&i| is_prime[(i - lo) as usize]).collect()
    }

    pub fn prime_factors(&mut self, mut n: u64) -> Vec<(u64, u32)> {
        self.total_checks += 1;
        let mut factors = Vec::new();
        let mut d = 2u64;
        while d * d <= n {
            let mut count = 0;
            while n % d == 0 { n /= d; count += 1; }
            if count > 0 { factors.push((d, count)); }
            d += 1;
        }
        if n > 1 { factors.push((n, 1)); }
        factors
    }

    pub fn euler_totient(&mut self, n: u64) -> u64 {
        let factors = self.prime_factors(n);
        let mut result = n;
        for (p, _) in &factors { result = result / p * (p - 1); }
        result
    }

    pub fn total_checks(&self) -> u64 { self.total_checks }
    pub fn total_sieves(&self) -> u64 { self.total_sieves }
}

impl Default for PrimeX {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_primes() {
        let mut px = PrimeX::new();
        assert!(px.is_prime(2)); assert!(px.is_prime(3)); assert!(px.is_prime(5));
        assert!(!px.is_prime(0)); assert!(!px.is_prime(1)); assert!(!px.is_prime(4));
    }

    #[test]
    fn miller_rabin() {
        let mut px = PrimeX::new();
        assert!(px.miller_rabin(97, &[2, 3]));
        assert!(!px.miller_rabin(91, &[2, 3]));
    }

    #[test]
    fn sieve_basic() {
        let mut px = PrimeX::new();
        let primes = px.sieve(30);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
    }

    #[test]
    fn segmented() {
        let mut px = PrimeX::new();
        let primes = px.segmented_sieve(100, 120);
        assert!(primes.contains(&101));
        assert!(primes.contains(&103));
        assert!(primes.contains(&107));
    }

    #[test]
    fn factors() {
        let mut px = PrimeX::new();
        let f = px.prime_factors(60);
        assert_eq!(f, vec![(2, 2), (3, 1), (5, 1)]);
    }

    #[test]
    fn totient() {
        let mut px = PrimeX::new();
        assert_eq!(px.euler_totient(12), 4);
        assert_eq!(px.euler_totient(7), 6);
    }

    #[test]
    fn large_prime() {
        let mut px = PrimeX::new();
        assert!(px.is_prime_fast(1_000_000_007));
        assert!(!px.is_prime_fast(1_000_000_008));
    }

    #[test]
    fn fast_consistent() {
        let mut px = PrimeX::new();
        for i in 0..200u64 { assert_eq!(px.is_prime(i), px.is_prime_fast(i), "mismatch at {i}"); }
    }

    #[test]
    fn stats() {
        let mut px = PrimeX::new();
        px.is_prime(5); px.sieve(10);
        assert_eq!(px.total_checks(), 1);
        assert_eq!(px.total_sieves(), 1);
    }
}
