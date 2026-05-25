use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum WsError {
    Empty,
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::Empty => write!(f, "deque empty"),
        }
    }
}

impl std::error::Error for WsError {}

pub struct WorkSteal<T> {
    local: VecDeque<T>,
    stolen_count: u64,
    push_count: u64,
    pop_count: u64,
    steal_count: u64,
}

impl<T> WorkSteal<T> {
    pub fn new() -> Self { Self { local: VecDeque::new(), stolen_count: 0, push_count: 0, pop_count: 0, steal_count: 0 } }

    pub fn push(&mut self, item: T) {
        self.push_count += 1;
        self.local.push_back(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.pop_count += 1;
        self.local.pop_back()
    }

    pub fn steal(&mut self) -> Option<T> {
        self.steal_count += 1;
        self.stolen_count += 1;
        self.local.pop_front()
    }

    pub fn steal_batch(&mut self, max: usize) -> Vec<T> {
        let count = max.min(self.local.len() / 2).max(1).min(self.local.len());
        let mut batch = Vec::with_capacity(count);
        for _ in 0..count {
            match self.local.pop_front() {
                Some(item) => { batch.push(item); self.stolen_count += 1; }
                None => break,
            }
        }
        self.steal_count += 1;
        batch
    }

    pub fn len(&self) -> usize { self.local.len() }
    pub fn is_empty(&self) -> bool { self.local.is_empty() }
    pub fn stolen_count(&self) -> u64 { self.stolen_count }
    pub fn push_count(&self) -> u64 { self.push_count }
    pub fn pop_count(&self) -> u64 { self.pop_count }
    pub fn steal_count(&self) -> u64 { self.steal_count }
}

impl<T> Default for WorkSteal<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ws() { let ws: WorkSteal<i32> = WorkSteal::new(); assert!(ws.is_empty()); }

    #[test]
    fn push_pop() {
        let mut ws = WorkSteal::new();
        ws.push(1); ws.push(2); ws.push(3);
        assert_eq!(ws.pop(), Some(3));
        assert_eq!(ws.pop(), Some(2));
    }

    #[test]
    fn steal() {
        let mut ws = WorkSteal::new();
        ws.push(1); ws.push(2); ws.push(3);
        assert_eq!(ws.steal(), Some(1));
        assert_eq!(ws.steal(), Some(2));
    }

    #[test]
    fn steal_batch() {
        let mut ws = WorkSteal::new();
        for i in 0..10 { ws.push(i); }
        let batch = ws.steal_batch(3);
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0], 0);
    }

    #[test]
    fn lifo_fifo() {
        let mut ws = WorkSteal::new();
        ws.push(1); ws.push(2); ws.push(3);
        assert_eq!(ws.pop(), Some(3));
        assert_eq!(ws.steal(), Some(1));
    }

    #[test]
    fn empty_ops() {
        let mut ws: WorkSteal<i32> = WorkSteal::new();
        assert_eq!(ws.pop(), None);
        assert_eq!(ws.steal(), None);
    }

    #[test]
    fn steal_batch_empty() {
        let mut ws: WorkSteal<i32> = WorkSteal::new();
        assert!(ws.steal_batch(5).is_empty());
    }

    #[test]
    fn stats() {
        let mut ws = WorkSteal::new();
        ws.push(1); ws.push(2); ws.pop(); ws.steal();
        assert_eq!(ws.push_count(), 2);
        assert_eq!(ws.pop_count(), 1);
        assert_eq!(ws.steal_count(), 1);
    }

    #[test]
    fn drain_all() {
        let mut ws = WorkSteal::new();
        for i in 0..5 { ws.push(i); }
        while ws.pop().is_some() {}
        assert!(ws.is_empty());
    }

    #[test]
    fn error_display() { assert!(WsError::Empty.to_string().contains("empty")); }
}
