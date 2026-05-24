use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CacheTierError {
    KeyNotFound { key: u64 },
}

impl std::fmt::Display for CacheTierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheTierError::KeyNotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for CacheTierError {}

struct Tier<V> {
    entries: BTreeMap<u64, V>,
    order: Vec<u64>,
    capacity: usize,
}

impl<V: Clone> Tier<V> {
    fn new(capacity: usize) -> Self { Self { entries: BTreeMap::new(), order: Vec::new(), capacity } }

    fn get(&mut self, k: u64) -> Option<V> {
        if self.entries.contains_key(&k) {
            self.order.retain(|&x| x != k);
            self.order.push(k);
            self.entries.get(&k).cloned()
        } else { None }
    }

    fn put(&mut self, k: u64, v: V) -> Option<(u64, V)> {
        if self.entries.insert(k, v).is_some() {
            self.order.retain(|&x| x != k);
            self.order.push(k);
            return None;
        }
        self.order.push(k);
        if self.entries.len() > self.capacity {
            if let Some(evict) = self.order.first().copied() {
                self.order.remove(0);
                let val = self.entries.remove(&evict).unwrap();
                return Some((evict, val));
            }
        }
        None
    }

    fn remove(&mut self, k: u64) -> Option<V> {
        self.order.retain(|&x| x != k);
        self.entries.remove(&k)
    }

    fn len(&self) -> usize { self.entries.len() }
    fn is_empty(&self) -> bool { self.entries.is_empty() }
    fn capacity(&self) -> usize { self.capacity }
    fn contains(&self, k: u64) -> bool { self.entries.contains_key(&k) }
}

pub struct CacheTiers<V: Clone> {
    hot: Tier<V>,
    warm: Tier<V>,
    total_gets: u64,
    total_puts: u64,
    total_hits: u64,
    total_misses: u64,
    total_promotions: u64,
    total_demotions: u64,
    total_evictions: u64,
}

impl<V: Clone> CacheTiers<V> {
    pub fn new(hot_capacity: usize, warm_capacity: usize) -> Self {
        Self { hot: Tier::new(hot_capacity), warm: Tier::new(warm_capacity), total_gets: 0, total_puts: 0, total_hits: 0, total_misses: 0, total_promotions: 0, total_demotions: 0, total_evictions: 0 }
    }

    pub fn put(&mut self, key: u64, value: V) {
        self.total_puts += 1;
        if self.warm.contains(key) {
            self.warm.remove(key);
        }
        if let Some((evicted_k, evicted_v)) = self.hot.put(key, value) {
            self.total_evictions += 1;
            self.warm.put(evicted_k, evicted_v);
        }
    }

    pub fn get(&mut self, key: u64) -> Option<V> {
        self.total_gets += 1;
        if self.hot.contains(key) {
            self.total_hits += 1;
            return self.hot.get(key);
        }
        if self.warm.contains(key) {
            let v = self.warm.remove(key).unwrap();
            self.total_hits += 1;
            self.total_promotions += 1;
            if let Some((evicted_k, evicted_v)) = self.hot.put(key, v.clone()) {
                self.total_evictions += 1;
                self.warm.put(evicted_k, evicted_v);
            }
            return Some(v);
        }
        self.total_misses += 1;
        None
    }

    pub fn remove(&mut self, key: u64) -> Option<V> {
        if self.hot.contains(key) { return self.hot.remove(key); }
        self.warm.remove(key)
    }

    pub fn hot_count(&self) -> usize { self.hot.len() }
    pub fn warm_count(&self) -> usize { self.warm.len() }
    pub fn total_count(&self) -> usize { self.hot.len() + self.warm.len() }
    pub fn hot_capacity(&self) -> usize { self.hot.capacity() }
    pub fn warm_capacity(&self) -> usize { self.warm.capacity() }
    pub fn hit_rate(&self) -> f64 { if self.total_gets == 0 { 0.0 } else { self.total_hits as f64 / self.total_gets as f64 } }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_hits(&self) -> u64 { self.total_hits }
    pub fn total_misses(&self) -> u64 { self.total_misses }
    pub fn total_promotions(&self) -> u64 { self.total_promotions }
    pub fn total_demotions(&self) -> u64 { self.total_demotions }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache() { let c: CacheTiers<i32> = CacheTiers::new(2, 4); assert_eq!(c.total_count(), 0); }

    #[test]
    fn put_get() {
        let mut c = CacheTiers::new(2, 4);
        c.put(1, 10);
        assert_eq!(c.get(1), Some(10));
    }

    #[test]
    fn miss() {
        let mut c: CacheTiers<i32> = CacheTiers::new(2, 4);
        assert_eq!(c.get(1), None);
        assert_eq!(c.total_misses(), 1);
    }

    #[test]
    fn promotion() {
        let mut c = CacheTiers::new(2, 4);
        c.put(1, 10); c.put(2, 20); c.put(3, 30);
        let v = c.get(1);
        assert!(v.is_some());
        assert!(c.total_promotions() > 0);
    }

    #[test]
    fn eviction_to_warm() {
        let mut c = CacheTiers::new(2, 4);
        c.put(1, 10); c.put(2, 20); c.put(3, 30);
        assert!(c.total_count() >= 2);
    }

    #[test]
    fn remove() {
        let mut c = CacheTiers::new(2, 4);
        c.put(1, 10);
        assert_eq!(c.remove(1), Some(10));
        assert_eq!(c.get(1), None);
    }

    #[test]
    fn hit_rate() {
        let mut c = CacheTiers::new(2, 4);
        c.put(1, 10);
        c.get(1); c.get(2);
        let rate = c.hit_rate();
        assert!(rate > 0.0 && rate < 1.0);
    }

    #[test]
    fn overwrite() {
        let mut c = CacheTiers::new(2, 4);
        c.put(1, 10); c.put(1, 20);
        assert_eq!(c.get(1), Some(20));
    }

    #[test]
    fn stats() {
        let mut c = CacheTiers::new(2, 4);
        c.put(1, 10);
        c.get(1);
        assert_eq!(c.total_puts(), 1);
        assert_eq!(c.total_gets(), 1);
        assert_eq!(c.total_hits(), 1);
    }

    #[test]
    fn error_display() { assert!(CacheTierError::KeyNotFound { key: 3 }.to_string().contains("3")); }
}
