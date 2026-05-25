use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LmError {
    NotFound { key: u64 },
    TableFull,
}

impl std::fmt::Display for LmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LmError::NotFound { key } => write!(f, "key {key} not found"),
            LmError::TableFull => write!(f, "table full"),
        }
    }
}

impl std::error::Error for LmError {}

#[derive(Clone)]
struct Slot {
    key: u64,
    value: Vec<u8>,
    occupied: bool,
    dist: usize,
}

pub struct LinMap {
    slots: Vec<Slot>,
    cap: usize,
    len: usize,
    total_inserts: u64,
    total_lookups: u64,
    max_dist: usize,
}

impl LinMap {
    pub fn new(cap: usize) -> Self {
        Self { slots: vec![Slot { key: 0, value: Vec::new(), occupied: false, dist: 0 }; cap], cap, len: 0, total_inserts: 0, total_lookups: 0, max_dist: 0 }
    }

    fn hash(&self, key: u64) -> usize {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in key.to_le_bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
        (h as usize) % self.cap
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) -> Result<(), LmError> {
        self.total_inserts += 1;
        let home = self.hash(key);
        let mut k = key;
        let mut v = value;
        let mut dist = 0usize;
        for _ in 0..self.cap {
            let idx = (home + dist) % self.cap;
            if !self.slots[idx].occupied {
                self.slots[idx] = Slot { key: k, value: v, occupied: true, dist };
                self.len += 1;
                if dist > self.max_dist { self.max_dist = dist; }
                return Ok(());
            }
            if self.slots[idx].key == k {
                self.slots[idx].value = v;
                return Ok(());
            }
            if self.slots[idx].dist < dist {
                let old_key = self.slots[idx].key;
                let old_val = std::mem::replace(&mut self.slots[idx].value, Vec::new());
                let old_dist = self.slots[idx].dist;
                self.slots[idx] = Slot { key: k, value: v, occupied: true, dist };
                if dist > self.max_dist { self.max_dist = dist; }
                k = old_key; v = old_val; dist = old_dist;
            }
            dist += 1;
        }
        Err(LmError::TableFull)
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let home = self.hash(key);
        for dist in 0..=self.max_dist {
            let idx = (home + dist) % self.cap;
            if !self.slots[idx].occupied { return None; }
            if self.slots[idx].key == key { return Some(&self.slots[idx].value); }
        }
        None
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, LmError> {
        let home = self.hash(key);
        let mut found_idx: Option<usize> = None;
        for dist in 0..=self.max_dist {
            let idx = (home + dist) % self.cap;
            if !self.slots[idx].occupied { break; }
            if self.slots[idx].key == key { found_idx = Some(idx); break; }
        }
        let mut idx = found_idx.ok_or(LmError::NotFound { key })?;
        self.len -= 1;
        loop {
            let next = (idx + 1) % self.cap;
            if !self.slots[next].occupied || self.slots[next].dist == 0 { break; }
            self.slots[idx] = self.slots[next].clone();
            self.slots[idx].dist -= 1;
            idx = next;
        }
        let val = std::mem::take(&mut self.slots[idx].value);
        self.slots[idx] = Slot { key: 0, value: Vec::new(), occupied: false, dist: 0 };
        Ok(val)
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn max_displacement(&self) -> usize { self.max_dist }
    pub fn load_factor(&self) -> f64 { self.len as f64 / self.cap as f64 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lm() { let lm = LinMap::new(16); assert!(lm.is_empty()); }

    #[test]
    fn insert_get() {
        let mut lm = LinMap::new(16);
        lm.insert(1, b"one".to_vec()).unwrap(); lm.insert(2, b"two".to_vec()).unwrap();
        assert_eq!(lm.get(1), Some(&b"one"[..]));
        assert_eq!(lm.get(2), Some(&b"two"[..]));
    }

    #[test]
    fn overwrite() {
        let mut lm = LinMap::new(16);
        lm.insert(1, b"old".to_vec()).unwrap(); lm.insert(1, b"new".to_vec()).unwrap();
        assert_eq!(lm.get(1), Some(&b"new"[..]));
        assert_eq!(lm.len(), 1);
    }

    #[test]
    fn remove() {
        let mut lm = LinMap::new(16);
        lm.insert(1, b"a".to_vec()).unwrap();
        lm.remove(1).unwrap();
        assert!(!lm.contains(1));
    }

    #[test]
    fn remove_not_found() { assert!(LinMap::new(16).remove(1).is_err()); }

    #[test]
    fn many() {
        let mut lm = LinMap::new(64);
        for i in 0..50u64 { lm.insert(i, vec![i as u8]).unwrap(); }
        assert_eq!(lm.len(), 50);
        for i in 0..50u64 { assert!(lm.contains(i)); }
    }

    #[test]
    fn displacement() {
        let mut lm = LinMap::new(16);
        for i in 0..10 { lm.insert(i, vec![]).unwrap(); }
        assert!(lm.max_displacement() > 0);
    }

    #[test]
    fn load_factor() {
        let mut lm = LinMap::new(10);
        for i in 0..5 { lm.insert(i, vec![]).unwrap(); }
        assert!(lm.load_factor() >= 0.5);
    }

    #[test]
    fn stats() {
        let mut lm = LinMap::new(16);
        lm.insert(1, vec![]).unwrap(); lm.get(1);
        assert_eq!(lm.total_inserts(), 1);
        assert_eq!(lm.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(LmError::TableFull.to_string().contains("full")); }
}
