use std::collections::BTreeMap;

pub struct Wavelet {
    data: Vec<u64>,
    hist: BTreeMap<u64, Vec<usize>>,
    total_rank: u64,
    total_access: u64,
}

impl Wavelet {
    pub fn new(data: Vec<u64>) -> Self {
        let mut hist = BTreeMap::new();
        for (i, &v) in data.iter().enumerate() {
            hist.entry(v).or_insert_with(Vec::new).push(i);
        }
        Self { data, hist, total_rank: 0, total_access: 0 }
    }

    pub fn rank(&mut self, val: u64, pos: usize) -> usize {
        self.total_rank += 1;
        let indices = match self.hist.get(&val) {
            Some(v) => v,
            None => return 0,
        };
        indices.iter().take_while(|&&i| i < pos).count()
    }

    pub fn access(&mut self, pos: usize) -> Option<u64> {
        self.total_access += 1;
        self.data.get(pos).copied()
    }

    pub fn quantile(&self, lo: usize, hi: usize, k: usize) -> Option<u64> {
        if hi <= lo || k >= hi - lo { return None; }
        let mut slice: Vec<u64> = self.data[lo..hi].to_vec();
        let n = slice.len();
        slice.select_nth_unstable(k.min(n - 1));
        Some(slice[k.min(n - 1)])
    }

    pub fn range_count(&mut self, val: u64, lo: usize, hi: usize) -> usize {
        self.rank(val, hi) - self.rank(val, lo)
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn total_rank(&self) -> u64 { self.total_rank }
    pub fn total_access(&self) -> u64 { self.total_access }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank() {
        let mut w = Wavelet::new(vec![3, 1, 4, 1, 5, 9, 2, 6]);
        assert_eq!(w.rank(1, 4), 2);
        assert_eq!(w.rank(9, 8), 1);
        assert_eq!(w.rank(7, 8), 0);
    }

    #[test]
    fn access() {
        let mut w = Wavelet::new(vec![10, 20, 30]);
        assert_eq!(w.access(1), Some(20));
        assert_eq!(w.access(5), None);
    }

    #[test]
    fn quantile() {
        let w = Wavelet::new(vec![5, 3, 1, 4, 2]);
        assert_eq!(w.quantile(0, 5, 0), Some(1));
        assert_eq!(w.quantile(0, 5, 4), Some(5));
    }

    #[test]
    fn range_count() {
        let mut w = Wavelet::new(vec![1, 2, 1, 3, 1]);
        assert_eq!(w.range_count(1, 1, 4), 1);
        assert_eq!(w.range_count(1, 0, 5), 3);
    }

    #[test]
    fn empty() { assert!(Wavelet::new(vec![]).is_empty()); }

    #[test]
    fn stats() {
        let mut w = Wavelet::new(vec![1, 2, 3]);
        w.rank(1, 3); w.access(0);
        assert_eq!(w.total_rank(), 1);
        assert_eq!(w.total_access(), 1);
    }
}
