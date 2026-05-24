const FNV_OFFSET: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn fnv1a(data: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(FNV_PRIME); }
    h
}

fn fnv1a_32(data: &[u8]) -> u32 {
    fnv1a(data) as u32
}

const BUCKET_BITS: usize = 4;
const SLOTS_PER_BUCKET: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Empty,
    Occupied { fingerprint: u32, count: u32 },
}

#[derive(Debug, Clone)]
pub struct FpSet {
    buckets: Vec<[Slot; SLOTS_PER_BUCKET]>,
    capacity: usize,
    count: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_hits: u64,
}

impl FpSet {
    pub fn new(capacity: usize) -> Self {
        let num_buckets = ((capacity + SLOTS_PER_BUCKET - 1) / SLOTS_PER_BUCKET).next_power_of_two();
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push([Slot::Empty; SLOTS_PER_BUCKET]);
        }
        Self { buckets, capacity, count: 0, total_inserts: 0, total_lookups: 0, total_hits: 0 }
    }

    fn bucket_idx(&self, fp: u32) -> usize {
        (fp as usize) & (self.buckets.len() - 1)
    }

    fn alt_bucket(&self, idx: usize, fp: u32) -> usize {
        let hash2 = fnv1a_32(&fp.to_le_bytes());
        (idx ^ (hash2 as usize)) & (self.buckets.len() - 1)
    }

    fn fingerprint(data: &[u8]) -> u32 {
        let h = fnv1a(data);
        let fp = ((h >> BUCKET_BITS) & 0xFFFF_FFFF) as u32;
        if fp == 0 { 1 } else { fp }
    }

    pub fn insert(&mut self, data: &[u8]) -> bool {
        if self.count >= self.capacity { return false; }
        let fp = Self::fingerprint(data);
        let idx1 = self.bucket_idx(fp);
        let idx2 = self.alt_bucket(idx1, fp);
        for &idx in &[idx1, idx2] {
            let bucket = &mut self.buckets[idx];
            for slot in bucket.iter_mut() {
                if let Slot::Occupied { fingerprint, .. } = slot {
                    if *fingerprint == fp {
                        if let Slot::Occupied { count, .. } = slot { *count += 1; }
                        self.total_inserts += 1;
                        return true;
                    }
                }
            }
        }
        for &idx in &[idx1, idx2] {
            let bucket = &mut self.buckets[idx];
            for slot in bucket.iter_mut() {
                if *slot == Slot::Empty {
                    *slot = Slot::Occupied { fingerprint: fp, count: 1 };
                    self.count += 1;
                    self.total_inserts += 1;
                    return true;
                }
            }
        }
        self.total_inserts += 1;
        false
    }

    pub fn contains(&mut self, data: &[u8]) -> bool {
        self.total_lookups += 1;
        let fp = Self::fingerprint(data);
        let idx1 = self.bucket_idx(fp);
        let idx2 = self.alt_bucket(idx1, fp);
        for &idx in &[idx1, idx2] {
            for slot in &self.buckets[idx] {
                if let Slot::Occupied { fingerprint, .. } = slot {
                    if *fingerprint == fp {
                        self.total_hits += 1;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn merge(&mut self, other: &FpSet) -> u64 {
        let mut added = 0u64;
        for bucket in &other.buckets {
            for slot in bucket {
                if let Slot::Occupied { fingerprint, .. } = slot {
                    let fp_bytes = fingerprint.to_le_bytes();
                    let fp = *fingerprint;
                    let idx1 = self.bucket_idx(fp);
                    let idx2 = self.alt_bucket(idx1, fp);
                    let mut found = false;
                    for &idx in &[idx1, idx2] {
                        for s in &self.buckets[idx] {
                            if let Slot::Occupied { fingerprint: f, .. } = s {
                                if *f == fp { found = true; break; }
                            }
                        }
                        if found { break; }
                    }
                    if !found {
                        for &idx in &[idx1, idx2] {
                            let bucket = &mut self.buckets[idx];
                            for s in bucket.iter_mut() {
                                if *s == Slot::Empty {
                                    *s = Slot::Occupied { fingerprint: fp, count: 1 };
                                    self.count += 1;
                                    added += 1;
                                    found = true;
                                    break;
                                }
                            }
                            if found { break; }
                        }
                    }
                    let _ = fp_bytes;
                }
            }
        }
        added
    }

    pub fn count(&self) -> usize { self.count }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 { 0.0 } else { self.total_hits as f64 / self.total_lookups as f64 }
    }
}

impl Slot {
    fn count(&self) -> u32 {
        match self { Slot::Empty => 0, Slot::Occupied { count, .. } => *count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set() {
        let s = FpSet::new(100);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn insert_contains() {
        let mut s = FpSet::new(100);
        assert!(s.insert(b"hello"));
        assert!(s.contains(b"hello"));
        assert!(!s.contains(b"world"));
    }

    #[test]
    fn multiple_inserts() {
        let mut s = FpSet::new(100);
        for i in 0..50u8 { assert!(s.insert(&[i])); }
        assert_eq!(s.count(), 50);
    }

    #[test]
    fn capacity_limit() {
        let mut s = FpSet::new(4);
        for i in 0..10u8 { let _ = s.insert(&[i]); }
        assert!(s.count() <= 4);
    }

    #[test]
    fn hit_rate() {
        let mut s = FpSet::new(100);
        s.insert(b"x");
        s.contains(b"x");
        s.contains(b"y");
        assert!(s.hit_rate() > 0.0);
    }

    #[test]
    fn merge_sets() {
        let mut s1 = FpSet::new(100);
        let mut s2 = FpSet::new(100);
        s1.insert(b"a");
        s1.insert(b"b");
        s2.insert(b"b");
        s2.insert(b"c");
        let added = s1.merge(&s2);
        assert!(s1.contains(b"a"));
        assert!(s1.contains(b"b"));
        assert!(s1.contains(b"c"));
        assert_eq!(added, 1);
    }

    #[test]
    fn duplicate_insert() {
        let mut s = FpSet::new(100);
        assert!(s.insert(b"x"));
        assert!(s.insert(b"x"));
        assert_eq!(s.total_inserts(), 2);
    }

    #[test]
    fn empty_contains() {
        let mut s = FpSet::new(100);
        assert!(!s.contains(b"anything"));
    }

    #[test]
    fn stats() {
        let mut s = FpSet::new(100);
        s.insert(b"a");
        s.insert(b"b");
        assert_eq!(s.total_inserts(), 2);
    }

    #[test]
    fn fingerprint_deterministic() {
        let fp1 = FpSet::fingerprint(b"test");
        let fp2 = FpSet::fingerprint(b"test");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn large_insert() {
        let mut s = FpSet::new(1000);
        for i in 0..500u32 {
            assert!(s.insert(&i.to_le_bytes()));
        }
        assert_eq!(s.count(), 500);
        for i in 0..500u32 {
            assert!(s.contains(&i.to_le_bytes()));
        }
    }

    #[test]
    fn lookup_count() {
        let mut s = FpSet::new(100);
        s.insert(b"x");
        s.contains(b"x");
        s.contains(b"x");
        assert_eq!(s.total_lookups(), 2);
    }
}
