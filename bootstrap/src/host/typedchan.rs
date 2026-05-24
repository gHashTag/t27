use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum ChanError {
    Full { capacity: usize },
    Closed,
    Empty,
}

impl std::fmt::Display for ChanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChanError::Full { capacity } => write!(f, "channel full ({capacity})"),
            ChanError::Closed => write!(f, "channel closed"),
            ChanError::Empty => write!(f, "channel empty"),
        }
    }
}

impl std::error::Error for ChanError {}

pub struct TypedChannel<T> {
    buf: VecDeque<T>,
    capacity: usize,
    closed: bool,
    total_sent: u64,
    total_recv: u64,
    total_dropped: u64,
    high_water_mark: usize,
}

impl<T> TypedChannel<T> {
    pub fn new(capacity: usize) -> Self {
        Self { buf: VecDeque::with_capacity(capacity), capacity, closed: false, total_sent: 0, total_recv: 0, total_dropped: 0, high_water_mark: 0 }
    }

    pub fn send(&mut self, item: T) -> Result<(), ChanError> {
        if self.closed { return Err(ChanError::Closed); }
        if self.buf.len() >= self.capacity { return Err(ChanError::Full { capacity: self.capacity }); }
        self.buf.push_back(item);
        self.total_sent += 1;
        if self.buf.len() > self.high_water_mark { self.high_water_mark = self.buf.len(); }
        Ok(())
    }

    pub fn try_send(&mut self, item: T) -> Result<(), T> {
        if self.closed || self.buf.len() >= self.capacity { return Err(item); }
        self.buf.push_back(item);
        self.total_sent += 1;
        if self.buf.len() > self.high_water_mark { self.high_water_mark = self.buf.len(); }
        Ok(())
    }

    pub fn recv(&mut self) -> Option<T> {
        let item = self.buf.pop_front();
        if item.is_some() { self.total_recv += 1; }
        item
    }

    pub fn recv_all(&mut self) -> Vec<T> {
        let items: Vec<T> = self.buf.drain(..).collect();
        self.total_recv += items.len() as u64;
        items
    }

    pub fn close(&mut self) -> Vec<T> {
        self.closed = true;
        let remaining: Vec<T> = self.buf.drain(..).collect();
        self.total_dropped += remaining.len() as u64;
        remaining
    }

    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn is_closed(&self) -> bool { self.closed }
    pub fn total_sent(&self) -> u64 { self.total_sent }
    pub fn total_recv(&self) -> u64 { self.total_recv }
    pub fn total_dropped(&self) -> u64 { self.total_dropped }
    pub fn high_water_mark(&self) -> usize { self.high_water_mark }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chan() { let c: TypedChannel<i32> = TypedChannel::new(10); assert!(c.is_empty()); }

    #[test]
    fn send_recv() {
        let mut c = TypedChannel::new(10);
        c.send(42).unwrap();
        assert_eq!(c.recv(), Some(42));
        assert!(c.is_empty());
    }

    #[test]
    fn fifo_order() {
        let mut c = TypedChannel::new(10);
        c.send(1).unwrap(); c.send(2).unwrap(); c.send(3).unwrap();
        assert_eq!(c.recv(), Some(1));
        assert_eq!(c.recv(), Some(2));
        assert_eq!(c.recv(), Some(3));
    }

    #[test]
    fn full() {
        let mut c = TypedChannel::new(2);
        c.send(1).unwrap(); c.send(2).unwrap();
        let err = c.send(3).unwrap_err();
        assert!(matches!(err, ChanError::Full { .. }));
    }

    #[test]
    fn closed_send() {
        let mut c: TypedChannel<i32> = TypedChannel::new(10);
        c.close();
        let err = c.send(1).unwrap_err();
        assert!(matches!(err, ChanError::Closed));
    }

    #[test]
    fn try_send_overflow() {
        let mut c = TypedChannel::new(1);
        c.send(1).unwrap();
        let result = c.try_send(2);
        assert_eq!(result, Err(2));
    }

    #[test]
    fn recv_all() {
        let mut c = TypedChannel::new(10);
        c.send(1).unwrap(); c.send(2).unwrap(); c.send(3).unwrap();
        let items = c.recv_all();
        assert_eq!(items, vec![1, 2, 3]);
        assert!(c.is_empty());
    }

    #[test]
    fn close_drops_remaining() {
        let mut c = TypedChannel::new(10);
        c.send(1).unwrap(); c.send(2).unwrap();
        let dropped = c.close();
        assert_eq!(dropped.len(), 2);
        assert!(c.is_closed());
        assert_eq!(c.total_dropped(), 2);
    }

    #[test]
    fn high_water_mark() {
        let mut c = TypedChannel::new(10);
        c.send(1).unwrap(); c.send(2).unwrap(); c.send(3).unwrap();
        c.recv(); c.recv(); c.recv();
        assert_eq!(c.high_water_mark(), 3);
    }

    #[test]
    fn stats() {
        let mut c = TypedChannel::new(10);
        c.send(1).unwrap();
        c.recv();
        assert_eq!(c.total_sent(), 1);
        assert_eq!(c.total_recv(), 1);
    }

    #[test]
    fn error_display() { assert!(ChanError::Empty.to_string().contains("empty")); }
}
