pub struct RingBuf2 {
    buf: Vec<u8>,
    head: usize,
    tail: usize,
    len: usize,
    total_write: u64,
    total_read: u64,
}

impl RingBuf2 {
    pub fn new(cap: usize) -> Self {
        Self { buf: vec![0; cap.max(1)], head: 0, tail: 0, len: 0, total_write: 0, total_read: 0 }
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let cap = self.buf.len();
        let available = cap - self.len;
        let n = data.len().min(available);
        for i in 0..n {
            self.buf[self.tail] = data[i];
            self.tail = (self.tail + 1) % cap;
        }
        self.len += n;
        self.total_write += n as u64;
        n
    }

    pub fn read(&mut self, out: &mut [u8]) -> usize {
        let n = out.len().min(self.len);
        let cap = self.buf.len();
        for i in 0..n {
            out[i] = self.buf[self.head];
            self.head = (self.head + 1) % cap;
        }
        self.len -= n;
        self.total_read += n as u64;
        n
    }

    pub fn peek(&self, out: &mut [u8]) -> usize {
        let n = out.len().min(self.len);
        for i in 0..n { out[i] = self.buf[(self.head + i) % self.buf.len()]; }
        n
    }

    pub fn skip(&mut self, n: usize) -> usize {
        let n = n.min(self.len);
        self.head = (self.head + n) % self.buf.len();
        self.len -= n;
        n
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn available(&self) -> usize { self.buf.len() - self.len }
    pub fn cap(&self) -> usize { self.buf.len() }
    pub fn total_write(&self) -> u64 { self.total_write }
    pub fn total_read(&self) -> u64 { self.total_read }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let mut rb = RingBuf2::new(8);
        assert_eq!(rb.write(b"hello"), 5);
        let mut out = [0u8; 5];
        assert_eq!(rb.read(&mut out), 5);
        assert_eq!(&out, b"hello");
    }

    #[test]
    fn wrap_around() {
        let mut rb = RingBuf2::new(4);
        rb.write(b"abcd");
        let mut out = [0u8; 2];
        rb.read(&mut out);
        assert_eq!(rb.write(b"ef"), 2);
        assert_eq!(rb.len(), 4);
    }

    #[test]
    fn full_write() {
        let mut rb = RingBuf2::new(4);
        assert_eq!(rb.write(b"abcdef"), 4);
        assert_eq!(rb.write(b"g"), 0);
    }

    #[test]
    fn peek() {
        let mut rb = RingBuf2::new(8);
        rb.write(b"hello");
        let mut out = [0u8; 3];
        rb.peek(&mut out);
        assert_eq!(&out, b"hel");
        assert_eq!(rb.len(), 5);
    }

    #[test]
    fn skip() {
        let mut rb = RingBuf2::new(8);
        rb.write(b"hello");
        assert_eq!(rb.skip(2), 2);
        let mut out = [0u8; 3];
        rb.read(&mut out);
        assert_eq!(&out, b"llo");
    }

    #[test]
    fn stats() {
        let mut rb = RingBuf2::new(8);
        rb.write(b"abc"); let mut out = [0u8; 2]; rb.read(&mut out);
        assert_eq!(rb.total_write(), 3);
        assert_eq!(rb.total_read(), 2);
    }
}
