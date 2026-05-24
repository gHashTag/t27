use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum KvError {
    KeyNotFound { key: u64 },
    Tombstone { key: u64 },
}

impl std::fmt::Display for KvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvError::KeyNotFound { key } => write!(f, "key {key} not found"),
            KvError::Tombstone { key } => write!(f, "key {key} is tombstoned"),
        }
    }
}

impl std::error::Error for KvError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry { Value(Vec<u8>), Tombstone }

struct Segment {
    id: u64,
    data: BTreeMap<u64, Entry>,
}

pub struct KvEngine {
    memtable: BTreeMap<u64, Entry>,
    segments: Vec<Segment>,
    memtable_limit: usize,
    next_segment: u64,
    total_puts: u64,
    total_gets: u64,
    total_deletes: u64,
    total_flushes: u64,
    total_compactions: u64,
}

impl KvEngine {
    pub fn new(memtable_limit: usize) -> Self {
        Self { memtable: BTreeMap::new(), segments: Vec::new(), memtable_limit, next_segment: 1, total_puts: 0, total_gets: 0, total_deletes: 0, total_flushes: 0, total_compactions: 0 }
    }

    pub fn put(&mut self, key: u64, value: Vec<u8>) {
        self.memtable.insert(key, Entry::Value(value));
        self.total_puts += 1;
        if self.memtable.len() >= self.memtable_limit { self.flush(); }
    }

    pub fn get(&mut self, key: u64) -> Result<Vec<u8>, KvError> {
        self.total_gets += 1;
        if let Some(entry) = self.memtable.get(&key) {
            return match entry {
                Entry::Value(v) => Ok(v.clone()),
                Entry::Tombstone => Err(KvError::Tombstone { key }),
            };
        }
        for seg in self.segments.iter().rev() {
            if let Some(entry) = seg.data.get(&key) {
                return match entry {
                    Entry::Value(v) => Ok(v.clone()),
                    Entry::Tombstone => Err(KvError::Tombstone { key }),
                };
            }
        }
        Err(KvError::KeyNotFound { key })
    }

    pub fn delete(&mut self, key: u64) {
        self.memtable.insert(key, Entry::Tombstone);
        self.total_deletes += 1;
        if self.memtable.len() >= self.memtable_limit { self.flush(); }
    }

    pub fn flush(&mut self) {
        if self.memtable.is_empty() { return; }
        let id = self.next_segment;
        self.next_segment += 1;
        let data = std::mem::take(&mut self.memtable);
        self.segments.push(Segment { id, data });
        self.total_flushes += 1;
    }

    pub fn compact(&mut self) {
        let mut merged: BTreeMap<u64, Entry> = BTreeMap::new();
        for seg in &self.segments {
            for (&k, v) in &seg.data {
                merged.insert(k, v.clone());
            }
        }
        let mut mem_merged: BTreeMap<u64, Entry> = BTreeMap::new();
        std::mem::swap(&mut mem_merged, &mut self.memtable);
        for (&k, v) in &mem_merged {
            merged.insert(k, v.clone());
        }
        merged.retain(|_, v| *v != Entry::Tombstone);
        self.segments.clear();
        self.memtable = merged;
        self.total_compactions += 1;
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_ok() }
    pub fn segment_count(&self) -> usize { self.segments.len() }
    pub fn memtable_size(&self) -> usize { self.memtable.len() }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_flushes(&self) -> u64 { self.total_flushes }
    pub fn total_compactions(&self) -> u64 { self.total_compactions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_engine() { let kv = KvEngine::new(100); assert_eq!(kv.memtable_size(), 0); }

    #[test]
    fn put_get() {
        let mut kv = KvEngine::new(100);
        kv.put(1, b"val".to_vec());
        assert_eq!(kv.get(1), Ok(b"val".to_vec()));
    }

    #[test]
    fn auto_flush() {
        let mut kv = KvEngine::new(3);
        kv.put(1, b"a".to_vec()); kv.put(2, b"b".to_vec()); kv.put(3, b"c".to_vec());
        assert_eq!(kv.segment_count(), 1);
        assert_eq!(kv.total_flushes(), 1);
    }

    #[test]
    fn get_from_segment() {
        let mut kv = KvEngine::new(2);
        kv.put(1, b"a".to_vec()); kv.put(2, b"b".to_vec());
        kv.put(3, b"c".to_vec());
        assert_eq!(kv.get(1), Ok(b"a".to_vec()));
        assert_eq!(kv.get(3), Ok(b"c".to_vec()));
    }

    #[test]
    fn delete_tombstone() {
        let mut kv = KvEngine::new(100);
        kv.put(1, b"val".to_vec());
        kv.delete(1);
        let err = kv.get(1).unwrap_err();
        assert!(matches!(err, KvError::Tombstone { .. }));
    }

    #[test]
    fn not_found() {
        let mut kv = KvEngine::new(100);
        let err = kv.get(99).unwrap_err();
        assert!(matches!(err, KvError::KeyNotFound { .. }));
    }

    #[test]
    fn compact() {
        let mut kv = KvEngine::new(2);
        kv.put(1, b"a".to_vec()); kv.put(2, b"b".to_vec());
        kv.delete(1);
        kv.put(3, b"c".to_vec());
        kv.compact();
        assert_eq!(kv.segment_count(), 0);
        assert!(kv.contains(2));
        assert!(kv.contains(3));
        assert!(!kv.contains(1));
    }

    #[test]
    fn overwrite() {
        let mut kv = KvEngine::new(100);
        kv.put(1, b"old".to_vec());
        kv.put(1, b"new".to_vec());
        assert_eq!(kv.get(1), Ok(b"new".to_vec()));
    }

    #[test]
    fn contains() {
        let mut kv = KvEngine::new(100);
        kv.put(1, b"x".to_vec());
        assert!(kv.contains(1));
        assert!(!kv.contains(2));
    }

    #[test]
    fn stats() {
        let mut kv = KvEngine::new(2);
        kv.put(1, b"x".to_vec()); kv.put(2, b"x".to_vec());
        kv.get(1);
        assert_eq!(kv.total_puts(), 2);
        assert_eq!(kv.total_gets(), 1);
    }

    #[test]
    fn error_display() { assert!(KvError::KeyNotFound { key: 1 }.to_string().contains("1")); }
}
