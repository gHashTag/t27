use std::collections::BTreeMap;

struct L1Bucket { entries: BTreeMap<u64, Vec<u8>>, l2: BTreeMap<u64, Vec<u8>> }

pub struct HierHash {
    l1_buckets: usize,
    buckets: Vec<L1Bucket>,
    len: usize,
    total_inserts: u64,
    total_lookups: u64,
    l2_overflows: u64,
}

impl HierHash {
    pub fn new(l1_buckets: usize) -> Self {
        Self { l1_buckets, buckets: (0..l1_buckets).map(|_| L1Bucket { entries: BTreeMap::new(), l2: BTreeMap::new() }).collect(), len: 0, total_inserts: 0, total_lookups: 0, l2_overflows: 0 }
    }

    fn bucket_idx(&self, key: u64) -> usize { (key.wrapping_mul(0x9e3779b97f4a7c15) as usize) % self.l1_buckets }

    pub fn insert(&mut self, key: u64, value: Vec<u8>) {
        self.total_inserts += 1;
        let bi = self.bucket_idx(key);
        let b = &mut self.buckets[bi];
        let is_new = !b.entries.contains_key(&key) && !b.l2.contains_key(&key);
        if b.entries.len() < 8 { b.entries.insert(key, value); }
        else { b.l2.insert(key, value); self.l2_overflows += 1; }
        if is_new { self.len += 1; }
    }

    pub fn get(&mut self, key: u64) -> Option<&[u8]> {
        self.total_lookups += 1;
        let bi = self.bucket_idx(key);
        let b = &self.buckets[bi];
        b.entries.get(&key).or_else(|| b.l2.get(&key)).map(|v| v.as_slice())
    }

    pub fn remove(&mut self, key: u64) -> Option<Vec<u8>> {
        let bi = self.bucket_idx(key);
        let b = &mut self.buckets[bi];
        if let Some(v) = b.entries.remove(&key) { self.len -= 1; return Some(v); }
        if let Some(v) = b.l2.remove(&key) { self.len -= 1; return Some(v); }
        None
    }

    pub fn contains(&mut self, key: u64) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn l2_overflow_count(&self) -> u64 { self.l2_overflows }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut hh = HierHash::new(16);
        hh.insert(1, b"v".to_vec());
        assert_eq!(hh.get(1), Some(&b"v"[..]));
    }

    #[test]
    fn remove() {
        let mut hh = HierHash::new(16);
        hh.insert(1, b"v".to_vec());
        assert_eq!(hh.remove(1), Some(b"v".to_vec()));
        assert!(hh.is_empty());
    }

    #[test]
    fn l2_overflow() {
        let mut hh = HierHash::new(1);
        for i in 0..20u64 { hh.insert(i, vec![]); }
        assert!(hh.l2_overflow_count() > 0);
    }

    #[test]
    fn l2_lookup() {
        let mut hh = HierHash::new(1);
        for i in 0..20u64 { hh.insert(i, vec![i as u8]); }
        assert_eq!(hh.get(19), Some(&[19u8][..]));
    }

    #[test]
    fn missing() { let mut hh = HierHash::new(16); assert!(hh.get(99).is_none()); }

    #[test]
    fn overwrite() {
        let mut hh = HierHash::new(16);
        hh.insert(1, b"old".to_vec()); hh.insert(1, b"new".to_vec());
        assert_eq!(hh.get(1), Some(&b"new"[..]));
        assert_eq!(hh.len(), 1);
    }

    #[test]
    fn stats() {
        let mut hh = HierHash::new(16);
        hh.insert(1, vec![]); hh.get(1);
        assert_eq!(hh.total_inserts(), 1);
        assert_eq!(hh.total_lookups(), 1);
    }
}
