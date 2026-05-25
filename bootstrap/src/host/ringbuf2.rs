#[derive(Debug, Clone, PartialEq)]
pub enum Rb2Error {
    Full,
    Empty,
}

impl std::fmt::Display for Rb2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rb2Error::Full => write!(f, "buffer full"),
            Rb2Error::Empty => write!(f, "buffer empty"),
        }
    }
}

impl std::error::Error for Rb2Error {}

pub struct RingBuf2 {
    data: Vec<u8>,
    cap: usize,
    head: usize,
    tail: usize,
    len: usize,
    total_writes: u64,
    total_reads: u64,
    total_overwrites: u64,
}

impl RingBuf2 {
    pub fn new(cap: usize) -> Self { Self { data: vec![0; cap], cap, head: 0, tail: 0, len: 0, total_writes: 0, total_reads: 0, total_overwrites: 0 } }

    pub fn push(&mut self, byte: u8) -> Result<(), Rb2Error> {
        if self.len == self.cap { return Err(Rb2Error::Full); }
        self.data[self.tail] = byte;
        self.tail = (self.tail + 1) % self.cap;
        self.len += 1;
        self.total_writes += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<u8, Rb2Error> {
        if self.len == 0 { return Err(Rb2Error::Empty); }
        let val = self.data[self.head];
        self.head = (self.head + 1) % self.cap;
        self.len -= 1;
        self.total_reads += 1;
        Ok(val)
    }

    pub fn write_batch(&mut self, data: &[u8]) -> usize {
        let mut written = 0;
        for &b in data {
            if self.push(b).is_err() { break; }
            written += 1;
        }
        written
    }

    pub fn read_batch(&mut self, buf: &mut [u8]) -> usize {
        let mut read = 0;
        for b in buf.iter_mut() {
            match self.pop() { Ok(v) => { *b = v; read += 1; } Err(_) => break }
        }
        read
    }

    pub fn push_overwrite(&mut self, byte: u8) {
        if self.len == self.cap {
            self.head = (self.head + 1) % self.cap;
            self.len -= 1;
            self.total_overwrites += 1;
        }
        self.data[self.tail] = byte;
        self.tail = (self.tail + 1) % self.cap;
        self.len += 1;
        self.total_writes += 1;
    }

    pub fn peek(&self) -> Result<u8, Rb2Error> {
        if self.len == 0 { return Err(Rb2Error::Empty); }
        Ok(self.data[self.head])
    }

    pub fn peek_back(&self) -> Result<u8, Rb2Error> {
        if self.len == 0 { return Err(Rb2Error::Empty); }
        let idx = (self.tail + self.cap - 1) % self.cap;
        Ok(self.data[idx])
    }

    pub fn clear(&mut self) { self.head = 0; self.tail = 0; self.len = 0; }

    pub fn contiguous_read_slices(&self) -> (&[u8], &[u8]) {
        if self.len == 0 { return (&[], &[]); }
        let end = if self.head + self.len <= self.cap { self.head + self.len } else { self.cap };
        let first = &self.data[self.head..end];
        let remaining = self.len.saturating_sub(end - self.head);
        let second = &self.data[..remaining];
        (first, second)
    }

    pub fn len(&self) -> usize { self.len }
    pub fn cap(&self) -> usize { self.cap }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn is_full(&self) -> bool { self.len == self.cap }
    pub fn available(&self) -> usize { self.cap - self.len }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_overwrites(&self) -> u64 { self.total_overwrites }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rb() { let rb = RingBuf2::new(8); assert_eq!(rb.cap(), 8); assert!(rb.is_empty()); }

    #[test]
    fn push_pop() {
        let mut rb = RingBuf2::new(4);
        rb.push(1).unwrap(); rb.push(2).unwrap();
        assert_eq!(rb.pop().unwrap(), 1);
        assert_eq!(rb.pop().unwrap(), 2);
    }

    #[test]
    fn full_err() {
        let mut rb = RingBuf2::new(2);
        rb.push(1).unwrap(); rb.push(2).unwrap();
        assert!(rb.push(3).is_err());
    }

    #[test]
    fn empty_err() { assert!(RingBuf2::new(4).pop().is_err()); }

    #[test]
    fn batch() {
        let mut rb = RingBuf2::new(8);
        assert_eq!(rb.write_batch(b"hello"), 5);
        let mut buf = [0u8; 3];
        assert_eq!(rb.read_batch(&mut buf), 3);
        assert_eq!(&buf, b"hel");
    }

    #[test]
    fn overwrite() {
        let mut rb = RingBuf2::new(3);
        rb.push_overwrite(1); rb.push_overwrite(2); rb.push_overwrite(3); rb.push_overwrite(4);
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.pop().unwrap(), 2);
    }

    #[test]
    fn peek() {
        let mut rb = RingBuf2::new(4);
        rb.push(42).unwrap();
        assert_eq!(rb.peek().unwrap(), 42);
        assert_eq!(rb.peek_back().unwrap(), 42);
    }

    #[test]
    fn contiguous_slices() {
        let mut rb = RingBuf2::new(4);
        rb.write_batch(b"abcd");
        rb.read_batch(&mut [0; 2]);
        rb.write_batch(b"ef");
        let (a, b) = rb.contiguous_read_slices();
        assert!(a.len() + b.len() == 4);
    }

    #[test]
    fn clear() {
        let mut rb = RingBuf2::new(4);
        rb.push(1).unwrap(); rb.clear();
        assert!(rb.is_empty()); assert_eq!(rb.available(), 4);
    }

    #[test]
    fn stats() {
        let mut rb = RingBuf2::new(4);
        rb.push(1).unwrap(); rb.pop().unwrap();
        assert_eq!(rb.total_writes(), 1);
        assert_eq!(rb.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(Rb2Error::Full.to_string().contains("full")); }
}
