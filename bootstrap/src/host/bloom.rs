const BITS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BloomError {
    AlreadyFull,
}

impl std::fmt::Display for BloomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BloomError::AlreadyFull => write!(f, "filter at capacity"),
        }
    }
}

impl std::error::Error for BloomError {}

fn hash1(key: u64) -> u64 {
    let mut h = key;
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51AFD7ED558CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CEB9FE1A85EC53);
    h ^= h >> 33;
    h
}

fn hash2(key: u64) -> u64 {
    let mut h = key.wrapping_mul(0x5851F42D4C957F2D);
    h ^= h >> 33;
    h = h.wrapping_mul(0x9E3779B97F4A7C15);
    h ^= h >> 33;
    h
}

#[derive(Debug, Clone)]
pub struct BloomStats {
    pub bit_count: usize,
    pub hash_count: usize,
    pub items_inserted: u64,
    pub bits_set: usize,
    pub fill_ratio: f64,
    pub estimated_fpp: f64,
}

#[derive(Debug, Clone)]
pub struct BloomFilter {
    bitmap: Vec<u64>,
    bit_count: usize,
    hash_count: usize,
    items_inserted: u64,
    max_items: usize,
}

impl BloomFilter {
    pub fn new(expected_items: usize, fp_rate: f64) -> Self {
        let ln2 = std::f64::consts::LN_2;
        let bits = ((-(fp_rate.ln()) * expected_items as f64) / (ln2 * ln2)).ceil() as usize;
        let bit_count = bits.max(BITS);
        let hash_count = ((bit_count as f64 / expected_items as f64) * ln2).ceil() as usize;
        let hash_count = hash_count.max(1).min(32);
        let words = (bit_count + BITS - 1) / BITS;
        Self {
            bitmap: vec![0u64; words],
            bit_count,
            hash_count,
            items_inserted: 0,
            max_items: expected_items,
        }
    }

    pub fn with_params(bit_count: usize, hash_count: usize) -> Self {
        let words = (bit_count + BITS - 1) / BITS;
        Self {
            bitmap: vec![0u64; words],
            bit_count,
            hash_count: hash_count.max(1).min(32),
            items_inserted: 0,
            max_items: bit_count,
        }
    }

    fn get_hashes(&self, key: u64) -> Vec<usize> {
        let h1 = hash1(key);
        let h2 = hash2(key);
        (0..self.hash_count)
            .map(|i| ((h1.wrapping_add((i as u64).wrapping_mul(h2))) % self.bit_count as u64) as usize)
            .collect()
    }

    pub fn insert(&mut self, key: u64) {
        for idx in self.get_hashes(key) {
            let word = idx / BITS;
            let bit = idx % BITS;
            self.bitmap[word] |= 1u64 << bit;
        }
        self.items_inserted += 1;
    }

    pub fn contains(&self, key: u64) -> bool {
        for idx in self.get_hashes(key) {
            let word = idx / BITS;
            let bit = idx % BITS;
            if self.bitmap[word] & (1u64 << bit) == 0 {
                return false;
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.bitmap.fill(0);
        self.items_inserted = 0;
    }

    pub fn bits_set(&self) -> usize {
        self.bitmap.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn fill_ratio(&self) -> f64 {
        self.bits_set() as f64 / self.bit_count as f64
    }

    pub fn estimated_fpp(&self) -> f64 {
        let k = self.hash_count as f64;
        let m = self.bit_count as f64;
        let n = self.items_inserted as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }

    pub fn items_inserted(&self) -> u64 {
        self.items_inserted
    }

    pub fn bit_count(&self) -> usize {
        self.bit_count
    }

    pub fn hash_count(&self) -> usize {
        self.hash_count
    }

    pub fn stats(&self) -> BloomStats {
        BloomStats {
            bit_count: self.bit_count,
            hash_count: self.hash_count,
            items_inserted: self.items_inserted,
            bits_set: self.bits_set(),
            fill_ratio: self.fill_ratio(),
            estimated_fpp: self.estimated_fpp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_filter() {
        let bf = BloomFilter::new(1000, 0.01);
        assert!(bf.bit_count() > 0);
        assert!(bf.hash_count() > 0);
        assert_eq!(bf.items_inserted(), 0);
    }

    #[test]
    fn insert_and_contains() {
        let mut bf = BloomFilter::new(100, 0.01);
        bf.insert(42);
        assert!(bf.contains(42));
        assert_eq!(bf.items_inserted(), 1);
    }

    #[test]
    fn absent_key() {
        let mut bf = BloomFilter::with_params(1024, 3);
        bf.insert(1);
        assert!(!bf.contains(2));
    }

    #[test]
    fn multiple_inserts() {
        let mut bf = BloomFilter::new(100, 0.01);
        for i in 0..50u64 { bf.insert(i); }
        for i in 0..50u64 { assert!(bf.contains(i)); }
        assert_eq!(bf.items_inserted(), 50);
    }

    #[test]
    fn clear() {
        let mut bf = BloomFilter::with_params(512, 3);
        bf.insert(42);
        bf.clear();
        assert!(!bf.contains(42));
        assert_eq!(bf.items_inserted(), 0);
    }

    #[test]
    fn fill_ratio_increases() {
        let mut bf = BloomFilter::with_params(512, 3);
        let r0 = bf.fill_ratio();
        for i in 0..100u64 { bf.insert(i); }
        assert!(bf.fill_ratio() > r0);
    }

    #[test]
    fn estimated_fpp() {
        let mut bf = BloomFilter::new(100, 0.01);
        for i in 0..50u64 { bf.insert(i); }
        let fpp = bf.estimated_fpp();
        assert!(fpp >= 0.0 && fpp <= 1.0);
    }

    #[test]
    fn no_false_negatives() {
        let mut bf = BloomFilter::new(200, 0.01);
        let keys: Vec<u64> = (0..100).collect();
        for &k in &keys { bf.insert(k); }
        for &k in &keys { assert!(bf.contains(k), "false negative for {k}"); }
    }

    #[test]
    fn deterministic() {
        let mut bf1 = BloomFilter::with_params(512, 3);
        let mut bf2 = BloomFilter::with_params(512, 3);
        for i in 0..20u64 { bf1.insert(i); bf2.insert(i); }
        assert_eq!(bf1.bits_set(), bf2.bits_set());
    }

    #[test]
    fn stats() {
        let mut bf = BloomFilter::new(100, 0.01);
        bf.insert(1);
        bf.insert(2);
        let s = bf.stats();
        assert_eq!(s.items_inserted, 2);
        assert!(s.bits_set > 0);
    }

    #[test]
    fn with_params() {
        let bf = BloomFilter::with_params(256, 5);
        assert_eq!(bf.bit_count(), 256);
        assert_eq!(bf.hash_count(), 5);
    }

    #[test]
    fn error_display() {
        assert!(BloomError::AlreadyFull.to_string().contains("capacity"));
    }
}
