use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvictPolicy { DropNewest, DropOldest }

#[derive(Debug, Clone, PartialEq)]
pub enum CqError {
    Full { capacity: usize },
    Empty,
}

impl std::fmt::Display for CqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CqError::Full { capacity } => write!(f, "full (cap {capacity})"),
            CqError::Empty => write!(f, "empty"),
        }
    }
}

impl std::error::Error for CqError {}

pub struct CompactQueue<T> {
    buf: VecDeque<T>,
    capacity: usize,
    policy: EvictPolicy,
    total_push: u64,
    total_pop: u64,
    total_evicted: u64,
}

impl<T> CompactQueue<T> {
    pub fn new(capacity: usize, policy: EvictPolicy) -> Self {
        Self { buf: VecDeque::with_capacity(capacity), capacity, policy, total_push: 0, total_pop: 0, total_evicted: 0 }
    }

    pub fn push(&mut self, item: T) -> Result<(), CqError> {
        if self.buf.len() >= self.capacity {
            match self.policy {
                EvictPolicy::DropOldest => { self.buf.pop_front(); self.total_evicted += 1; }
                EvictPolicy::DropNewest => { return Err(CqError::Full { capacity: self.capacity }); }
            }
        }
        self.buf.push_back(item);
        self.total_push += 1;
        Ok(())
    }

    pub fn push_force(&mut self, item: T) {
        if self.buf.len() >= self.capacity {
            self.buf.pop_front();
            self.total_evicted += 1;
        }
        self.buf.push_back(item);
        self.total_push += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.buf.pop_front();
        if item.is_some() { self.total_pop += 1; }
        item
    }

    pub fn pop_batch(&mut self, max: usize) -> Vec<T> {
        let mut result = Vec::with_capacity(max.min(self.buf.len()));
        while result.len() < max {
            match self.pop() { Some(item) => result.push(item), None => break }
        }
        result
    }

    pub fn peek(&self) -> Option<&T> { self.buf.front() }
    pub fn peek_back(&self) -> Option<&T> { self.buf.back() }
    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_push(&self) -> u64 { self.total_push }
    pub fn total_pop(&self) -> u64 { self.total_pop }
    pub fn total_evicted(&self) -> u64 { self.total_evicted }

    pub fn compact(&mut self, keep: usize) -> usize {
        let remove = self.buf.len().saturating_sub(keep);
        for _ in 0..remove { self.buf.pop_front(); }
        self.total_evicted += remove as u64;
        remove
    }

    pub fn clear(&mut self) {
        let len = self.buf.len();
        self.total_evicted += len as u64;
        self.buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() {
        let q: CompactQueue<i32> = CompactQueue::new(4, EvictPolicy::DropOldest);
        assert!(q.is_empty());
    }

    #[test]
    fn push_pop() {
        let mut q = CompactQueue::new(4, EvictPolicy::DropOldest);
        q.push(1).unwrap(); q.push(2).unwrap();
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
    }

    #[test]
    fn drop_oldest() {
        let mut q = CompactQueue::new(2, EvictPolicy::DropOldest);
        q.push(1).unwrap(); q.push(2).unwrap(); q.push(3).unwrap();
        assert_eq!(q.len(), 2);
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.total_evicted(), 1);
    }

    #[test]
    fn drop_newest_rejects() {
        let mut q = CompactQueue::new(2, EvictPolicy::DropNewest);
        q.push(1).unwrap(); q.push(2).unwrap();
        let err = q.push(3).unwrap_err();
        assert!(matches!(err, CqError::Full { .. }));
        assert_eq!(q.peek_back(), Some(&2));
    }

    #[test]
    fn force_push() {
        let mut q = CompactQueue::new(2, EvictPolicy::DropNewest);
        q.push(1).unwrap(); q.push(2).unwrap();
        q.push_force(3);
        assert_eq!(q.len(), 2);
        assert_eq!(q.pop(), Some(2));
    }

    #[test]
    fn peek() {
        let mut q = CompactQueue::new(4, EvictPolicy::DropOldest);
        q.push(42).unwrap();
        assert_eq!(q.peek(), Some(&42));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn compact() {
        let mut q = CompactQueue::new(10, EvictPolicy::DropOldest);
        for i in 0..10 { q.push(i).unwrap(); }
        let removed = q.compact(3);
        assert_eq!(removed, 7);
        assert_eq!(q.len(), 3);
    }

    #[test]
    fn batch_pop() {
        let mut q = CompactQueue::new(10, EvictPolicy::DropOldest);
        for i in 0..5 { q.push(i).unwrap(); }
        let batch = q.pop_batch(3);
        assert_eq!(batch, vec![0, 1, 2]);
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn clear() {
        let mut q = CompactQueue::new(10, EvictPolicy::DropOldest);
        for i in 0..5 { q.push(i).unwrap(); }
        q.clear();
        assert!(q.is_empty());
        assert_eq!(q.total_evicted(), 5);
    }

    #[test]
    fn stats() {
        let mut q = CompactQueue::new(2, EvictPolicy::DropOldest);
        q.push(1).unwrap(); q.push(2).unwrap(); q.push(3).unwrap();
        q.pop();
        assert_eq!(q.total_push(), 3);
        assert_eq!(q.total_pop(), 1);
    }

    #[test]
    fn error_display() { assert!(CqError::Empty.to_string().contains("empty")); }
}
