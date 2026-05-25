pub struct CuckooFilter {
    buckets: Vec<Vec<[u8; 4]>>,
    count: usize,
}

impl CuckooFilter {
    pub fn new(capacity: usize) -> Self {
        let num_buckets = (capacity / 4).max(1);
        Self { buckets: vec![vec![]; num_buckets], count: 0 }
    }

    fn hash(item: &[u8]) -> (u64, u64) {
        let mut h1: u64 = 0xcbf29ce484222325;
        for &b in item {
            h1 ^= b as u64;
            h1 = h1.wrapping_mul(0x100000001b3);
        }
        let fp = (h1 & 0xFF) as u8;
        let mut h2: u64 = 0x100000001b3;
        h2 ^= fp as u64;
        h2 = h2.wrapping_mul(0xcbf29ce484222325);
        (h1, h2)
    }

    fn fingerprint(item: &[u8]) -> [u8; 4] {
        let (h, _) = Self::hash(item);
        [(h & 0xFF) as u8, ((h >> 8) & 0xFF) as u8, ((h >> 16) & 0xFF) as u8, ((h >> 24) & 0xFF) as u8]
    }

    fn index(&self, hash: u64) -> usize { (hash as usize) % self.buckets.len() }

    fn alt_index(&self, idx: usize, fp: [u8; 4]) -> usize {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in &fp { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
        (idx ^ (h as usize)) % self.buckets.len()
    }

    pub fn insert(&mut self, item: &[u8]) -> bool {
        let fp = Self::fingerprint(item);
        let (h1, _) = Self::hash(item);
        let i1 = self.index(h1);
        let i2 = self.alt_index(i1, fp);
        let bucket = if self.buckets[i1].len() < 4 { &mut self.buckets[i1] }
                     else if self.buckets[i2].len() < 4 { &mut self.buckets[i2] }
                     else { return false };
        bucket.push(fp);
        self.count += 1;
        true
    }

    pub fn contains(&self, item: &[u8]) -> bool {
        let fp = Self::fingerprint(item);
        let (h1, _) = Self::hash(item);
        let i1 = self.index(h1);
        let i2 = self.alt_index(i1, fp);
        self.buckets[i1].iter().any(|f| f == &fp) || self.buckets[i2].iter().any(|f| f == &fp)
    }

    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_lookup() {
        let mut cf = CuckooFilter::new(64);
        assert!(cf.insert(b"hello"));
        assert!(cf.contains(b"hello"));
    }

    #[test]
    fn absent() {
        let mut cf = CuckooFilter::new(64);
        cf.insert(b"hello");
        assert!(!cf.contains(b"world"));
    }

    #[test]
    fn count() {
        let mut cf = CuckooFilter::new(256);
        for i in 0u8..20 { assert!(cf.insert(&[i])); }
        assert_eq!(cf.len(), 20);
    }

    #[test]
    fn empty() {
        let cf = CuckooFilter::new(64);
        assert!(cf.is_empty());
        assert_eq!(cf.len(), 0);
    }

    #[test]
    fn no_false_negative_basic() {
        let mut cf = CuckooFilter::new(256);
        let items: Vec<Vec<u8>> = (0..30).map(|i| vec![i]).collect();
        for item in &items { cf.insert(item); }
        for item in &items { assert!(cf.contains(item)); }
    }
}
