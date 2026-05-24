use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagError {
    NotFound { tag: u32, key: u64 },
    TagExists { tag: u32 },
}

impl std::fmt::Display for TagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagError::NotFound { tag, key } => write!(f, "tag {tag} key {key} not found"),
            TagError::TagExists { tag } => write!(f, "tag {tag} exists"),
        }
    }
}

impl std::error::Error for TagError {}

#[derive(Debug, Clone)]
pub struct TagStats {
    pub tag_count: usize,
    pub total_entries: usize,
    pub total_lookups: u64,
    pub total_hits: u64,
}

#[derive(Debug, Clone)]
pub struct TagStore<V> {
    store: BTreeMap<u32, BTreeMap<u64, V>>,
    total_lookups: u64,
    total_hits: u64,
    total_inserts: u64,
}

impl<V> TagStore<V> {
    pub fn new() -> Self {
        Self { store: BTreeMap::new(), total_lookups: 0, total_hits: 0, total_inserts: 0 }
    }

    pub fn create_tag(&mut self, tag: u32) -> Result<(), TagError> {
        if self.store.contains_key(&tag) {
            return Err(TagError::TagExists { tag });
        }
        self.store.insert(tag, BTreeMap::new());
        Ok(())
    }

    pub fn remove_tag(&mut self, tag: u32) -> usize {
        self.store.remove(&tag).map(|m| m.len()).unwrap_or(0)
    }

    pub fn insert(&mut self, tag: u32, key: u64, value: V) -> Result<Option<V>, TagError> {
        let ns = self.store.get_mut(&tag).ok_or(TagError::NotFound { tag, key })?;
        let old = ns.insert(key, value);
        self.total_inserts += 1;
        Ok(old)
    }

    pub fn get(&self, tag: u32, key: u64) -> Option<&V> {
        self.store.get(&tag).and_then(|ns| ns.get(&key))
    }

    pub fn get_mut(&mut self, tag: u32, key: u64) -> Option<&mut V> {
        self.store.get_mut(&tag).and_then(|ns| ns.get_mut(&key))
    }

    pub fn lookup(&mut self, tag: u32, key: u64) -> Option<&V> {
        self.total_lookups += 1;
        let result = self.store.get(&tag).and_then(|ns| ns.get(&key));
        if result.is_some() { self.total_hits += 1; }
        result
    }

    pub fn remove(&mut self, tag: u32, key: u64) -> Option<V> {
        self.store.get_mut(&tag).and_then(|ns| ns.remove(&key))
    }

    pub fn contains(&self, tag: u32, key: u64) -> bool {
        self.store.get(&tag).map(|ns| ns.contains_key(&key)).unwrap_or(false)
    }

    pub fn tag_keys(&self, tag: u32) -> Vec<u64> {
        self.store.get(&tag).map(|ns| ns.keys().copied().collect()).unwrap_or_default()
    }

    pub fn tag_len(&self, tag: u32) -> usize {
        self.store.get(&tag).map(|ns| ns.len()).unwrap_or(0)
    }

    pub fn drain_tag(&mut self, tag: u32) -> Vec<(u64, V)> {
        let keys: Vec<u64> = self.store.get(&tag).map(|ns| ns.keys().copied().collect()).unwrap_or_default();
        let mut result = Vec::with_capacity(keys.len());
        if let Some(ns) = self.store.get_mut(&tag) {
            for k in keys {
                if let Some(v) = ns.remove(&k) { result.push((k, v)); }
            }
        }
        result
    }

    pub fn tag_count(&self) -> usize {
        self.store.len()
    }

    pub fn total_entries(&self) -> usize {
        self.store.values().map(|ns| ns.len()).sum()
    }

    pub fn has_tag(&self, tag: u32) -> bool {
        self.store.contains_key(&tag)
    }

    pub fn tag_ids(&self) -> Vec<u32> {
        self.store.keys().copied().collect()
    }

    pub fn stats(&self) -> TagStats {
        TagStats {
            tag_count: self.tag_count(),
            total_entries: self.total_entries(),
            total_lookups: self.total_lookups,
            total_hits: self.total_hits,
        }
    }

    pub fn clear(&mut self) {
        self.store.clear();
    }
}

impl<V> Default for TagStore<V> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_store() {
        let ts: TagStore<i32> = TagStore::new();
        assert_eq!(ts.tag_count(), 0);
    }

    #[test]
    fn create_tag() {
        let mut ts: TagStore<i32> = TagStore::new();
        ts.create_tag(1).unwrap();
        assert_eq!(ts.tag_count(), 1);
        assert!(ts.has_tag(1));
    }

    #[test]
    fn duplicate_tag() {
        let mut ts: TagStore<i32> = TagStore::new();
        ts.create_tag(1).unwrap();
        let err = ts.create_tag(1).unwrap_err();
        assert!(matches!(err, TagError::TagExists { tag: 1 }));
    }

    #[test]
    fn insert_and_get() {
        let mut ts = TagStore::new();
        ts.create_tag(1).unwrap();
        ts.insert(1, 100, 42).unwrap();
        assert_eq!(ts.get(1, 100), Some(&42));
    }

    #[test]
    fn insert_missing_tag() {
        let mut ts: TagStore<i32> = TagStore::new();
        let err = ts.insert(99, 1, 10).unwrap_err();
        assert!(matches!(err, TagError::NotFound { tag: 99, .. }));
    }

    #[test]
    fn lookup_hit_miss() {
        let mut ts = TagStore::new();
        ts.create_tag(1).unwrap();
        ts.insert(1, 1, 10).unwrap();
        ts.lookup(1, 1);
        ts.lookup(1, 2);
        let s = ts.stats();
        assert_eq!(s.total_lookups, 2);
        assert_eq!(s.total_hits, 1);
    }

    #[test]
    fn remove_entry() {
        let mut ts = TagStore::new();
        ts.create_tag(1).unwrap();
        ts.insert(1, 1, 10).unwrap();
        let v = ts.remove(1, 1).unwrap();
        assert_eq!(v, 10);
        assert!(!ts.contains(1, 1));
    }

    #[test]
    fn tag_keys() {
        let mut ts = TagStore::new();
        ts.create_tag(1).unwrap();
        ts.insert(1, 10, 1).unwrap();
        ts.insert(1, 20, 2).unwrap();
        assert_eq!(ts.tag_keys(1), vec![10, 20]);
    }

    #[test]
    fn drain_tag() {
        let mut ts = TagStore::new();
        ts.create_tag(1).unwrap();
        ts.insert(1, 1, 10).unwrap();
        ts.insert(1, 2, 20).unwrap();
        let items = ts.drain_tag(1);
        assert_eq!(items.len(), 2);
        assert_eq!(ts.tag_len(1), 0);
    }

    #[test]
    fn remove_tag() {
        let mut ts = TagStore::new();
        ts.create_tag(1).unwrap();
        ts.insert(1, 1, 10).unwrap();
        let count = ts.remove_tag(1);
        assert_eq!(count, 1);
        assert!(!ts.has_tag(1));
    }

    #[test]
    fn namespace_isolation() {
        let mut ts = TagStore::new();
        ts.create_tag(1).unwrap();
        ts.create_tag(2).unwrap();
        ts.insert(1, 1, 10).unwrap();
        ts.insert(2, 1, 20).unwrap();
        assert_eq!(ts.get(1, 1), Some(&10));
        assert_eq!(ts.get(2, 1), Some(&20));
    }

    #[test]
    fn error_display() {
        assert!(TagError::NotFound { tag: 1, key: 2 }.to_string().contains("1"));
    }
}
