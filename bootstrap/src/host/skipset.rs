use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SkError {
    NotFound { key: u64 },
    AlreadyExists { key: u64 },
}

impl std::fmt::Display for SkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkError::NotFound { key } => write!(f, "key {key} not found"),
            SkError::AlreadyExists { key } => write!(f, "key {key} exists"),
        }
    }
}

impl std::error::Error for SkError {}

struct SkipNode {
    key: u64,
    forward: Vec<Option<usize>>,
}

pub struct SkipSet {
    nodes: Vec<SkipNode>,
    max_level: usize,
    head: usize,
    len: usize,
    rng_state: u64,
    total_inserts: u64,
    total_removes: u64,
    total_searches: u64,
    total_promotions: u64,
}

impl SkipSet {
    pub fn new(max_level: usize) -> Self {
        let head = 0;
        let nodes = vec![SkipNode { key: u64::MAX, forward: vec![None; max_level] }];
        Self { nodes, max_level, head, len: 0, rng_state: 0x1234567890ABCDEF, total_inserts: 0, total_removes: 0, total_searches: 0, total_promotions: 0 }
    }

    fn random_level(&mut self) -> usize {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mut level = 1;
        let mut bits = self.rng_state;
        while level < self.max_level && bits & 1 == 0 { level += 1; bits >>= 1; }
        level
    }

    pub fn insert(&mut self, key: u64) -> Result<(), SkError> {
        self.total_inserts += 1;
        let mut update = vec![self.head; self.max_level];
        let mut cur = Some(self.head);
        for lvl in (0..self.max_level).rev() {
            while let Some(ci) = cur {
                let next = self.nodes[ci].forward[lvl];
                match next {
                    Some(ni) if self.nodes[ni].key < key => { cur = Some(ni); }
                    Some(ni) if self.nodes[ni].key == key => { return Err(SkError::AlreadyExists { key }); }
                    _ => break,
                }
            }
            if let Some(ci) = cur { update[lvl] = ci; }
        }
        let first_next = self.nodes[update[0]].forward[0];
        if let Some(ni) = first_next {
            if self.nodes[ni].key == key { return Err(SkError::AlreadyExists { key }); }
        }
        let new_level = self.random_level();
        let new_idx = self.nodes.len();
        self.total_promotions += new_level as u64;
        self.nodes.push(SkipNode { key, forward: vec![None; self.max_level] });
        for lvl in 0..new_level.min(self.max_level) {
            self.nodes[new_idx].forward[lvl] = self.nodes[update[lvl]].forward[lvl];
            self.nodes[update[lvl]].forward[lvl] = Some(new_idx);
        }
        self.len += 1;
        Ok(())
    }

    pub fn contains(&mut self, key: u64) -> bool {
        self.total_searches += 1;
        let mut cur = Some(self.head);
        for lvl in (0..self.max_level).rev() {
            while let Some(ci) = cur {
                let next = self.nodes[ci].forward[lvl];
                match next {
                    Some(ni) if self.nodes[ni].key < key => { cur = Some(ni); }
                    Some(ni) if self.nodes[ni].key == key => { return true; }
                    _ => break,
                }
            }
        }
        if let Some(ci) = cur {
            if let Some(ni) = self.nodes[ci].forward[0] {
                return self.nodes[ni].key == key;
            }
        }
        false
    }

    pub fn remove(&mut self, key: u64) -> Result<(), SkError> {
        self.total_removes += 1;
        let mut update = vec![self.head; self.max_level];
        let mut cur = Some(self.head);
        for lvl in (0..self.max_level).rev() {
            while let Some(ci) = cur {
                let next = self.nodes[ci].forward[lvl];
                match next {
                    Some(ni) if self.nodes[ni].key < key => { cur = Some(ni); }
                    _ => break,
                }
            }
            if let Some(ci) = cur { update[lvl] = ci; }
        }
        let target = self.nodes[update[0]].forward[0];
        let ti = target.ok_or(SkError::NotFound { key })?;
        if self.nodes[ti].key != key { return Err(SkError::NotFound { key }); }
        for lvl in 0..self.max_level {
            if self.nodes[update[lvl]].forward[lvl] == Some(ti) {
                self.nodes[update[lvl]].forward[lvl] = self.nodes[ti].forward[lvl];
            }
        }
        self.nodes[ti].key = u64::MAX;
        self.nodes[ti].forward = vec![None; self.max_level];
        self.len -= 1;
        Ok(())
    }

    pub fn iter(&self) -> Vec<u64> {
        let mut result = Vec::new();
        let mut cur = self.nodes[self.head].forward[0];
        while let Some(ci) = cur {
            result.push(self.nodes[ci].key);
            cur = self.nodes[ci].forward[0];
        }
        result
    }

    pub fn min(&self) -> Option<u64> { self.nodes[self.head].forward[0].map(|ni| self.nodes[ni].key) }
    pub fn max(&self) -> Option<u64> {
        let mut cur = self.head;
        for lvl in (0..self.max_level).rev() {
            while let Some(ni) = self.nodes[cur].forward[lvl] { cur = ni; }
        }
        if cur == self.head { None } else { Some(self.nodes[cur].key) }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_searches(&self) -> u64 { self.total_searches }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ss() { let s = SkipSet::new(8); assert!(s.is_empty()); }

    #[test]
    fn insert_contains() {
        let mut s = SkipSet::new(8);
        s.insert(10).unwrap(); s.insert(20).unwrap(); s.insert(30).unwrap();
        assert!(s.contains(10)); assert!(s.contains(20)); assert!(s.contains(30));
        assert!(!s.contains(15));
    }

    #[test]
    fn duplicate() {
        let mut s = SkipSet::new(4);
        s.insert(1).unwrap();
        assert!(s.insert(1).is_err());
    }

    #[test]
    fn remove() {
        let mut s = SkipSet::new(8);
        s.insert(1).unwrap(); s.insert(2).unwrap();
        s.remove(1).unwrap();
        assert!(!s.contains(1)); assert!(s.contains(2));
    }

    #[test]
    fn remove_not_found() {
        let mut s = SkipSet::new(4);
        assert!(s.remove(1).is_err());
    }

    #[test]
    fn iter_sorted() {
        let mut s = SkipSet::new(8);
        for &k in &[30, 10, 20, 50, 40] { s.insert(k).unwrap(); }
        assert_eq!(s.iter(), vec![10, 20, 30, 40, 50]);
    }

    #[test]
    fn min_max() {
        let mut s = SkipSet::new(8);
        for i in [5, 1, 9, 3] { s.insert(i).unwrap(); }
        assert_eq!(s.min(), Some(1));
        assert_eq!(s.max(), Some(9));
    }

    #[test]
    fn many() {
        let mut s = SkipSet::new(16);
        for i in 0..200u64 { s.insert(i).unwrap(); }
        assert_eq!(s.len(), 200);
        for i in 0..200u64 { assert!(s.contains(i)); }
    }

    #[test]
    fn stats() {
        let mut s = SkipSet::new(4);
        s.insert(1).unwrap(); s.contains(1);
        assert_eq!(s.total_inserts(), 1);
        assert_eq!(s.total_searches(), 1);
    }

    #[test]
    fn error_display() { assert!(SkError::NotFound { key: 1 }.to_string().contains("not found")); }
}
