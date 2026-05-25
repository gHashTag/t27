use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggKind {
    Sum,
    Min,
    Max,
    Avg,
    Count,
}

pub struct SlidingAgg {
    window: VecDeque<i64>,
    cap: usize,
    sum: i64,
    total_pushes: u64,
    total_queries: u64,
}

impl SlidingAgg {
    pub fn new(window_size: usize) -> Self { Self { window: VecDeque::with_capacity(window_size), cap: window_size, sum: 0, total_pushes: 0, total_queries: 0 } }

    pub fn push(&mut self, val: i64) {
        self.total_pushes += 1;
        if self.window.len() == self.cap {
            if let Some(&old) = self.window.pop_front() { self.sum -= old; }
        }
        self.window.push_back(val);
        self.sum += val;
    }

    pub fn query(&mut self, kind: AggKind) -> Option<f64> {
        self.total_queries += 1;
        if self.window.is_empty() { return None; }
        Some(match kind {
            AggKind::Sum => self.sum as f64,
            AggKind::Avg => self.sum as f64 / self.window.len() as f64,
            AggKind::Count => self.window.len() as f64,
            AggKind::Min => (*self.window.iter().min().unwrap()) as f64,
            AggKind::Max => (*self.window.iter().max().unwrap()) as f64,
        })
    }

    pub fn sum(&mut self) -> i64 { self.query(AggKind::Sum).map(|v| v as i64).unwrap_or(0) }
    pub fn avg(&mut self) -> f64 { self.query(AggKind::Avg).unwrap_or(0.0) }
    pub fn min(&mut self) -> i64 { self.query(AggKind::Min).map(|v| v as i64).unwrap_or(0) }
    pub fn max(&mut self) -> i64 { self.query(AggKind::Max).map(|v| v as i64).unwrap_or(0) }
    pub fn count(&mut self) -> usize { self.window.len() }

    pub fn len(&self) -> usize { self.window.len() }
    pub fn is_empty(&self) -> bool { self.window.is_empty() }
    pub fn window_size(&self) -> usize { self.cap }
    pub fn total_pushes(&self) -> u64 { self.total_pushes }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sa() { let sa = SlidingAgg::new(3); assert!(sa.is_empty()); assert_eq!(sa.window_size(), 3); }

    #[test]
    fn sum_window() {
        let mut sa = SlidingAgg::new(3);
        sa.push(1); sa.push(2); sa.push(3);
        assert_eq!(sa.sum(), 6);
    }

    #[test]
    fn slide_out() {
        let mut sa = SlidingAgg::new(3);
        sa.push(1); sa.push(2); sa.push(3); sa.push(4);
        assert_eq!(sa.sum(), 9);
        assert_eq!(sa.count(), 3);
    }

    #[test]
    fn avg() {
        let mut sa = SlidingAgg::new(4);
        sa.push(10); sa.push(20); sa.push(30);
        assert!((sa.avg() - 20.0).abs() < 0.01);
    }

    #[test]
    fn min_max() {
        let mut sa = SlidingAgg::new(5);
        sa.push(5); sa.push(1); sa.push(9); sa.push(3);
        assert_eq!(sa.min(), 1);
        assert_eq!(sa.max(), 9);
    }

    #[test]
    fn empty_query() {
        let mut sa = SlidingAgg::new(3);
        assert!(sa.query(AggKind::Sum).is_none());
    }

    #[test]
    fn negative() {
        let mut sa = SlidingAgg::new(3);
        sa.push(-5); sa.push(10); sa.push(-3);
        assert_eq!(sa.sum(), 2);
    }

    #[test]
    fn count() {
        let mut sa = SlidingAgg::new(2);
        sa.push(1); sa.push(2); sa.push(3);
        assert_eq!(sa.count(), 2);
    }

    #[test]
    fn stats() {
        let mut sa = SlidingAgg::new(3);
        sa.push(1); sa.sum();
        assert_eq!(sa.total_pushes(), 1);
        assert_eq!(sa.total_queries(), 1);
    }
}
