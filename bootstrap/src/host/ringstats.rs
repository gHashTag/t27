use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingStatsError {
    WindowOverflow { max_window: usize },
}

impl std::fmt::Display for RingStatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingStatsError::WindowOverflow { max_window } => {
                write!(f, "window exceeds max {max_window}")
            }
        }
    }
}

impl std::error::Error for RingStatsError {}

#[derive(Debug, Clone)]
pub struct RingSnapshot {
    pub capacity: usize,
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub free: usize,
    pub total_push: u64,
    pub total_pop: u64,
    pub total_overflow: u64,
    pub total_underflow: u64,
}

impl RingSnapshot {
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 { 0.0 } else { self.count as f64 / self.capacity as f64 }
    }
}

#[derive(Debug, Clone)]
pub struct RingStatsCollector {
    capacity: usize,
    total_push: u64,
    total_pop: u64,
    total_overflow: u64,
    total_underflow: u64,
    max_used: usize,
    window_push: Vec<u64>,
    window_pop: Vec<u64>,
    window_idx: usize,
    window_size: usize,
}

impl RingStatsCollector {
    pub fn new(capacity: usize, window_size: usize) -> Self {
        let ws = window_size.max(1);
        Self {
            capacity,
            total_push: 0,
            total_pop: 0,
            total_overflow: 0,
            total_underflow: 0,
            max_used: 0,
            window_push: vec![0; ws],
            window_pop: vec![0; ws],
            window_idx: 0,
            window_size: ws,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn record_push(&mut self, count: usize, was_full: bool) {
        self.total_push += 1;
        if was_full { self.total_overflow += 1; }
        if count > self.max_used { self.max_used = count; }
        self.window_push[self.window_idx % self.window_size] += 1;
    }

    pub fn record_pop(&mut self, count: usize, was_empty: bool) {
        self.total_pop += 1;
        if was_empty { self.total_underflow += 1; }
        self.window_pop[self.window_idx % self.window_size] += 1;
    }

    pub fn advance_window(&mut self) {
        self.window_idx += 1;
        let slot = self.window_idx % self.window_size;
        self.window_push[slot] = 0;
        self.window_pop[slot] = 0;
    }

    pub fn snapshot(&self, head: usize, tail: usize, count: usize) -> RingSnapshot {
        RingSnapshot {
            capacity: self.capacity,
            head,
            tail,
            count,
            free: self.capacity.saturating_sub(count),
            total_push: self.total_push,
            total_pop: self.total_pop,
            total_overflow: self.total_overflow,
            total_underflow: self.total_underflow,
        }
    }

    pub fn max_used(&self) -> usize {
        self.max_used
    }

    pub fn total_push(&self) -> u64 {
        self.total_push
    }

    pub fn total_pop(&self) -> u64 {
        self.total_pop
    }

    pub fn total_overflow(&self) -> u64 {
        self.total_overflow
    }

    pub fn total_underflow(&self) -> u64 {
        self.total_underflow
    }

    pub fn window_throughput(&self) -> (u64, u64) {
        let push_sum: u64 = self.window_push.iter().sum();
        let pop_sum: u64 = self.window_pop.iter().sum();
        (push_sum, pop_sum)
    }

    pub fn throughput_ratio(&self) -> f64 {
        let (pushes, pops) = self.window_throughput();
        if pushes == 0 { 0.0 } else { pops as f64 / pushes as f64 }
    }

    pub fn reset(&mut self) {
        self.total_push = 0;
        self.total_pop = 0;
        self.total_overflow = 0;
        self.total_underflow = 0;
        self.max_used = 0;
        self.window_push.fill(0);
        self.window_pop.fill(0);
        self.window_idx = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_collector() {
        let sc = RingStatsCollector::new(64, 8);
        assert_eq!(sc.capacity(), 64);
    }

    #[test]
    fn record_push() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_push(1, false);
        assert_eq!(sc.total_push(), 1);
        assert_eq!(sc.total_overflow(), 0);
    }

    #[test]
    fn record_push_overflow() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_push(16, true);
        assert_eq!(sc.total_overflow(), 1);
    }

    #[test]
    fn record_pop() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_pop(1, false);
        assert_eq!(sc.total_pop(), 1);
        assert_eq!(sc.total_underflow(), 0);
    }

    #[test]
    fn record_pop_underflow() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_pop(0, true);
        assert_eq!(sc.total_underflow(), 1);
    }

    #[test]
    fn max_used_tracking() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_push(5, false);
        sc.record_push(10, false);
        sc.record_push(3, false);
        assert_eq!(sc.max_used(), 10);
    }

    #[test]
    fn snapshot() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_push(3, false);
        let snap = sc.snapshot(3, 0, 3);
        assert_eq!(snap.count, 3);
        assert_eq!(snap.free, 13);
        assert_eq!(snap.total_push, 1);
        assert!((snap.utilization() - 0.1875).abs() < 0.001);
    }

    #[test]
    fn window_throughput() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_push(1, false);
        sc.record_push(1, false);
        sc.record_pop(1, false);
        let (p, c) = sc.window_throughput();
        assert_eq!(p, 2);
        assert_eq!(c, 1);
    }

    #[test]
    fn advance_window() {
        let mut sc = RingStatsCollector::new(16, 2);
        sc.record_push(1, false);
        sc.advance_window();
        sc.record_push(1, false);
        let (p, _) = sc.window_throughput();
        assert_eq!(p, 2);
    }

    #[test]
    fn throughput_ratio() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_push(1, false);
        sc.record_push(1, false);
        sc.record_pop(1, false);
        assert!((sc.throughput_ratio() - 0.5).abs() < 0.001);
    }

    #[test]
    fn reset() {
        let mut sc = RingStatsCollector::new(16, 4);
        sc.record_push(5, true);
        sc.record_pop(2, false);
        sc.reset();
        assert_eq!(sc.total_push(), 0);
        assert_eq!(sc.total_pop(), 0);
        assert_eq!(sc.total_overflow(), 0);
        assert_eq!(sc.max_used(), 0);
    }

    #[test]
    fn error_display() {
        let e = RingStatsError::WindowOverflow { max_window: 256 };
        assert!(e.to_string().contains("256"));
    }
}
