pub struct Primes {
    sieve: Vec<bool>,
    limit: usize,
    total_is_prime: u64,
    total_factorize: u64,
}

impl Primes {
    pub fn new(limit: usize) -> Self {
        let limit = limit.max(2);
        let mut sieve = vec![true; limit + 1];
        sieve[0] = false; sieve[1] = false;
        let mut i = 2;
        while i * i <= limit { if sieve[i] { let mut j = i * i; while j <= limit { sieve[j] = false; j += i; } } i += 1; }
        Self { sieve, limit, total_is_prime: 0, total_factorize: 0 }
    }

    pub fn is_prime(&mut self, n: u64) -> bool {
        self.total_is_prime += 1;
        if n as usize > self.limit { return self.trial_divide(n); }
        self.sieve[n as usize]
    }

    fn trial_divide(&self, n: u64) -> bool {
        if n < 2 { return false; }
        let mut d = 2u64;
        while d * d <= n { if n % d == 0 { return false; } d += 1; }
        true
    }

    pub fn primes(&self) -> Vec<u64> {
        (0..=self.limit).filter(|&i| self.sieve[i]).map(|i| i as u64).collect()
    }

    pub fn factorize(&mut self, mut n: u64) -> Vec<(u64, u32)> {
        self.total_factorize += 1;
        let mut factors = Vec::new();
        let mut d = 2u64;
        while d * d <= n {
            if n % d == 0 {
                let mut exp = 0u32;
                while n % d == 0 { n /= d; exp += 1; }
                factors.push((d, exp));
            }
            d += 1;
        }
        if n > 1 { factors.push((n, 1)); }
        factors
    }

    pub fn euler_totient(&mut self, n: u64) -> u64 {
        let factors = self.factorize(n);
        let mut result = n as f64;
        for &(p, _) in &factors { result *= 1.0 - 1.0 / p as f64; }
        result as u64
    }

    pub fn count(&self) -> usize { self.sieve.iter().filter(|&&b| b).count() }
    pub fn limit(&self) -> usize { self.limit }
    pub fn total_is_prime(&self) -> u64 { self.total_is_prime }
    pub fn total_factorize(&self) -> u64 { self.total_factorize }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_primes() {
        let mut p = Primes::new(30);
        assert!(p.is_prime(2)); assert!(p.is_prime(3)); assert!(p.is_prime(5));
        assert!(!p.is_prime(4)); assert!(!p.is_prime(1));
    }

    #[test]
    fn count() { assert_eq!(Primes::new(100).count(), 25); }

    #[test]
    fn factorize() {
        let mut p = Primes::new(100);
        assert_eq!(p.factorize(12), vec![(2, 2), (3, 1)]);
        assert_eq!(p.factorize(7), vec![(7, 1)]);
    }

    #[test]
    fn euler_totient() {
        let mut p = Primes::new(100);
        assert_eq!(p.euler_totient(12), 4);
    }

    #[test]
    fn beyond_sieve() {
        let mut p = Primes::new(10);
        assert!(p.is_prime(97));
        assert!(!p.is_prime(100));
    }

    #[test]
    fn stats() {
        let mut p = Primes::new(100);
        p.is_prime(7); p.factorize(12);
        assert_eq!(p.total_is_prime(), 1);
        assert_eq!(p.total_factorize(), 1);
    }
}
