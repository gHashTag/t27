pub struct Mo {
    data: Vec<u64>,
    total_queries: u64,
}

impl Mo {
    pub fn new(data: Vec<u64>) -> Self { Self { data, total_queries: 0 } }

    pub fn range_sum(&mut self, lo: usize, hi: usize) -> u64 {
        self.total_queries += 1;
        self.data[lo..hi].iter().sum()
    }

    pub fn range_min(&mut self, lo: usize, hi: usize) -> u64 {
        self.total_queries += 1;
        *self.data[lo..hi].iter().min().unwrap_or(&0)
    }

    pub fn range_max(&mut self, lo: usize, hi: usize) -> u64 {
        self.total_queries += 1;
        *self.data[lo..hi].iter().max().unwrap_or(&0)
    }

    pub fn range_xor(&mut self, lo: usize, hi: usize) -> u64 {
        self.total_queries += 1;
        self.data[lo..hi].iter().fold(0, |a, &b| a ^ b)
    }

    pub fn frequency(&mut self, lo: usize, hi: usize, target: u64) -> usize {
        self.total_queries += 1;
        self.data[lo..hi].iter().filter(|&&v| v == target).count()
    }

    pub fn mode(&mut self, lo: usize, hi: usize) -> u64 {
        self.total_queries += 1;
        use std::collections::BTreeMap;
        let mut freq: BTreeMap<u64, usize> = BTreeMap::new();
        for &v in &self.data[lo..hi] { *freq.entry(v).or_insert(0) += 1; }
        freq.into_iter().max_by_key(|(_, c)| *c).map(|(v, _)| v).unwrap_or(0)
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> Vec<u64> { vec![3, 1, 4, 1, 5, 9, 2, 6] }

    #[test]
    fn sum() { let mut m = Mo::new(data()); assert_eq!(m.range_sum(0, 3), 8); }

    #[test]
    fn min() { let mut m = Mo::new(data()); assert_eq!(m.range_min(0, 4), 1); }

    #[test]
    fn max() { let mut m = Mo::new(data()); assert_eq!(m.range_max(4, 8), 9); }

    #[test]
    fn xor() { let mut m = Mo::new(data()); assert_eq!(m.range_xor(0, 3), 3^1^4); }

    #[test]
    fn frequency() { let mut m = Mo::new(data()); assert_eq!(m.frequency(0, 8, 1), 2); }

    #[test]
    fn mode() { let mut m = Mo::new(vec![1, 2, 1, 3, 1]); assert_eq!(m.mode(0, 5), 1); }

    #[test]
    fn stats() { let mut m = Mo::new(data()); m.range_sum(0, 4); assert_eq!(m.total_queries(), 1); }
}
