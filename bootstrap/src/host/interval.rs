const DEFAULT_WINDOW: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalError {
    WindowFull { capacity: usize },
}

impl std::fmt::Display for IntervalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntervalError::WindowFull { capacity } => write!(f, "window full ({capacity})"),
        }
    }
}

impl std::error::Error for IntervalError {}

#[derive(Debug, Clone)]
pub struct IntervalTracker {
    samples: Vec<u64>,
    window: usize,
    total_count: u64,
    total_sum: u64,
    min: u64,
    max: u64,
}

impl IntervalTracker {
    pub fn new(window: usize) -> Self {
        Self {
            samples: Vec::with_capacity(window),
            window: window.max(1),
            total_count: 0,
            total_sum: 0,
            min: u64::MAX,
            max: 0,
        }
    }

    pub fn record(&mut self, value: u64) {
        if self.samples.len() >= self.window {
            self.samples.remove(0);
        }
        self.samples.push(value);
        self.total_count += 1;
        self.total_sum += value;
        if value < self.min { self.min = value; }
        if value > self.max { self.max = value; }
    }

    pub fn count(&self) -> usize {
        self.samples.len()
    }

    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn min(&self) -> u64 {
        if self.samples.is_empty() { 0 } else { self.min }
    }

    pub fn max(&self) -> u64 {
        self.max
    }

    pub fn avg(&self) -> f64 {
        if self.samples.is_empty() { 0.0 } else { self.samples.iter().sum::<u64>() as f64 / self.samples.len() as f64 }
    }

    pub fn total_avg(&self) -> f64 {
        if self.total_count == 0 { 0.0 } else { self.total_sum as f64 / self.total_count as f64 }
    }

    pub fn percentile(&self, p: f64) -> u64 {
        if self.samples.is_empty() { return 0; }
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    pub fn p50(&self) -> u64 { self.percentile(50.0) }
    pub fn p90(&self) -> u64 { self.percentile(90.0) }
    pub fn p99(&self) -> u64 { self.percentile(99.0) }

    pub fn stddev(&self) -> f64 {
        if self.samples.len() < 2 { return 0.0; }
        let mean = self.avg();
        let variance = self.samples.iter()
            .map(|&v| { let d = v as f64 - mean; d * d })
            .sum::<f64>() / (self.samples.len() - 1) as f64;
        variance.sqrt()
    }

    pub fn range(&self) -> u64 {
        self.max.saturating_sub(self.min())
    }

    pub fn reset(&mut self) {
        self.samples.clear();
        self.total_count = 0;
        self.total_sum = 0;
        self.min = u64::MAX;
        self.max = 0;
    }

    pub fn window(&self) -> usize {
        self.window
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker() {
        let t = IntervalTracker::new(100);
        assert_eq!(t.window(), 100);
        assert!(t.is_empty());
    }

    #[test]
    fn record_and_count() {
        let mut t = IntervalTracker::new(10);
        t.record(5);
        t.record(10);
        t.record(15);
        assert_eq!(t.count(), 3);
        assert_eq!(t.total_count(), 3);
    }

    #[test]
    fn min_max() {
        let mut t = IntervalTracker::new(10);
        t.record(5);
        t.record(10);
        t.record(15);
        assert_eq!(t.min(), 5);
        assert_eq!(t.max(), 15);
    }

    #[test]
    fn avg() {
        let mut t = IntervalTracker::new(10);
        t.record(10);
        t.record(20);
        t.record(30);
        assert!((t.avg() - 20.0).abs() < 0.001);
    }

    #[test]
    fn total_avg() {
        let mut t = IntervalTracker::new(3);
        t.record(10);
        t.record(20);
        t.record(30);
        t.record(40);
        assert!((t.total_avg() - 25.0).abs() < 0.001);
    }

    #[test]
    fn rolling_window() {
        let mut t = IntervalTracker::new(3);
        t.record(10);
        t.record(20);
        t.record(30);
        t.record(40);
        assert_eq!(t.count(), 3);
        assert!((t.avg() - 30.0).abs() < 0.001);
    }

    #[test]
    fn percentile() {
        let mut t = IntervalTracker::new(100);
        for i in 1..=100 { t.record(i); }
        assert!(t.p50() >= 49 && t.p50() <= 51, "p50={}", t.p50());
        assert!(t.p90() >= 89 && t.p90() <= 91, "p90={}", t.p90());
        assert!(t.p99() >= 98 && t.p99() <= 100, "p99={}", t.p99());
    }

    #[test]
    fn stddev() {
        let mut t = IntervalTracker::new(10);
        t.record(10);
        t.record(10);
        t.record(10);
        assert!(t.stddev() < 0.001);
    }

    #[test]
    fn range() {
        let mut t = IntervalTracker::new(10);
        t.record(5);
        t.record(20);
        assert_eq!(t.range(), 15);
    }

    #[test]
    fn empty_stats() {
        let t = IntervalTracker::new(10);
        assert_eq!(t.min(), 0);
        assert_eq!(t.max(), 0);
        assert!(t.avg() < 0.001);
        assert_eq!(t.p50(), 0);
    }

    #[test]
    fn reset() {
        let mut t = IntervalTracker::new(10);
        t.record(42);
        t.reset();
        assert!(t.is_empty());
        assert_eq!(t.total_count(), 0);
    }

    #[test]
    fn error_display() {
        let e = IntervalError::WindowFull { capacity: 64 };
        assert!(e.to_string().contains("64"));
    }
}
