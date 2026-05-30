#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingError {
    Full,
    Empty,
}

impl std::fmt::Display for RingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingError::Full => write!(f, "ring buffer full"),
            RingError::Empty => write!(f, "ring buffer empty"),
        }
    }
}

impl std::error::Error for RingError {}

#[derive(Debug, Clone)]
pub struct RingBuffer<T: Clone> {
    buf: Vec<T>,
    cap: usize,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: vec![T::default(); capacity],
            cap: capacity,
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn with_default(capacity: usize, default: T) -> Self {
        Self {
            buf: vec![default; capacity],
            cap: capacity,
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.cap
    }

    pub fn available(&self) -> usize {
        self.cap - self.len
    }

    pub fn push(&mut self, item: T) -> Result<(), RingError> {
        if self.is_full() {
            return Err(RingError::Full);
        }
        self.buf[self.tail] = item;
        self.tail = (self.tail + 1) % self.cap;
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T, RingError> {
        if self.is_empty() {
            return Err(RingError::Empty);
        }
        let item = self.buf[self.head].clone();
        self.head = (self.head + 1) % self.cap;
        self.len -= 1;
        Ok(item)
    }

    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.buf[self.head])
        }
    }

    pub fn push_batch(&mut self, items: &[T]) -> Result<usize, RingError> {
        let to_write = items.len().min(self.available());
        if to_write == 0 && !items.is_empty() {
            return Err(RingError::Full);
        }
        for item in &items[..to_write] {
            self.buf[self.tail] = item.clone();
            self.tail = (self.tail + 1) % self.cap;
        }
        self.len += to_write;
        Ok(to_write)
    }

    pub fn pop_batch(&mut self, out: &mut [T]) -> Result<usize, RingError> {
        let to_read = out.len().min(self.len);
        if to_read == 0 && !out.is_empty() {
            return Err(RingError::Empty);
        }
        for slot in &mut out[..to_read] {
            *slot = self.buf[self.head].clone();
            self.head = (self.head + 1) % self.cap;
        }
        self.len -= to_read;
        Ok(to_read)
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_empty() {
        let rb: RingBuffer<u64> = RingBuffer::new(8);
        assert!(rb.is_empty());
        assert_eq!(rb.len(), 0);
        assert_eq!(rb.capacity(), 8);
    }

    #[test]
    fn push_pop_single() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        rb.push(42).unwrap();
        assert_eq!(rb.len(), 1);
        assert_eq!(rb.pop().unwrap(), 42);
        assert!(rb.is_empty());
    }

    #[test]
    fn push_to_full_errors() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(2);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        assert!(rb.is_full());
        assert!(matches!(rb.push(3), Err(RingError::Full)));
    }

    #[test]
    fn pop_from_empty_errors() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        assert!(matches!(rb.pop(), Err(RingError::Empty)));
    }

    #[test]
    fn fifo_order() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        rb.push(10).unwrap();
        rb.push(20).unwrap();
        rb.push(30).unwrap();
        assert_eq!(rb.pop().unwrap(), 10);
        assert_eq!(rb.pop().unwrap(), 20);
        assert_eq!(rb.pop().unwrap(), 30);
    }

    #[test]
    fn wrap_around() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(3);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.push(3).unwrap();
        assert_eq!(rb.pop().unwrap(), 1);
        rb.push(4).unwrap();
        assert_eq!(rb.pop().unwrap(), 2);
        assert_eq!(rb.pop().unwrap(), 3);
        assert_eq!(rb.pop().unwrap(), 4);
    }

    #[test]
    fn peek_returns_front() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        assert!(rb.peek().is_none());
        rb.push(99).unwrap();
        assert_eq!(*rb.peek().unwrap(), 99);
        assert_eq!(rb.len(), 1);
    }

    #[test]
    fn available_space() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        assert_eq!(rb.available(), 4);
        rb.push(1).unwrap();
        assert_eq!(rb.available(), 3);
    }

    #[test]
    fn push_batch() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        let written = rb.push_batch(&[10, 20, 30]).unwrap();
        assert_eq!(written, 3);
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.pop().unwrap(), 10);
    }

    #[test]
    fn push_batch_partial() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(2);
        rb.push(1).unwrap();
        let written = rb.push_batch(&[2, 3, 4]).unwrap();
        assert_eq!(written, 1);
        assert_eq!(rb.len(), 2);
    }

    #[test]
    fn push_batch_full_errors() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(2);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        assert!(matches!(rb.push_batch(&[3]), Err(RingError::Full)));
    }

    #[test]
    fn pop_batch() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        rb.push_batch(&[1, 2, 3]).unwrap();
        let mut out = [0u64; 2];
        let read = rb.pop_batch(&mut out).unwrap();
        assert_eq!(read, 2);
        assert_eq!(out, [1, 2]);
        assert_eq!(rb.len(), 1);
    }

    #[test]
    fn pop_batch_empty_errors() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        let mut out = [0u64; 2];
        assert!(matches!(rb.pop_batch(&mut out), Err(RingError::Empty)));
    }

    #[test]
    fn clear_resets() {
        let mut rb: RingBuffer<u64> = RingBuffer::new(4);
        rb.push(1).unwrap();
        rb.push(2).unwrap();
        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.available(), 4);
    }

    #[test]
    fn with_default() {
        let mut rb = RingBuffer::with_default(3, 0xFFu64);
        assert_eq!(rb.capacity(), 3);
        rb.push(1).unwrap();
        assert_eq!(rb.pop().unwrap(), 1);
    }

    #[test]
    fn error_display() {
        assert!(RingError::Full.to_string().contains("full"));
        assert!(RingError::Empty.to_string().contains("empty"));
    }
}
