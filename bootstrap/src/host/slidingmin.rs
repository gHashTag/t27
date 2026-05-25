use std::collections::VecDeque;

pub struct SlidingMin {
    window: usize,
    buf: VecDeque<(usize, i64)>,
    idx: usize,
    total_push: u64,
    total_query: u64,
}

impl SlidingMin {
    pub fn new(window: usize) -> Self { Self { window: window.max(1), buf: VecDeque::new(), idx: 0, total_push: 0, total_query: 0 } }

    pub fn push(&mut self, val: i64) {
        self.total_push += 1;
        while !self.buf.is_empty() && self.buf.back().map(|(_, v)| v).unwrap() >= &val { self.buf.pop_back(); }
        self.buf.push_back((self.idx, val));
        while self.buf.front().map(|(i, _)| *i).unwrap_or(0) + self.window <= self.idx { self.buf.pop_front(); }
        self.idx += 1;
    }

    pub fn min(&mut self) -> Option<i64> {
        self.total_query += 1;
        self.buf.front().map(|(_, v)| *v)
    }

    pub fn window(&self) -> usize { self.window }
    pub fn len(&self) -> usize { self.buf.len() }
    pub fn total_push(&self) -> u64 { self.total_push }
    pub fn total_query(&self) -> u64 { self.total_query }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut sm = SlidingMin::new(3);
        sm.push(5); sm.push(3); sm.push(4);
        assert_eq!(sm.min(), Some(3));
    }

    #[test]
    fn slides_out() {
        let mut sm = SlidingMin::new(2);
        sm.push(5); sm.push(1); sm.push(3);
        assert_eq!(sm.min(), Some(1));
    }

    #[test]
    fn increasing() {
        let mut sm = SlidingMin::new(3);
        for v in 1..=10i64 { sm.push(v); }
        assert_eq!(sm.min(), Some(8));
    }

    #[test]
    fn decreasing() {
        let mut sm = SlidingMin::new(3);
        for v in (1..=10i64).rev() { sm.push(v); }
        assert_eq!(sm.min(), Some(1));
    }

    #[test]
    fn duplicates() {
        let mut sm = SlidingMin::new(2);
        sm.push(5); sm.push(5); sm.push(5);
        assert_eq!(sm.min(), Some(5));
    }

    #[test]
    fn stats() {
        let mut sm = SlidingMin::new(3);
        sm.push(1); sm.min();
        assert_eq!(sm.total_push(), 1);
        assert_eq!(sm.total_query(), 1);
    }
}
