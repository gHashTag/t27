const FINGERPRINT_BITS: u8 = 8;
const BUCKET_SIZE: usize = 4;
const MAX_KICKS: usize = 500;

fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

fn hash_index(data: &[u8], num_buckets: usize) -> usize {
    (fnv1a(data) as usize) % num_buckets
}

fn fingerprint(data: &[u8]) -> u8 {
    let h = fnv1a(data);
    let fp = (h & 0xFF) as u8;
    if fp == 0 { 1 } else { fp }
}

fn alt_index(idx: usize, fp: u8, num_buckets: usize) -> usize {
    let h = fnv1a(&[fp]);
    (idx ^ h as usize) % num_buckets
}

#[derive(Clone)]
struct Bucket {
    fps: [u8; BUCKET_SIZE],
    count: usize,
}

pub struct CuckooFilter {
    buckets: Vec<Bucket>,
    num_buckets: usize,
    count: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_deletes: u64,
}

impl CuckooFilter {
    pub fn new(capacity: usize) -> Self {
        let num_buckets = (capacity + BUCKET_SIZE - 1) / BUCKET_SIZE;
        let num_buckets = num_buckets.next_power_of_two().max(1);
        let buckets = (0..num_buckets).map(|_| Bucket { fps: [0; BUCKET_SIZE], count: 0 }).collect();
        Self { buckets, num_buckets, count: 0, total_inserts: 0, total_lookups: 0, total_deletes: 0 }
    }

    pub fn insert(&mut self, data: &[u8]) -> bool {
        self.total_inserts += 1;
        let fp = fingerprint(data);
        let i1 = hash_index(data, self.num_buckets);
        let i2 = alt_index(i1, fp, self.num_buckets);
        if self.put_fp(i1, fp) || self.put_fp(i2, fp) { self.count += 1; return true; }
        let mut idx = if fnv1a(data) % 2 == 0 { i1 } else { i2 };
        let mut fp_cur = fp;
        for _ in 0..MAX_KICKS {
            let evict_pos = (fnv1a(&[fp_cur, idx as u8]) as usize) % BUCKET_SIZE;
            let evicted_fp = self.buckets[idx].fps[evict_pos];
            self.buckets[idx].fps[evict_pos] = fp_cur;
            if evicted_fp == 0 { self.buckets[idx].count += 1; self.count += 1; return true; }
            fp_cur = evicted_fp;
            idx = alt_index(idx, fp_cur, self.num_buckets);
            if self.put_fp(idx, fp_cur) { self.count += 1; return true; }
        }
        false
    }

    fn put_fp(&mut self, idx: usize, fp: u8) -> bool {
        let b = &mut self.buckets[idx];
        if b.count < BUCKET_SIZE {
            b.fps[b.count] = fp;
            b.count += 1;
            true
        } else { false }
    }

    pub fn contains(&mut self, data: &[u8]) -> bool {
        self.total_lookups += 1;
        let fp = fingerprint(data);
        let i1 = hash_index(data, self.num_buckets);
        let i2 = alt_index(i1, fp, self.num_buckets);
        self.has_fp(i1, fp) || self.has_fp(i2, fp)
    }

    fn has_fp(&self, idx: usize, fp: u8) -> bool {
        let b = &self.buckets[idx];
        for i in 0..b.count { if b.fps[i] == fp { return true; } }
        false
    }

    pub fn delete(&mut self, data: &[u8]) -> bool {
        self.total_deletes += 1;
        let fp = fingerprint(data);
        let i1 = hash_index(data, self.num_buckets);
        let i2 = alt_index(i1, fp, self.num_buckets);
        if self.remove_fp(i1, fp) || self.remove_fp(i2, fp) { self.count -= 1; return true; }
        false
    }

    fn remove_fp(&mut self, idx: usize, fp: u8) -> bool {
        let b = &mut self.buckets[idx];
        for i in 0..b.count {
            if b.fps[i] == fp {
                b.count -= 1;
                b.fps[i] = b.fps[b.count];
                b.fps[b.count] = 0;
                return true;
            }
        }
        false
    }

    pub fn count(&self) -> usize { self.count }
    pub fn capacity(&self) -> usize { self.num_buckets * BUCKET_SIZE }
    pub fn load_factor(&self) -> f64 { self.count as f64 / self.capacity().max(1) as f64 }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_contains() {
        let mut cf = CuckooFilter::new(100);
        cf.insert(b"hello");
        assert!(cf.contains(b"hello"));
    }

    #[test]
    fn absent() {
        let mut cf = CuckooFilter::new(100);
        assert!(!cf.contains(b"world"));
    }

    #[test]
    fn delete() {
        let mut cf = CuckooFilter::new(100);
        cf.insert(b"hello");
        assert!(cf.delete(b"hello"));
        assert!(!cf.contains(b"hello"));
    }

    #[test]
    fn delete_absent() { assert!(!CuckooFilter::new(100).delete(b"x")); }

    #[test]
    fn many() {
        let mut cf = CuckooFilter::new(1000);
        for i in 0..200u64 { assert!(cf.insert(&i.to_le_bytes())); }
        assert_eq!(cf.count(), 200);
        for i in 0..200u64 { assert!(cf.contains(&i.to_le_bytes())); }
    }

    #[test]
    fn load_factor() {
        let mut cf = CuckooFilter::new(100);
        cf.insert(b"a"); cf.insert(b"b");
        assert!(cf.load_factor() > 0.0);
    }

    #[test]
    fn capacity() { assert!(CuckooFilter::new(100).capacity() >= 100); }

    #[test]
    fn stats() {
        let mut cf = CuckooFilter::new(100);
        cf.insert(b"x"); cf.contains(b"x"); cf.delete(b"x");
        assert_eq!(cf.total_inserts(), 1);
        assert_eq!(cf.total_lookups(), 1);
        assert_eq!(cf.total_deletes(), 1);
    }
}
