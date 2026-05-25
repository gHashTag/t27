pub struct Histogram {
    bins: Vec<usize>,
    min: f64,
    max: f64,
    bin_width: f64,
    count: usize,
}

impl Histogram {
    pub fn new(min: f64, max: f64, num_bins: usize) -> Self {
        let bin_width = (max - min) / num_bins as f64;
        Self { bins: vec![0; num_bins], min, max, bin_width, count: 0 }
    }

    pub fn add(&mut self, value: f64) {
        if value < self.min || value >= self.max { return; }
        let idx = ((value - self.min) / self.bin_width) as usize;
        let idx = idx.min(self.bins.len() - 1);
        self.bins[idx] += 1;
        self.count += 1;
    }

    pub fn bins(&self) -> &[usize] { &self.bins }

    pub fn count(&self) -> usize { self.count }

    pub fn mean_bin(&self) -> Option<usize> {
        if self.count == 0 { return None; }
        let mut max_idx = 0;
        let mut max_val = 0;
        for (i, &v) in self.bins.iter().enumerate() {
            if v > max_val { max_val = v; max_idx = i; }
        }
        Some(max_idx)
    }

    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.count == 0 { return None; }
        let target = (p / 100.0 * self.count as f64).ceil() as usize;
        let mut cumulative = 0;
        for (i, &v) in self.bins.iter().enumerate() {
            cumulative += v;
            if cumulative >= target {
                return Some(self.min + (i as f64 + 0.5) * self.bin_width);
            }
        }
        Some(self.max)
    }

    pub fn entropy(&self) -> f64 {
        if self.count == 0 { return 0.0; }
        self.bins.iter().filter(|&&v| v > 0).map(|&v| {
            let p = v as f64 / self.count as f64;
            -p * p.log2()
        }).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut h = Histogram::new(0.0, 10.0, 10);
        for v in 0..10 { h.add(v as f64 + 0.5); }
        assert_eq!(h.count(), 10);
    }

    #[test]
    fn out_of_range() {
        let mut h = Histogram::new(0.0, 10.0, 10);
        h.add(-1.0); h.add(11.0);
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn mode() {
        let mut h = Histogram::new(0.0, 10.0, 10);
        h.add(5.5); h.add(5.5); h.add(5.5); h.add(1.5);
        assert_eq!(h.mean_bin(), Some(5));
    }

    #[test]
    fn percentile() {
        let mut h = Histogram::new(0.0, 100.0, 100);
        for v in 0..100 { h.add(v as f64 + 0.5); }
        let p50 = h.percentile(50.0).unwrap();
        assert!(p50 > 40.0 && p50 < 60.0);
    }

    #[test]
    fn entropy() {
        let mut h = Histogram::new(0.0, 4.0, 4);
        for _ in 0..10 { h.add(0.5); h.add(1.5); h.add(2.5); h.add(3.5); }
        let e = h.entropy();
        assert!((e - 2.0).abs() < 0.01);
    }
}
