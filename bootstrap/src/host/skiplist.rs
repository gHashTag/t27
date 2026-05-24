use std::collections::BTreeMap;

const MAX_LEVEL: usize = 16;

fn random_level() -> usize {
    let mut level = 1;
    let mut v = pseudo_random();
    while v & 1 == 1 && level < MAX_LEVEL { level += 1; v >>= 1; }
    level
}

static mut SEED: u64 = 0x12345678;
fn pseudo_random() -> u64 {
    unsafe {
        SEED = SEED.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        SEED
    }
}

fn reset_seed() {
    unsafe { SEED = 0x12345678; }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkipError {
    KeyExists { key: u64 },
    KeyNotFound { key: u64 },
}

impl std::fmt::Display for SkipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipError::KeyExists { key } => write!(f, "key {key} exists"),
            SkipError::KeyNotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for SkipError {}

#[derive(Debug, Clone)]
pub struct SkipList {
    levels: Vec<BTreeMap<u64, Vec<u64>>>,
    total_inserts: u64,
    total_deletes: u64,
}

impl SkipList {
    pub fn new() -> Self {
        let levels = (0..MAX_LEVEL).map(|_| BTreeMap::new()).collect();
        Self { levels, total_inserts: 0, total_deletes: 0 }
    }

    pub fn insert(&mut self, key: u64, value: u64) -> Result<(), SkipError> {
        if self.contains(key) { return Err(SkipError::KeyExists { key }); }
        let level = random_level();
        for l in 0..level.min(MAX_LEVEL) {
            self.levels[l].insert(key, vec![value]);
        }
        self.total_inserts += 1;
        Ok(())
    }

    pub fn get(&self, key: u64) -> Option<u64> {
        self.levels[0].get(&key).and_then(|v| v.first().copied())
    }

    pub fn contains(&self, key: u64) -> bool {
        self.levels[0].contains_key(&key)
    }

    pub fn remove(&mut self, key: u64) -> Result<u64, SkipError> {
        let val = self.levels[0].remove(&key)
            .and_then(|v| v.first().copied())
            .ok_or(SkipError::KeyNotFound { key })?;
        for level in &mut self.levels.iter_mut().skip(1) {
            level.remove(&key);
        }
        self.total_deletes += 1;
        Ok(val)
    }

    pub fn update(&mut self, key: u64, value: u64) -> Result<u64, SkipError> {
        if !self.contains(key) { return Err(SkipError::KeyNotFound { key }); }
        let old = self.levels[0].get(&key).and_then(|v| v.first().copied()).unwrap();
        self.levels[0].insert(key, vec![value]);
        Ok(old)
    }

    pub fn range(&self, min: u64, max: u64) -> Vec<(u64, u64)> {
        self.levels[0].range(min..=max)
            .map(|(k, v)| (*k, v.first().copied().unwrap()))
            .collect()
    }

    pub fn min(&self) -> Option<u64> { self.levels[0].keys().next().copied() }
    pub fn max(&self) -> Option<u64> { self.levels[0].keys().next_back().copied() }

    pub fn len(&self) -> usize { self.levels[0].len() }
    pub fn is_empty(&self) -> bool { self.levels[0].is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn levels(&self) -> usize { MAX_LEVEL }
}

impl Default for SkipList {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() { reset_seed(); }

    #[test]
    fn new_list() {
        setup();
        let sl = SkipList::new();
        assert!(sl.is_empty());
    }

    #[test]
    fn insert_get() {
        setup();
        let mut sl = SkipList::new();
        sl.insert(10, 100).unwrap();
        sl.insert(20, 200).unwrap();
        assert_eq!(sl.get(10), Some(100));
        assert_eq!(sl.get(20), Some(200));
        assert_eq!(sl.len(), 2);
    }

    #[test]
    fn duplicate() {
        setup();
        let mut sl = SkipList::new();
        sl.insert(1, 10).unwrap();
        let err = sl.insert(1, 20).unwrap_err();
        assert!(matches!(err, SkipError::KeyExists { .. }));
    }

    #[test]
    fn remove() {
        setup();
        let mut sl = SkipList::new();
        sl.insert(5, 50).unwrap();
        let v = sl.remove(5).unwrap();
        assert_eq!(v, 50);
        assert!(sl.is_empty());
    }

    #[test]
    fn remove_missing() {
        setup();
        let mut sl = SkipList::new();
        let err = sl.remove(99).unwrap_err();
        assert!(matches!(err, SkipError::KeyNotFound { .. }));
    }

    #[test]
    fn update() {
        setup();
        let mut sl = SkipList::new();
        sl.insert(1, 10).unwrap();
        let old = sl.update(1, 20).unwrap();
        assert_eq!(old, 10);
        assert_eq!(sl.get(1), Some(20));
    }

    #[test]
    fn range_query() {
        setup();
        let mut sl = SkipList::new();
        for i in 0..10 { sl.insert(i, i * 10).unwrap(); }
        let r = sl.range(3, 7);
        assert_eq!(r.len(), 5);
        assert_eq!(r[0], (3, 30));
    }

    #[test]
    fn min_max() {
        setup();
        let mut sl = SkipList::new();
        sl.insert(5, 50).unwrap();
        sl.insert(1, 10).unwrap();
        sl.insert(9, 90).unwrap();
        assert_eq!(sl.min(), Some(1));
        assert_eq!(sl.max(), Some(9));
    }

    #[test]
    fn ordered_keys() {
        setup();
        let mut sl = SkipList::new();
        for &k in &[5, 3, 8, 1, 9] { sl.insert(k, k).unwrap(); }
        let all = sl.range(0, u64::MAX);
        let keys: Vec<u64> = all.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![1, 3, 5, 8, 9]);
    }

    #[test]
    fn stats() {
        setup();
        let mut sl = SkipList::new();
        sl.insert(1, 1).unwrap();
        sl.insert(2, 2).unwrap();
        sl.remove(1).unwrap();
        assert_eq!(sl.total_inserts(), 2);
        assert_eq!(sl.total_deletes(), 1);
    }

    #[test]
    fn contains() {
        setup();
        let mut sl = SkipList::new();
        sl.insert(42, 0).unwrap();
        assert!(sl.contains(42));
        assert!(!sl.contains(43));
    }

    #[test]
    fn error_display() {
        assert!(SkipError::KeyNotFound { key: 5 }.to_string().contains("5"));
    }
}
