use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReplicaId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VectorClock {
    pub replica: ReplicaId,
    pub seq: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrdtError {
    ReplicaExists { replica: u64 },
    ReplicaNotFound { replica: u64 },
}

impl std::fmt::Display for CrdtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrdtError::ReplicaExists { replica } => write!(f, "replica {replica} exists"),
            CrdtError::ReplicaNotFound { replica } => write!(f, "replica {replica} not found"),
        }
    }
}

impl std::error::Error for CrdtError {}

#[derive(Debug, Clone)]
pub struct LwwRegister {
    value: Vec<u8>,
    timestamp: u64,
    author: ReplicaId,
}

impl LwwRegister {
    pub fn new(replica: ReplicaId, initial: Vec<u8>) -> Self {
        Self { value: initial, timestamp: 0, author: replica }
    }

    pub fn set(&mut self, value: Vec<u8>, timestamp: u64, replica: ReplicaId) {
        if timestamp >= self.timestamp {
            self.value = value;
            self.timestamp = timestamp;
            self.author = replica;
        }
    }

    pub fn get(&self) -> &[u8] { &self.value }
    pub fn timestamp(&self) -> u64 { self.timestamp }
    pub fn author(&self) -> ReplicaId { self.author }

    pub fn merge(&mut self, other: &LwwRegister) {
        if other.timestamp > self.timestamp || (other.timestamp == self.timestamp && other.author.0 > self.author.0) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
            self.author = other.author;
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrSet {
    adds: BTreeMap<Vec<u8>, BTreeSet<(u64, ReplicaId)>>,
    removes: BTreeSet<(Vec<u8>, (u64, ReplicaId))>,
}

impl OrSet {
    pub fn new() -> Self { Self { adds: BTreeMap::new(), removes: BTreeSet::new() } }

    pub fn add(&mut self, element: Vec<u8>, seq: u64, replica: ReplicaId) {
        self.adds.entry(element.clone()).or_default().insert((seq, replica));
    }

    pub fn remove(&mut self, element: &[u8]) -> usize {
        if let Some(tags) = self.adds.get(element).cloned() {
            let count = tags.len();
            for tag in &tags { self.removes.insert((element.to_vec(), *tag)); }
            count
        } else { 0 }
    }

    pub fn contains(&self, element: &[u8]) -> bool {
        let tags = self.adds.get(element);
        match tags {
            Some(t) => t.iter().any(|tag| !self.removes.contains(&(element.to_vec(), *tag))),
            None => false,
        }
    }

    pub fn elements(&self) -> Vec<Vec<u8>> {
        self.adds.keys()
            .filter(|e| self.contains(e))
            .cloned()
            .collect()
    }

    pub fn merge(&mut self, other: &OrSet) {
        for (elem, tags) in &other.adds {
            self.adds.entry(elem.clone()).or_default().extend(tags.iter().copied());
        }
        for item in &other.removes { self.removes.insert(item.clone()); }
    }

    pub fn len(&self) -> usize { self.elements().len() }
    pub fn is_empty(&self) -> bool { self.elements().is_empty() }
}

impl Default for OrSet {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lww_new() {
        let r = LwwRegister::new(ReplicaId(1), vec![0]);
        assert_eq!(r.get(), &[0]);
    }

    #[test]
    fn lww_set_get() {
        let mut r = LwwRegister::new(ReplicaId(1), vec![]);
        r.set(vec![42], 10, ReplicaId(1));
        assert_eq!(r.get(), &[42]);
        assert_eq!(r.timestamp(), 10);
    }

    #[test]
    fn lww_last_writer_wins() {
        let mut r = LwwRegister::new(ReplicaId(1), vec![1]);
        r.set(vec![2], 5, ReplicaId(2));
        r.set(vec![3], 3, ReplicaId(1));
        assert_eq!(r.get(), &[2]);
    }

    #[test]
    fn lww_merge() {
        let mut r1 = LwwRegister::new(ReplicaId(1), vec![1]);
        let mut r2 = LwwRegister::new(ReplicaId(2), vec![2]);
        r1.set(vec![10], 5, ReplicaId(1));
        r2.set(vec![20], 10, ReplicaId(2));
        r1.merge(&r2);
        assert_eq!(r1.get(), &[20]);
    }

    #[test]
    fn lww_merge_tiebreak() {
        let mut r1 = LwwRegister::new(ReplicaId(1), vec![1]);
        let mut r2 = LwwRegister::new(ReplicaId(2), vec![2]);
        r1.set(vec![10], 5, ReplicaId(1));
        r2.set(vec![20], 5, ReplicaId(2));
        r1.merge(&r2);
        assert_eq!(r1.get(), &[20]);
    }

    #[test]
    fn orset_add_contains() {
        let mut s = OrSet::new();
        s.add(b"hello".to_vec(), 1, ReplicaId(1));
        assert!(s.contains(b"hello"));
        assert!(!s.contains(b"world"));
    }

    #[test]
    fn orset_remove() {
        let mut s = OrSet::new();
        s.add(b"x".to_vec(), 1, ReplicaId(1));
        s.remove(b"x");
        assert!(!s.contains(b"x"));
    }

    #[test]
    fn orset_add_remove_add() {
        let mut s = OrSet::new();
        s.add(b"x".to_vec(), 1, ReplicaId(1));
        s.remove(b"x");
        s.add(b"x".to_vec(), 2, ReplicaId(1));
        assert!(s.contains(b"x"));
    }

    #[test]
    fn orset_merge() {
        let mut s1 = OrSet::new();
        let mut s2 = OrSet::new();
        s1.add(b"a".to_vec(), 1, ReplicaId(1));
        s2.add(b"b".to_vec(), 1, ReplicaId(2));
        s1.merge(&s2);
        assert!(s1.contains(b"a"));
        assert!(s1.contains(b"b"));
    }

    #[test]
    fn orset_elements() {
        let mut s = OrSet::new();
        s.add(b"a".to_vec(), 1, ReplicaId(1));
        s.add(b"b".to_vec(), 1, ReplicaId(1));
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn orset_empty() {
        let s = OrSet::new();
        assert!(s.is_empty());
    }

    #[test]
    fn crdt_error_display() {
        assert!(CrdtError::ReplicaNotFound { replica: 3 }.to_string().contains("3"));
    }
}
