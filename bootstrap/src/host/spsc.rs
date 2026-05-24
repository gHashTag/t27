#[derive(Debug, Clone, PartialEq)]
pub enum SpscError {
    Full { capacity: usize },
    Empty,
}

impl std::fmt::Display for SpscError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpscError::Full { capacity } => write!(f, "full (cap {capacity})"),
            SpscError::Empty => write!(f, "empty"),
        }
    }
}

impl std::error::Error for SpscError {}

pub struct SpscRing<T> {
    buf: Vec<Option<T>>,
    head: usize,
    tail: usize,
    len: usize,
    total_push: u64,
    total_pop: u64,
}

impl<T> SpscRing<T> {
    pub fn new(capacity: usize) -> Self {
        Self { buf: (0..capacity).map(|_| None).collect(), head: 0, tail: 0, len: 0, total_push: 0, total_pop: 0 }
    }

    pub fn push(&mut self, item: T) -> Result<(), SpscError> {
        if self.len >= self.buf.len() { return Err(SpscError::Full { capacity: self.buf.len() }); }
        self.buf[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.buf.len();
        self.len += 1;
        self.total_push += 1;
        Ok(())
    }

    pub fn push_batch(&mut self, items: Vec<T>) -> Result<usize, SpscError> {
        let avail = self.buf.len() - self.len;
        let to_write = items.len().min(avail);
        if to_write == 0 && !items.is_empty() { return Err(SpscError::Full { capacity: self.buf.len() }); }
        for item in items.into_iter().take(to_write) { self.push(item).unwrap(); }
        Ok(to_write)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 { return None; }
        let item = self.buf[self.head].take();
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        self.total_pop += 1;
        item
    }

    pub fn pop_batch(&mut self, max: usize) -> Vec<T> {
        let mut result = Vec::with_capacity(max.min(self.len));
        while result.len() < max {
            match self.pop() {
                Some(item) => result.push(item),
                None => break,
            }
        }
        result
    }

    pub fn peek(&self) -> Option<&T> { self.buf[self.head].as_ref() }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn capacity(&self) -> usize { self.buf.len() }
    pub fn total_push(&self) -> u64 { self.total_push }
    pub fn total_pop(&self) -> u64 { self.total_pop }
    pub fn available(&self) -> usize { self.buf.len() - self.len }

    pub fn clear(&mut self) {
        for slot in &mut self.buf { *slot = None; }
        self.head = 0; self.tail = 0; self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ring() {
        let r: SpscRing<i32> = SpscRing::new(4);
        assert!(r.is_empty());
        assert_eq!(r.capacity(), 4);
    }

    #[test]
    fn push_pop() {
        let mut r = SpscRing::new(4);
        r.push(10).unwrap();
        r.push(20).unwrap();
        assert_eq!(r.pop(), Some(10));
        assert_eq!(r.pop(), Some(20));
        assert!(r.is_empty());
    }

    #[test]
    fn full() {
        let mut r = SpscRing::new(2);
        r.push(1).unwrap(); r.push(2).unwrap();
        let err = r.push(3).unwrap_err();
        assert!(matches!(err, SpscError::Full { .. }));
    }

    #[test]
    fn wrap_around() {
        let mut r = SpscRing::new(3);
        r.push(1).unwrap(); r.push(2).unwrap();
        r.pop(); r.pop();
        r.push(3).unwrap(); r.push(4).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r.pop(), Some(3));
    }

    #[test]
    fn batch_push() {
        let mut r = SpscRing::new(5);
        let written = r.push_batch(vec![1, 2, 3]).unwrap();
        assert_eq!(written, 3);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn batch_push_partial() {
        let mut r = SpscRing::new(2);
        r.push(1).unwrap();
        let written = r.push_batch(vec![2, 3, 4]).unwrap();
        assert_eq!(written, 1);
    }

    #[test]
    fn batch_pop() {
        let mut r = SpscRing::new(10);
        for i in 0..5 { r.push(i).unwrap(); }
        let items = r.pop_batch(3);
        assert_eq!(items, vec![0, 1, 2]);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn peek() {
        let mut r = SpscRing::new(4);
        r.push(42).unwrap();
        assert_eq!(r.peek(), Some(&42));
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn available() {
        let mut r = SpscRing::new(4);
        r.push(1).unwrap();
        assert_eq!(r.available(), 3);
    }

    #[test]
    fn clear() {
        let mut r = SpscRing::new(4);
        r.push(1).unwrap(); r.push(2).unwrap();
        r.clear();
        assert!(r.is_empty());
    }

    #[test]
    fn stats() {
        let mut r = SpscRing::new(4);
        r.push(1).unwrap(); r.pop();
        assert_eq!(r.total_push(), 1);
        assert_eq!(r.total_pop(), 1);
    }

    #[test]
    fn pop_empty() { assert!((SpscRing::<i32>::new(4)).pop().is_none()); }
}
