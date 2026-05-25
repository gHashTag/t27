use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DmError {
    KeyExists { key: u64 },
    ValueExists { value: u64 },
    NotFound { key: u64 },
}

impl std::fmt::Display for DmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DmError::KeyExists { key } => write!(f, "key {key} exists"),
            DmError::ValueExists { value } => write!(f, "value {value} exists"),
            DmError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for DmError {}

pub struct DoubleMap {
    forward: BTreeMap<u64, u64>,
    reverse: BTreeMap<u64, u64>,
    total_inserts: u64,
    total_removes: u64,
    total_lookups: u64,
}

impl DoubleMap {
    pub fn new() -> Self { Self { forward: BTreeMap::new(), reverse: BTreeMap::new(), total_inserts: 0, total_removes: 0, total_lookups: 0 } }

    pub fn insert(&mut self, key: u64, value: u64) -> Result<(), DmError> {
        if self.forward.contains_key(&key) { return Err(DmError::KeyExists { key }); }
        if self.reverse.contains_key(&value) { return Err(DmError::ValueExists { value }); }
        self.forward.insert(key, value);
        self.reverse.insert(value, key);
        self.total_inserts += 1;
        Ok(())
    }

    pub fn get_by_key(&mut self, key: u64) -> Option<u64> {
        self.total_lookups += 1;
        self.forward.get(&key).copied()
    }

    pub fn get_by_value(&mut self, value: u64) -> Option<u64> {
        self.total_lookups += 1;
        self.reverse.get(&value).copied()
    }

    pub fn remove_by_key(&mut self, key: u64) -> Result<u64, DmError> {
        let value = self.forward.remove(&key).ok_or(DmError::NotFound { key })?;
        self.reverse.remove(&value);
        self.total_removes += 1;
        Ok(value)
    }

    pub fn remove_by_value(&mut self, value: u64) -> Result<u64, DmError> {
        let key = self.reverse.remove(&value).ok_or(DmError::NotFound { key: value })?;
        self.forward.remove(&key);
        self.total_removes += 1;
        Ok(key)
    }

    pub fn contains_key(&self, key: u64) -> bool { self.forward.contains_key(&key) }
    pub fn contains_value(&self, value: u64) -> bool { self.reverse.contains_key(&value) }

    pub fn upsert(&mut self, key: u64, value: u64) -> Option<u64> {
        if let Some(old_val) = self.forward.insert(key, value) {
            self.reverse.remove(&old_val);
            self.reverse.insert(value, key);
            Some(old_val)
        } else {
            self.reverse.insert(value, key);
            self.total_inserts += 1;
            None
        }
    }

    pub fn len(&self) -> usize { self.forward.len() }
    pub fn is_empty(&self) -> bool { self.forward.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

impl Default for DoubleMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dm() { assert!(DoubleMap::new().is_empty()); }

    #[test]
    fn insert_lookup() {
        let mut dm = DoubleMap::new();
        dm.insert(1, 100).unwrap(); dm.insert(2, 200).unwrap();
        assert_eq!(dm.get_by_key(1), Some(100));
        assert_eq!(dm.get_by_value(200), Some(2));
    }

    #[test]
    fn key_exists() {
        let mut dm = DoubleMap::new();
        dm.insert(1, 100).unwrap();
        assert!(matches!(dm.insert(1, 999), Err(DmError::KeyExists { .. })));
    }

    #[test]
    fn value_exists() {
        let mut dm = DoubleMap::new();
        dm.insert(1, 100).unwrap();
        assert!(matches!(dm.insert(2, 100), Err(DmError::ValueExists { .. })));
    }

    #[test]
    fn remove_by_key() {
        let mut dm = DoubleMap::new();
        dm.insert(1, 100).unwrap();
        let v = dm.remove_by_key(1).unwrap();
        assert_eq!(v, 100);
        assert!(!dm.contains_value(100));
    }

    #[test]
    fn remove_by_value() {
        let mut dm = DoubleMap::new();
        dm.insert(1, 100).unwrap();
        let k = dm.remove_by_value(100).unwrap();
        assert_eq!(k, 1);
        assert!(!dm.contains_key(1));
    }

    #[test]
    fn upsert() {
        let mut dm = DoubleMap::new();
        dm.insert(1, 100).unwrap();
        let old = dm.upsert(1, 200);
        assert_eq!(old, Some(100));
        assert_eq!(dm.get_by_key(1), Some(200));
        assert_eq!(dm.get_by_value(200), Some(1));
    }

    #[test]
    fn not_found() { assert!(DoubleMap::new().remove_by_key(1).is_err()); }

    #[test]
    fn stats() {
        let mut dm = DoubleMap::new();
        dm.insert(1, 100).unwrap(); dm.get_by_key(1);
        assert_eq!(dm.total_inserts(), 1);
        assert_eq!(dm.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(DmError::KeyExists { key: 1 }.to_string().contains("exists")); }
}
