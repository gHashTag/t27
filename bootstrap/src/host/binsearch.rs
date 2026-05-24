use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BsError {
    NotFound { value: i64 },
    EmptyArray,
}

impl std::fmt::Display for BsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BsError::NotFound { value } => write!(f, "value {value} not found"),
            BsError::EmptyArray => write!(f, "empty array"),
        }
    }
}

impl std::error::Error for BsError {}

pub struct BinSearch {
    data: Vec<i64>,
    total_inserts: u64,
    total_searches: u64,
    total_removes: u64,
}

impl BinSearch {
    pub fn new() -> Self { Self { data: Vec::new(), total_inserts: 0, total_searches: 0, total_removes: 0 } }

    pub fn insert(&mut self, value: i64) {
        let pos = self.data.partition_point(|&x| x < value);
        self.data.insert(pos, value);
        self.total_inserts += 1;
    }

    pub fn insert_dedup(&mut self, value: i64) -> bool {
        let pos = self.data.partition_point(|&x| x < value);
        if pos < self.data.len() && self.data[pos] == value { return false; }
        self.data.insert(pos, value);
        self.total_inserts += 1;
        true
    }

    pub fn search(&mut self, value: i64) -> Option<usize> {
        self.total_searches += 1;
        let pos = self.data.partition_point(|&x| x < value);
        if pos < self.data.len() && self.data[pos] == value { Some(pos) } else { None }
    }

    pub fn lower_bound(&self, value: i64) -> usize { self.data.partition_point(|&x| x < value) }

    pub fn upper_bound(&self, value: i64) -> usize { self.data.partition_point(|&x| x <= value) }

    pub fn rank(&self, value: i64) -> usize { self.lower_bound(value) }

    pub fn count_range(&self, lo: i64, hi: i64) -> usize { self.upper_bound(lo - 1).min(self.data.len()) - self.lower_bound(hi + 1).min(self.data.len()) }

    pub fn remove(&mut self, value: i64) -> bool {
        if let Some(pos) = self.search(value) {
            self.data.remove(pos);
            self.total_removes += 1;
            true
        } else { false }
    }

    pub fn min(&self) -> Option<i64> { self.data.first().copied() }
    pub fn max(&self) -> Option<i64> { self.data.last().copied() }

    pub fn kth(&self, k: usize) -> Option<i64> { self.data.get(k).copied() }

    pub fn range(&self, lo: i64, hi: i64) -> Vec<i64> {
        let from = self.lower_bound(lo);
        let to = self.upper_bound(hi);
        self.data[from..to].to_vec()
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_searches(&self) -> u64 { self.total_searches }
    pub fn total_removes(&self) -> u64 { self.total_removes }
}

impl Default for BinSearch {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bs() { assert!(BinSearch::new().is_empty()); }

    #[test]
    fn insert_search() {
        let mut bs = BinSearch::new();
        bs.insert(5); bs.insert(1); bs.insert(10);
        assert_eq!(bs.search(5), Some(1));
        assert_eq!(bs.search(1), Some(0));
        assert_eq!(bs.search(99), None);
    }

    #[test]
    fn sorted_order() {
        let mut bs = BinSearch::new();
        bs.insert(3); bs.insert(1); bs.insert(2);
        assert_eq!(bs.data, vec![1, 2, 3]);
    }

    #[test]
    fn dedup() {
        let mut bs = BinSearch::new();
        assert!(bs.insert_dedup(1));
        assert!(!bs.insert_dedup(1));
        assert_eq!(bs.len(), 1);
    }

    #[test]
    fn range_query() {
        let mut bs = BinSearch::new();
        for i in 1..=10 { bs.insert(i); }
        let r = bs.range(3, 7);
        assert_eq!(r, vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn min_max_kth() {
        let mut bs = BinSearch::new();
        bs.insert(10); bs.insert(5); bs.insert(20);
        assert_eq!(bs.min(), Some(5));
        assert_eq!(bs.max(), Some(20));
        assert_eq!(bs.kth(1), Some(10));
    }

    #[test]
    fn remove() {
        let mut bs = BinSearch::new();
        bs.insert(5); bs.insert(10);
        assert!(bs.remove(5));
        assert_eq!(bs.search(5), None);
        assert!(!bs.remove(99));
    }

    #[test]
    fn rank() {
        let mut bs = BinSearch::new();
        for i in [10, 20, 30, 40] { bs.insert(i); }
        assert_eq!(bs.rank(25), 2);
    }

    #[test]
    fn bounds() {
        let mut bs = BinSearch::new();
        for i in [10, 20, 30] { bs.insert(i); }
        assert_eq!(bs.lower_bound(20), 1);
        assert_eq!(bs.upper_bound(20), 2);
    }

    #[test]
    fn stats() {
        let mut bs = BinSearch::new();
        bs.insert(1); bs.insert(2);
        bs.search(1);
        bs.remove(2);
        assert_eq!(bs.total_inserts(), 2);
        assert!(bs.total_searches() >= 1);
        assert_eq!(bs.total_removes(), 1);
    }

    #[test]
    fn error_display() { assert!(BsError::NotFound { value: 1 }.to_string().contains("1")); }
}
