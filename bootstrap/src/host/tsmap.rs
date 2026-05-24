use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TsError {
    KeyNotFound { key: String },
    Tombstoned { key: String },
}

impl std::fmt::Display for TsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsError::KeyNotFound { key } => write!(f, "key {key} not found"),
            TsError::Tombstoned { key } => write!(f, "key {key} tombstoned"),
        }
    }
}

impl std::error::Error for TsError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryState { Active, Tombstoned }

struct Entry {
    value: Vec<u8>,
    version: u64,
    state: EntryState,
}

pub struct TombstoneMap {
    entries: BTreeMap<String, Entry>,
    next_version: u64,
    total_puts: u64,
    total_deletes: u64,
    total_gc: u64,
}

impl TombstoneMap {
    pub fn new() -> Self { Self { entries: BTreeMap::new(), next_version: 1, total_puts: 0, total_deletes: 0, total_gc: 0 } }

    pub fn put(&mut self, key: &str, value: Vec<u8>) -> u64 {
        let ver = self.next_version;
        self.next_version += 1;
        self.entries.insert(key.to_string(), Entry { value, version: ver, state: EntryState::Active });
        self.total_puts += 1;
        ver
    }

    pub fn get(&self, key: &str) -> Result<&[u8], TsError> {
        let e = self.entries.get(key).ok_or_else(|| TsError::KeyNotFound { key: key.to_string() })?;
        match e.state {
            EntryState::Active => Ok(&e.value),
            EntryState::Tombstoned => Err(TsError::Tombstoned { key: key.to_string() }),
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.get(key).map(|e| e.state == EntryState::Active).unwrap_or(false)
    }

    pub fn delete(&mut self, key: &str) -> Result<u64, TsError> {
        let e = self.entries.get_mut(key).ok_or_else(|| TsError::KeyNotFound { key: key.to_string() })?;
        if e.state == EntryState::Tombstoned { return Err(TsError::Tombstoned { key: key.to_string() }); }
        e.state = EntryState::Tombstoned;
        self.total_deletes += 1;
        Ok(e.version)
    }

    pub fn resurrect(&mut self, key: &str, value: Vec<u8>) -> u64 {
        let ver = self.next_version;
        self.next_version += 1;
        self.entries.insert(key.to_string(), Entry { value, version: ver, state: EntryState::Active });
        ver
    }

    pub fn gc(&mut self) -> u64 {
        let before = self.entries.len();
        self.entries.retain(|_, e| e.state == EntryState::Active);
        let removed = (before - self.entries.len()) as u64;
        self.total_gc += removed;
        removed
    }

    pub fn gc_older_than(&mut self, version: u64) -> u64 {
        let before = self.entries.len();
        self.entries.retain(|_, e| e.state == EntryState::Active || e.version > version);
        let removed = (before - self.entries.len()) as u64;
        self.total_gc += removed;
        removed
    }

    pub fn version(&self, key: &str) -> Option<u64> { self.entries.get(key).map(|e| e.version) }
    pub fn is_tombstoned(&self, key: &str) -> bool {
        self.entries.get(key).map(|e| e.state == EntryState::Tombstoned).unwrap_or(false)
    }
    pub fn len(&self) -> usize { self.entries.values().filter(|e| e.state == EntryState::Active).count() }
    pub fn tombstone_count(&self) -> usize { self.entries.values().filter(|e| e.state == EntryState::Tombstoned).count() }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_gc(&self) -> u64 { self.total_gc }
}

impl Default for TombstoneMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { assert!(TombstoneMap::new().len() == 0); }

    #[test]
    fn put_get() {
        let mut m = TombstoneMap::new();
        m.put("k", vec![42]);
        assert_eq!(m.get("k").unwrap(), &[42]);
    }

    #[test]
    fn not_found() {
        let m = TombstoneMap::new();
        assert!(matches!(m.get("x"), Err(TsError::KeyNotFound { .. })));
    }

    #[test]
    fn delete_tombstones() {
        let mut m = TombstoneMap::new();
        m.put("k", vec![1]);
        m.delete("k").unwrap();
        assert!(matches!(m.get("k"), Err(TsError::Tombstoned { .. })));
        assert!(m.is_tombstoned("k"));
    }

    #[test]
    fn delete_missing() {
        let mut m = TombstoneMap::new();
        assert!(matches!(m.delete("x"), Err(TsError::KeyNotFound { .. })));
    }

    #[test]
    fn double_delete() {
        let mut m = TombstoneMap::new();
        m.put("k", vec![1]);
        m.delete("k").unwrap();
        assert!(matches!(m.delete("k"), Err(TsError::Tombstoned { .. })));
    }

    #[test]
    fn resurrect() {
        let mut m = TombstoneMap::new();
        m.put("k", vec![1]);
        m.delete("k").unwrap();
        m.resurrect("k", vec![2]);
        assert_eq!(m.get("k").unwrap(), &[2]);
    }

    #[test]
    fn gc() {
        let mut m = TombstoneMap::new();
        m.put("a", vec![1]); m.put("b", vec![2]);
        m.delete("a").unwrap();
        let removed = m.gc();
        assert_eq!(removed, 1);
        assert_eq!(m.len(), 1);
        assert!(!m.is_tombstoned("a"));
    }

    #[test]
    fn gc_older_than() {
        let mut m = TombstoneMap::new();
        m.put("a", vec![1]);
        let v = m.put("b", vec![2]);
        m.delete("a").unwrap();
        m.gc_older_than(v);
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn version() {
        let mut m = TombstoneMap::new();
        m.put("k", vec![1]);
        assert_eq!(m.version("k"), Some(1));
    }

    #[test]
    fn stats() {
        let mut m = TombstoneMap::new();
        m.put("a", vec![]); m.put("b", vec![]);
        m.delete("a").unwrap();
        assert_eq!(m.total_puts(), 2);
        assert_eq!(m.total_deletes(), 1);
        assert_eq!(m.tombstone_count(), 1);
    }

    #[test]
    fn error_display() { assert!(TsError::Tombstoned { key: "k".into() }.to_string().contains("k")); }
}
