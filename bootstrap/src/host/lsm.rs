use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LsmError {
    KeyNotFound { key: String },
    Tombstone { key: String },
}

impl std::fmt::Display for LsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LsmError::KeyNotFound { key } => write!(f, "key {key} not found"),
            LsmError::Tombstone { key } => write!(f, "key {key} deleted"),
        }
    }
}

impl std::error::Error for LsmError {}

#[derive(Debug, Clone)]
struct Entry {
    value: Option<Vec<u8>>,
    seq: u64,
}

#[derive(Debug, Clone)]
pub struct LsmTree {
    memtable: BTreeMap<String, Entry>,
    levels: Vec<BTreeMap<String, Entry>>,
    memtable_limit: usize,
    level_ratio: usize,
    next_seq: u64,
    total_puts: u64,
    total_deletes: u64,
    total_flushes: u64,
    total_compactions: u64,
}

impl LsmTree {
    pub fn new(memtable_limit: usize, level_ratio: usize) -> Self {
        Self {
            memtable: BTreeMap::new(), levels: vec![BTreeMap::new()], memtable_limit, level_ratio,
            next_seq: 1, total_puts: 0, total_deletes: 0, total_flushes: 0, total_compactions: 0,
        }
    }

    pub fn put(&mut self, key: &str, value: Vec<u8>) {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.memtable.insert(key.to_string(), Entry { value: Some(value), seq });
        self.total_puts += 1;
        if self.memtable.len() >= self.memtable_limit { self.flush(); }
    }

    pub fn delete(&mut self, key: &str) {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.memtable.insert(key.to_string(), Entry { value: None, seq });
        self.total_deletes += 1;
        if self.memtable.len() >= self.memtable_limit { self.flush(); }
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>, LsmError> {
        if let Some(e) = self.memtable.get(key) {
            return e.value.clone().ok_or_else(|| LsmError::Tombstone { key: key.to_string() });
        }
        for level in &self.levels {
            if let Some(e) = level.get(key) {
                return e.value.clone().ok_or_else(|| LsmError::Tombstone { key: key.to_string() });
            }
        }
        Err(LsmError::KeyNotFound { key: key.to_string() })
    }

    pub fn contains(&self, key: &str) -> bool {
        if let Some(e) = self.memtable.get(key) { return e.value.is_some(); }
        for level in &self.levels {
            if let Some(e) = level.get(key) { return e.value.is_some(); }
        }
        false
    }

    fn flush(&mut self) {
        let new_memtable = std::mem::take(&mut self.memtable);
        self.levels[0] = merge_maps(std::mem::replace(&mut self.levels[0], new_memtable), &self.levels[0]);
        self.total_flushes += 1;
        self.compact();
    }

    fn compact(&mut self) {
        for lvl in 0..self.levels.len() {
            let size_limit = self.level_ratio.pow((lvl + 1) as u32) * self.memtable_limit;
            if self.levels[lvl].len() >= size_limit {
                let data = std::mem::take(&mut self.levels[lvl]);
                if lvl + 1 >= self.levels.len() { self.levels.push(BTreeMap::new()); }
                self.levels[lvl + 1] = merge_maps(data, &self.levels[lvl + 1]);
                self.total_compactions += 1;
            }
        }
    }

    pub fn memtable_size(&self) -> usize { self.memtable.len() }
    pub fn level_count(&self) -> usize { self.levels.len() }
    pub fn level_size(&self, lvl: usize) -> Option<usize> { self.levels.get(lvl).map(|l| l.len()) }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_flushes(&self) -> u64 { self.total_flushes }
    pub fn total_compactions(&self) -> u64 { self.total_compactions }
}

fn merge_maps(newer: BTreeMap<String, Entry>, older: &BTreeMap<String, Entry>) -> BTreeMap<String, Entry> {
    let mut result = newer;
    for (k, v) in older {
        result.entry(k.clone()).or_insert(v.clone());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree() {
        let t = LsmTree::new(4, 2);
        assert_eq!(t.memtable_size(), 0);
    }

    #[test]
    fn put_get() {
        let mut t = LsmTree::new(100, 2);
        t.put("k1", vec![1, 2, 3]);
        assert_eq!(t.get("k1").unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn key_not_found() {
        let t = LsmTree::new(100, 2);
        let err = t.get("nope").unwrap_err();
        assert!(matches!(err, LsmError::KeyNotFound { .. }));
    }

    #[test]
    fn delete() {
        let mut t = LsmTree::new(100, 2);
        t.put("k1", vec![1]);
        t.delete("k1");
        assert!(!t.contains("k1"));
        let err = t.get("k1").unwrap_err();
        assert!(matches!(err, LsmError::Tombstone { .. }));
    }

    #[test]
    fn overwrite() {
        let mut t = LsmTree::new(100, 2);
        t.put("k1", vec![1]);
        t.put("k1", vec![2]);
        assert_eq!(t.get("k1").unwrap(), vec![2]);
    }

    #[test]
    fn flush_triggers() {
        let mut t = LsmTree::new(3, 2);
        t.put("a", vec![1]); t.put("b", vec![2]); t.put("c", vec![3]);
        assert!(t.total_flushes() >= 1);
        assert_eq!(t.get("a").unwrap(), vec![1]);
    }

    #[test]
    fn contains() {
        let mut t = LsmTree::new(100, 2);
        t.put("k", vec![1]);
        assert!(t.contains("k"));
        assert!(!t.contains("x"));
    }

    #[test]
    fn stats() {
        let mut t = LsmTree::new(100, 2);
        t.put("a", vec![1]); t.put("b", vec![2]);
        t.delete("a");
        assert_eq!(t.total_puts(), 2);
        assert_eq!(t.total_deletes(), 1);
    }

    #[test]
    fn level_sizes() {
        let mut t = LsmTree::new(2, 2);
        for i in 0..10u8 { t.put(&format!("k{i}"), vec![i]); }
        assert!(t.level_count() >= 1);
    }

    #[test]
    fn delete_resurrect() {
        let mut t = LsmTree::new(100, 2);
        t.put("k", vec![1]); t.delete("k"); t.put("k", vec![2]);
        assert_eq!(t.get("k").unwrap(), vec![2]);
    }

    #[test]
    fn many_puts() {
        let mut t = LsmTree::new(4, 2);
        for i in 0..20u8 { t.put(&format!("k{i}"), vec![i]); }
        for i in 0..20u8 { assert_eq!(t.get(&format!("k{i}")).unwrap(), vec![i]); }
    }

    #[test]
    fn error_display() {
        assert!(LsmError::KeyNotFound { key: "x".into() }.to_string().contains("x"));
    }
}
