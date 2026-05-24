use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum WsError {
    Empty,
    Closed,
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::Empty => write!(f, "deque empty"),
            WsError::Closed => write!(f, "deque closed"),
        }
    }
}

impl std::error::Error for WsError {}

pub struct WsDeque<T> {
    local: VecDeque<T>,
    stolen: u64,
    pushed: u64,
    popped: u64,
    steal_count: u64,
    capacity: usize,
    closed: bool,
}

impl<T> WsDeque<T> {
    pub fn new(capacity: usize) -> Self {
        Self { local: VecDeque::with_capacity(capacity), stolen: 0, pushed: 0, popped: 0, steal_count: 0, capacity, closed: false }
    }

    pub fn push(&mut self, item: T) -> Result<(), WsError> {
        if self.closed { return Err(WsError::Closed); }
        self.local.push_back(item);
        self.pushed += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.local.pop_back();
        if item.is_some() { self.popped += 1; }
        item
    }

    pub fn steal(&mut self, count: usize) -> Vec<T> {
        let available = self.local.len().min(count);
        let mut stolen = Vec::with_capacity(available);
        for _ in 0..available {
            if let Some(item) = self.local.pop_front() {
                stolen.push(item);
            }
        }
        self.stolen += stolen.len() as u64;
        self.steal_count += 1;
        stolen
    }

    pub fn steal_half(&mut self) -> Vec<T> {
        let half = self.local.len() / 2;
        if half == 0 { return Vec::new(); }
        self.steal(half)
    }

    pub fn len(&self) -> usize { self.local.len() }
    pub fn is_empty(&self) -> bool { self.local.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn close(&mut self) { self.closed = true; }
    pub fn is_closed(&self) -> bool { self.closed }
    pub fn total_pushed(&self) -> u64 { self.pushed }
    pub fn total_popped(&self) -> u64 { self.popped }
    pub fn total_stolen(&self) -> u64 { self.stolen }
    pub fn total_steal_ops(&self) -> u64 { self.steal_count }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_deque() { let d: WsDeque<i32> = WsDeque::new(100); assert!(d.is_empty()); }

    #[test]
    fn push_pop() {
        let mut d = WsDeque::new(100);
        d.push(1).unwrap(); d.push(2).unwrap(); d.push(3).unwrap();
        assert_eq!(d.pop(), Some(3));
        assert_eq!(d.pop(), Some(2));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn lifo_order() {
        let mut d = WsDeque::new(100);
        d.push(1).unwrap(); d.push(2).unwrap();
        assert_eq!(d.pop(), Some(2));
        assert_eq!(d.pop(), Some(1));
    }

    #[test]
    fn steal_fifo() {
        let mut d = WsDeque::new(100);
        d.push(1).unwrap(); d.push(2).unwrap(); d.push(3).unwrap();
        let stolen = d.steal(2);
        assert_eq!(stolen, vec![1, 2]);
        assert_eq!(d.pop(), Some(3));
    }

    #[test]
    fn steal_half() {
        let mut d = WsDeque::new(100);
        for i in 0..10 { d.push(i).unwrap(); }
        let stolen = d.steal_half();
        assert_eq!(stolen.len(), 5);
        assert_eq!(d.len(), 5);
    }

    #[test]
    fn steal_empty() {
        let mut d: WsDeque<i32> = WsDeque::new(100);
        let stolen = d.steal(5);
        assert!(stolen.is_empty());
    }

    #[test]
    fn close_blocks_push() {
        let mut d = WsDeque::new(100);
        d.close();
        let err = d.push(1).unwrap_err();
        assert!(matches!(err, WsError::Closed));
    }

    #[test]
    fn stats() {
        let mut d = WsDeque::new(100);
        d.push(1).unwrap(); d.push(2).unwrap();
        d.pop(); d.steal(1);
        assert_eq!(d.total_pushed(), 2);
        assert_eq!(d.total_popped(), 1);
        assert_eq!(d.total_stolen(), 1);
    }

    #[test]
    fn steal_more_than_available() {
        let mut d = WsDeque::new(100);
        d.push(1).unwrap(); d.push(2).unwrap();
        let stolen = d.steal(10);
        assert_eq!(stolen.len(), 2);
    }

    #[test]
    fn pop_empty() { let mut d: WsDeque<i32> = WsDeque::new(100); assert!(d.pop().is_none()); }

    #[test]
    fn error_display() { assert!(WsError::Empty.to_string().contains("empty")); }
}
