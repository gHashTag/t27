use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum AbErr {
    Closed,
    Full { cap: usize },
}

impl std::fmt::Display for AbErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AbErr::Closed => write!(f, "buffer closed"),
            AbErr::Full { cap } => write!(f, "buffer full {cap}"),
        }
    }
}

impl std::error::Error for AbErr {}

pub struct AsyncBuf {
    buf: VecDeque<Vec<u8>>,
    cap: usize,
    closed: bool,
    total_writes: u64,
    total_reads: u64,
    bytes_written: u64,
    bytes_read: u64,
}

impl AsyncBuf {
    pub fn new(cap: usize) -> Self { Self { buf: VecDeque::with_capacity(cap), cap, closed: false, total_writes: 0, total_reads: 0, bytes_written: 0, bytes_read: 0 } }

    pub fn write(&mut self, data: Vec<u8>) -> Result<(), AbErr> {
        if self.closed { return Err(AbErr::Closed); }
        if self.buf.len() >= self.cap { return Err(AbErr::Full { cap: self.cap }); }
        self.bytes_written += data.len() as u64;
        self.total_writes += 1;
        self.buf.push_back(data);
        Ok(())
    }

    pub fn read(&mut self) -> Option<Vec<u8>> {
        let data = self.buf.pop_front()?;
        self.bytes_read += data.len() as u64;
        self.total_reads += 1;
        Some(data)
    }

    pub fn close(&mut self) { self.closed = true; }

    pub fn write_all(&mut self, chunks: Vec<Vec<u8>>) -> Result<usize, AbErr> {
        let mut written = 0usize;
        for chunk in chunks {
            match self.write(chunk) {
                Ok(()) => written += 1,
                Err(e) => { if written > 0 { return Ok(written); } else { return Err(e); } }
            }
        }
        Ok(written)
    }

    pub fn read_all(&mut self) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        while let Some(data) = self.read() { out.push(data); }
        out
    }

    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn is_closed(&self) -> bool { self.closed }
    pub fn cap(&self) -> usize { self.cap }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn bytes_written(&self) -> u64 { self.bytes_written }
    pub fn bytes_read(&self) -> u64 { self.bytes_read }
    pub fn throughput_ratio(&self) -> f64 { if self.bytes_written == 0 { 0.0 } else { self.bytes_read as f64 / self.bytes_written as f64 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read() {
        let mut ab = AsyncBuf::new(8);
        ab.write(b"hello".to_vec()).unwrap();
        let d = ab.read().unwrap();
        assert_eq!(d, b"hello");
    }

    #[test]
    fn full() {
        let mut ab = AsyncBuf::new(2);
        ab.write(b"a".to_vec()).unwrap();
        ab.write(b"b".to_vec()).unwrap();
        assert!(ab.write(b"c".to_vec()).is_err());
    }

    #[test]
    fn closed() {
        let mut ab = AsyncBuf::new(8);
        ab.close();
        assert!(ab.write(b"x".to_vec()).is_err());
    }

    #[test]
    fn write_all_read_all() {
        let mut ab = AsyncBuf::new(8);
        ab.write_all(vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]).unwrap();
        let all = ab.read_all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn empty_read() { assert!(AsyncBuf::new(8).read().is_none()); }

    #[test]
    fn bytes_tracking() {
        let mut ab = AsyncBuf::new(8);
        ab.write(b"abc".to_vec()).unwrap();
        ab.write(b"de".to_vec()).unwrap();
        ab.read();
        assert_eq!(ab.bytes_written(), 5);
        assert_eq!(ab.bytes_read(), 3);
    }

    #[test]
    fn throughput() {
        let mut ab = AsyncBuf::new(8);
        ab.write(b"abc".to_vec()).unwrap(); ab.read();
        assert!((ab.throughput_ratio() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn stats() {
        let mut ab = AsyncBuf::new(8);
        ab.write(vec![]).unwrap(); ab.read();
        assert_eq!(ab.total_writes(), 1);
        assert_eq!(ab.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(AbErr::Closed.to_string().contains("closed")); }
}
