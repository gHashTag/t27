use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WsError {
    Full { capacity: usize },
    Empty,
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::Full { capacity } => write!(f, "queue full (cap={capacity})"),
            WsError::Empty => write!(f, "queue empty"),
        }
    }
}

impl std::error::Error for WsError {}

#[derive(Debug, Clone)]
pub struct WorkStealQueue<T> {
    deque: VecDeque<T>,
    capacity: usize,
    total_push: u64,
    total_pop: u64,
    total_steal: u64,
    peak_len: usize,
}

impl<T> WorkStealQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            deque: VecDeque::with_capacity(capacity),
            capacity,
            total_push: 0,
            total_pop: 0,
            total_steal: 0,
            peak_len: 0,
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), WsError> {
        if self.deque.len() >= self.capacity {
            return Err(WsError::Full { capacity: self.capacity });
        }
        self.deque.push_back(item);
        self.total_push += 1;
        if self.deque.len() > self.peak_len {
            self.peak_len = self.deque.len();
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.deque.pop_front();
        if item.is_some() { self.total_pop += 1; }
        item
    }

    pub fn steal(&mut self) -> Option<T> {
        let item = self.deque.pop_back();
        if item.is_some() { self.total_steal += 1; }
        item
    }

    pub fn steal_batch(&mut self, max: usize) -> Vec<T> {
        let count = max.min(self.deque.len());
        let mut batch = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.deque.pop_back() {
                batch.push(item);
                self.total_steal += 1;
            }
        }
        batch
    }

    pub fn peek_front(&self) -> Option<&T> {
        self.deque.front()
    }

    pub fn peek_back(&self) -> Option<&T> {
        self.deque.back()
    }

    pub fn len(&self) -> usize {
        self.deque.len()
    }

    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn remaining(&self) -> usize {
        self.capacity - self.deque.len()
    }

    pub fn total_push(&self) -> u64 {
        self.total_push
    }

    pub fn total_pop(&self) -> u64 {
        self.total_pop
    }

    pub fn total_steal(&self) -> u64 {
        self.total_steal
    }

    pub fn peak_len(&self) -> usize {
        self.peak_len
    }

    pub fn clear(&mut self) {
        self.deque.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() {
        let q: WorkStealQueue<i32> = WorkStealQueue::new(16);
        assert_eq!(q.capacity(), 16);
        assert!(q.is_empty());
        assert_eq!(q.remaining(), 16);
    }

    #[test]
    fn push_pop_fifo() {
        let mut q = WorkStealQueue::new(16);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.push(3).unwrap();
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), Some(3));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn steal_from_back() {
        let mut q = WorkStealQueue::new(16);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.push(3).unwrap();
        assert_eq!(q.steal(), Some(3));
        assert_eq!(q.steal(), Some(2));
        assert_eq!(q.pop(), Some(1));
    }

    #[test]
    fn steal_batch() {
        let mut q = WorkStealQueue::new(16);
        for i in 1..=5 { q.push(i).unwrap(); }
        let batch = q.steal_batch(3);
        assert_eq!(batch, vec![5, 4, 3]);
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn steal_batch_limited_by_len() {
        let mut q = WorkStealQueue::new(16);
        q.push(1).unwrap();
        let batch = q.steal_batch(10);
        assert_eq!(batch, vec![1]);
        assert!(q.is_empty());
    }

    #[test]
    fn full_queue() {
        let mut q = WorkStealQueue::new(2);
        q.push(1).unwrap();
        q.push(2).unwrap();
        let err = q.push(3).unwrap_err();
        assert!(matches!(err, WsError::Full { capacity: 2 }));
    }

    #[test]
    fn pop_empty() {
        let mut q: WorkStealQueue<i32> = WorkStealQueue::new(4);
        assert_eq!(q.pop(), None);
        assert_eq!(q.steal(), None);
    }

    #[test]
    fn peek() {
        let mut q = WorkStealQueue::new(8);
        assert_eq!(q.peek_front(), None);
        q.push(10).unwrap();
        q.push(20).unwrap();
        assert_eq!(q.peek_front(), Some(&10));
        assert_eq!(q.peek_back(), Some(&20));
    }

    #[test]
    fn stats() {
        let mut q = WorkStealQueue::new(16);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.push(3).unwrap();
        q.pop();
        q.steal();
        assert_eq!(q.total_push(), 3);
        assert_eq!(q.total_pop(), 1);
        assert_eq!(q.total_steal(), 1);
        assert_eq!(q.peak_len(), 3);
    }

    #[test]
    fn clear() {
        let mut q = WorkStealQueue::new(8);
        q.push(1).unwrap();
        q.push(2).unwrap();
        q.clear();
        assert!(q.is_empty());
        assert_eq!(q.remaining(), 8);
    }

    #[test]
    fn mixed_pop_steal() {
        let mut q = WorkStealQueue::new(16);
        for i in 1..=6 { q.push(i).unwrap(); }
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.steal(), Some(6));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.steal(), Some(5));
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn error_display() {
        assert!(WsError::Full { capacity: 8 }.to_string().contains("8"));
        assert!(WsError::Empty.to_string().contains("empty"));
    }
}
