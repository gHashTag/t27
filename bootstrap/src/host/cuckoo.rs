fn fnv_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum CfError {
    FilterFull,
    AlreadyExists { fp: u8 },
}

impl std::fmt::Display for CfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfError::FilterFull => write!(f, "filter full"),
            CfError::AlreadyExists { fp } => write!(f, "fingerprint {fp} exists"),
        }
    }
}

impl std::error::Error for CfError {}

pub struct CuckooFilter {
    buckets: Vec<Vec<u8>>,
    bucket_size: usize,
    num_buckets: usize,
    max_kicks: usize,
    total_inserts: u64,
    total_lookups: u64,
    total_deletes: u64,
    total_kicks: u64,
}

impl CuckooFilter {
    pub fn new(num_buckets: usize, bucket_size: usize, max_kicks: usize) -> Self {
        Self { buckets: vec![Vec::new(); num_buckets], bucket_size, num_buckets, max_kicks, total_inserts: 0, total_lookups: 0, total_deletes: 0, total_kicks: 0 }
    }

    fn fingerprint(&self, item: u64) -> u8 {
        let h = fnv_hash(&item.to_le_bytes());
        let fp = (h & 0xFF) as u8;
        if fp == 0 { 1 } else { fp }
    }

    fn idx1(&self, item: u64) -> usize { (fnv_hash(&item.to_le_bytes()) % self.num_buckets as u64) as usize }

    fn idx2(&self, idx1: usize, fp: u8) -> usize {
        let h = fnv_hash(&[fp]);
        (idx1 as u64 ^ h) as usize % self.num_buckets
    }

    pub fn insert(&mut self, item: u64) -> Result<(), CfError> {
        let fp = self.fingerprint(item);
        let i1 = self.idx1(item);
        let i2 = self.idx2(i1, fp);
        if self.buckets[i1].len() < self.bucket_size {
            self.buckets[i1].push(fp);
            self.total_inserts += 1;
            return Ok(());
        }
        if self.buckets[i2].len() < self.bucket_size {
            self.buckets[i2].push(fp);
            self.total_inserts += 1;
            return Ok(());
        }
        let mut rand_idx = if i1 > i2 { i1 } else { i2 };
        let mut evicted_fp = fp;
        for _ in 0..self.max_kicks {
            let slot = 0;
            let tmp = self.buckets[rand_idx][slot];
            self.buckets[rand_idx][slot] = evicted_fp;
            evicted_fp = tmp;
            rand_idx = self.idx2(rand_idx, evicted_fp);
            self.total_kicks += 1;
            if self.buckets[rand_idx].len() < self.bucket_size {
                self.buckets[rand_idx].push(evicted_fp);
                self.total_inserts += 1;
                return Ok(());
            }
        }
        Err(CfError::FilterFull)
    }

    pub fn contains(&mut self, item: u64) -> bool {
        self.total_lookups += 1;
        let fp = self.fingerprint(item);
        let i1 = self.idx1(item);
        let i2 = self.idx2(i1, fp);
        self.buckets[i1].contains(&fp) || self.buckets[i2].contains(&fp)
    }

    pub fn delete(&mut self, item: u64) -> bool {
        self.total_deletes += 1;
        let fp = self.fingerprint(item);
        let i1 = self.idx1(item);
        let i2 = self.idx2(i1, fp);
        if let Some(pos) = self.buckets[i1].iter().position(|&f| f == fp) {
            self.buckets[i1].remove(pos);
            return true;
        }
        if let Some(pos) = self.buckets[i2].iter().position(|&f| f == fp) {
            self.buckets[i2].remove(pos);
            return true;
        }
        false
    }

    pub fn len(&self) -> usize { self.buckets.iter().map(|b| b.len()).sum() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn capacity(&self) -> usize { self.num_buckets * self.bucket_size }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_kicks(&self) -> u64 { self.total_kicks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_filter() { let f = CuckooFilter::new(16, 4, 50); assert!(f.is_empty()); }

    #[test]
    fn insert_contains() {
        let mut f = CuckooFilter::new(64, 4, 50);
        f.insert(42).unwrap();
        assert!(f.contains(42));
    }

    #[test]
    fn not_present() {
        let mut f = CuckooFilter::new(64, 4, 50);
        assert!(!f.contains(42));
    }

    #[test]
    fn delete() {
        let mut f = CuckooFilter::new(64, 4, 50);
        f.insert(42).unwrap();
        assert!(f.delete(42));
        assert!(!f.contains(42));
    }

    #[test]
    fn delete_missing() {
        let mut f = CuckooFilter::new(64, 4, 50);
        assert!(!f.delete(42));
    }

    #[test]
    fn many_items() {
        let mut f = CuckooFilter::new(256, 4, 50);
        for i in 0..100 { f.insert(i).unwrap(); }
        for i in 0..100 { assert!(f.contains(i)); }
    }

    #[test]
    fn len() {
        let mut f = CuckooFilter::new(64, 4, 50);
        f.insert(1).unwrap(); f.insert(2).unwrap();
        assert_eq!(f.len(), 2);
    }

    #[test]
    fn stats() {
        let mut f = CuckooFilter::new(64, 4, 50);
        f.insert(1).unwrap();
        f.contains(1);
        f.delete(1);
        assert_eq!(f.total_inserts(), 1);
        assert_eq!(f.total_lookups(), 1);
        assert_eq!(f.total_deletes(), 1);
    }

    #[test]
    fn capacity() {
        let f = CuckooFilter::new(16, 4, 50);
        assert_eq!(f.capacity(), 64);
    }

    #[test]
    fn error_display() { assert!(CfError::FilterFull.to_string().contains("full")); }
}
