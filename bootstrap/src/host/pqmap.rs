use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PqmError {
    KeyNotFound { key: u64 },
    KeyExists { key: u64 },
}

impl std::fmt::Display for PqmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PqmError::KeyNotFound { key } => write!(f, "key {key} not found"),
            PqmError::KeyExists { key } => write!(f, "key {key} exists"),
        }
    }
}

impl std::error::Error for PqmError {}

struct Entry {
    key: u64,
    priority: i64,
    data: Vec<u8>,
}

pub struct PqMap {
    entries: BTreeMap<u64, Entry>,
    total_inserts: u64,
    total_pops: u64,
    total_updates: u64,
}

impl PqMap {
    pub fn new() -> Self { Self { entries: BTreeMap::new(), total_inserts: 0, total_pops: 0, total_updates: 0 } }

    pub fn insert(&mut self, key: u64, priority: i64, data: Vec<u8>) -> Result<(), PqmError> {
        if self.entries.contains_key(&key) { return Err(PqmError::KeyExists { key }); }
        self.entries.insert(key, Entry { key, priority, data });
        self.total_inserts += 1;
        Ok(())
    }

    pub fn pop_min(&mut self) -> Option<(u64, i64, Vec<u8>)> {
        let min_key = self.entries.values().min_by_key(|e| e.priority)?.key;
        let e = self.entries.remove(&min_key)?;
        self.total_pops += 1;
        Some((e.key, e.priority, e.data))
    }

    pub fn pop_max(&mut self) -> Option<(u64, i64, Vec<u8>)> {
        let max_key = self.entries.values().max_by_key(|e| e.priority)?.key;
        let e = self.entries.remove(&max_key)?;
        self.total_pops += 1;
        Some((e.key, e.priority, e.data))
    }

    pub fn update_priority(&mut self, key: u64, new_priority: i64) -> Result<i64, PqmError> {
        let e = self.entries.get_mut(&key).ok_or(PqmError::KeyNotFound { key })?;
        let old = e.priority;
        e.priority = new_priority;
        self.total_updates += 1;
        Ok(old)
    }

    pub fn decrease_key(&mut self, key: u64, new_priority: i64) -> Result<i64, PqmError> {
        let e = self.entries.get_mut(&key).ok_or(PqmError::KeyNotFound { key })?;
        if new_priority >= e.priority { return Ok(e.priority); }
        let old = e.priority;
        e.priority = new_priority;
        self.total_updates += 1;
        Ok(old)
    }

    pub fn get(&self, key: u64) -> Option<(i64, &Vec<u8>)> { self.entries.get(&key).map(|e| (e.priority, &e.data)) }

    pub fn top_k(&self, k: usize) -> Vec<(u64, i64)> {
        let mut entries: Vec<&Entry> = self.entries.values().collect();
        entries.sort_by_key(|e| e.priority);
        entries.into_iter().take(k).map(|e| (e.key, e.priority)).collect()
    }

    pub fn bottom_k(&self, k: usize) -> Vec<(u64, i64)> {
        let mut entries: Vec<&Entry> = self.entries.values().collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.priority));
        entries.into_iter().take(k).map(|e| (e.key, e.priority)).collect()
    }

    pub fn contains(&self, key: u64) -> bool { self.entries.contains_key(&key) }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_pops(&self) -> u64 { self.total_pops }
    pub fn total_updates(&self) -> u64 { self.total_updates }
}

impl Default for PqMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pqm() { assert!(PqMap::new().is_empty()); }

    #[test]
    fn insert_pop_min() {
        let mut pq = PqMap::new();
        pq.insert(1, 10, b"a".to_vec()).unwrap();
        pq.insert(2, 5, b"b".to_vec()).unwrap();
        let (k, p, _) = pq.pop_min().unwrap();
        assert_eq!(k, 2); assert_eq!(p, 5);
    }

    #[test]
    fn pop_max() {
        let mut pq = PqMap::new();
        pq.insert(1, 10, b"a".to_vec()).unwrap();
        pq.insert(2, 5, b"b".to_vec()).unwrap();
        let (k, _, _) = pq.pop_max().unwrap();
        assert_eq!(k, 1);
    }

    #[test]
    fn update_priority() {
        let mut pq = PqMap::new();
        pq.insert(1, 10, b"a".to_vec()).unwrap();
        let old = pq.update_priority(1, 1).unwrap();
        assert_eq!(old, 10);
        let (k, _, _) = pq.pop_min().unwrap();
        assert_eq!(k, 1);
    }

    #[test]
    fn decrease_key() {
        let mut pq = PqMap::new();
        pq.insert(1, 10, b"a".to_vec()).unwrap();
        pq.insert(2, 5, b"b".to_vec()).unwrap();
        pq.decrease_key(1, 1).unwrap();
        let (k, _, _) = pq.pop_min().unwrap();
        assert_eq!(k, 1);
    }

    #[test]
    fn decrease_key_noop() {
        let mut pq = PqMap::new();
        pq.insert(1, 5, b"a".to_vec()).unwrap();
        let result = pq.decrease_key(1, 10).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn top_k() {
        let mut pq = PqMap::new();
        for i in 1..=10 { pq.insert(i, i as i64 * 10, vec![]).unwrap(); }
        let top = pq.top_k(3);
        assert_eq!(top[0], (1, 10));
        assert_eq!(top[2], (3, 30));
    }

    #[test]
    fn duplicate_key() {
        let mut pq = PqMap::new();
        pq.insert(1, 1, vec![]).unwrap();
        let err = pq.insert(1, 2, vec![]).unwrap_err();
        assert!(matches!(err, PqmError::KeyExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut pq = PqMap::new();
        let err = pq.update_priority(99, 1).unwrap_err();
        assert!(matches!(err, PqmError::KeyNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut pq = PqMap::new();
        pq.insert(1, 1, vec![]).unwrap();
        pq.pop_min();
        assert_eq!(pq.total_inserts(), 1);
        assert_eq!(pq.total_pops(), 1);
    }

    #[test]
    fn error_display() { assert!(PqmError::KeyNotFound { key: 1 }.to_string().contains("1")); }
}
