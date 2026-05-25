const SEEDS: [u64; 4] = [0xcbf29ce484222325, 0x9e3779b97f4a7c15, 0x100000001b3, 0xff51afd7ed558ccd];

fn hash_n(data: &[u8], seed: u64) -> u64 {
    let mut h = seed;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

pub struct Probemap2 {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
    total_inserts: u64,
    total_lookups: u64,
}

impl Probemap2 {
    pub fn new(capacity: usize, fp_rate: f64) -> Self {
        let ln2 = std::f64::consts::LN_2;
        let num_bits = (-(capacity as f64 * fp_rate.ln()) / (ln2 * ln2)).ceil() as usize;
        let num_bits = num_bits.max(64);
        let num_hashes = ((num_bits as f64 / capacity.max(1) as f64) * ln2).ceil() as usize;
        let num_hashes = num_hashes.clamp(1, SEEDS.len());
        let words = (num_bits + 63) / 64;
        Self { bits: vec![0u64; words], num_bits, num_hashes, total_inserts: 0, total_lookups: 0 }
    }

    fn indices(&self, data: &[u8]) -> Vec<usize> {
        (0..self.num_hashes).map(|i| (hash_n(data, SEEDS[i]) as usize) % self.num_bits).collect()
    }

    pub fn insert(&mut self, data: &[u8]) {
        self.total_inserts += 1;
        for idx in self.indices(data) {
            self.bits[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    pub fn contains(&mut self, data: &[u8]) -> bool {
        self.total_lookups += 1;
        for idx in self.indices(data) {
            if self.bits[idx / 64] & (1u64 << (idx % 64)) == 0 { return false; }
        }
        true
    }

    pub fn estimated_fp_rate(&self) -> f64 {
        let set_bits: usize = self.bits.iter().map(|w| w.count_ones() as usize).sum();
        let k = self.num_hashes as f64;
        let m = self.num_bits as f64;
        let n = set_bits as f64;
        (1.0 - (1.0 - n / m).powf(k)).powf(k)
    }

    pub fn fill_ratio(&self) -> f64 {
        let set_bits: usize = self.bits.iter().map(|w| w.count_ones() as usize).sum();
        set_bits as f64 / self.num_bits as f64
    }

    pub fn num_bits(&self) -> usize { self.num_bits }
    pub fn num_hashes(&self) -> usize { self.num_hashes }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_contains() {
        let mut pm = Probemap2::new(100, 0.01);
        pm.insert(b"hello");
        assert!(pm.contains(b"hello"));
    }

    #[test]
    fn absent() {
        let mut pm = Probemap2::new(100, 0.01);
        pm.insert(b"hello");
        assert!(!pm.contains(b"world"));
    }

    #[test]
    fn many_inserts() {
        let mut pm = Probemap2::new(1000, 0.01);
        for i in 0..100u64 { pm.insert(&i.to_le_bytes()); }
        for i in 0..100u64 { assert!(pm.contains(&i.to_le_bytes())); }
    }

    #[test]
    fn estimated_fp() {
        let mut pm = Probemap2::new(100, 0.01);
        for i in 0..50u64 { pm.insert(&i.to_le_bytes()); }
        assert!(pm.estimated_fp_rate() < 0.5);
    }

    #[test]
    fn fill_ratio() {
        let mut pm = Probemap2::new(100, 0.01);
        for i in 0..10u64 { pm.insert(&i.to_le_bytes()); }
        assert!(pm.fill_ratio() > 0.0);
    }

    #[test]
    fn hashes_configured() {
        let pm = Probemap2::new(100, 0.01);
        assert!(pm.num_hashes() >= 1);
        assert!(pm.num_bits() >= 64);
    }

    #[test]
    fn stats() {
        let mut pm = Probemap2::new(100, 0.01);
        pm.insert(b"x"); pm.contains(b"x");
        assert_eq!(pm.total_inserts(), 1);
        assert_eq!(pm.total_lookups(), 1);
    }
}
