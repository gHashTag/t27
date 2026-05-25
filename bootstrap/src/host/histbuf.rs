pub struct HistBuf {
    bins: Vec<u64>,
    lo: f64,
    hi: f64,
    width: f64,
    total: u64,
    underflow: u64,
    overflow: u64,
    sum: f64,
    sum_sq: f64,
}

impl HistBuf {
    pub fn new(bins: usize, lo: f64, hi: f64) -> Self {
        let bins = bins.max(1);
        let width = (hi - lo) / bins as f64;
        Self { bins: vec![0; bins], lo, hi, width, total: 0, underflow: 0, overflow: 0, sum: 0.0, sum_sq: 0.0 }
    }

    pub fn add(&mut self, val: f64) {
        self.total += 1;
        self.sum += val;
        self.sum_sq += val * val;
        if val < self.lo { self.underflow += 1; return; }
        if val >= self.hi { self.overflow += 1; return; }
        let idx = ((val - self.lo) / self.width) as usize;
        let idx = idx.min(self.bins.len() - 1);
        self.bins[idx] += 1;
    }

    pub fn bin(&self, i: usize) -> u64 { self.bins.get(i).copied().unwrap_or(0) }
    pub fn bins(&self) -> &[u64] { &self.bins }
    pub fn num_bins(&self) -> usize { self.bins.len() }
    pub fn total(&self) -> u64 { self.total }
    pub fn underflow(&self) -> u64 { self.underflow }
    pub fn overflow(&self) -> u64 { self.overflow }
    pub fn mean(&self) -> f64 { if self.total == 0 { 0.0 } else { self.sum / self.total as f64 } }
    pub fn variance(&self) -> f64 {
        if self.total == 0 { return 0.0; }
        let n = self.total as f64;
        (self.sum_sq / n) - (self.sum / n).powi(2)
    }
    pub fn max_bin(&self) -> usize {
        self.bins.iter().enumerate().max_by_key(|(_, &c)| c).map(|(i, _)| i).unwrap_or(0)
    }
    pub fn reset(&mut self) {
        for b in &mut self.bins { *b = 0; }
        self.total = 0; self.underflow = 0; self.overflow = 0; self.sum = 0.0; self.sum_sq = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut h = HistBuf::new(10, 0.0, 10.0);
        h.add(1.5); h.add(5.0); h.add(8.5);
        assert_eq!(h.total(), 3);
        assert_eq!(h.underflow(), 0);
        assert_eq!(h.overflow(), 0);
    }

    #[test]
    fn overflow_underflow() {
        let mut h = HistBuf::new(5, 0.0, 10.0);
        h.add(-1.0); h.add(11.0);
        assert_eq!(h.underflow(), 1);
        assert_eq!(h.overflow(), 1);
    }

    #[test]
    fn mean_var() {
        let mut h = HistBuf::new(10, 0.0, 100.0);
        h.add(10.0); h.add(20.0); h.add(30.0);
        assert!((h.mean() - 20.0).abs() < 1e-9);
        assert!(h.variance() > 0.0);
    }

    #[test]
    fn max_bin() {
        let mut h = HistBuf::new(5, 0.0, 5.0);
        h.add(0.5); h.add(0.5); h.add(0.5); h.add(3.5);
        assert_eq!(h.max_bin(), 0);
    }

    #[test]
    fn reset() {
        let mut h = HistBuf::new(5, 0.0, 10.0);
        h.add(5.0); h.reset();
        assert_eq!(h.total(), 0);
        assert_eq!(h.bin(2), 0);
    }

    #[test]
    fn edge_bin() {
        let mut h = HistBuf::new(2, 0.0, 10.0);
        h.add(4.9); h.add(5.0);
        assert_eq!(h.bin(0), 1);
        assert_eq!(h.bin(1), 1);
    }

    #[test]
    fn empty() {
        let h = HistBuf::new(5, 0.0, 10.0);
        assert_eq!(h.total(), 0);
        assert_eq!(h.mean(), 0.0);
    }
}
