pub struct RollingHash {
    base: u64,
    modulus: u64,
    power: Vec<u64>,
}

impl RollingHash {
    pub fn new(base: u64, modulus: u64, max_len: usize) -> Self {
        let mut power = vec![1u64; max_len + 1];
        for i in 1..=max_len {
            power[i] = (power[i - 1] as u128 * base as u128 % modulus as u128) as u64;
        }
        Self { base, modulus, power }
    }

    pub fn hash(&self, data: &[u8]) -> u64 {
        let mut h = 0u64;
        for (i, &b) in data.iter().enumerate() {
            h = (h as u128 + (b as u128 * self.power[data.len() - 1 - i] as u128) % self.modulus as u128) as u64 % self.modulus;
        }
        h
    }

    pub fn search(&self, text: &[u8], pattern: &[u8]) -> Vec<usize> {
        let n = text.len();
        let m = pattern.len();
        if m == 0 || m > n { return Vec::new(); }
        let ph = self.hash(pattern);
        let mut th = self.hash(&text[..m]);
        let mut matches = Vec::new();
        for i in 0..=n - m {
            if th == ph && text[i..i + m] == pattern[..] {
                matches.push(i);
            }
            if i + m < n {
                let old = (text[i] as u128 * self.power[m - 1] as u128 % self.modulus as u128) as u64;
                th = ((th as u128 + self.modulus as u128 - old as u128) % self.modulus as u128 * self.base as u128 % self.modulus as u128 + text[i + m] as u128) as u64 % self.modulus;
            }
        }
        matches
    }

    pub fn double_hash(data: &[u8]) -> (u64, u64) {
        const B1: u64 = 131;
        const M1: u64 = 1_000_000_007;
        const B2: u64 = 137;
        const M2: u64 = 1_000_000_009;
        let mut h1 = 0u64;
        let mut h2 = 0u64;
        for &b in data {
            h1 = ((h1 as u128 * B1 as u128 + b as u128) % M1 as u128) as u64;
            h2 = ((h2 as u128 * B2 as u128 + b as u128) % M2 as u128) as u64;
        }
        (h1, h2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_deterministic() {
        let rh = RollingHash::new(131, 1_000_000_007, 100);
        let h1 = rh.hash(b"hello");
        let h2 = rh.hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_different() {
        let rh = RollingHash::new(131, 1_000_000_007, 100);
        assert_ne!(rh.hash(b"abc"), rh.hash(b"abd"));
    }

    #[test]
    fn search_found() {
        let rh = RollingHash::new(131, 1_000_000_007, 100);
        let m = rh.search(b"abcabcabc", b"abc");
        assert_eq!(m, vec![0, 3, 6]);
    }

    #[test]
    fn search_not_found() {
        let rh = RollingHash::new(131, 1_000_000_007, 100);
        assert!(rh.search(b"abcdef", b"xyz").is_empty());
    }

    #[test]
    fn search_empty_pattern() {
        let rh = RollingHash::new(131, 1_000_000_007, 100);
        assert!(rh.search(b"abc", b"").is_empty());
    }

    #[test]
    fn double_hash_equal() {
        let (h1a, h1b) = RollingHash::double_hash(b"test");
        let (h2a, h2b) = RollingHash::double_hash(b"test");
        assert_eq!(h1a, h2a);
        assert_eq!(h1b, h2b);
    }
}
