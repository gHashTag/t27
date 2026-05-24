const FNV: u64 = 0xcbf29ce484222325;
const PRIME: u64 = 0x100000001b3;

fn hash_fn(data: &[u8], seed: u64) -> u64 {
    let mut h = FNV ^ seed;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(PRIME); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum BloomError {
    CapacityExceeded,
}

impl std::fmt::Display for BloomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { BloomError::CapacityExceeded => write!(f, "capacity exceeded") }
    }
}

impl std::error::Error for BloomError {}

pub struct CountingBloom {
    counters: Vec<u8>,
    num_hashes: usize,
    total_inserts: u64,
    total_removes: u64,
    total_queries: u64,
    total_hits: u64,
}

impl CountingBloom {
    pub fn new(size: usize, num_hashes: usize) -> Self {
        Self { counters: vec![0; size], num_hashes, total_inserts: 0, total_removes: 0, total_queries: 0, total_hits: 0 }
    }

    fn indices(&self, data: &[u8]) -> Vec<usize> {
        (0..self.num_hashes).map(|i| {
            let h = hash_fn(data, i as u64);
            (h as usize) % self.counters.len()
        }).collect()
    }

    pub fn insert(&mut self, data: &[u8]) {
        for idx in self.indices(data) {
            if self.counters[idx] < 255 { self.counters[idx] += 1; }
        }
        self.total_inserts += 1;
    }

    pub fn remove(&mut self, data: &[u8]) -> bool {
        let indices = self.indices(data);
        if indices.iter().all(|&i| self.counters[i] > 0) {
            for &idx in &indices { self.counters[idx] = self.counters[idx].saturating_sub(1); }
            self.total_removes += 1;
            true
        } else { false }
    }

    pub fn contains(&mut self, data: &[u8]) -> bool {
        self.total_queries += 1;
        let result = self.indices(data).iter().all(|&i| self.counters[i] > 0);
        if result { self.total_hits += 1; }
        result
    }

    pub fn estimated_fp_rate(&self) -> f64 {
        let filled = self.counters.iter().filter(|&&c| c > 0).count() as f64;
        let total = self.counters.len() as f64;
        let ratio = filled / total;
        ratio.powi(self.num_hashes as i32)
    }

    pub fn reset(&mut self) {
        for c in &mut self.counters { *c = 0; }
    }

    pub fn len(&self) -> usize { self.counters.len() }
    pub fn num_hashes(&self) -> usize { self.num_hashes }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_queries(&self) -> u64 { self.total_queries }
    pub fn total_hits(&self) -> u64 { self.total_hits }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_filter() {
        let bf = CountingBloom::new(1024, 3);
        assert_eq!(bf.len(), 1024);
    }

    #[test]
    fn insert_contains() {
        let mut bf = CountingBloom::new(1024, 3);
        bf.insert(b"hello");
        assert!(bf.contains(b"hello"));
    }

    #[test]
    fn not_present() {
        let mut bf = CountingBloom::new(1024, 3);
        assert!(!bf.contains(b"world"));
    }

    #[test]
    fn remove() {
        let mut bf = CountingBloom::new(1024, 3);
        bf.insert(b"key");
        assert!(bf.remove(b"key"));
        assert!(!bf.contains(b"key"));
    }

    #[test]
    fn remove_not_present() { assert!(!CountingBloom::new(1024, 3).remove(b"x")); }

    #[test]
    fn double_insert_remove() {
        let mut bf = CountingBloom::new(1024, 3);
        bf.insert(b"k"); bf.insert(b"k");
        bf.remove(b"k");
        assert!(bf.contains(b"k"));
        bf.remove(b"k");
        assert!(!bf.contains(b"k"));
    }

    #[test]
    fn many_items() {
        let mut bf = CountingBloom::new(4096, 5);
        for i in 0..100u32 { bf.insert(&i.to_le_bytes()); }
        for i in 0..100u32 { assert!(bf.contains(&i.to_le_bytes())); }
    }

    #[test]
    fn fp_rate() {
        let mut bf = CountingBloom::new(1024, 3);
        bf.estimated_fp_rate();
        assert!(bf.estimated_fp_rate() < 0.01);
        for i in 0..100u32 { bf.insert(&i.to_le_bytes()); }
        assert!(bf.estimated_fp_rate() > 0.0);
    }

    #[test]
    fn reset() {
        let mut bf = CountingBloom::new(1024, 3);
        bf.insert(b"k");
        bf.reset();
        assert!(!bf.contains(b"k"));
    }

    #[test]
    fn stats() {
        let mut bf = CountingBloom::new(1024, 3);
        bf.insert(b"a"); bf.insert(b"b");
        bf.contains(b"a"); bf.contains(b"x");
        assert_eq!(bf.total_inserts(), 2);
        assert_eq!(bf.total_queries(), 2);
        assert_eq!(bf.total_hits(), 1);
    }

    #[test]
    fn error_display() { assert!(BloomError::CapacityExceeded.to_string().contains("exceeded")); }
}
