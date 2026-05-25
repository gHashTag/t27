const HASH_BITS: usize = 64;

fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

pub struct SimHash {
    total_computed: u64,
    total_comparisons: u64,
}

impl SimHash {
    pub fn new() -> Self { Self { total_computed: 0, total_comparisons: 0 } }

    pub fn compute(&mut self, features: &[Vec<u8>]) -> u64 {
        self.total_computed += 1;
        let mut counts = [0i32; HASH_BITS];
        for feat in features {
            let h = fnv1a(feat);
            for i in 0..HASH_BITS {
                if h & (1u64 << i) != 0 { counts[i] += 1; } else { counts[i] -= 1; }
            }
        }
        let mut result = 0u64;
        for i in 0..HASH_BITS { if counts[i] > 0 { result |= 1u64 << i; } }
        result
    }

    pub fn hamming_distance(&mut self, a: u64, b: u64) -> u32 {
        self.total_comparisons += 1;
        (a ^ b).count_ones()
    }

    pub fn similarity(&mut self, a: u64, b: u64) -> f64 {
        self.total_comparisons += 1;
        let dist = (a ^ b).count_ones();
        1.0 - dist as f64 / HASH_BITS as f64
    }

    pub fn is_near_duplicate(&mut self, a: u64, b: u64, threshold: u32) -> bool {
        self.hamming_distance(a, b) <= threshold
    }

    pub fn bulk_compare(&mut self, query: u64, candidates: &[(u64, Vec<u8>)], threshold: u32) -> Vec<(u64, u32)> {
        self.total_comparisons += candidates.len() as u64;
        candidates.iter().filter_map(|&(hash, _)| {
            let d = (query ^ hash).count_ones();
            if d <= threshold { Some((hash, d)) } else { None }
        }).collect()
    }

    pub fn total_computed(&self) -> u64 { self.total_computed }
    pub fn total_comparisons(&self) -> u64 { self.total_comparisons }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical() {
        let mut sh = SimHash::new();
        let h1 = sh.compute(&[b"hello".to_vec(), b"world".to_vec()]);
        let h2 = sh.compute(&[b"hello".to_vec(), b"world".to_vec()]);
        assert_eq!(sh.hamming_distance(h1, h2), 0);
    }

    #[test]
    fn different() {
        let mut sh = SimHash::new();
        let h1 = sh.compute(&[b"aaa".to_vec()]);
        let h2 = sh.compute(&[b"zzz".to_vec()]);
        assert!(sh.hamming_distance(h1, h2) > 0);
    }

    #[test]
    fn near_duplicate() {
        let mut sh = SimHash::new();
        let h1 = sh.compute(&[b"the quick brown fox".to_vec()]);
        let h2 = sh.compute(&[b"the quick brown fox".to_vec(), b"jumps".to_vec()]);
        assert!(sh.similarity(h1, h2) > 0.5);
    }

    #[test]
    fn similarity_range() {
        let mut sh = SimHash::new();
        let sim = sh.similarity(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bulk() {
        let mut sh = SimHash::new();
        let h = sh.compute(&[b"test".to_vec()]);
        let cands = vec![(h, vec![]), (0u64, vec![])];
        let near = sh.bulk_compare(h, &cands, 3);
        assert!(near.len() >= 1);
    }

    #[test]
    fn is_near_dup() {
        let mut sh = SimHash::new();
        assert!(sh.is_near_duplicate(0b1111, 0b1111, 0));
        assert!(!sh.is_near_duplicate(0b1111, 0b0000, 1));
    }

    #[test]
    fn stats() {
        let mut sh = SimHash::new();
        sh.compute(&[b"x".to_vec()]); sh.hamming_distance(0, 0);
        assert_eq!(sh.total_computed(), 1);
        assert_eq!(sh.total_comparisons(), 1);
    }
}
