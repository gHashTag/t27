use std::collections::{BTreeMap, VecDeque};

pub struct Lru2 {
    cap: usize,
    order: VecDeque<u64>,
    data: BTreeMap<u64, Vec<u8>>,
    total_hits: u64,
    total_misses: u64,
    total_evictions: u64,
}

impl Lru2 {
    pub fn new(cap: usize) -> Self { Self { cap: cap.max(1), order: VecDeque::new(), data: BTreeMap::new(), total_hits: 0, total_misses: 0, total_evictions: 0 } }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        if self.data.contains_key(&key) {
            self.total_hits += 1;
            self.touch(key);
            Some(self.data.get(&key).unwrap().as_slice())
        } else { self.total_misses += 1; None }
    }

    fn touch(&mut self, key: u64) {
        self.order.retain(|&k| k != key);
        self.order.push_back(key);
    }

    pub fn put(&mut self, key: u64, value: Vec<u8>) {
        if self.data.contains_key(&key) { self.data.insert(key, value); self.touch(key); return; }
        while self.data.len() >= self.cap {
            if let Some(old) = self.order.pop_front() { self.data.remove(&old); self.total_evictions += 1; }
        }
        self.data.insert(key, value);
        self.order.push_back(key);
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn cap(&self) -> usize { self.cap }
    pub fn total_hits(&self) -> u64 { self.total_hits }
    pub fn total_misses(&self) -> u64 { self.total_misses }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get() {
        let mut l = Lru2::new(3);
        l.put(1, b"one".to_vec());
        assert_eq!(l.get(1), Some(&b"one"[..]));
    }

    #[test]
    fn miss() { let mut l = Lru2::new(3); assert!(l.get(1).is_none()); assert_eq!(l.total_misses(), 1); }

    #[test]
    fn evict_lru() {
        let mut l = Lru2::new(2);
        l.put(1, vec![]); l.put(2, vec![]);
        l.get(1);
        l.put(3, vec![]);
        assert!(l.get(2).is_none());
        assert!(l.get(1).is_some());
        assert_eq!(l.total_evictions(), 1);
    }

    #[test]
    fn overwrite() {
        let mut l = Lru2::new(2);
        l.put(1, b"old".to_vec()); l.put(1, b"new".to_vec());
        assert_eq!(l.get(1), Some(&b"new"[..]));
        assert_eq!(l.len(), 1);
    }

    #[test]
    fn access_order() {
        let mut l = Lru2::new(3);
        l.put(1, vec![]); l.put(2, vec![]); l.put(3, vec![]);
        l.get(1);
        l.put(4, vec![]);
        assert!(l.get(2).is_none());
        assert!(l.get(1).is_some());
    }

    #[test]
    fn stats() {
        let mut l = Lru2::new(2);
        l.put(1, vec![]); l.put(2, vec![]); l.get(1); l.get(3);
        assert_eq!(l.total_hits(), 1);
        assert_eq!(l.total_misses(), 1);
    }
}
