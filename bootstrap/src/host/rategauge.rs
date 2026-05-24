#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GaugeError {
    WindowOverflow { requested: usize, max: usize },
}

impl std::fmt::Display for GaugeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GaugeError::WindowOverflow { requested, max } => {
                write!(f, "window {requested} > max {max}")
            }
        }
    }
}

impl std::error::Error for GaugeError {}

const MAX_WINDOW: usize = 1024;

#[derive(Debug, Clone)]
pub struct RateGauge {
    window: Vec<u64>,
    window_size: usize,
    idx: usize,
    filled: bool,
    total_events: u64,
    total_ticks: u64,
    peak_rate: f64,
}

impl RateGauge {
    pub fn new(window_size: usize) -> Self {
        let ws = window_size.min(MAX_WINDOW).max(1);
        Self {
            window: vec![0; ws],
            window_size: ws,
            idx: 0,
            filled: false,
            total_events: 0,
            total_ticks: 0,
            peak_rate: 0.0,
        }
    }

    pub fn record(&mut self, count: u64) {
        self.window[self.idx] += count;
        self.total_events += count;
    }

    pub fn tick(&mut self) {
        let rate = self.rate();
        if rate > self.peak_rate { self.peak_rate = rate; }
        self.idx = (self.idx + 1) % self.window_size;
        if self.idx == 0 { self.filled = true; }
        self.window[self.idx] = 0;
        self.total_ticks += 1;
    }

    pub fn rate(&self) -> f64 {
        let count = if self.filled { self.window_size } else { self.idx + 1 };
        if count == 0 { return 0.0; }
        let sum: u64 = self.window.iter().take(count).sum();
        sum as f64 / count as f64
    }

    pub fn total_events(&self) -> u64 {
        self.total_events
    }

    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    pub fn peak_rate(&self) -> f64 {
        self.peak_rate
    }

    pub fn window_sum(&self) -> u64 {
        let count = if self.filled { self.window_size } else { self.idx + 1 };
        self.window.iter().take(count).sum()
    }

    pub fn current_window(&self) -> u64 {
        self.window[self.idx]
    }

    pub fn window_size(&self) -> usize {
        self.window_size
    }

    pub fn reset(&mut self) {
        self.window.fill(0);
        self.idx = 0;
        self.filled = false;
        self.total_events = 0;
        self.total_ticks = 0;
        self.peak_rate = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_gauge() {
        let rg = RateGauge::new(8);
        assert_eq!(rg.window_size(), 8);
        assert_eq!(rg.total_events(), 0);
    }

    #[test]
    fn record_and_rate() {
        let mut rg = RateGauge::new(4);
        rg.record(10);
        assert!((rg.rate() - 10.0).abs() < 0.01);
    }

    #[test]
    fn tick_advances_window() {
        let mut rg = RateGauge::new(3);
        rg.record(10);
        rg.tick();
        rg.record(20);
        rg.tick();
        rg.record(30);
        let rate = rg.rate();
        assert!((rate - 20.0).abs() < 1.0);
    }

    #[test]
    fn sliding_window() {
        let mut rg = RateGauge::new(2);
        rg.record(100);
        rg.tick();
        rg.record(0);
        rg.tick();
        rg.record(50);
        let rate = rg.rate();
        assert!((rate - 25.0).abs() < 0.01);
    }

    #[test]
    fn peak_rate() {
        let mut rg = RateGauge::new(4);
        rg.record(100);
        rg.tick();
        rg.record(1);
        rg.tick();
        assert!(rg.peak_rate() > 50.0);
    }

    #[test]
    fn total_events() {
        let mut rg = RateGauge::new(4);
        rg.record(10);
        rg.record(20);
        rg.record(30);
        assert_eq!(rg.total_events(), 60);
    }

    #[test]
    fn window_sum() {
        let mut rg = RateGauge::new(4);
        rg.record(5);
        rg.tick();
        rg.record(10);
        assert_eq!(rg.window_sum(), 15);
    }

    #[test]
    fn current_window() {
        let mut rg = RateGauge::new(4);
        rg.record(42);
        assert_eq!(rg.current_window(), 42);
    }

    #[test]
    fn reset() {
        let mut rg = RateGauge::new(4);
        rg.record(100);
        rg.tick();
        rg.reset();
        assert_eq!(rg.total_events(), 0);
        assert_eq!(rg.total_ticks(), 0);
    }

    #[test]
    fn empty_rate() {
        let rg = RateGauge::new(4);
        assert!((rg.rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn total_ticks() {
        let mut rg = RateGauge::new(4);
        rg.tick();
        rg.tick();
        rg.tick();
        assert_eq!(rg.total_ticks(), 3);
    }

    #[test]
    fn error_display() {
        let e = GaugeError::WindowOverflow { requested: 2000, max: 1024 };
        assert!(e.to_string().contains("2000"));
    }
}
