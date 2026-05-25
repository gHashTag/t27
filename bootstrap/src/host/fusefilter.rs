const SEEDS: [u64; 3] = [0xcbf29ce484222325, 0x9e3779b97f4a7c15, 0x100000001b3];

fn hash_n(data: &[u8], seed: u64) -> usize { let mut h = seed; for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); } h as usize }

pub struct FuseFilter {
    tier1_bits: Vec<u64>,
    tier2_bits: Vec<u64>,
    tier1_size: usize,
    tier2_size: usize,
    total_inserts: u64,
    total_lookups: u64,
    tier1_hits: u64,
    tier2_hits: u64,
}

impl FuseFilter {
    pub fn new(capacity: usize) -> Self {
        let t1 = (capacity as f64 * 4.0) as usize;
        let t2 = (capacity as f64 * 8.0) as usize;
        Self { tier1_bits: vec![0u64; (t1 + 63) / 64], tier2_bits: vec![0u64; (t2 + 63) / 64], tier1_size: t1, tier2_size: t2, total_inserts: 0, total_lookups: 0, tier1_hits: 0, tier2_hits: 0 }
    }

    fn set_bit(bits: &mut [u64], size: usize, idx: usize) { let i = idx % size; bits[i / 64] |= 1u64 << (i % 64); }
    fn get_bit(bits: &[u64], size: usize, idx: usize) -> bool { let i = idx % size; bits[i / 64] & (1u64 << (i % 64)) != 0 }

    pub fn insert(&mut self, data: &[u8]) {
        self.total_inserts += 1;
        for &seed in &SEEDS { Self::set_bit(&mut self.tier1_bits, self.tier1_size, hash_n(data, seed)); }
        for &seed in &SEEDS { Self::set_bit(&mut self.tier2_bits, self.tier2_size, hash_n(data, seed.wrapping_add(1))); }
    }

    pub fn contains(&mut self, data: &[u8]) -> bool {
        self.total_lookups += 1;
        let t1 = SEEDS.iter().all(|&s| Self::get_bit(&self.tier1_bits, self.tier1_size, hash_n(data, s)));
        if t1 {
            self.tier1_hits += 1;
            let t2 = SEEDS.iter().all(|&s| Self::get_bit(&self.tier2_bits, self.tier2_size, hash_n(data, s.wrapping_add(1))));
            if t2 { self.tier2_hits += 1; }
            t2
        } else { false }
    }

    pub fn tier1_hit_rate(&self) -> f64 { if self.total_lookups == 0 { 0.0 } else { self.tier1_hits as f64 / self.total_lookups as f64 } }
    pub fn tier2_hit_rate(&self) -> f64 { if self.tier1_hits == 0 { 0.0 } else { self.tier2_hits as f64 / self.tier1_hits as f64 } }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn memory_bytes(&self) -> usize { (self.tier1_bits.len() + self.tier2_bits.len()) * 8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_contains() {
        let mut ff = FuseFilter::new(100);
        ff.insert(b"hello");
        assert!(ff.contains(b"hello"));
    }

    #[test]
    fn absent() {
        let mut ff = FuseFilter::new(100);
        assert!(!ff.contains(b"world"));
    }

    #[test]
    fn many() {
        let mut ff = FuseFilter::new(1000);
        for i in 0..100u64 { ff.insert(&i.to_le_bytes()); }
        for i in 0..100u64 { assert!(ff.contains(&i.to_le_bytes())); }
    }

    #[test]
    fn tiered_hits() {
        let mut ff = FuseFilter::new(100);
        ff.insert(b"x");
        ff.contains(b"x");
        assert!(ff.tier1_hit_rate() > 0.0);
        assert!(ff.tier2_hit_rate() > 0.0);
    }

    #[test]
    fn memory() { let ff = FuseFilter::new(100); assert!(ff.memory_bytes() > 0); }

    #[test]
    fn stats() {
        let mut ff = FuseFilter::new(100);
        ff.insert(b"x"); ff.contains(b"x");
        assert_eq!(ff.total_inserts(), 1);
        assert_eq!(ff.total_lookups(), 1);
    }
}
