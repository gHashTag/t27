use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum SpError {
    Full,
    Empty,
}

impl std::fmt::Display for SpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpError::Full => write!(f, "buffer full"),
            SpError::Empty => write!(f, "buffer empty"),
        }
    }
}

impl std::error::Error for SpError {}

pub struct Spsc {
    buf: VecDeque<Vec<u8>>,
    capacity: usize,
    total_pushed: u64,
    total_popped: u64,
    total_overflow: u64,
}

impl Spsc {
    pub fn new(capacity: usize) -> Self { Self { buf: VecDeque::with_capacity(capacity), capacity, total_pushed: 0, total_popped: 0, total_overflow: 0 } }

    pub fn try_push(&mut self, data: Vec<u8>) -> Result<(), SpError> {
        if self.buf.len() >= self.capacity { return Err(SpError::Full); }
        self.buf.push_back(data);
        self.total_pushed += 1;
        Ok(())
    }

    pub fn try_pop(&mut self) -> Option<Vec<u8>> {
        let v = self.buf.pop_front()?;
        self.total_popped += 1;
        Some(v)
    }

    pub fn push_overwrite(&mut self, data: Vec<u8>) -> Option<Vec<u8>> {
        if self.buf.len() >= self.capacity {
            self.total_overflow += 1;
            let old = self.buf.pop_front();
            self.buf.push_back(data);
            self.total_pushed += 1;
            old
        } else {
            self.buf.push_back(data);
            self.total_pushed += 1;
            None
        }
    }

    pub fn peek(&self) -> Option<&Vec<u8>> { self.buf.front() }

    pub fn peek_back(&self) -> Option<&Vec<u8>> { self.buf.back() }

    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn is_full(&self) -> bool { self.buf.len() >= self.capacity }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_pushed(&self) -> u64 { self.total_pushed }
    pub fn total_popped(&self) -> u64 { self.total_popped }
    pub fn total_overflow(&self) -> u64 { self.total_overflow }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buf() { let s = Spsc::new(4); assert!(s.is_empty()); assert_eq!(s.capacity(), 4); }

    #[test]
    fn push_pop() {
        let mut s = Spsc::new(4);
        s.try_push(b"hello".to_vec()).unwrap();
        let data = s.try_pop().unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn full() {
        let mut s = Spsc::new(2);
        s.try_push(b"a".to_vec()).unwrap();
        s.try_push(b"b".to_vec()).unwrap();
        assert!(s.is_full());
        let err = s.try_push(b"c".to_vec()).unwrap_err();
        assert!(matches!(err, SpError::Full));
    }

    #[test]
    fn empty_pop() { assert!(Spsc::new(4).try_pop().is_none()); }

    #[test]
    fn overwrite() {
        let mut s = Spsc::new(2);
        s.try_push(b"a".to_vec()).unwrap();
        s.try_push(b"b".to_vec()).unwrap();
        let old = s.push_overwrite(b"c".to_vec());
        assert_eq!(old, Some(b"a".to_vec()));
        assert_eq!(s.len(), 2);
        assert_eq!(s.total_overflow(), 1);
    }

    #[test]
    fn peek() {
        let mut s = Spsc::new(4);
        s.try_push(b"first".to_vec()).unwrap();
        s.try_push(b"second".to_vec()).unwrap();
        assert_eq!(s.peek(), Some(&b"first".to_vec()));
        assert_eq!(s.peek_back(), Some(&b"second".to_vec()));
    }

    #[test]
    fn fifo_order() {
        let mut s = Spsc::new(8);
        for i in 0..5 { s.try_push(vec![i]).unwrap(); }
        for i in 0..5 { assert_eq!(s.try_pop(), Some(vec![i])); }
    }

    #[test]
    fn stats() {
        let mut s = Spsc::new(4);
        s.try_push(b"x".to_vec()).unwrap();
        s.try_pop();
        assert_eq!(s.total_pushed(), 1);
        assert_eq!(s.total_popped(), 1);
    }

    #[test]
    fn large_capacity() {
        let mut s = Spsc::new(1024);
        for i in 0..1024 { s.try_push(vec![i as u8]).unwrap(); }
        assert!(s.is_full());
        assert_eq!(s.len(), 1024);
    }

    #[test]
    fn error_display() { assert!(SpError::Full.to_string().contains("full")); }
}
