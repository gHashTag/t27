pub struct RabinHash {
    base: u64,
    modulus: u64,
    window_size: usize,
    power: Vec<u64>,
    hash: u64,
    window: Vec<u8>,
    pos: usize,
}

impl RabinHash {
    pub fn new(base: u64, modulus: u64, window_size: usize) -> Self {
        let mut power = vec![1u64; window_size + 1];
        for i in 1..=window_size {
            power[i] = power[i - 1].wrapping_mul(base) % modulus;
        }
        Self { base, modulus, power, window_size, hash: 0, window: vec![0; window_size], pos: 0 }
    }

    pub fn slide(&mut self, byte: u8) -> u64 {
        let old = self.window[self.pos];
        self.hash = (self.hash + self.modulus - (old as u64 * self.power[self.window_size - 1] % self.modulus) % self.modulus) % self.modulus;
        self.hash = (self.hash * self.base + byte as u64) % self.modulus;
        self.window[self.pos] = byte;
        self.pos = (self.pos + 1) % self.window_size;
        self.hash
    }

    pub fn hash(&self) -> u64 { self.hash }

    pub fn fingerprint(data: &[u8], base: u64, modulus: u64) -> u64 {
        let mut h = 0u64;
        for &b in data {
            h = (h.wrapping_mul(base) + b as u64) % modulus;
        }
        h
    }

    pub fn find_duplicates(data: &[u8], window_size: usize, base: u64, modulus: u64) -> Vec<(usize, usize)> {
        let mut rh = Self::new(base, modulus, window_size);
        let mut seen = std::collections::HashMap::new();
        let mut dups = Vec::new();
        for (i, &b) in data.iter().enumerate() {
            if i >= window_size {
                rh.slide(b);
                if let Some(&first) = seen.get(&rh.hash()) {
                    dups.push((first, i - window_size + 1));
                } else {
                    seen.insert(rh.hash(), i - window_size + 1);
                }
            } else {
                rh.slide(b);
                if i == window_size - 1 { seen.insert(rh.hash(), 0); }
            }
        }
        dups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: u64 = 257;
    const MOD: u64 = 1_000_000_007;

    #[test]
    fn rolling_consistent() {
        let mut rh = RabinHash::new(BASE, MOD, 4);
        for &b in b"abcd" { rh.slide(b); }
        let h1 = rh.hash();
        for &b in b"abcd" { rh.slide(b); }
        let h2 = rh.hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn fingerprint_equal() {
        let h1 = RabinHash::fingerprint(b"hello", BASE, MOD);
        let h2 = RabinHash::fingerprint(b"hello", BASE, MOD);
        assert_eq!(h1, h2);
    }

    #[test]
    fn fingerprint_different() {
        let h1 = RabinHash::fingerprint(b"abc", BASE, MOD);
        let h2 = RabinHash::fingerprint(b"abd", BASE, MOD);
        assert_ne!(h1, h2);
    }

    #[test]
    fn slide_changes_hash() {
        let mut rh = RabinHash::new(BASE, MOD, 3);
        rh.slide(b'a'); rh.slide(b'b'); rh.slide(b'c');
        let h1 = rh.hash();
        rh.slide(b'd');
        assert_ne!(h1, rh.hash());
    }

    #[test]
    fn find_duplicates() {
        let data = b"abcabcxyzabc";
        let dups = RabinHash::find_duplicates(data, 3, BASE, MOD);
        assert!(!dups.is_empty());
    }
}
