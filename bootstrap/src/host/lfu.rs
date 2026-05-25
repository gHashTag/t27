use std::collections::BTreeMap;

pub struct Lfu {
    cap: usize,
    freq: BTreeMap<u64, u64>,
    data: BTreeMap<u64, Vec<u8>>,
    min_freq: u64,
    total_hits: u64,
    total_misses: u64,
    total_evictions: u64,
}

impl Lfu {
    pub fn new(cap: usize) -> Self { Self { cap: cap.max(1), freq: BTreeMap::new(), data: BTreeMap::new(), min_freq: 0, total_hits: 0, total_misses: 0, total_evictions: 0 } }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        if self.data.contains_key(&key) {
            self.total_hits += 1;
            *self.freq.get_mut(&key).unwrap() += 1;
            Some(self.data.get(&key).unwrap().as_slice())
        } else { self.total_misses += 1; None }
    }

    pub fn put(&mut self, key: u64, value: Vec<u8>) {
        if self.cap == 0 { return; }
        if self.data.contains_key(&key) { self.data.insert(key, value); *self.freq.get_mut(&key).unwrap() += 1; return; }
        if self.data.len() >= self.cap { self.evict(); }
        self.freq.insert(key, 1);
        self.data.insert(key, value);
        self.min_freq = 1;
    }

    fn evict(&mut self) {
        let mut min_f = u64::MAX;
        let mut victim = None;
        for (&k, &f) in &self.freq {
            if f < min_f { min_f = f; victim = Some(k); }
        }
        if let Some(k) = victim {
            self.data.remove(&k);
            self.freq.remove(&k);
            self.total_evictions += 1;
        }
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn cap(&self) -> usize { self.cap }
    pub fn freq_of(&self, key: u64) -> u64 { self.freq.get(&key).copied().unwrap_or(0) }
    pub fn total_hits(&self) -> u64 { self.total_hits }
    pub fn total_misses(&self) -> u64 { self.total_misses }
    pub fn total_evictions(&self) -> u64 { self.total_evictions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get() {
        let mut l = Lfu::new(3);
        l.put(1, b"one".to_vec());
        assert_eq!(l.get(1), Some(&b"one"[..]));
    }

    #[test]
    fn miss() {
        let mut l = Lfu::new(3);
        assert!(l.get(99).is_none());
        assert_eq!(l.total_misses(), 1);
    }

    #[test]
    fn evict_lfu() {
        let mut l = Lfu::new(2);
        l.put(1, vec![]); l.put(2, vec![]);
        l.get(1); l.get(1);
        l.get(2);
        l.put(3, vec![]);
        assert!(l.get(2).is_none());
        assert_eq!(l.total_evictions(), 1);
    }

    #[test]
    fn freq_track() {
        let mut l = Lfu::new(5);
        l.put(1, vec![]);
        assert_eq!(l.freq_of(1), 1);
        l.get(1); l.get(1);
        assert_eq!(l.freq_of(1), 3);
    }

    #[test]
    fn overwrite() {
        let mut l = Lfu::new(2);
        l.put(1, b"old".to_vec()); l.put(1, b"new".to_vec());
        assert_eq!(l.get(1), Some(&b"new"[..]));
        assert_eq!(l.len(), 1);
    }

    #[test]
    fn stats() {
        let mut l = Lfu::new(2);
        l.put(1, vec![]); l.put(2, vec![]); l.put(3, vec![]);
        assert_eq!(l.total_evictions(), 1);
    }
}
