pub struct BoyerMoore {
    total_queries: u64,
}

impl BoyerMoore {
    pub fn new() -> Self { Self { total_queries: 0 } }

    pub fn majority(&mut self, data: &[u64]) -> Option<u64> {
        self.total_queries += 1;
        if data.is_empty() { return None; }
        let mut candidate = data[0];
        let mut count = 1i64;
        for &v in &data[1..] {
            if count == 0 { candidate = v; count = 1; }
            else if v == candidate { count += 1; }
            else { count -= 1; }
        }
        let freq = data.iter().filter(|&&v| v == candidate).count();
        if freq > data.len() / 2 { Some(candidate) } else { None }
    }

    pub fn majority_n(&mut self, data: &[u64], n: usize) -> Vec<u64> {
        self.total_queries += 1;
        if n == 0 || data.is_empty() { return vec![]; }
        let mut candidates: Vec<(u64, i64)> = Vec::new();
        for &v in data {
            let mut found = false;
            for c in &mut candidates { if c.0 == v { c.1 += 1; found = true; break; } }
            if found { continue; }
            if candidates.len() < n - 1 { candidates.push((v, 1)); }
            else { for c in &mut candidates { c.1 -= 1; } candidates.retain(|c| c.1 > 0); }
        }
        let threshold = data.len() / n;
        candidates.iter().map(|c| c.0).filter(|&c| data.iter().filter(|&&v| v == c).count() > threshold).collect()
    }

    pub fn frequency(&mut self, data: &[u64], target: u64) -> usize {
        self.total_queries += 1;
        data.iter().filter(|&&v| v == target).count()
    }

    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majority_yes() {
        let mut bm = BoyerMoore::new();
        assert_eq!(bm.majority(&[1, 2, 1, 1, 3]), Some(1));
    }

    #[test]
    fn majority_no() {
        let mut bm = BoyerMoore::new();
        assert_eq!(bm.majority(&[1, 2, 3]), None);
    }

    #[test]
    fn empty() { assert_eq!(BoyerMoore::new().majority(&[]), None); }

    #[test]
    fn single() { assert_eq!(BoyerMoore::new().majority(&[42]), Some(42)); }

    #[test]
    fn majority_n() {
        let mut bm = BoyerMoore::new();
        let r = bm.majority_n(&[1, 1, 2, 2, 3], 3);
        assert!(r.contains(&1));
        assert!(r.contains(&2));
    }

    #[test]
    fn frequency() {
        let mut bm = BoyerMoore::new();
        assert_eq!(bm.frequency(&[1, 2, 1, 3, 1], 1), 3);
    }

    #[test]
    fn stats() {
        let mut bm = BoyerMoore::new();
        bm.majority(&[1]); bm.frequency(&[1], 1);
        assert_eq!(bm.total_queries(), 2);
    }
}
