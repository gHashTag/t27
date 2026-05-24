pub const UNIVERSE: u32 = 1_000_000;

pub struct SparseSet {
    sparse: Vec<u32>,
    dense: Vec<u32>,
    len: usize,
    total_inserts: u64,
    total_removes: u64,
    total_contains: u64,
    total_clears: u64,
}

impl SparseSet {
    pub fn new() -> Self { Self { sparse: vec![UNIVERSE; 256], dense: Vec::new(), len: 0, total_inserts: 0, total_removes: 0, total_contains: 0, total_clears: 0 } }

    fn ensure_capacity(&mut self, val: u32) {
        let needed = val as usize + 1;
        if self.sparse.len() < needed { self.sparse.resize(needed, UNIVERSE); }
    }

    pub fn insert(&mut self, val: u32) -> bool {
        self.ensure_capacity(val);
        if self.contains(val) { return false; }
        let idx = self.dense.len() as u32;
        self.dense.push(val);
        self.sparse[val as usize] = idx;
        self.len += 1;
        self.total_inserts += 1;
        true
    }

    pub fn remove(&mut self, val: u32) -> bool {
        if val as usize >= self.sparse.len() { return false; }
        let idx = self.sparse[val as usize];
        if idx as usize >= self.dense.len() || self.dense[idx as usize] != val { return false; }
        let last = *self.dense.last().unwrap();
        self.dense[idx as usize] = last;
        self.sparse[last as usize] = idx;
        self.dense.pop();
        self.sparse[val as usize] = UNIVERSE;
        self.len -= 1;
        self.total_removes += 1;
        true
    }

    pub fn contains(&mut self, val: u32) -> bool {
        self.total_contains += 1;
        if val as usize >= self.sparse.len() { return false; }
        let idx = self.sparse[val as usize];
        (idx as usize) < self.dense.len() && self.dense[idx as usize] == val
    }

    pub fn clear(&mut self) {
        self.dense.clear();
        for s in self.sparse.iter_mut() { *s = UNIVERSE; }
        self.len = 0;
        self.total_clears += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.dense.iter().copied()
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_contains(&self) -> u64 { self.total_contains }
    pub fn total_clears(&self) -> u64 { self.total_clears }
}

impl Default for SparseSet {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() { assert!(SparseSet::new().is_empty()); }

    #[test]
    fn insert_contains() {
        let mut s = SparseSet::new();
        s.insert(5);
        assert!(s.contains(5));
        assert!(!s.contains(6));
    }

    #[test]
    fn duplicate_insert() {
        let mut s = SparseSet::new();
        assert!(s.insert(1));
        assert!(!s.insert(1));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn remove() {
        let mut s = SparseSet::new();
        s.insert(1); s.insert(2); s.insert(3);
        assert!(s.remove(2));
        assert!(!s.contains(2));
        assert!(s.contains(1));
        assert!(s.contains(3));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn remove_nonexistent() {
        let mut s = SparseSet::new();
        assert!(!s.remove(99));
    }

    #[test]
    fn swap_remove_maintains_dense() {
        let mut s = SparseSet::new();
        s.insert(1); s.insert(2); s.insert(3);
        s.remove(1);
        let items: Vec<u32> = s.iter().collect();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&2));
        assert!(items.contains(&3));
    }

    #[test]
    fn clear() {
        let mut s = SparseSet::new();
        s.insert(1); s.insert(2); s.insert(3);
        s.clear();
        assert!(s.is_empty());
        assert!(!s.contains(1));
    }

    #[test]
    fn large_values() {
        let mut s = SparseSet::new();
        s.insert(1000);
        assert!(s.contains(1000));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn iteration() {
        let mut s = SparseSet::new();
        s.insert(3); s.insert(1); s.insert(2);
        let items: Vec<u32> = s.iter().collect();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn stats() {
        let mut s = SparseSet::new();
        s.insert(1); s.insert(2);
        s.remove(1);
        s.contains(99);
        assert_eq!(s.total_inserts(), 2);
        assert_eq!(s.total_removes(), 1);
        assert!(s.total_contains() >= 1);
    }

    #[test]
    fn insert_after_remove() {
        let mut s = SparseSet::new();
        s.insert(5);
        s.remove(5);
        s.insert(5);
        assert!(s.contains(5));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn clear_stats() {
        let mut s = SparseSet::new();
        s.insert(1); s.insert(2);
        s.clear();
        assert_eq!(s.total_clears(), 1);
    }
}
