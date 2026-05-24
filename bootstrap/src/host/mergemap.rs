use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergePolicy {
    KeepLeft,
    KeepRight,
    Sum,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MmError {
    NotFound { key: u64 },
}

impl std::fmt::Display for MmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MmError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for MmError {}

pub struct MergeMap {
    data: BTreeMap<u64, i64>,
    total_inserts: u64,
    total_merges: u64,
    total_lookups: u64,
}

impl MergeMap {
    pub fn new() -> Self { Self { data: BTreeMap::new(), total_inserts: 0, total_merges: 0, total_lookups: 0 } }

    pub fn insert(&mut self, key: u64, value: i64) {
        self.total_inserts += 1;
        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: u64) -> Option<i64> {
        self.total_lookups += 1;
        self.data.get(&key).copied()
    }

    pub fn merge(&mut self, other: &MergeMap, policy: MergePolicy) {
        self.total_merges += 1;
        for (&k, &v) in &other.data {
            match self.data.get(&k) {
                Some(existing) => {
                    let new_val = match policy {
                        MergePolicy::KeepLeft => *existing,
                        MergePolicy::KeepRight => v,
                        MergePolicy::Sum => *existing + v,
                    };
                    self.data.insert(k, new_val);
                }
                None => { self.data.insert(k, v); }
            }
        }
    }

    pub fn merge_drain(&mut self, other: &mut MergeMap, policy: MergePolicy) {
        self.total_merges += 1;
        let other_data = std::mem::take(&mut other.data);
        for (k, v) in other_data {
            match self.data.get(&k) {
                Some(existing) => {
                    let new_val = match policy {
                        MergePolicy::KeepLeft => *existing,
                        MergePolicy::KeepRight => v,
                        MergePolicy::Sum => *existing + v,
                    };
                    self.data.insert(k, new_val);
                }
                None => { self.data.insert(k, v); }
            }
        }
    }

    pub fn diff(&self, other: &MergeMap) -> (MergeMap, MergeMap, MergeMap) {
        let mut only_left = MergeMap::new();
        let mut only_right = MergeMap::new();
        let mut changed = MergeMap::new();
        for (&k, &v) in &self.data {
            match other.data.get(&k) {
                Some(&ov) if ov != v => { changed.insert(k, v); }
                Some(_) => {}
                None => { only_left.insert(k, v); }
            }
        }
        for (&k, &v) in &other.data {
            if !self.data.contains_key(&k) { only_right.insert(k, v); }
        }
        (only_left, only_right, changed)
    }

    pub fn intersect(&self, other: &MergeMap) -> MergeMap {
        let mut result = MergeMap::new();
        for (&k, &v) in &self.data {
            if other.data.contains_key(&k) { result.insert(k, v); }
        }
        result
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_merges(&self) -> u64 { self.total_merges }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

impl Default for MergeMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mm() { assert!(MergeMap::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut mm = MergeMap::new();
        mm.insert(1, 10); mm.insert(2, 20);
        assert_eq!(mm.get(1), Some(10));
        assert_eq!(mm.get(3), None);
    }

    #[test]
    fn merge_keep_left() {
        let mut m1 = MergeMap::new(); m1.insert(1, 10);
        let mut m2 = MergeMap::new(); m2.insert(1, 99); m2.insert(2, 20);
        m1.merge(&m2, MergePolicy::KeepLeft);
        assert_eq!(m1.get(1), Some(10));
        assert_eq!(m1.get(2), Some(20));
    }

    #[test]
    fn merge_keep_right() {
        let mut m1 = MergeMap::new(); m1.insert(1, 10);
        let m2 = MergeMap::new(); let mut m2 = m2; m2.insert(1, 99);
        m1.merge(&m2, MergePolicy::KeepRight);
        assert_eq!(m1.get(1), Some(99));
    }

    #[test]
    fn merge_sum() {
        let mut m1 = MergeMap::new(); m1.insert(1, 10);
        let mut m2 = MergeMap::new(); m2.insert(1, 20);
        m1.merge(&m2, MergePolicy::Sum);
        assert_eq!(m1.get(1), Some(30));
    }

    #[test]
    fn merge_drain() {
        let mut m1 = MergeMap::new(); m1.insert(1, 10);
        let mut m2 = MergeMap::new(); m2.insert(2, 20);
        m1.merge_drain(&mut m2, MergePolicy::KeepRight);
        assert_eq!(m1.len(), 2);
        assert!(m2.is_empty());
    }

    #[test]
    fn diff() {
        let mut m1 = MergeMap::new(); m1.insert(1, 10); m1.insert(2, 20); m1.insert(3, 30);
        let mut m2 = MergeMap::new(); m2.insert(2, 99); m2.insert(4, 40);
        let (left, right, changed) = m1.diff(&m2);
        assert_eq!(left.len(), 2);
        assert_eq!(right.len(), 1);
        assert_eq!(changed.len(), 1);
    }

    #[test]
    fn intersect() {
        let mut m1 = MergeMap::new(); m1.insert(1, 10); m1.insert(2, 20);
        let mut m2 = MergeMap::new(); m2.insert(2, 99); m2.insert(3, 30);
        let mut inter = m1.intersect(&m2);
        assert_eq!(inter.len(), 1);
        assert_eq!(inter.get(2), Some(20));
    }

    #[test]
    fn stats() {
        let mut mm = MergeMap::new(); mm.insert(1, 1); mm.get(1);
        assert_eq!(mm.total_inserts(), 1);
        assert_eq!(mm.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(MmError::NotFound { key: 1 }.to_string().contains("not found")); }
}
