use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub enum CrdtError {
    AlreadyPresent { element: u64 },
}

impl std::fmt::Display for CrdtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrdtError::AlreadyPresent { element } => write!(f, "element {element} already present"),
        }
    }
}

impl std::error::Error for CrdtError {}

#[derive(Clone)]
pub struct CrdtGSet {
    node_id: u64,
    elements: BTreeSet<u64>,
    vector_clock: BTreeMap<u64, u64>,
    total_adds: u64,
    total_merges: u64,
}

impl CrdtGSet {
    pub fn new(node_id: u64) -> Self {
        Self { node_id, elements: BTreeSet::new(), vector_clock: BTreeMap::new(), total_adds: 0, total_merges: 0 }
    }

    pub fn add(&mut self, element: u64) {
        self.elements.insert(element);
        *self.vector_clock.entry(self.node_id).or_insert(0) += 1;
        self.total_adds += 1;
    }

    pub fn contains(&self, element: u64) -> bool { self.elements.contains(&element) }

    pub fn merge(&mut self, other: &CrdtGSet) {
        for &e in &other.elements { self.elements.insert(e); }
        for (&node, &count) in &other.vector_clock {
            let my_count = self.vector_clock.entry(node).or_insert(0);
            if count > *my_count { *my_count = count; }
        }
        self.total_merges += 1;
    }

    pub fn elements(&self) -> Vec<u64> { self.elements.iter().copied().collect() }
    pub fn len(&self) -> usize { self.elements.len() }
    pub fn is_empty(&self) -> bool { self.elements.is_empty() }
    pub fn node_id(&self) -> u64 { self.node_id }
    pub fn vector_clock(&self) -> &BTreeMap<u64, u64> { &self.vector_clock }
    pub fn total_adds(&self) -> u64 { self.total_adds }
    pub fn total_merges(&self) -> u64 { self.total_merges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set() { let s = CrdtGSet::new(1); assert!(s.is_empty()); assert_eq!(s.node_id(), 1); }

    #[test]
    fn add_contains() {
        let mut s = CrdtGSet::new(1);
        s.add(42);
        assert!(s.contains(42));
        assert!(!s.contains(43));
    }

    #[test]
    fn add_idempotent() {
        let mut s = CrdtGSet::new(1);
        s.add(1);
        s.add(1);
        assert_eq!(s.len(), 1);
        assert_eq!(s.total_adds(), 2);
    }

    #[test]
    fn merge_union() {
        let mut s1 = CrdtGSet::new(1);
        let mut s2 = CrdtGSet::new(2);
        s1.add(1); s1.add(2);
        s2.add(2); s2.add(3);
        s1.merge(&s2);
        assert!(s1.contains(1));
        assert!(s1.contains(2));
        assert!(s1.contains(3));
    }

    #[test]
    fn merge_idempotent() {
        let mut s1 = CrdtGSet::new(1);
        let mut s2 = CrdtGSet::new(2);
        s1.add(1); s2.add(2);
        s1.merge(&s2);
        s1.merge(&s2);
        assert_eq!(s1.len(), 2);
        assert_eq!(s1.total_merges(), 2);
    }

    #[test]
    fn merge_commutative() {
        let mut s1 = CrdtGSet::new(1);
        let mut s2 = CrdtGSet::new(2);
        s1.add(1); s2.add(2);
        let mut s1_copy = s1.clone();
        s1_copy.merge(&s2);
        let mut s2_copy = s2.clone();
        s2_copy.merge(&s1);
        assert_eq!(s1_copy.elements(), s2_copy.elements());
    }

    #[test]
    fn vector_clock() {
        let mut s = CrdtGSet::new(1);
        s.add(1); s.add(2);
        assert_eq!(s.vector_clock().get(&1), Some(&2));
    }

    #[test]
    fn merge_updates_clock() {
        let mut s1 = CrdtGSet::new(1);
        let mut s2 = CrdtGSet::new(2);
        s1.add(1);
        s2.add(2); s2.add(3);
        s1.merge(&s2);
        assert_eq!(s1.vector_clock().get(&2), Some(&2));
    }

    #[test]
    fn elements_sorted() {
        let mut s = CrdtGSet::new(1);
        s.add(3); s.add(1); s.add(2);
        assert_eq!(s.elements(), vec![1, 2, 3]);
    }

    #[test]
    fn stats() {
        let mut s = CrdtGSet::new(1);
        s.add(1);
        let mut other = CrdtGSet::new(2);
        other.add(2);
        s.merge(&other);
        assert_eq!(s.total_adds(), 1);
        assert_eq!(s.total_merges(), 1);
    }

    #[test]
    fn error_display() { assert!(CrdtError::AlreadyPresent { element: 1 }.to_string().contains("1")); }
}
