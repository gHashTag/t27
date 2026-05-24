use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum RvError {
    BufferEmpty,
    BufferFull,
}

impl std::fmt::Display for RvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RvError::BufferEmpty => write!(f, "buffer empty"),
            RvError::BufferFull => write!(f, "buffer full"),
        }
    }
}

impl std::error::Error for RvError {}

pub struct RingVec {
    buf: VecDeque<u64>,
    capacity: usize,
    total_pushed: u64,
    total_popped: u64,
    total_overflow: u64,
}

impl RingVec {
    pub fn new(capacity: usize) -> Self {
        Self { buf: VecDeque::with_capacity(capacity), capacity, total_pushed: 0, total_popped: 0, total_overflow: 0 }
    }

    pub fn push_back(&mut self, val: u64) -> Result<(), RvError> {
        if self.buf.len() >= self.capacity {
            self.buf.pop_front();
            self.total_overflow += 1;
        }
        self.buf.push_back(val);
        self.total_pushed += 1;
        Ok(())
    }

    pub fn push_front(&mut self, val: u64) -> Result<(), RvError> {
        if self.buf.len() >= self.capacity {
            self.buf.pop_back();
            self.total_overflow += 1;
        }
        self.buf.push_front(val);
        self.total_pushed += 1;
        Ok(())
    }

    pub fn pop_front(&mut self) -> Option<u64> {
        let v = self.buf.pop_front()?;
        self.total_popped += 1;
        Some(v)
    }

    pub fn pop_back(&mut self) -> Option<u64> {
        let v = self.buf.pop_back()?;
        self.total_popped += 1;
        Some(v)
    }

    pub fn extend_from_slice(&mut self, vals: &[u64]) {
        for &v in vals { self.push_back(v).unwrap(); }
    }

    pub fn drain_to(&mut self, target: &mut RingVec, count: usize) -> usize {
        let mut transferred = 0;
        for _ in 0..count {
            if let Some(v) = self.pop_front() {
                if target.push_back(v).is_ok() {
                    transferred += 1;
                }
            }
        }
        transferred
    }

    pub fn sliding_sum(&self, window: usize) -> Vec<u64> {
        let mut result = Vec::new();
        if self.buf.is_empty() || window == 0 { return result; }
        let vals: Vec<u64> = self.buf.iter().copied().collect();
        for i in 0..vals.len().saturating_sub(window - 1) + 1 {
            let end = (i + window).min(vals.len());
            if end - i == window {
                result.push(vals[i..end].iter().sum());
            }
        }
        result
    }

    pub fn front(&self) -> Option<&u64> { self.buf.front() }
    pub fn back(&self) -> Option<&u64> { self.buf.back() }
    pub fn get(&self, idx: usize) -> Option<&u64> { self.buf.get(idx) }
    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_pushed(&self) -> u64 { self.total_pushed }
    pub fn total_popped(&self) -> u64 { self.total_popped }
    pub fn total_overflow(&self) -> u64 { self.total_overflow }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buf() { let rv = RingVec::new(4); assert_eq!(rv.capacity(), 4); assert!(rv.is_empty()); }

    #[test]
    fn push_pop() {
        let mut rv = RingVec::new(4);
        rv.push_back(1).unwrap(); rv.push_back(2).unwrap();
        assert_eq!(rv.pop_front(), Some(1));
        assert_eq!(rv.pop_front(), Some(2));
    }

    #[test]
    fn overflow_front() {
        let mut rv = RingVec::new(2);
        rv.push_back(1).unwrap(); rv.push_back(2).unwrap();
        rv.push_back(3).unwrap();
        assert_eq!(rv.len(), 2);
        assert_eq!(rv.front(), Some(&2));
        assert_eq!(rv.back(), Some(&3));
        assert_eq!(rv.total_overflow(), 1);
    }

    #[test]
    fn push_front() {
        let mut rv = RingVec::new(3);
        rv.push_front(3).unwrap(); rv.push_front(2).unwrap(); rv.push_front(1).unwrap();
        assert_eq!(rv.pop_front(), Some(1));
    }

    #[test]
    fn extend() {
        let mut rv = RingVec::new(5);
        rv.extend_from_slice(&[1, 2, 3]);
        assert_eq!(rv.len(), 3);
    }

    #[test]
    fn drain_to() {
        let mut src = RingVec::new(10);
        let mut dst = RingVec::new(10);
        src.extend_from_slice(&[1, 2, 3, 4]);
        let n = src.drain_to(&mut dst, 2);
        assert_eq!(n, 2);
        assert_eq!(src.len(), 2);
        assert_eq!(dst.len(), 2);
    }

    #[test]
    fn sliding_sum() {
        let mut rv = RingVec::new(10);
        rv.extend_from_slice(&[1, 2, 3, 4, 5]);
        let sums = rv.sliding_sum(3);
        assert_eq!(sums, vec![6, 9, 12]);
    }

    #[test]
    fn get() {
        let mut rv = RingVec::new(5);
        rv.push_back(10).unwrap(); rv.push_back(20).unwrap();
        assert_eq!(rv.get(0), Some(&10));
        assert_eq!(rv.get(1), Some(&20));
        assert_eq!(rv.get(5), None);
    }

    #[test]
    fn stats() {
        let mut rv = RingVec::new(2);
        rv.push_back(1).unwrap(); rv.push_back(2).unwrap(); rv.push_back(3).unwrap();
        rv.pop_front();
        assert_eq!(rv.total_pushed(), 3);
        assert_eq!(rv.total_overflow(), 1);
        assert_eq!(rv.total_popped(), 1);
    }

    #[test]
    fn error_display() { assert!(RvError::BufferEmpty.to_string().contains("empty")); }
}
