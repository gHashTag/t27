pub const NUM_BINS: usize = 10;
pub const BIN_BOUNDARIES: [u64; NUM_BINS + 1] = [
    0, 100, 250, 500, 1000, 2500, 5000, 10000, 25000, 50000, u64::MAX,
];
pub const BIN_LABELS: [&str; NUM_BINS] = [
    "<100us", "100-250us", "250-500us", "0.5-1ms", "1-2.5ms",
    "2.5-5ms", "5-10ms", "10-25ms", "25-50ms", ">50ms",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Histogram {
    bins: [u64; NUM_BINS],
    count: u64,
    sum_us: u64,
    min_us: u64,
    max_us: u64,
}

impl Histogram {
    pub fn new() -> Self {
        Self {
            bins: [0; NUM_BINS],
            count: 0,
            sum_us: 0,
            min_us: u64::MAX,
            max_us: 0,
        }
    }

    pub fn record(&mut self, latency_us: u64) {
        let bin = self.find_bin(latency_us);
        self.bins[bin] += 1;
        self.count += 1;
        self.sum_us += latency_us;
        if latency_us < self.min_us {
            self.min_us = latency_us;
        }
        if latency_us > self.max_us {
            self.max_us = latency_us;
        }
    }

    fn find_bin(&self, value: u64) -> usize {
        for i in 0..NUM_BINS {
            if value < BIN_BOUNDARIES[i + 1] {
                return i;
            }
        }
        NUM_BINS - 1
    }

    pub fn bins(&self) -> &[u64; NUM_BINS] {
        &self.bins
    }

    pub fn bin(&self, index: usize) -> u64 {
        self.bins[index]
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn sum_us(&self) -> u64 {
        self.sum_us
    }

    pub fn min_us(&self) -> u64 {
        if self.count == 0 { 0 } else { self.min_us }
    }

    pub fn max_us(&self) -> u64 {
        self.max_us
    }

    pub fn mean_us(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_us as f64 / self.count as f64
        }
    }

    pub fn percentile(&self, p: f64) -> u64 {
        if self.count == 0 {
            return 0;
        }
        let target = (p / 100.0 * self.count as f64).ceil() as u64;
        let mut accumulated = 0u64;
        for i in 0..NUM_BINS {
            accumulated += self.bins[i];
            if accumulated >= target {
                return BIN_BOUNDARIES[i + 1].saturating_sub(1);
            }
        }
        self.max_us
    }

    pub fn p50(&self) -> u64 {
        self.percentile(50.0)
    }

    pub fn p99(&self) -> u64 {
        self.percentile(99.0)
    }

    pub fn merge(&mut self, other: &Histogram) {
        for i in 0..NUM_BINS {
            self.bins[i] += other.bins[i];
        }
        self.count += other.count;
        self.sum_us += other.sum_us;
        if other.count > 0 {
            if other.min_us < self.min_us {
                self.min_us = other.min_us;
            }
            if other.max_us > self.max_us {
                self.max_us = other.max_us;
            }
        }
    }

    pub fn reset(&mut self) {
        self.bins = [0; NUM_BINS];
        self.count = 0;
        self.sum_us = 0;
        self.min_us = u64::MAX;
        self.max_us = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn summary(&self) -> HistogramSummary {
        HistogramSummary {
            count: self.count,
            min_us: self.min_us(),
            max_us: self.max_us,
            mean_us: self.mean_us(),
            p50: self.p50(),
            p99: self.p99(),
        }
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistogramSummary {
    pub count: u64,
    pub min_us: u64,
    pub max_us: u64,
    pub mean_us: f64,
    pub p50: u64,
    pub p99: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_histogram() {
        let h = Histogram::new();
        assert!(h.is_empty());
        assert_eq!(h.count(), 0);
        assert_eq!(h.mean_us(), 0.0);
        assert_eq!(h.min_us(), 0);
    }

    #[test]
    fn record_single() {
        let mut h = Histogram::new();
        h.record(150);
        assert_eq!(h.count(), 1);
        assert_eq!(h.sum_us(), 150);
        assert_eq!(h.min_us(), 150);
        assert_eq!(h.max_us(), 150);
    }

    #[test]
    fn bin_placement() {
        let mut h = Histogram::new();
        h.record(50);
        h.record(150);
        h.record(600);
        h.record(50000);
        assert_eq!(h.bin(0), 1);
        assert_eq!(h.bin(1), 1);
        assert_eq!(h.bin(3), 1);
        assert_eq!(h.bin(9), 1);
    }

    #[test]
    fn min_max() {
        let mut h = Histogram::new();
        h.record(100);
        h.record(500);
        h.record(1000);
        assert_eq!(h.min_us(), 100);
        assert_eq!(h.max_us(), 1000);
    }

    #[test]
    fn mean() {
        let mut h = Histogram::new();
        h.record(100);
        h.record(300);
        assert!((h.mean_us() - 200.0).abs() < 0.001);
    }

    #[test]
    fn percentile_p50() {
        let mut h = Histogram::new();
        for _ in 0..50 {
            h.record(50);
        }
        for _ in 0..50 {
            h.record(5000);
        }
        let p50 = h.p50();
        assert!(p50 < 100);
    }

    #[test]
    fn percentile_p99() {
        let mut h = Histogram::new();
        for _ in 0..98 {
            h.record(50);
        }
        h.record(50000);
        h.record(60000);
        let p99 = h.p99();
        assert!(p99 >= 50000);
    }

    #[test]
    fn merge() {
        let mut a = Histogram::new();
        let mut b = Histogram::new();
        a.record(100);
        b.record(500);
        a.merge(&b);
        assert_eq!(a.count(), 2);
        assert_eq!(a.min_us(), 100);
        assert_eq!(a.max_us(), 500);
    }

    #[test]
    fn reset() {
        let mut h = Histogram::new();
        h.record(100);
        h.reset();
        assert!(h.is_empty());
        assert_eq!(h.count(), 0);
    }

    #[test]
    fn summary() {
        let mut h = Histogram::new();
        h.record(100);
        h.record(200);
        let s = h.summary();
        assert_eq!(s.count, 2);
        assert_eq!(s.min_us, 100);
        assert_eq!(s.max_us, 200);
        assert!((s.mean_us - 150.0).abs() < 0.001);
    }

    #[test]
    fn bin_labels_match_boundaries() {
        assert_eq!(BIN_LABELS.len(), NUM_BINS);
        assert_eq!(BIN_BOUNDARIES.len(), NUM_BINS + 1);
    }

    #[test]
    fn default_trait() {
        let h = Histogram::default();
        assert!(h.is_empty());
    }
}
