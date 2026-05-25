use std::collections::VecDeque;

pub struct StatWin {
    window: VecDeque<f64>,
    cap: usize,
    sum: f64,
    sum_sq: f64,
    min: f64,
    max: f64,
    total_pushed: u64,
    total_queries: u64,
}

impl StatWin {
    pub fn new(cap: usize) -> Self {
        Self { window: VecDeque::with_capacity(cap), cap, sum: 0.0, sum_sq: 0.0, min: f64::MAX, max: f64::MIN, total_pushed: 0, total_queries: 0 }
    }

    pub fn push(&mut self, value: f64) {
        self.total_pushed += 1;
        if self.window.len() == self.cap {
            if let Some(old) = self.window.pop_front() {
                self.sum -= old;
                self.sum_sq -= old * old;
            }
        }
        if value < self.min { self.min = value; }
        if value > self.max { self.max = value; }
        self.sum += value;
        self.sum_sq += value * value;
        self.window.push_back(value);
    }

    pub fn mean(&mut self) -> Option<f64> {
        self.total_queries += 1;
        if self.window.is_empty() { return None; }
        Some(self.sum / self.window.len() as f64)
    }

    pub fn variance(&mut self) -> Option<f64> {
        self.total_queries += 1;
        if self.window.len() < 2 { return None; }
        let m = self.sum / self.window.len() as f64;
        Some(self.sum_sq / self.window.len() as f64 - m * m)
    }

    pub fn stddev(&mut self) -> Option<f64> { self.variance().map(|v| v.sqrt()) }

    pub fn min(&self) -> Option<f64> { if self.window.is_empty() { None } else { Some(self.min) } }
    pub fn max(&self) -> Option<f64> { if self.window.is_empty() { None } else { Some(self.max) } }

    pub fn recalc_bounds(&mut self) {
        self.min = f64::MAX;
        self.max = f64::MIN;
        for &v in &self.window {
            if v < self.min { self.min = v; }
            if v > self.max { self.max = v; }
        }
    }

    pub fn len(&self) -> usize { self.window.len() }
    pub fn is_empty(&self) -> bool { self.window.is_empty() }
    pub fn cap(&self) -> usize { self.cap }
    pub fn sum(&self) -> f64 { self.sum }
    pub fn total_pushed(&self) -> u64 { self.total_pushed }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_mean() {
        let mut sw = StatWin::new(5);
        sw.push(2.0); sw.push(4.0); sw.push(6.0);
        assert!((sw.mean().unwrap() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn variance() {
        let mut sw = StatWin::new(5);
        sw.push(2.0); sw.push(4.0); sw.push(6.0);
        assert!(sw.variance().unwrap() > 0.0);
    }

    #[test]
    fn stddev() {
        let mut sw = StatWin::new(5);
        sw.push(1.0); sw.push(3.0);
        assert!(sw.stddev().unwrap() > 0.0);
    }

    #[test]
    fn window_slide() {
        let mut sw = StatWin::new(3);
        sw.push(10.0); sw.push(20.0); sw.push(30.0); sw.push(40.0);
        assert!((sw.mean().unwrap() - 30.0).abs() < 1e-10);
        assert_eq!(sw.len(), 3);
    }

    #[test]
    fn min_max() {
        let mut sw = StatWin::new(5);
        sw.push(1.0); sw.push(5.0); sw.push(3.0);
        assert!((sw.min().unwrap() - 1.0).abs() < 1e-10);
        assert!((sw.max().unwrap() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn empty_stats() {
        let mut sw = StatWin::new(5);
        assert!(sw.mean().is_none());
        assert!(sw.variance().is_none());
    }

    #[test]
    fn single_variance_none() {
        let mut sw = StatWin::new(5);
        sw.push(1.0);
        assert!(sw.variance().is_none());
    }

    #[test]
    fn bounds_recalc_after_slide() {
        let mut sw = StatWin::new(2);
        sw.push(100.0); sw.push(1.0);
        assert!((sw.min().unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn stats() {
        let mut sw = StatWin::new(5);
        sw.push(1.0);
        sw.mean();
        assert_eq!(sw.total_pushed(), 1);
        assert_eq!(sw.total_queries(), 1);
    }
}
