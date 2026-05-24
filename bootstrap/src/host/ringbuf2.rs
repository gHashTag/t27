#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingBufError {
    Full { capacity: usize },
    Empty,
}

impl std::fmt::Display for RingBufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingBufError::Full { capacity } => write!(f, "full (cap={capacity})"),
            RingBufError::Empty => write!(f, "empty"),
        }
    }
}

impl std::error::Error for RingBufError {}

#[derive(Debug, Clone)]
pub struct RingBuf<T> {
    buf: Vec<Option<T>>,
    head: usize,
    tail: usize,
    count: usize,
    capacity: usize,
    total_push: u64,
    total_pop: u64,
    peak_len: usize,
}

impl<T> RingBuf<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: (0..capacity).map(|_| None).collect(),
            head: 0,
            tail: 0,
            count: 0,
            capacity,
            total_push: 0,
            total_pop: 0,
            peak_len: 0,
        }
    }

    pub fn capacity(&self) -> usize { self.capacity }
    pub fn len(&self) -> usize { self.count }
    pub fn is_empty(&self) -> bool { self.count == 0 }
    pub fn is_full(&self) -> bool { self.count == self.capacity }

    pub fn push(&mut self, item: T) -> Result<(), RingBufError> {
        if self.count >= self.capacity {
            return Err(RingBufError::Full { capacity: self.capacity });
        }
        self.buf[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;
        self.count += 1;
        self.total_push += 1;
        if self.count > self.peak_len { self.peak_len = self.count; }
        Ok(())
    }

    pub fn push_overwrite(&mut self, item: T) -> Option<T> {
        let evicted = if self.count >= self.capacity {
            let old = self.buf[self.head].take();
            self.head = (self.head + 1) % self.capacity;
            self.count -= 1;
            old
        } else {
            None
        };
        self.buf[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;
        self.count += 1;
        self.total_push += 1;
        if self.count > self.peak_len { self.peak_len = self.count; }
        evicted
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 { return None; }
        let item = self.buf[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.count -= 1;
        self.total_pop += 1;
        item
    }

    pub fn peek(&self) -> Option<&T> {
        if self.count == 0 { None } else { self.buf[self.head].as_ref() }
    }

    pub fn peek_back(&self) -> Option<&T> {
        if self.count == 0 { return None; }
        let idx = (self.tail + self.capacity - 1) % self.capacity;
        self.buf[idx].as_ref()
    }

    pub fn push_batch(&mut self, items: &[T]) -> usize
    where T: Clone
    {
        let mut pushed = 0;
        for item in items {
            if self.push(item.clone()).is_err() { break; }
            pushed += 1;
        }
        pushed
    }

    pub fn pop_batch(&mut self, max: usize) -> Vec<T> {
        let n = max.min(self.count);
        let mut result = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(item) = self.pop() { result.push(item); }
        }
        result
    }

    pub fn iter(&self) -> RingBufIter<'_, T> {
        RingBufIter { ring: self, pos: 0 }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.buf { slot.take(); }
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }

    pub fn total_push(&self) -> u64 { self.total_push }
    pub fn total_pop(&self) -> u64 { self.total_pop }
    pub fn peak_len(&self) -> usize { self.peak_len }
}

pub struct RingBufIter<'a, T> {
    ring: &'a RingBuf<T>,
    pos: usize,
}

impl<'a, T> Iterator for RingBufIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.ring.count { return None; }
        let idx = (self.ring.head + self.pos) % self.ring.capacity;
        self.pos += 1;
        self.ring.buf[idx].as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ringbuf() {
        let rb: RingBuf<i32> = RingBuf::new(8);
        assert_eq!(rb.capacity(), 8);
        assert!(rb.is_empty());
    }

    #[test]
    fn push_pop_fifo() {
        let mut rb = RingBuf::new(4);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.push(3).unwrap();
        assert_eq!(rb.pop(), Some(1));
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn full_rejected() {
        let mut rb = RingBuf::new(2);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        let err = rb.push(3).unwrap_err();
        assert!(matches!(err, RingBufError::Full { .. }));
    }

    #[test]
    fn push_overwrite() {
        let mut rb = RingBuf::new(3);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.push(3).unwrap();
        let evicted = rb.push_overwrite(4);
        assert_eq!(evicted, Some(1));
        assert_eq!(rb.len(), 3);
    }

    #[test]
    fn peek_front_back() {
        let mut rb = RingBuf::new(4);
        rb.push(10).unwrap();
        rb.push(20).unwrap();
        rb.push(30).unwrap();
        assert_eq!(rb.peek(), Some(&10));
        assert_eq!(rb.peek_back(), Some(&30));
    }

    #[test]
    fn wrap_around() {
        let mut rb = RingBuf::new(3);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.pop();
        rb.push(3).unwrap();
        rb.push(4).unwrap();
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), Some(4));
    }

    #[test]
    fn push_batch() {
        let mut rb = RingBuf::new(4);
        let pushed = rb.push_batch(&[1, 2, 3, 4, 5]);
        assert_eq!(pushed, 4);
        assert!(rb.is_full());
    }

    #[test]
    fn pop_batch() {
        let mut rb = RingBuf::new(8);
        rb.push(1).unwrap(); rb.push(2).unwrap(); rb.push(3).unwrap();
        let batch = rb.pop_batch(2);
        assert_eq!(batch, vec![1, 2]);
        assert_eq!(rb.len(), 1);
    }

    #[test]
    fn iterator() {
        let mut rb = RingBuf::new(8);
        rb.push(10).unwrap();
        rb.push(20).unwrap();
        rb.push(30).unwrap();
        let items: Vec<&i32> = rb.iter().collect();
        assert_eq!(items, vec![&10, &20, &30]);
    }

    #[test]
    fn peak_len() {
        let mut rb = RingBuf::new(8);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.push(3).unwrap();
        rb.pop();
        assert_eq!(rb.peak_len(), 3);
    }

    #[test]
    fn stats() {
        let mut rb = RingBuf::new(8);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.pop();
        assert_eq!(rb.total_push(), 2);
        assert_eq!(rb.total_pop(), 1);
    }

    #[test]
    fn clear() {
        let mut rb = RingBuf::new(4);
        rb.push(1).unwrap();
        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
    }
}
