use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum WtError {
    EmptySequence,
    SymbolOutOfRange { symbol: u8, max: u8 },
    IndexOutOfRange { idx: usize, len: usize },
}

impl std::fmt::Display for WtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WtError::EmptySequence => write!(f, "empty sequence"),
            WtError::SymbolOutOfRange { symbol, max } => write!(f, "symbol {symbol} exceeds max {max}"),
            WtError::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
        }
    }
}

impl std::error::Error for WtError {}

pub struct WaveletTree {
    data: Vec<u8>,
    max_symbol: u8,
    total_ranks: u64,
    total_accesses: u64,
}

impl WaveletTree {
    pub fn new(max_symbol: u8) -> Self {
        Self { data: Vec::new(), max_symbol, total_ranks: 0, total_accesses: 0 }
    }

    pub fn build(max_symbol: u8, data: &[u8]) -> Result<Self, WtError> {
        if data.is_empty() { return Err(WtError::EmptySequence); }
        for &s in data {
            if s > max_symbol { return Err(WtError::SymbolOutOfRange { symbol: s, max: max_symbol }); }
        }
        Ok(Self { data: data.to_vec(), max_symbol, total_ranks: 0, total_accesses: 0 })
    }

    pub fn access(&mut self, idx: usize) -> Option<u8> {
        self.total_accesses += 1;
        self.data.get(idx).copied()
    }

    pub fn rank(&mut self, symbol: u8, upto: usize) -> usize {
        self.total_ranks += 1;
        self.data[..upto.min(self.data.len())].iter().filter(|&&s| s == symbol).count()
    }

    pub fn select(&mut self, symbol: u8, k: usize) -> Option<usize> {
        if k == 0 { return None; }
        let mut count = 0;
        for (i, &s) in self.data.iter().enumerate() {
            if s == symbol { count += 1; if count == k { return Some(i); } }
        }
        None
    }

    pub fn histogram(&self) -> BTreeMap<u8, usize> {
        let mut m = BTreeMap::new();
        for &s in &self.data { *m.entry(s).or_insert(0) += 1; }
        m
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn max_symbol(&self) -> u8 { self.max_symbol }
    pub fn total_ranks(&self) -> u64 { self.total_ranks }
    pub fn total_accesses(&self) -> u64 { self.total_accesses }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_basic() {
        let mut wt = WaveletTree::build(3, &[0, 1, 2, 3, 1, 0]).unwrap();
        assert_eq!(wt.len(), 6);
    }

    #[test]
    fn access() {
        let mut wt = WaveletTree::build(5, &[3, 1, 4, 1, 5]).unwrap();
        assert_eq!(wt.access(0), Some(3));
        assert_eq!(wt.access(2), Some(4));
        assert_eq!(wt.access(4), Some(5));
        assert_eq!(wt.access(5), None);
    }

    #[test]
    fn rank() {
        let mut wt = WaveletTree::build(3, &[0, 1, 2, 1, 0, 1]).unwrap();
        assert_eq!(wt.rank(1, 6), 3);
        assert_eq!(wt.rank(0, 6), 2);
        assert_eq!(wt.rank(2, 6), 1);
        assert_eq!(wt.rank(3, 6), 0);
    }

    #[test]
    fn select() {
        let mut wt = WaveletTree::build(3, &[0, 1, 2, 1, 0, 1]).unwrap();
        assert_eq!(wt.select(1, 1), Some(1));
        assert_eq!(wt.select(1, 2), Some(3));
        assert_eq!(wt.select(1, 3), Some(5));
        assert_eq!(wt.select(1, 4), None);
    }

    #[test]
    fn histogram() {
        let wt = WaveletTree::build(2, &[0, 1, 2, 1, 0]).unwrap();
        let h = wt.histogram();
        assert_eq!(h[&0], 2);
        assert_eq!(h[&1], 2);
        assert_eq!(h[&2], 1);
    }

    #[test]
    fn empty_err() { assert!(WaveletTree::build(3, &[]).is_err()); }

    #[test]
    fn symbol_err() { assert!(WaveletTree::build(2, &[0, 3]).is_err()); }

    #[test]
    fn new_empty() {
        let mut wt = WaveletTree::new(5);
        assert!(wt.is_empty());
        assert_eq!(wt.rank(0, 0), 0);
    }

    #[test]
    fn stats() {
        let mut wt = WaveletTree::build(3, &[0, 1, 2]).unwrap();
        wt.rank(1, 3); wt.access(0);
        assert_eq!(wt.total_ranks(), 1);
        assert_eq!(wt.total_accesses(), 1);
    }

    #[test]
    fn error_display() { assert!(WtError::EmptySequence.to_string().contains("empty")); }
}
