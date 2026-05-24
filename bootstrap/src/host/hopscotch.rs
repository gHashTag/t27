use std::collections::BTreeMap;

const EMPTY: u8 = 0;
const OCCUPIED: u8 = 1;
const HOP_RANGE: usize = 32;

#[derive(Debug, Clone, PartialEq)]
pub enum HhError {
    KeyExists { key: u64 },
    KeyNotFound { key: u64 },
    TableFull,
}

impl std::fmt::Display for HhError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HhError::KeyExists { key } => write!(f, "key {key} exists"),
            HhError::KeyNotFound { key } => write!(f, "key {key} not found"),
            HhError::TableFull => write!(f, "table full"),
        }
    }
}

impl std::error::Error for HhError {}

struct Slot {
    key: u64,
    value: Vec<u8>,
    state: u8,
    hop_info: u32,
}

pub struct Hopscotch {
    slots: Vec<Slot>,
    capacity: usize,
    count: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_removes: u64,
    total_swaps: u64,
}

impl Hopscotch {
    pub fn new(capacity: usize) -> Self {
        Self { slots: (0..capacity).map(|_| Slot { key: 0, value: Vec::new(), state: EMPTY, hop_info: 0 }).collect(), capacity, count: 0, total_inserts: 0, total_lookups: 0, total_removes: 0, total_swaps: 0 }
    }

    fn hash(&self, key: u64) -> usize {
        let mut h = key;
        h ^= h >> 33; h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33; h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        (h as usize) % self.capacity
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) -> Result<(), HhError> {
        if self.count >= self.capacity { return Err(HhError::TableFull); }
        let home = self.hash(key);
        for i in 0..HOP_RANGE {
            let idx = (home + i) % self.capacity;
            if self.slots[idx].state == OCCUPIED && self.slots[idx].key == key { return Err(HhError::KeyExists { key }); }
        }
        let mut free = None;
        for i in 0..self.capacity {
            let idx = (home + i) % self.capacity;
            if self.slots[idx].state == EMPTY { free = Some(idx); break; }
        }
        let mut free_idx = free.ok_or(HhError::TableFull)?;
        while free_idx >= home + HOP_RANGE || (free_idx < home && free_idx + self.capacity >= home + HOP_RANGE) {
            let dist = if free_idx >= home { free_idx - home } else { free_idx + self.capacity - home };
            if dist < HOP_RANGE { break; }
            let mut swapped = false;
            for j in (1..HOP_RANGE).rev() {
                let cand = if free_idx >= j { free_idx - j } else { free_idx + self.capacity - j };
                let cand_home = self.hash(self.slots[cand].key);
                let cand_dist = if free_idx >= cand_home { free_idx - cand_home } else { free_idx + self.capacity - cand_home };
                if cand_dist < HOP_RANGE && self.slots[cand].state == OCCUPIED {
                    self.slots[free_idx].key = self.slots[cand].key;
                    self.slots[free_idx].value = std::mem::take(&mut self.slots[cand].value);
                    self.slots[free_idx].state = OCCUPIED;
                    self.slots[free_idx].hop_info = 0;
                    self.slots[cand].state = EMPTY;
                    self.total_swaps += 1;
                    let h = self.hash(self.slots[free_idx].key);
                    self.slots[h].hop_info |= 1 << (if free_idx >= h { free_idx - h } else { free_idx + self.capacity - h });
                    self.slots[h].hop_info &= !(1 << (if cand >= h { cand - h } else { cand + self.capacity - h }));
                    free_idx = cand;
                    swapped = true;
                    break;
                }
            }
            if !swapped { return Err(HhError::TableFull); }
        }
        self.slots[free_idx].key = key;
        self.slots[free_idx].value = value;
        self.slots[free_idx].state = OCCUPIED;
        self.slots[home].hop_info |= 1 << (if free_idx >= home { free_idx - home } else { free_idx + self.capacity - home });
        self.count += 1;
        self.total_inserts += 1;
        Ok(())
    }

    pub fn get(&mut self, key: u64) -> Option<&Vec<u8>> {
        self.total_lookups += 1;
        let home = self.hash(key);
        for i in 0..HOP_RANGE {
            let idx = (home + i) % self.capacity;
            if self.slots[idx].state == OCCUPIED && self.slots[idx].key == key { return Some(&self.slots[idx].value); }
        }
        None
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, HhError> {
        let home = self.hash(key);
        for i in 0..HOP_RANGE {
            let idx = (home + i) % self.capacity;
            if self.slots[idx].state == OCCUPIED && self.slots[idx].key == key {
                self.slots[idx].state = EMPTY;
                self.slots[home].hop_info &= !(1 << i);
                self.count -= 1;
                self.total_removes += 1;
                return Ok(std::mem::take(&mut self.slots[idx].value));
            }
        }
        Err(HhError::KeyNotFound { key })
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_swaps(&self) -> u64 { self.total_swaps }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_table() { let h = Hopscotch::new(64); assert!(h.is_empty()); assert_eq!(h.capacity(), 64); }

    #[test]
    fn insert_get() {
        let mut h = Hopscotch::new(64);
        h.insert(1, b"val".to_vec()).unwrap();
        assert_eq!(h.get(1), Some(&b"val".to_vec()));
    }

    #[test]
    fn duplicate() {
        let mut h = Hopscotch::new(64);
        h.insert(1, b"a".to_vec()).unwrap();
        let err = h.insert(1, b"b".to_vec()).unwrap_err();
        assert!(matches!(err, HhError::KeyExists { .. }));
    }

    #[test]
    fn remove() {
        let mut h = Hopscotch::new(64);
        h.insert(1, b"val".to_vec()).unwrap();
        let v = h.remove(1).unwrap();
        assert_eq!(v, b"val");
        assert!(h.is_empty());
    }

    #[test]
    fn remove_missing() {
        let mut h = Hopscotch::new(64);
        let err = h.remove(99).unwrap_err();
        assert!(matches!(err, HhError::KeyNotFound { .. }));
    }

    #[test]
    fn many_items() {
        let mut h = Hopscotch::new(256);
        for i in 0..100 { h.insert(i, vec![i as u8]).unwrap(); }
        for i in 0..100 { assert!(h.contains(i)); }
    }

    #[test]
    fn contains() {
        let mut h = Hopscotch::new(64);
        h.insert(42, b"x".to_vec()).unwrap();
        assert!(h.contains(42));
        assert!(!h.contains(43));
    }

    #[test]
    fn get_missing() {
        let mut h = Hopscotch::new(64);
        assert_eq!(h.get(99), None);
    }

    #[test]
    fn stats() {
        let mut h = Hopscotch::new(64);
        h.insert(1, b"x".to_vec()).unwrap();
        h.get(1);
        assert_eq!(h.total_inserts(), 1);
        assert_eq!(h.total_lookups(), 1);
    }

    #[test]
    fn len() {
        let mut h = Hopscotch::new(64);
        h.insert(1, vec![]).unwrap(); h.insert(2, vec![]).unwrap();
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn error_display() { assert!(HhError::TableFull.to_string().contains("full")); }
}
