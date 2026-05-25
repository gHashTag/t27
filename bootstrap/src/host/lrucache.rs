use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LruError {
    NotFound { key: u64 },
}

impl std::fmt::Display for LruError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LruError::NotFound { key } => write!(f, "key {key} not found"),
        }
    }
}

impl std::error::Error for LruError {}

struct LruNode {
    key: u64,
    value: Vec<u8>,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct LruCache {
    nodes: Vec<LruNode>,
    map: BTreeMap<u64, usize>,
    head: Option<usize>,
    tail: Option<usize>,
    free: Vec<usize>,
    cap: usize,
    total_puts: u64,
    total_gets: u64,
    total_evictions: u64,
    total_hits: u64,
    total_misses: u64,
}

impl LruCache {
    pub fn new(cap: usize) -> Self {
        Self { nodes: Vec::new(), map: BTreeMap::new(), head: None, tail: None, free: Vec::new(), cap, total_puts: 0, total_gets: 0, total_evictions: 0, total_hits: 0, total_misses: 0 }
    }

    fn alloc(&mut self, key: u64, value: Vec<u8>) -> usize {
        if let Some(idx) = self.free.pop() {
            self.nodes[idx] = LruNode { key, value, prev: None, next: None };
            return idx;
        }
        let idx = self.nodes.len();
        self.nodes.push(LruNode { key, value, prev: None, next: None });
        idx
    }

    fn detach(&mut self, idx: usize) {
        let (prev, next) = (self.nodes[idx].prev, self.nodes[idx].next);
        match prev {
            Some(p) => { self.nodes[p].next = next; }
            None => { self.head = next; }
        }
        match next {
            Some(n) => { self.nodes[n].prev = prev; }
            None => { self.tail = prev; }
        }
        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;
    }

    fn push_front(&mut self, idx: usize) {
        self.nodes[idx].next = self.head;
        self.nodes[idx].prev = None;
        if let Some(h) = self.head { self.nodes[h].prev = Some(idx); }
        self.head = Some(idx);
        if self.tail.is_none() { self.tail = Some(idx); }
    }

    fn evict_lru(&mut self) {
        let tail_idx = match self.tail {
            Some(t) => t,
            None => return,
        };
        let key = self.nodes[tail_idx].key;
        self.detach(tail_idx);
        self.map.remove(&key);
        self.free.push(tail_idx);
        self.total_evictions += 1;
    }

    pub fn put(&mut self, key: u64, value: Vec<u8>) -> Option<Vec<u8>> {
        self.total_puts += 1;
        if let Some(&idx) = self.map.get(&key) {
            let old = std::mem::replace(&mut self.nodes[idx].value, value);
            self.detach(idx);
            self.push_front(idx);
            return Some(old);
        }
        if self.map.len() >= self.cap { self.evict_lru(); }
        let idx = self.alloc(key, value);
        self.push_front(idx);
        self.map.insert(key, idx);
        None
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_gets += 1;
        let idx = *self.map.get(&key)?;
        self.total_hits += 1;
        let val_ptr = &self.nodes[idx].value as *const Vec<u8>;
        self.detach(idx);
        self.push_front(idx);
        unsafe { Some(&*val_ptr) }
    }

    pub fn peek(&self, key: u64) -> Option<&[u8]> {
        let &idx = self.map.get(&key)?;
        Some(&self.nodes[idx].value)
    }

    pub fn remove(&mut self, key: u64) -> Option<Vec<u8>> {
        let idx = self.map.remove(&key)?;
        self.detach(idx);
        let val = std::mem::take(&mut self.nodes[idx].value);
        self.free.push(idx);
        Some(val)
    }

    pub fn contains(&self, key: u64) -> bool { self.map.contains_key(&key) }
    pub fn len(&self) -> usize { self.map.len() }
    pub fn is_empty(&self) -> bool { self.map.is_empty() }
    pub fn cap(&self) -> usize { self.cap }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
    pub fn hit_rate(&self) -> f64 { if self.total_gets == 0 { 0.0 } else { self.total_hits as f64 / self.total_gets as f64 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lru() { let lru = LruCache::new(3); assert!(lru.is_empty()); assert_eq!(lru.cap(), 3); }

    #[test]
    fn put_get() {
        let mut lru = LruCache::new(5);
        lru.put(1, b"one".to_vec()); lru.put(2, b"two".to_vec());
        assert_eq!(lru.get(1), Some(&b"one"[..]));
        assert_eq!(lru.get(2), Some(&b"two"[..]));
    }

    #[test]
    fn evict() {
        let mut lru = LruCache::new(2);
        lru.put(1, b"a".to_vec()); lru.put(2, b"b".to_vec()); lru.put(3, b"c".to_vec());
        assert!(lru.contains(3)); assert!(!lru.contains(1));
        assert_eq!(lru.total_evictions(), 1);
    }

    #[test]
    fn update_moves_front() {
        let mut lru = LruCache::new(2);
        lru.put(1, b"a".to_vec()); lru.put(2, b"b".to_vec());
        lru.get(1);
        lru.put(3, b"c".to_vec());
        assert!(lru.contains(1)); assert!(!lru.contains(2));
    }

    #[test]
    fn peek_no_promote() {
        let mut lru = LruCache::new(2);
        lru.put(1, b"a".to_vec()); lru.put(2, b"b".to_vec());
        assert_eq!(lru.peek(1), Some(&b"a"[..]));
        lru.put(3, b"c".to_vec());
        assert!(!lru.contains(1));
    }

    #[test]
    fn remove() {
        let mut lru = LruCache::new(5);
        lru.put(1, b"a".to_vec());
        let v = lru.remove(1).unwrap();
        assert_eq!(v, b"a".to_vec());
        assert!(!lru.contains(1));
    }

    #[test]
    fn overwrite() {
        let mut lru = LruCache::new(5);
        let old = lru.put(1, b"old".to_vec());
        assert!(old.is_none());
        let old = lru.put(1, b"new".to_vec());
        assert_eq!(old, Some(b"old".to_vec()));
    }

    #[test]
    fn hit_rate() {
        let mut lru = LruCache::new(5);
        lru.put(1, b"a".to_vec()); lru.put(2, b"b".to_vec());
        lru.get(1); lru.get(2); lru.get(3);
        assert!(lru.hit_rate() > 0.5);
    }

    #[test]
    fn stats() {
        let mut lru = LruCache::new(5);
        lru.put(1, vec![]); lru.get(1);
        assert_eq!(lru.total_puts(), 1);
        assert_eq!(lru.total_gets(), 1);
    }

    #[test]
    fn error_display() { assert!(LruError::NotFound { key: 1 }.to_string().contains("not found")); }
}
