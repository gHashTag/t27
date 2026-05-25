use std::collections::BTreeMap;

const BUCKET_W: usize = 4;
const MAX_KICKS: usize = 256;

fn h1(k: u64, n: usize) -> usize { (k.wrapping_mul(0x9e3779b97f4a7c15) as usize) % n }
fn h2(k: u64, n: usize) -> usize { (k.wrapping_mul(0xcbf29ce484222325) as usize) % n }

#[derive(Clone)]
struct Slot { key: u64, value: Vec<u8>, occupied: bool }
struct Bucket { slots: [Slot; BUCKET_W], count: usize }

impl Bucket { fn new() -> Self { Self { slots: std::array::from_fn(|_| Slot { key: 0, value: Vec::new(), occupied: false }), count: 0 } } }

#[derive(Debug, Clone, PartialEq)]
pub enum Ch2Err { Full, NotFound(u64), }

impl std::fmt::Display for Ch2Err {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Ch2Err::Full => write!(f, "table full"), Ch2Err::NotFound(k) => write!(f, "key {k} not found") }
    }
}

impl std::error::Error for Ch2Err {}

pub struct CuckooHash2 {
    buckets: Vec<Bucket>,
    stash: BTreeMap<u64, Vec<u8>>,
    num_buckets: usize,
    len: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_kicks: u64,
}

impl CuckooHash2 {
    pub fn new(capacity: usize) -> Self {
        let nb = (capacity + BUCKET_W - 1) / BUCKET_W;
        let nb = nb.next_power_of_two().max(2);
        Self { buckets: (0..nb).map(|_| Bucket::new()).collect(), stash: BTreeMap::new(), num_buckets: nb, len: 0, total_inserts: 0, total_lookups: 0, total_kicks: 0 }
    }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) -> Result<(), Ch2Err> {
        self.total_inserts += 1;
        let i1 = h1(key, self.num_buckets);
        let i2 = h2(key, self.num_buckets);
        if let Some(is_new) = self.put_in_bucket(i1, key, value.clone()) {
            if is_new { self.len += 1; }
            return Ok(());
        }
        if let Some(is_new) = self.put_in_bucket(i2, key, value.clone()) {
            if is_new { self.len += 1; }
            return Ok(());
        }
        let mut cur_key = key;
        let mut cur_val = value;
        let mut idx = if key % 2 == 0 { i1 } else { i2 };
        for _ in 0..MAX_KICKS {
            self.total_kicks += 1;
            let evict_slot = (cur_key.wrapping_add(idx as u64)) as usize % BUCKET_W;
            let evicted_key = self.buckets[idx].slots[evict_slot].key;
            let evicted_val = std::mem::replace(&mut self.buckets[idx].slots[evict_slot].value, cur_val);
            let was_occupied = self.buckets[idx].slots[evict_slot].occupied;
            self.buckets[idx].slots[evict_slot].key = cur_key;
            self.buckets[idx].slots[evict_slot].occupied = true;
            if !was_occupied { self.buckets[idx].count += 1; self.len += 1; return Ok(()); }
            cur_key = evicted_key;
            cur_val = evicted_val;
            let alt = if idx == h1(evicted_key, self.num_buckets) { h2(evicted_key, self.num_buckets) } else { h1(evicted_key, self.num_buckets) };
            if let Some(is_new) = self.put_in_bucket(alt, cur_key, cur_val.clone()) {
                if is_new { self.len += 1; }
                return Ok(());
            }
            idx = alt;
        }
        self.stash.insert(cur_key, cur_val);
        self.len += 1;
        Ok(())
    }

    fn put_in_bucket(&mut self, bi: usize, key: u64, value: Vec<u8>) -> Option<bool> {
        let b = &mut self.buckets[bi];
        for i in 0..b.count { if b.slots[i].key == key { b.slots[i].value = value; return Some(false); } }
        if b.count < BUCKET_W { b.slots[b.count] = Slot { key, value, occupied: true }; b.count += 1; return Some(true); }
        None
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        if let Some(v) = self.stash.get(&key) { return Some(v.as_slice()); }
        for &bi in &[h1(key, self.num_buckets), h2(key, self.num_buckets)] {
            let b = &self.buckets[bi];
            for i in 0..b.count { if b.slots[i].key == key { return Some(b.slots[i].value.as_slice()); } }
        }
        None
    }

    pub fn remove(&mut self, key: u64) -> Result<Vec<u8>, Ch2Err> {
        if let Some(v) = self.stash.remove(&key) { self.len -= 1; return Ok(v); }
        for &bi in &[h1(key, self.num_buckets), h2(key, self.num_buckets)] {
            let b = &mut self.buckets[bi];
            for i in 0..b.count {
                if b.slots[i].key == key {
                    let val = std::mem::take(&mut b.slots[i].value);
                    b.count -= 1;
                    if i < b.count { b.slots[i] = b.slots[b.count].clone(); }
                    b.slots[b.count] = Slot { key: 0, value: Vec::new(), occupied: false };
                    self.len -= 1;
                    return Ok(val);
                }
            }
        }
        Err(Ch2Err::NotFound(key))
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn stash_len(&self) -> usize { self.stash.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_kicks(&self) -> u64 { self.total_kicks }
    pub fn load_factor(&self) -> f64 { self.len as f64 / (self.num_buckets * BUCKET_W) as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut ch = CuckooHash2::new(64);
        ch.insert(1, b"one".to_vec()).unwrap();
        assert_eq!(ch.get(1), Some(&b"one"[..]));
    }

    #[test]
    fn remove() {
        let mut ch = CuckooHash2::new(64);
        ch.insert(1, b"v".to_vec()).unwrap();
        assert_eq!(ch.remove(1).unwrap(), b"v");
        assert!(ch.is_empty());
    }

    #[test]
    fn remove_missing() { assert!(CuckooHash2::new(64).remove(1).is_err()); }

    #[test]
    fn many() {
        let mut ch = CuckooHash2::new(256);
        for i in 0..100u64 { ch.insert(i, vec![i as u8]).unwrap(); }
        assert_eq!(ch.len(), 100);
        for i in 0..100u64 { assert!(ch.get(i).is_some()); }
    }

    #[test]
    fn overwrite() {
        let mut ch = CuckooHash2::new(64);
        ch.insert(1, b"old".to_vec()).unwrap();
        ch.insert(1, b"new".to_vec()).unwrap();
        assert_eq!(ch.get(1), Some(&b"new"[..]));
        assert_eq!(ch.len(), 1);
    }

    #[test]
    fn load_factor() {
        let mut ch = CuckooHash2::new(64);
        for i in 0..10u64 { ch.insert(i, vec![]).unwrap(); }
        assert!(ch.load_factor() > 0.0);
    }

    #[test]
    fn stash_used() {
        let mut ch = CuckooHash2::new(8);
        for i in 0..50u64 { ch.insert(i, vec![]).unwrap(); }
        assert!(ch.stash_len() > 0 || ch.total_kicks() > 0);
    }

    #[test]
    fn stats() {
        let mut ch = CuckooHash2::new(64);
        ch.insert(1, vec![]).unwrap(); ch.get(1);
        assert_eq!(ch.total_inserts(), 1);
        assert_eq!(ch.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(Ch2Err::Full.to_string().contains("full")); }
}
