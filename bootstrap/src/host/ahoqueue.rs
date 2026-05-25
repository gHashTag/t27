use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum AqError {
    Empty,
}

impl std::fmt::Display for AqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AqError::Empty => write!(f, "queue empty"),
        }
    }
}

impl std::error::Error for AqError {}

pub struct AhoQueue<T> {
    front: VecDeque<T>,
    back: Vec<T>,
    total_enqueue: u64,
    total_dequeue: u64,
    total_drains: u64,
}

impl<T> AhoQueue<T> {
    pub fn new() -> Self { Self { front: VecDeque::new(), back: Vec::new(), total_enqueue: 0, total_dequeue: 0, total_drains: 0 } }

    pub fn enqueue(&mut self, item: T) {
        self.total_enqueue += 1;
        self.back.push(item);
    }

    pub fn enqueue_batch(&mut self, items: Vec<T>) {
        self.total_enqueue += items.len() as u64;
        self.back.extend(items);
    }

    fn maybe_flip(&mut self) {
        if self.front.is_empty() && !self.back.is_empty() {
            self.front.extend(self.back.drain(..));
        }
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.maybe_flip();
        self.total_dequeue += 1;
        self.front.pop_front()
    }

    pub fn peek(&mut self) -> Option<&T> {
        self.maybe_flip();
        self.front.front()
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.total_drains += 1;
        self.maybe_flip();
        self.front.drain(..).chain(self.back.drain(..)).collect()
    }

    pub fn drain_n(&mut self, n: usize) -> Vec<T> {
        let mut result = Vec::with_capacity(n);
        for _ in 0..n {
            match self.dequeue() {
                Some(item) => result.push(item),
                None => break,
            }
        }
        result
    }

    pub fn len(&self) -> usize { self.front.len() + self.back.len() }
    pub fn is_empty(&self) -> bool { self.front.is_empty() && self.back.is_empty() }
    pub fn front_len(&self) -> usize { self.front.len() }
    pub fn back_len(&self) -> usize { self.back.len() }
    pub fn total_enqueue(&self) -> u64 { self.total_enqueue }
    pub fn total_dequeue(&self) -> u64 { self.total_dequeue }
    pub fn total_drains(&self) -> u64 { self.total_drains }
}

impl<T> Default for AhoQueue<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_aq() { let q: AhoQueue<i32> = AhoQueue::new(); assert!(q.is_empty()); }

    #[test]
    fn enqueue_dequeue() {
        let mut q = AhoQueue::new();
        q.enqueue(1); q.enqueue(2); q.enqueue(3);
        assert_eq!(q.dequeue(), Some(1));
        assert_eq!(q.dequeue(), Some(2));
        assert_eq!(q.dequeue(), Some(3));
        assert_eq!(q.dequeue(), None);
    }

    #[test]
    fn batch() {
        let mut q = AhoQueue::new();
        q.enqueue_batch(vec![10, 20, 30]);
        assert_eq!(q.len(), 3);
        assert_eq!(q.dequeue(), Some(10));
    }

    #[test]
    fn peek() {
        let mut q = AhoQueue::new();
        q.enqueue(42);
        assert_eq!(q.peek(), Some(&42));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn drain() {
        let mut q = AhoQueue::new();
        for i in 1..=5 { q.enqueue(i); }
        let all = q.drain();
        assert_eq!(all, vec![1, 2, 3, 4, 5]);
        assert!(q.is_empty());
    }

    #[test]
    fn drain_n() {
        let mut q = AhoQueue::new();
        for i in 1..=10 { q.enqueue(i); }
        let batch = q.drain_n(3);
        assert_eq!(batch, vec![1, 2, 3]);
        assert_eq!(q.len(), 7);
    }

    #[test]
    fn flip_amortized() {
        let mut q = AhoQueue::new();
        for i in 0..100 { q.enqueue(i); }
        for i in 0..100 { assert_eq!(q.dequeue(), Some(i)); }
    }

    #[test]
    fn interleave() {
        let mut q = AhoQueue::new();
        q.enqueue(1); q.enqueue(2);
        assert_eq!(q.dequeue(), Some(1));
        q.enqueue(3);
        assert_eq!(q.dequeue(), Some(2));
        assert_eq!(q.dequeue(), Some(3));
    }

    #[test]
    fn stats() {
        let mut q = AhoQueue::new();
        q.enqueue(1); q.dequeue(); q.drain();
        assert_eq!(q.total_enqueue(), 1);
        assert_eq!(q.total_dequeue(), 1);
        assert_eq!(q.total_drains(), 1);
    }

    #[test]
    fn error_display() { assert!(AqError::Empty.to_string().contains("empty")); }
}
