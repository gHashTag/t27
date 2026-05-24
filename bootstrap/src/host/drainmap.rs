use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainMapError {
    KeyNotFound { key: u64 },
    CapacityExceeded { capacity: usize },
}

impl std::fmt::Display for DrainMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrainMapError::KeyNotFound { key } => write!(f, "key {key} not found"),
            DrainMapError::CapacityExceeded { capacity } => write!(f, "cap {capacity} exceeded"),
        }
    }
}

impl std::error::Error for DrainMapError {}

#[derive(Debug, Clone)]
pub struct DrainMap<V> {
    map: BTreeMap<u64, V>,
    capacity: Option<usize>,
    total_inserts: u64,
    total_removes: u64,
    total_drains: u64,
    peak_len: usize,
}

impl<V> DrainMap<V> {
    pub fn new() -> Self {
        Self { map: BTreeMap::new(), capacity: None, total_inserts: 0, total_removes: 0, total_drains: 0, peak_len: 0 }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { map: BTreeMap::new(), capacity: Some(cap), total_inserts: 0, total_removes: 0, total_drains: 0, peak_len: 0 }
    }

    pub fn insert(&mut self, key: u64, value: V) -> Result<Option<V>, DrainMapError> {
        if let Some(cap) = self.capacity {
            if !self.map.contains_key(&key) && self.map.len() >= cap {
                return Err(DrainMapError::CapacityExceeded { capacity: cap });
            }
        }
        let old = self.map.insert(key, value);
        self.total_inserts += 1;
        if self.map.len() > self.peak_len { self.peak_len = self.map.len(); }
        Ok(old)
    }

    pub fn remove(&mut self, key: u64) -> Result<V, DrainMapError> {
        let val = self.map.remove(&key).ok_or(DrainMapError::KeyNotFound { key })?;
        self.total_removes += 1;
        Ok(val)
    }

    pub fn get(&self, key: u64) -> Option<&V> {
        self.map.get(&key)
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut V> {
        self.map.get_mut(&key)
    }

    pub fn contains(&self, key: u64) -> bool {
        self.map.contains_key(&key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn drain_all(&mut self) -> Vec<(u64, V)> {
        let keys: Vec<u64> = self.map.keys().copied().collect();
        let mut result = Vec::with_capacity(keys.len());
        for k in keys {
            if let Some(v) = self.map.remove(&k) {
                result.push((k, v));
            }
        }
        self.total_drains += result.len() as u64;
        self.total_removes += result.len() as u64;
        result
    }

    pub fn drain_where<F>(&mut self, mut pred: F) -> Vec<(u64, V)>
    where
        F: FnMut(&u64, &V) -> bool,
    {
        let keys: Vec<u64> = self.map.iter()
            .filter(|(k, v)| pred(k, v))
            .map(|(&k, _)| k)
            .collect();
        let mut result = Vec::with_capacity(keys.len());
        for k in keys {
            if let Some(v) = self.map.remove(&k) {
                result.push((k, v));
            }
        }
        self.total_drains += result.len() as u64;
        self.total_removes += result.len() as u64;
        result
    }

    pub fn drain_range(&mut self, lo: u64, hi: u64) -> Vec<(u64, V)> {
        let keys: Vec<u64> = self.map.range(lo..hi).map(|(&k, _)| k).collect();
        let mut result = Vec::with_capacity(keys.len());
        for k in keys {
            if let Some(v) = self.map.remove(&k) {
                result.push((k, v));
            }
        }
        self.total_drains += result.len() as u64;
        self.total_removes += result.len() as u64;
        result
    }

    pub fn retain<F>(&mut self, mut pred: F)
    where
        F: FnMut(&u64, &V) -> bool,
    {
        let before = self.map.len();
        self.map.retain(|k, v| pred(k, v));
        let removed = before - self.map.len();
        self.total_removes += removed as u64;
    }

    pub fn keys(&self) -> Vec<u64> {
        self.map.keys().copied().collect()
    }

    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }

    pub fn total_inserts(&self) -> u64 {
        self.total_inserts
    }

    pub fn total_removes(&self) -> u64 {
        self.total_removes
    }

    pub fn total_drains(&self) -> u64 {
        self.total_drains
    }

    pub fn peak_len(&self) -> usize {
        self.peak_len
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl<V> Default for DrainMap<V> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() {
        let dm: DrainMap<i32> = DrainMap::new();
        assert!(dm.is_empty());
    }

    #[test]
    fn insert_and_get() {
        let mut dm = DrainMap::new();
        dm.insert(1, 10).unwrap();
        dm.insert(2, 20).unwrap();
        assert_eq!(dm.get(1), Some(&10));
        assert_eq!(dm.len(), 2);
    }

    #[test]
    fn remove() {
        let mut dm = DrainMap::new();
        dm.insert(1, 10).unwrap();
        let v = dm.remove(1).unwrap();
        assert_eq!(v, 10);
        assert!(dm.is_empty());
    }

    #[test]
    fn remove_not_found() {
        let mut dm: DrainMap<i32> = DrainMap::new();
        let err = dm.remove(99).unwrap_err();
        assert!(matches!(err, DrainMapError::KeyNotFound { .. }));
    }

    #[test]
    fn capacity_limit() {
        let mut dm = DrainMap::with_capacity(2);
        dm.insert(1, 10).unwrap();
        dm.insert(2, 20).unwrap();
        let err = dm.insert(3, 30).unwrap_err();
        assert!(matches!(err, DrainMapError::CapacityExceeded { capacity: 2 }));
    }

    #[test]
    fn drain_all() {
        let mut dm = DrainMap::new();
        dm.insert(1, 10).unwrap();
        dm.insert(2, 20).unwrap();
        dm.insert(3, 30).unwrap();
        let items = dm.drain_all();
        assert_eq!(items.len(), 3);
        assert!(dm.is_empty());
        assert_eq!(dm.total_drains(), 3);
    }

    #[test]
    fn drain_where() {
        let mut dm = DrainMap::new();
        dm.insert(1, 10).unwrap();
        dm.insert(2, 20).unwrap();
        dm.insert(3, 30).unwrap();
        let items = dm.drain_where(|_, &v| v > 15);
        assert_eq!(items.len(), 2);
        assert_eq!(dm.len(), 1);
    }

    #[test]
    fn drain_range() {
        let mut dm = DrainMap::new();
        for i in 0..10 { dm.insert(i, i * 10).unwrap(); }
        let items = dm.drain_range(3, 7);
        assert_eq!(items.len(), 4);
        assert_eq!(dm.len(), 6);
    }

    #[test]
    fn retain() {
        let mut dm = DrainMap::new();
        for i in 0..10 { dm.insert(i, i).unwrap(); }
        dm.retain(|_, &v| v % 2 == 0);
        assert_eq!(dm.len(), 5);
    }

    #[test]
    fn peak_len() {
        let mut dm = DrainMap::new();
        for i in 0..5 { dm.insert(i, i).unwrap(); }
        dm.drain_all();
        assert_eq!(dm.peak_len(), 5);
    }

    #[test]
    fn stats() {
        let mut dm = DrainMap::new();
        dm.insert(1, 10).unwrap();
        dm.insert(2, 20).unwrap();
        dm.remove(1).unwrap();
        assert_eq!(dm.total_inserts(), 2);
        assert_eq!(dm.total_removes(), 1);
    }

    #[test]
    fn error_display() {
        assert!(DrainMapError::KeyNotFound { key: 42 }.to_string().contains("42"));
    }
}
