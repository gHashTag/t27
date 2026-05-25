use std::collections::BTreeSet;

pub struct VebTree {
    set: BTreeSet<u64>,
    universe: u64,
    total_inserts: u64,
    total_deletes: u64,
    total_queries: u64,
}

impl VebTree {
    pub fn new(universe: u64) -> Self { Self { set: BTreeSet::new(), universe, total_inserts: 0, total_deletes: 0, total_queries: 0 } }

    pub fn insert(&mut self, val: u64) {
        self.total_inserts += 1;
        self.set.insert(val);
    }

    pub fn remove(&mut self, val: u64) -> bool {
        self.total_deletes += 1;
        self.set.remove(&val)
    }

    pub fn contains(&mut self, val: u64) -> bool { self.total_queries += 1; self.set.contains(&val) }

    pub fn min(&self) -> Option<u64> { self.set.iter().next().copied() }
    pub fn max(&self) -> Option<u64> { self.set.iter().next_back().copied() }

    pub fn successor(&mut self, val: u64) -> Option<u64> {
        self.total_queries += 1;
        self.set.range((val + 1)..).next().copied()
    }

    pub fn predecessor(&mut self, val: u64) -> Option<u64> {
        self.total_queries += 1;
        self.set.range(..val).next_back().copied()
    }

    pub fn len(&self) -> usize { self.set.len() }
    pub fn is_empty(&self) -> bool { self.set.is_empty() }
    pub fn universe(&self) -> u64 { self.universe }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_contains() {
        let mut v = VebTree::new(1024);
        v.insert(42);
        assert!(v.contains(42));
        assert!(!v.contains(43));
    }

    #[test]
    fn min_max() {
        let mut v = VebTree::new(1024);
        v.insert(10); v.insert(50); v.insert(30);
        assert_eq!(v.min(), Some(10));
        assert_eq!(v.max(), Some(50));
    }

    #[test]
    fn successor_predecessor() {
        let mut v = VebTree::new(1024);
        v.insert(10); v.insert(30); v.insert(50);
        assert_eq!(v.successor(10), Some(30));
        assert_eq!(v.predecessor(50), Some(30));
    }

    #[test]
    fn remove() {
        let mut v = VebTree::new(1024);
        v.insert(1); v.insert(2);
        assert!(v.remove(1));
        assert!(!v.contains(1));
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn empty() {
        let mut v = VebTree::new(256);
        assert!(v.min().is_none());
        assert!(v.successor(0).is_none());
    }

    #[test]
    fn dense() {
        let mut v = VebTree::new(256);
        for i in 0..100u64 { v.insert(i); }
        for i in 0..99u64 { assert_eq!(v.successor(i), Some(i + 1)); }
    }

    #[test]
    fn stats() {
        let mut v = VebTree::new(256);
        v.insert(1); v.contains(1); v.remove(1);
        assert_eq!(v.total_inserts(), 1);
        assert_eq!(v.total_queries(), 1);
        assert_eq!(v.total_deletes(), 1);
    }
}
