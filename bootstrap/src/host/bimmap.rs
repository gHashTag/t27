use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BmErr {
    KeyExists(u64),
    ValExists,
    NotFound(u64),
}

impl std::fmt::Display for BmErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BmErr::KeyExists(k) => write!(f, "key {k} exists"),
            BmErr::ValExists => write!(f, "val exists"),
            BmErr::NotFound(k) => write!(f, "{k} not found"),
        }
    }
}

impl std::error::Error for BmErr {}

pub struct BimMap<V: Ord + Clone> {
    forward: BTreeMap<u64, V>,
    reverse: BTreeMap<V, u64>,
    total_inserts: u64,
    total_lookups: u64,
}

impl<V: Ord + Clone> BimMap<V> {
    pub fn new() -> Self { Self { forward: BTreeMap::new(), reverse: BTreeMap::new(), total_inserts: 0, total_lookups: 0 } }

    pub fn insert(&mut self, key: u64, val: V) -> Result<(), BmErr> {
        if self.forward.contains_key(&key) { return Err(BmErr::KeyExists(key)); }
        if self.reverse.contains_key(&val) { return Err(BmErr::ValExists); }
        self.total_inserts += 1;
        self.forward.insert(key, val.clone());
        self.reverse.insert(val, key);
        Ok(())
    }

    pub fn get_by_key(&mut self, key: u64) -> Option<&V> { self.total_lookups += 1; self.forward.get(&key) }

    pub fn get_by_val(&mut self, val: &V) -> Option<u64> { self.total_lookups += 1; self.reverse.get(val).copied() }

    pub fn remove_key(&mut self, key: u64) -> Option<V> {
        let val = self.forward.remove(&key)?;
        self.reverse.remove(&val);
        Some(val)
    }

    pub fn remove_val(&mut self, val: &V) -> Option<u64> {
        let key = self.reverse.remove(val)?;
        self.forward.remove(&key);
        Some(key)
    }

    pub fn upsert(&mut self, key: u64, val: V) -> Option<V> {
        self.total_inserts += 1;
        if let Some(old_val) = self.forward.insert(key, val.clone()) { self.reverse.remove(&old_val); }
        self.reverse.insert(val, key);
        None
    }

    pub fn contains_key(&self, key: u64) -> bool { self.forward.contains_key(&key) }
    pub fn contains_val(&self, val: &V) -> bool { self.reverse.contains_key(val) }
    pub fn len(&self) -> usize { self.forward.len() }
    pub fn is_empty(&self) -> bool { self.forward.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_both_dirs() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(1, 100).unwrap();
        assert_eq!(bm.get_by_key(1), Some(&100));
        assert_eq!(bm.get_by_val(&100), Some(1));
    }

    #[test]
    fn dup_key() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(1, 100).unwrap();
        assert!(bm.insert(1, 200).is_err());
    }

    #[test]
    fn dup_val() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(1, 100).unwrap();
        assert!(bm.insert(2, 100).is_err());
    }

    #[test]
    fn remove_key() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(1, 100).unwrap();
        assert_eq!(bm.remove_key(1), Some(100));
        assert!(bm.is_empty());
    }

    #[test]
    fn remove_val() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(1, 100).unwrap();
        assert_eq!(bm.remove_val(&100), Some(1));
    }

    #[test]
    fn upsert() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(1, 100).unwrap();
        bm.upsert(1, 200);
        assert_eq!(bm.get_by_key(1), Some(&200));
        assert_eq!(bm.len(), 1);
        assert!(!bm.contains_val(&100));
    }

    #[test]
    fn contains() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(5, 50).unwrap();
        assert!(bm.contains_key(5));
        assert!(bm.contains_val(&50));
    }

    #[test]
    fn stats() {
        let mut bm: BimMap<u64> = BimMap::new();
        bm.insert(1, 10).unwrap(); bm.get_by_key(1);
        assert_eq!(bm.total_inserts(), 1);
        assert_eq!(bm.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(BmErr::NotFound(5).to_string().contains("not found")); }
}
