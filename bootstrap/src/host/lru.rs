use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum LruError {
    CapacityExceeded { cap: usize },
    KeyNotFound { id: u64 },
}

impl std::fmt::Display for LruError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LruError::CapacityExceeded { cap } => write!(f, "LRU cap {cap} exceeded"),
            LruError::KeyNotFound { id } => write!(f, "key {id} not found"),
        }
    }
}

impl std::error::Error for LruError {}

struct Entry<V> {
    id: u64,
    value: V,
}

#[derive(Debug, Clone)]
pub struct LruStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub inserts: u64,
}

pub struct LruCache<V> {
    order: VecDeque<u64>,
    entries: BTreeMap<u64, Entry<V>>,
    capacity: usize,
    stats: LruStats,
}

impl<V> LruCache<V> {
    pub fn new(capacity: usize) -> Self {
        Self { order: VecDeque::with_capacity(capacity), entries: BTreeMap::new(), capacity, stats: LruStats { hits: 0, misses: 0, evictions: 0, inserts: 0 } }
    }

    pub fn put(&mut self, id: u64, value: V) -> Option<V> {
        if let Some(e) = self.entries.get_mut(&id) {
            let old = std::mem::replace(&mut e.value, value);
            self.touch(id);
            self.stats.hits += 1;
            return Some(old);
        }
        if self.entries.len() >= self.capacity {
            if let Some(evicted) = self.order.pop_front() {
                self.entries.remove(&evicted);
                self.stats.evictions += 1;
            }
        }
        self.entries.insert(id, Entry { id, value });
        self.order.push_back(id);
        self.stats.inserts += 1;
        None
    }

    pub fn get(&mut self, id: u64) -> Option<&V> {
        if self.entries.contains_key(&id) {
            self.touch(id);
            self.stats.hits += 1;
            self.entries.get(&id).map(|e| &e.value)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut V> {
        if self.entries.contains_key(&id) {
            self.touch(id);
            self.stats.hits += 1;
            self.entries.get_mut(&id).map(|e| &mut e.value)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    pub fn remove(&mut self, id: u64) -> Option<V> {
        self.order.retain(|&k| k != id);
        self.entries.remove(&id).map(|e| e.value)
    }

    fn touch(&mut self, id: u64) {
        self.order.retain(|&k| k != id);
        self.order.push_back(id);
    }

    pub fn contains(&self, id: u64) -> bool { self.entries.contains_key(&id) }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn stats(&self) -> &LruStats { &self.stats }
    pub fn hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total == 0 { 0.0 } else { self.stats.hits as f64 / total as f64 }
    }

    pub fn clear(&mut self) {
        self.order.clear();
        self.entries.clear();
    }

    pub fn resize(&mut self, new_cap: usize) {
        self.capacity = new_cap;
        while self.entries.len() > new_cap {
            if let Some(evicted) = self.order.pop_front() {
                self.entries.remove(&evicted);
                self.stats.evictions += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache() {
        let c: LruCache<i32> = LruCache::new(4);
        assert_eq!(c.len(), 0);
        assert_eq!(c.capacity(), 4);
    }

    #[test]
    fn put_get() {
        let mut c = LruCache::new(4);
        c.put(1, 10);
        assert_eq!(*c.get(1).unwrap(), 10);
    }

    #[test]
    fn miss() {
        let mut c: LruCache<i32> = LruCache::new(4);
        assert!(c.get(99).is_none());
        assert_eq!(c.stats().misses, 1);
    }

    #[test]
    fn evict_lru() {
        let mut c = LruCache::new(2);
        c.put(1, 'a');
        c.put(2, 'b');
        c.put(3, 'c');
        assert!(!c.contains(1));
        assert!(c.contains(2));
        assert!(c.contains(3));
        assert_eq!(c.stats().evictions, 1);
    }

    #[test]
    fn touch_updates_order() {
        let mut c = LruCache::new(2);
        c.put(1, 'a');
        c.put(2, 'b');
        c.get(1);
        c.put(3, 'c');
        assert!(c.contains(1));
        assert!(!c.contains(2));
    }

    #[test]
    fn remove() {
        let mut c = LruCache::new(4);
        c.put(1, 10);
        let v = c.remove(1);
        assert_eq!(v, Some(10));
        assert!(!c.contains(1));
    }

    #[test]
    fn overwrite() {
        let mut c = LruCache::new(4);
        let old = c.put(1, 10);
        assert!(old.is_none());
        let old = c.put(1, 20);
        assert_eq!(old, Some(10));
        assert_eq!(*c.get(1).unwrap(), 20);
    }

    #[test]
    fn hit_rate() {
        let mut c = LruCache::new(4);
        c.put(1, 1);
        c.get(1);
        c.get(2);
        assert_eq!(c.hit_rate(), 0.5);
    }

    #[test]
    fn resize_shrink() {
        let mut c = LruCache::new(4);
        c.put(1, 'a'); c.put(2, 'b'); c.put(3, 'c');
        c.resize(2);
        assert_eq!(c.len(), 2);
        assert_eq!(c.stats().evictions, 1);
    }

    #[test]
    fn clear() {
        let mut c = LruCache::new(4);
        c.put(1, 1); c.put(2, 2);
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn get_mut() {
        let mut c = LruCache::new(4);
        c.put(1, 10);
        *c.get_mut(1).unwrap() = 20;
        assert_eq!(*c.get(1).unwrap(), 20);
    }

    #[test]
    fn error_display() {
        assert!(LruError::KeyNotFound { id: 3 }.to_string().contains("3"));
    }
}
