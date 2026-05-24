use std::collections::BTreeMap;

fn fnv_hash(data: &[u8], seed: u64) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325 ^ seed;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum PmError {
    KeyNotFound { key: u64 },
}

impl std::fmt::Display for PmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PmError::KeyNotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for PmError {}

pub struct ProbeMap {
    store: BTreeMap<u64, Vec<u8>>,
    bloom: Vec<u8>,
    num_hashes: usize,
    bloom_size: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_false_positives: u64,
    total_true_negatives: u64,
}

impl ProbeMap {
    pub fn new(bloom_size: usize, num_hashes: usize) -> Self {
        Self { store: BTreeMap::new(), bloom: vec![0; bloom_size], num_hashes, bloom_size, total_inserts: 0, total_lookups: 0, total_false_positives: 0, total_true_negatives: 0 }
    }

    fn bloom_indices(&self, key: u64) -> Vec<usize> {
        let kb = key.to_le_bytes();
        (0..self.num_hashes).map(|i| (fnv_hash(&kb, i as u64) as usize) % self.bloom_size).collect()
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        for idx in self.bloom_indices(key) { self.bloom[idx] = self.bloom[idx].saturating_add(1); }
        self.store.insert(key, value);
        self.total_inserts += 1;
    }

    pub fn might_contain(&mut self, key: u64) -> bool {
        self.total_lookups += 1;
        let indices = self.bloom_indices(key);
        let present = indices.iter().all(|&i| self.bloom[i] > 0);
        if present && !self.store.contains_key(&key) {
            self.total_false_positives += 1;
        }
        if !present && !self.store.contains_key(&key) {
            self.total_true_negatives += 1;
        }
        present
    }

    pub fn get(&mut self, key: u64) -> Option<&Vec<u8>> {
        self.total_lookups += 1;
        self.store.get(&key)
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, PmError> {
        let val = self.store.remove(&key).ok_or(PmError::KeyNotFound { key })?;
        for idx in self.bloom_indices(key) { self.bloom[idx] = self.bloom[idx].saturating_sub(1); }
        Ok(val)
    }

    pub fn len(&self) -> usize { self.store.len() }
    pub fn is_empty(&self) -> bool { self.store.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_false_positives(&self) -> u64 { self.total_false_positives }
    pub fn total_true_negatives(&self) -> u64 { self.total_true_negatives }
    pub fn false_positive_rate(&self) -> f64 {
        if self.total_lookups == 0 { return 0.0; }
        self.total_false_positives as f64 / self.total_lookups as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { let pm = ProbeMap::new(256, 3); assert!(pm.is_empty()); }

    #[test]
    fn insert_get() {
        let mut pm = ProbeMap::new(256, 3);
        pm.insert(1, b"val".to_vec());
        assert_eq!(pm.get(1), Some(&b"val".to_vec()));
    }

    #[test]
    fn might_contain_present() {
        let mut pm = ProbeMap::new(256, 3);
        pm.insert(42, b"x".to_vec());
        assert!(pm.might_contain(42));
    }

    #[test]
    fn might_contain_absent() {
        let mut pm = ProbeMap::new(1024, 5);
        assert!(!pm.might_contain(999));
    }

    #[test]
    fn remove() {
        let mut pm = ProbeMap::new(256, 3);
        pm.insert(1, b"x".to_vec());
        let v = pm.remove(1).unwrap();
        assert_eq!(v, b"x");
        assert!(pm.is_empty());
    }

    #[test]
    fn remove_missing() {
        let mut pm = ProbeMap::new(256, 3);
        let err = pm.remove(99).unwrap_err();
        assert!(matches!(err, PmError::KeyNotFound { .. }));
    }

    #[test]
    fn bloom_clears_on_remove() {
        let mut pm = ProbeMap::new(256, 3);
        pm.insert(42, b"x".to_vec());
        pm.remove(42).unwrap();
        assert!(!pm.might_contain(42));
    }

    #[test]
    fn many_items() {
        let mut pm = ProbeMap::new(1024, 5);
        for i in 0..50 { pm.insert(i, vec![i as u8]); }
        for i in 0..50 { assert!(pm.might_contain(i)); }
    }

    #[test]
    fn stats() {
        let mut pm = ProbeMap::new(256, 3);
        pm.insert(1, b"x".to_vec());
        pm.get(1);
        assert_eq!(pm.total_inserts(), 1);
        assert_eq!(pm.total_lookups(), 1);
    }

    #[test]
    fn fpr_zero() {
        let pm = ProbeMap::new(256, 3);
        assert_eq!(pm.false_positive_rate(), 0.0);
    }

    #[test]
    fn error_display() { assert!(PmError::KeyNotFound { key: 1 }.to_string().contains("1")); }
}
