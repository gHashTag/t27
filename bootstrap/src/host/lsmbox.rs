use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LsmError {
    KeyNotFound { key: u64 },
}

impl std::fmt::Display for LsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LsmError::KeyNotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for LsmError {}

#[derive(Debug, Clone, PartialEq)]
enum Entry {
    Val(Vec<u8>),
    Tombstone,
}

struct SSTable {
    data: BTreeMap<u64, Entry>,
    level: usize,
}

pub struct LsmBox {
    memtable: BTreeMap<u64, Entry>,
    sstables: Vec<SSTable>,
    memtable_limit: usize,
    total_puts: u64,
    total_gets: u64,
    total_deletes: u64,
    total_flushes: u64,
    total_compactions: u64,
}

impl LsmBox {
    pub fn new(memtable_limit: usize) -> Self {
        Self { memtable: BTreeMap::new(), sstables: Vec::new(), memtable_limit, total_puts: 0, total_gets: 0, total_deletes: 0, total_flushes: 0, total_compactions: 0 }
    }

    pub fn put(&mut self, key: u64, value: Vec<u8>) {
        self.total_puts += 1;
        self.memtable.insert(key, Entry::Val(value));
        if self.memtable.len() >= self.memtable_limit { self.flush(); }
    }

    pub fn delete(&mut self, key: u64) {
        self.total_deletes += 1;
        self.memtable.insert(key, Entry::Tombstone);
        if self.memtable.len() >= self.memtable_limit { self.flush(); }
    }

    fn flush(&mut self) {
        if self.memtable.is_empty() { return; }
        let data = std::mem::take(&mut self.memtable);
        self.sstables.push(SSTable { data, level: 0 });
        self.total_flushes += 1;
        self.maybe_compact();
    }

    fn maybe_compact(&mut self) {
        let level0: Vec<_> = self.sstables.iter().enumerate().filter(|(_, s)| s.level == 0).map(|(i, _)| i).collect();
        if level0.len() < 3 { return; }
        let mut merged = BTreeMap::new();
        for idx in &level0 {
            for (&k, v) in &self.sstables[*idx].data { merged.insert(k, v.clone()); }
        }
        for &idx in level0.iter().rev() { self.sstables.remove(idx); }
        self.sstables.push(SSTable { data: merged, level: 1 });
        self.total_compactions += 1;
    }

    pub fn get(&mut self, key: u64) -> Option<Vec<u8>> {
        self.total_gets += 1;
        if let Some(e) = self.memtable.get(&key) {
            return match e { Entry::Val(v) => Some(v.clone()), Entry::Tombstone => None };
        }
        for sst in self.sstables.iter().rev() {
            if let Some(e) = sst.data.get(&key) {
                return match e { Entry::Val(v) => Some(v.clone()), Entry::Tombstone => None };
            }
        }
        None
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }

    pub fn scan(&self, start: u64, end: u64) -> Vec<(u64, Vec<u8>)> {
        let mut result = BTreeMap::new();
        for (&k, e) in self.memtable.range(start..=end) {
            if let Entry::Val(v) = e { result.insert(k, v.clone()); }
        }
        for sst in &self.sstables {
            for (&k, e) in sst.data.range(start..=end) {
                if let Entry::Val(v) = e { result.insert(k, v.clone()); }
            }
        }
        result.into_iter().collect()
    }

    pub fn force_flush(&mut self) { self.flush(); }
    pub fn memtable_len(&self) -> usize { self.memtable.len() }
    pub fn sstable_count(&self) -> usize { self.sstables.len() }
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
    fn new_lsm() { let l = LsmBox::new(4); assert_eq!(l.memtable_len(), 0); }

    #[test]
    fn put_get() {
        let mut l = LsmBox::new(10);
        l.put(1, b"hello".to_vec());
        assert_eq!(l.get(1), Some(b"hello".to_vec()));
    }

    #[test]
    fn delete() {
        let mut l = LsmBox::new(10);
        l.put(1, b"hello".to_vec());
        l.delete(1);
        assert_eq!(l.get(1), None);
    }

    #[test]
    fn flush_trigger() {
        let mut l = LsmBox::new(3);
        l.put(1, b"a".to_vec()); l.put(2, b"b".to_vec()); l.put(3, b"c".to_vec());
        assert!(l.sstable_count() > 0);
        assert!(l.total_flushes() > 0);
    }

    #[test]
    fn compaction() {
        let mut l = LsmBox::new(2);
        for i in 0..6 { l.put(i, vec![i as u8]); }
        assert!(l.total_compactions() > 0);
        for i in 0..6u64 { assert_eq!(l.get(i), Some(vec![i as u8])); }
    }

    #[test]
    fn scan() {
        let mut l = LsmBox::new(10);
        for i in 1..=5 { l.put(i, vec![i as u8]); }
        let result = l.scan(2, 4);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn overwrite() {
        let mut l = LsmBox::new(10);
        l.put(1, b"old".to_vec()); l.put(1, b"new".to_vec());
        assert_eq!(l.get(1), Some(b"new".to_vec()));
    }

    #[test]
    fn tombstone_after_flush() {
        let mut l = LsmBox::new(2);
        l.put(1, b"a".to_vec()); l.put(2, b"b".to_vec());
        l.delete(1); l.put(3, b"c".to_vec());
        assert_eq!(l.get(1), None);
        assert_eq!(l.get(2), Some(b"b".to_vec()));
    }

    #[test]
    fn stats() {
        let mut l = LsmBox::new(10);
        l.put(1, b"x".to_vec()); l.get(1); l.delete(2);
        assert_eq!(l.total_puts(), 1);
        assert_eq!(l.total_gets(), 1);
        assert_eq!(l.total_deletes(), 1);
    }

    #[test]
    fn error_display() { assert!(LsmError::KeyNotFound { key: 1 }.to_string().contains("not found")); }
}
