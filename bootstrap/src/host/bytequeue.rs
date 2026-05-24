#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByteQueueError {
    Full { capacity: usize, requested: usize },
    Empty,
}

impl std::fmt::Display for ByteQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ByteQueueError::Full { capacity, requested } => {
                write!(f, "queue full: {capacity}/{requested}")
            }
            ByteQueueError::Empty => write!(f, "queue empty"),
        }
    }
}

impl std::error::Error for ByteQueueError {}

#[derive(Debug, Clone)]
pub struct ByteQueue {
    buf: Vec<u8>,
    head: usize,
    len: usize,
    total_pushed: u64,
    total_popped: u64,
}

impl ByteQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0; capacity],
            head: 0,
            len: 0,
            total_pushed: 0,
            total_popped: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn available(&self) -> usize {
        self.buf.len() - self.len
    }

    pub fn push(&mut self, byte: u8) -> Result<(), ByteQueueError> {
        if self.len == self.buf.len() {
            return Err(ByteQueueError::Full {
                capacity: self.buf.len(),
                requested: 1,
            });
        }
        let idx = (self.head + self.len) % self.buf.len();
        self.buf[idx] = byte;
        self.len += 1;
        self.total_pushed += 1;
        Ok(())
    }

    pub fn push_slice(&mut self, data: &[u8]) -> Result<usize, ByteQueueError> {
        let avail = self.available();
        let to_write = data.len().min(avail);
        for &b in &data[..to_write] {
            let idx = (self.head + self.len) % self.buf.len();
            self.buf[idx] = b;
            self.len += 1;
        }
        self.total_pushed += to_write as u64;
        Ok(to_write)
    }

    pub fn pop(&mut self) -> Result<u8, ByteQueueError> {
        if self.len == 0 {
            return Err(ByteQueueError::Empty);
        }
        let byte = self.buf[self.head];
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        self.total_popped += 1;
        Ok(byte)
    }

    pub fn pop_slice(&mut self, out: &mut [u8]) -> usize {
        let to_read = out.len().min(self.len);
        for i in 0..to_read {
            out[i] = self.buf[self.head];
            self.head = (self.head + 1) % self.buf.len();
            self.len -= 1;
        }
        self.total_popped += to_read as u64;
        to_read
    }

    pub fn peek(&self) -> Option<u8> {
        if self.len == 0 {
            None
        } else {
            Some(self.buf[self.head])
        }
    }

    pub fn peek_slice(&self, out: &mut [u8]) -> usize {
        let to_read = out.len().min(self.len);
        for i in 0..to_read {
            let idx = (self.head + i) % self.buf.len();
            out[i] = self.buf[idx];
        }
        to_read
    }

    pub fn skip(&mut self, count: usize) -> usize {
        let to_skip = count.min(self.len);
        self.head = (self.head + to_skip) % self.buf.len();
        self.len -= to_skip;
        self.total_popped += to_skip as u64;
        to_skip
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }

    pub fn total_pushed(&self) -> u64 {
        self.total_pushed
    }

    pub fn total_popped(&self) -> u64 {
        self.total_popped
    }

    pub fn throughput_ratio(&self) -> f64 {
        if self.total_pushed == 0 {
            0.0
        } else {
            self.total_popped as f64 / self.total_pushed as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() {
        let q = ByteQueue::new(8);
        assert_eq!(q.capacity(), 8);
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
        assert_eq!(q.available(), 8);
    }

    #[test]
    fn push_pop() {
        let mut q = ByteQueue::new(4);
        q.push(0x42).unwrap();
        assert_eq!(q.len(), 1);
        assert_eq!(q.pop().unwrap(), 0x42);
        assert!(q.is_empty());
    }

    #[test]
    fn push_full() {
        let mut q = ByteQueue::new(2);
        q.push(1).unwrap();
        q.push(2).unwrap();
        let err = q.push(3).unwrap_err();
        assert!(matches!(err, ByteQueueError::Full { .. }));
    }

    #[test]
    fn pop_empty() {
        let mut q = ByteQueue::new(4);
        let err = q.pop().unwrap_err();
        assert!(matches!(err, ByteQueueError::Empty));
    }

    #[test]
    fn wrap_around() {
        let mut q = ByteQueue::new(3);
        q.push_slice(b"abc").unwrap();
        q.pop().unwrap();
        q.pop().unwrap();
        q.push_slice(b"de").unwrap();
        let mut out = [0u8; 3];
        let n = q.pop_slice(&mut out);
        assert_eq!(n, 3);
        assert_eq!(&out, b"cde");
    }

    #[test]
    fn push_slice_partial() {
        let mut q = ByteQueue::new(2);
        let n = q.push_slice(b"hello").unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn pop_slice() {
        let mut q = ByteQueue::new(8);
        q.push_slice(b"hello").unwrap();
        let mut out = [0u8; 3];
        let n = q.pop_slice(&mut out);
        assert_eq!(n, 3);
        assert_eq!(&out, b"hel");
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn peek() {
        let mut q = ByteQueue::new(4);
        assert!(q.peek().is_none());
        q.push(0xAA).unwrap();
        assert_eq!(q.peek(), Some(0xAA));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn peek_slice() {
        let mut q = ByteQueue::new(8);
        q.push_slice(b"abc").unwrap();
        let mut out = [0u8; 2];
        let n = q.peek_slice(&mut out);
        assert_eq!(n, 2);
        assert_eq!(&out, b"ab");
        assert_eq!(q.len(), 3);
    }

    #[test]
    fn skip() {
        let mut q = ByteQueue::new(8);
        q.push_slice(b"hello").unwrap();
        let skipped = q.skip(3);
        assert_eq!(skipped, 3);
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn stats() {
        let mut q = ByteQueue::new(8);
        q.push_slice(b"hello").unwrap();
        q.pop_slice(&mut [0u8; 3]);
        assert_eq!(q.total_pushed(), 5);
        assert_eq!(q.total_popped(), 3);
        assert!((q.throughput_ratio() - 0.6).abs() < 0.01);
    }

    #[test]
    fn clear() {
        let mut q = ByteQueue::new(4);
        q.push(1).unwrap();
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn error_display() {
        assert!(ByteQueueError::Full { capacity: 4, requested: 5 }.to_string().contains("4/5"));
        assert!(ByteQueueError::Empty.to_string().contains("empty"));
    }
}
