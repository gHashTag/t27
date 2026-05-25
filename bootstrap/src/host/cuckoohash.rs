use std::collections::BTreeMap;

fn fnv_hash(seed: u64, key: u64) -> u64 {
    let mut h = seed;
    for &b in key.to_le_bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChError {
    NotFound { key: u64 },
    TableFull,
}

impl std::fmt::Display for ChError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChError::NotFound { key } => write!(f, "key {key} not found"),
            ChError::TableFull => write!(f, "table full"),
        }
    }
}

impl std::error::Error for ChError {}

const BUCKET_SIZE: usize = 4;
const MAX_KICKS: usize = 64;

struct Entry {
    key: u64,
    value: Vec<u8>,
    occupied: bool,
}

pub struct CuckooHash {
    buckets: Vec<[Entry; BUCKET_SIZE]>,
    num_buckets: usize,
    stash: BTreeMap<u64, Vec<u8>>,
    len: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_removes: u64,
    total_kicks: u64,
}

impl CuckooHash {
    pub fn new(num_buckets: usize) -> Self {
        let buckets = (0..num_buckets).map(|_| {
            std::array::from_fn(|_| Entry { key: 0, value: Vec::new(), occupied: false })
        }).collect();
        Self { buckets, num_buckets, stash: BTreeMap::new(), len: 0, total_inserts: 0, total_lookups: 0, total_removes: 0, total_kicks: 0 }
    }

    fn h1(&self, key: u64) -> usize { (fnv_hash(0xcbf29ce484222325, key) as usize) % self.num_buckets }
    fn h2(&self, key: u64) -> usize { (fnv_hash(0x9e3779b97f4a7c15, key) as usize) % self.num_buckets }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) -> Result<(), ChError> {
        self.total_inserts += 1;
        if self.stash.contains_key(&key) { self.stash.insert(key, value); return Ok(()); }
        for bucket in &[self.h1(key), self.h2(key)] {
            for slot in &mut self.buckets[*bucket] {
                if slot.occupied && slot.key == key { slot.value = value; return Ok(()); }
            }
            for slot in &mut self.buckets[*bucket] {
                if !slot.occupied { slot.key = key; slot.value = value; slot.occupied = true; self.len += 1; return Ok(()); }
            }
        }
        let mut cur_key = key;
        let mut cur_val = value;
        let mut bucket = self.h1(key);
        for _ in 0..MAX_KICKS {
            self.total_kicks += 1;
            let evict_idx = (fnv_hash(0x12345678, cur_key) as usize) % BUCKET_SIZE;
            let evicted_key = self.buckets[bucket][evict_idx].key;
            let evicted_val = std::mem::take(&mut self.buckets[bucket][evict_idx].value);
            self.buckets[bucket][evict_idx].key = cur_key;
            self.buckets[bucket][evict_idx].value = cur_val;
            cur_key = evicted_key;
            cur_val = evicted_val;
            let b1 = self.h1(cur_key);
            let b2 = self.h2(cur_key);
            for &try_bucket in &[b1, b2] {
                if try_bucket == bucket { continue; }
                for slot in &mut self.buckets[try_bucket] {
                    if !slot.occupied {
                        slot.key = cur_key; slot.value = cur_val; slot.occupied = true;
                        self.len += 1; return Ok(());
                    }
                }
            }
            bucket = if b1 == bucket { b2 } else { b1 };
        }
        self.stash.insert(cur_key, cur_val);
        self.len += 1;
        Ok(())
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        for bucket in &[self.h1(key), self.h2(key)] {
            for slot in &self.buckets[*bucket] {
                if slot.occupied && slot.key == key { return Some(&slot.value); }
            }
        }
        self.stash.get(&key).map(|v| v.as_slice())
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, ChError> {
        self.total_removes += 1;
        for bucket in &[self.h1(key), self.h2(key)] {
            for slot in &mut self.buckets[*bucket] {
                if slot.occupied && slot.key == key {
                    let val = std::mem::take(&mut slot.value);
                    slot.occupied = false;
                    self.len -= 1;
                    return Ok(val);
                }
            }
        }
        if let Some(val) = self.stash.remove(&key) { self.len -= 1; return Ok(val); }
        Err(ChError::NotFound { key })
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn stash_len(&self) -> usize { self.stash.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_kicks(&self) -> u64 { self.total_kicks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ch() { let ch = CuckooHash::new(16); assert!(ch.is_empty()); }

    #[test]
    fn insert_get() {
        let mut ch = CuckooHash::new(16);
        ch.insert(1, b"one".to_vec()).unwrap(); ch.insert(2, b"two".to_vec()).unwrap();
        assert_eq!(ch.get(1), Some(&b"one"[..]));
        assert_eq!(ch.get(2), Some(&b"two"[..]));
    }

    #[test]
    fn overwrite() {
        let mut ch = CuckooHash::new(16);
        ch.insert(1, b"old".to_vec()).unwrap(); ch.insert(1, b"new".to_vec()).unwrap();
        assert_eq!(ch.get(1), Some(&b"new"[..]));
        assert_eq!(ch.len(), 1);
    }

    #[test]
    fn remove() {
        let mut ch = CuckooHash::new(16);
        ch.insert(1, b"a".to_vec()).unwrap();
        ch.remove(1).unwrap();
        assert!(!ch.contains(1));
    }

    #[test]
    fn remove_not_found() { assert!(CuckooHash::new(16).remove(1).is_err()); }

    #[test]
    fn many() {
        let mut ch = CuckooHash::new(64);
        for i in 0..100u64 { ch.insert(i, vec![i as u8]).unwrap(); }
        assert_eq!(ch.len(), 100);
        for i in 0..100u64 { assert!(ch.contains(i)); }
    }

    #[test]
    fn kicks_occur() {
        let mut ch = CuckooHash::new(4);
        for i in 0..20u64 { ch.insert(i, vec![]).unwrap(); }
        assert!(ch.total_kicks() > 0 || ch.stash_len() > 0);
    }

    #[test]
    fn stats() {
        let mut ch = CuckooHash::new(16);
        ch.insert(1, vec![]).unwrap(); ch.get(1); ch.remove(1).unwrap();
        assert_eq!(ch.total_inserts(), 1);
        assert_eq!(ch.total_lookups(), 1);
        assert_eq!(ch.total_removes(), 1);
    }

    #[test]
    fn error_display() { assert!(ChError::TableFull.to_string().contains("full")); }
}
