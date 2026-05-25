use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq)]
pub enum AggFn {
    Sum,
    Max,
    Xor,
}

pub struct RingAgg {
    window: VecDeque<u64>,
    cap: usize,
    agg: AggFn,
    total_pushed: u64,
    total_queries: u64,
}

impl RingAgg {
    pub fn new(cap: usize, agg: AggFn) -> Self { Self { window: VecDeque::with_capacity(cap), cap, agg, total_pushed: 0, total_queries: 0 } }

    pub fn push(&mut self, value: u64) {
        self.total_pushed += 1;
        if self.window.len() == self.cap { self.window.pop_front(); }
        self.window.push_back(value);
    }

    pub fn query(&mut self) -> u64 {
        self.total_queries += 1;
        match self.agg {
            AggFn::Sum => self.window.iter().fold(0u64, |a, &v| a.wrapping_add(v)),
            AggFn::Max => self.window.iter().fold(0u64, |a, &v| a.max(v)),
            AggFn::Xor => self.window.iter().fold(0u64, |a, &v| a ^ v),
        }
    }

    pub fn query_prefix(&mut self, n: usize) -> u64 {
        self.total_queries += 1;
        let take = n.min(self.window.len());
        match self.agg {
            AggFn::Sum => self.window.iter().take(take).fold(0u64, |a, &v| a.wrapping_add(v)),
            AggFn::Max => self.window.iter().take(take).fold(0u64, |a, &v| a.max(v)),
            AggFn::Xor => self.window.iter().take(take).fold(0u64, |a, &v| a ^ v),
        }
    }

    pub fn len(&self) -> usize { self.window.len() }
    pub fn is_empty(&self) -> bool { self.window.is_empty() }
    pub fn cap(&self) -> usize { self.cap }
    pub fn agg_fn(&self) -> AggFn { self.agg }
    pub fn total_pushed(&self) -> u64 { self.total_pushed }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_agg() {
        let mut ra = RingAgg::new(3, AggFn::Sum);
        ra.push(1); ra.push(2); ra.push(3);
        assert_eq!(ra.query(), 6);
    }

    #[test]
    fn sum_slide() {
        let mut ra = RingAgg::new(3, AggFn::Sum);
        ra.push(1); ra.push(2); ra.push(3); ra.push(4);
        assert_eq!(ra.query(), 9);
    }

    #[test]
    fn max_agg() {
        let mut ra = RingAgg::new(5, AggFn::Max);
        ra.push(3); ra.push(7); ra.push(2);
        assert_eq!(ra.query(), 7);
    }

    #[test]
    fn xor_agg() {
        let mut ra = RingAgg::new(5, AggFn::Xor);
        ra.push(0xFF); ra.push(0x0F);
        assert_eq!(ra.query(), 0xF0);
    }

    #[test]
    fn prefix() {
        let mut ra = RingAgg::new(5, AggFn::Sum);
        ra.push(1); ra.push(2); ra.push(3);
        assert_eq!(ra.query_prefix(2), 3);
    }

    #[test]
    fn empty_query() {
        let mut ra = RingAgg::new(5, AggFn::Sum);
        assert_eq!(ra.query(), 0);
    }

    #[test]
    fn cap() { assert_eq!(RingAgg::new(10, AggFn::Sum).cap(), 10); }

    #[test]
    fn stats() {
        let mut ra = RingAgg::new(5, AggFn::Sum);
        ra.push(1); ra.query();
        assert_eq!(ra.total_pushed(), 1);
        assert_eq!(ra.total_queries(), 1);
    }
}
