use std::collections::BTreeMap;

pub struct SparseMap {
    explicit: BTreeMap<u64, u64>,
    default: u64,
    total_sets: u64,
    total_gets: u64,
}

impl SparseMap {
    pub fn new(default: u64) -> Self { Self { explicit: BTreeMap::new(), default, total_sets: 0, total_gets: 0 } }

    pub fn set(&mut self, key: u64, value: u64) {
        self.total_sets += 1;
        if value == self.default { self.explicit.remove(&key); } else { self.explicit.insert(key, value); }
    }

    pub fn get(&mut self, key: u64) -> u64 {
        self.total_gets += 1;
        self.explicit.get(&key).copied().unwrap_or(self.default)
    }

    pub fn bulk_set(&mut self, entries: &[(u64, u64)]) {
        for &(k, v) in entries { self.set(k, v); }
    }

    pub fn range_sum(&mut self, start: u64, end: u64) -> u64 {
        self.total_gets += 1;
        let explicit_sum: u64 = self.explicit.range(start..end).map(|(_, &v)| v).sum();
        let explicit_count = self.explicit.range(start..end).count() as u64;
        let total = end - start;
        explicit_sum + self.default * (total - explicit_count)
    }

    pub fn contains_explicit(&self, key: u64) -> bool { self.explicit.contains_key(&key) }
    pub fn explicit_count(&self) -> usize { self.explicit.len() }
    pub fn default_value(&self) -> u64 { self.default }
    pub fn density(&self) -> f64 { if self.explicit.is_empty() { 0.0 } else { 1.0 } }
    pub fn total_sets(&self) -> u64 { self.total_sets }
    pub fn total_gets(&self) -> u64 { self.total_gets }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let mut sm = SparseMap::new(0);
        sm.set(1, 42);
        assert_eq!(sm.get(1), 42);
        assert_eq!(sm.get(2), 0);
    }

    #[test]
    fn default_erase() {
        let mut sm = SparseMap::new(0);
        sm.set(1, 42); sm.set(1, 0);
        assert_eq!(sm.explicit_count(), 0);
    }

    #[test]
    fn bulk() {
        let mut sm = SparseMap::new(0);
        sm.bulk_set(&[(1, 10), (2, 20), (3, 30)]);
        assert_eq!(sm.get(2), 20);
    }

    #[test]
    fn range_sum() {
        let mut sm = SparseMap::new(1);
        sm.set(2, 10); sm.set(4, 20);
        assert_eq!(sm.range_sum(0, 5), 1 + 1 + 10 + 1 + 20);
    }

    #[test]
    fn non_zero_default() {
        let mut sm = SparseMap::new(99);
        assert_eq!(sm.get(100), 99);
    }

    #[test]
    fn stats() {
        let mut sm = SparseMap::new(0);
        sm.set(1, 1); sm.get(1);
        assert_eq!(sm.total_sets(), 1);
        assert_eq!(sm.total_gets(), 1);
    }
}
