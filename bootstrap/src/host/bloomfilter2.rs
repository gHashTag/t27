use std::cell::Cell;

fn fnv_hash(seed: u64, key: u64) -> u64 {
    let mut h = seed;
    for &b in key.to_le_bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum Bf2Error {
    AlreadyRemoved { key: u64 },
}

impl std::fmt::Display for Bf2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bf2Error::AlreadyRemoved { key } => write!(f, "key {key} already removed"),
        }
    }
}

impl std::error::Error for Bf2Error {}

pub struct BloomFilter2 {
    counters: Vec<u8>,
    num_hashes: usize,
    total_adds: Cell<u64>,
    total_removes: Cell<u64>,
    total_lookups: Cell<u64>,
}

impl BloomFilter2 {
    pub fn new(size: usize, num_hashes: usize) -> Self {
        Self { counters: vec![0; size], num_hashes, total_adds: Cell::new(0), total_removes: Cell::new(0), total_lookups: Cell::new(0) }
    }

    fn indices(&self, key: u64) -> Vec<usize> {
        (0..self.num_hashes).map(|i| (fnv_hash(0xcbf29ce484222325 + i as u64, key) as usize) % self.counters.len()).collect()
    }

    pub fn add(&self, key: u64) {
        self.total_adds.set(self.total_adds.get() + 1);
        for idx in self.indices(key) { self.counters[idx] = self.counters[idx].saturating_add(1); }
    }

    pub fn remove(&self, key: u64) -> Result<(), Bf2Error> {
        self.total_removes.set(self.total_removes.get() + 1);
        let indices = self.indices(key);
        for &idx in &indices {
            if self.counters[idx] == 0 { return Err(Bf2Error::AlreadyRemoved { key }); }
        }
        for idx in indices { self.counters[idx] -= 1; }
        Ok(())
    }

    pub fn contains(&self, key: u64) -> bool {
        self.total_lookups.set(self.total_lookups.get() + 1);
        self.indices(key).iter().all(|&idx| self.counters[idx] > 0)
    }

    pub fn estimate_count(&self, key: u64) -> usize {
        self.indices(key).iter().map(|&idx| self.counters[idx] as usize).min().unwrap_or(0)
    }

    pub fn clear(&mut self) { self.counters.fill(0); }

    pub fn saturation(&self) -> f64 {
        let nonzero: usize = self.counters.iter().filter(|&&c| c > 0).count();
        nonzero as f64 / self.counters.len() as f64
    }

    pub fn total_adds(&self) -> u64 { self.total_adds.get() }
    pub fn total_removes(&self) -> u64 { self.total_removes.get() }
    pub fn total_lookups(&self) -> u64 { self.total_lookups.get() }
    pub fn size(&self) -> usize { self.counters.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bf() { let bf = BloomFilter2::new(256, 3); assert_eq!(bf.size(), 256); }

    #[test]
    fn add_contains() {
        let bf = BloomFilter2::new(256, 3);
        bf.add(42);
        assert!(bf.contains(42));
    }

    #[test]
    fn not_present() {
        let bf = BloomFilter2::new(256, 3);
        assert!(!bf.contains(99));
    }

    #[test]
    fn remove() {
        let bf = BloomFilter2::new(256, 3);
        bf.add(42); bf.add(42);
        bf.remove(42).unwrap();
        assert!(bf.contains(42));
        bf.remove(42).unwrap();
        assert!(!bf.contains(42));
    }

    #[test]
    fn remove_not_present() {
        let bf = BloomFilter2::new(256, 3);
        assert!(bf.remove(42).is_err());
    }

    #[test]
    fn estimate_count() {
        let bf = BloomFilter2::new(256, 3);
        for _ in 0..5 { bf.add(42); }
        assert!(bf.estimate_count(42) >= 3);
    }

    #[test]
    fn clear() {
        let mut bf = BloomFilter2::new(256, 3);
        bf.add(1); bf.add(2); bf.clear();
        assert!(!bf.contains(1));
    }

    #[test]
    fn many_items() {
        let bf = BloomFilter2::new(1024, 5);
        for i in 0..200u64 { bf.add(i); }
        let mut tp = 0; let mut fp = 0;
        for i in 0..200u64 { if bf.contains(i) { tp += 1; } }
        for i in 200..400u64 { if bf.contains(i) { fp += 1; } }
        assert_eq!(tp, 200);
        assert!(fp < 20);
    }

    #[test]
    fn stats() {
        let bf = BloomFilter2::new(256, 3);
        bf.add(1); bf.contains(1);
        assert_eq!(bf.total_adds(), 1);
        assert_eq!(bf.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(Bf2Error::AlreadyRemoved { key: 1 }.to_string().contains("removed")); }
}
