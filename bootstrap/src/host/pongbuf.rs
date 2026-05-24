#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSide {
    A,
    B,
}

impl std::fmt::Display for BufferSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferSide::A => write!(f, "A"),
            BufferSide::B => write!(f, "B"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PongError {
    NotReady { side: BufferSide },
    AlreadyFull { side: BufferSide },
    BufferOverflow { side: BufferSide, capacity: usize, requested: usize },
}

impl std::fmt::Display for PongError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PongError::NotReady { side } => write!(f, "side {side} not ready"),
            PongError::AlreadyFull { side } => write!(f, "side {side} already full"),
            PongError::BufferOverflow { side, capacity, requested } => {
                write!(f, "side {side} overflow: {requested}/{capacity}")
            }
        }
    }
}

impl std::error::Error for PongError {}

#[derive(Debug, Clone)]
pub struct PingPongBuffer {
    buf_a: Vec<u8>,
    buf_b: Vec<u8>,
    len_a: usize,
    len_b: usize,
    write_side: BufferSide,
    a_valid: bool,
    b_valid: bool,
    swap_count: u64,
    total_written: u64,
    total_read: u64,
}

impl PingPongBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf_a: vec![0; capacity],
            buf_b: vec![0; capacity],
            len_a: 0,
            len_b: 0,
            write_side: BufferSide::A,
            a_valid: false,
            b_valid: false,
            swap_count: 0,
            total_written: 0,
            total_read: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf_a.len()
    }

    pub fn write_side(&self) -> BufferSide {
        self.write_side
    }

    pub fn read_side(&self) -> BufferSide {
        match self.write_side {
            BufferSide::A => BufferSide::B,
            BufferSide::B => BufferSide::A,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, PongError> {
        let (buf, len, side) = match self.write_side {
            BufferSide::A => (&mut self.buf_a, &mut self.len_a, BufferSide::A),
            BufferSide::B => (&mut self.buf_b, &mut self.len_b, BufferSide::B),
        };
        if *len + data.len() > buf.len() {
            return Err(PongError::BufferOverflow {
                side,
                capacity: buf.len(),
                requested: *len + data.len(),
            });
        }
        let to_write = data.len();
        buf[*len..*len + to_write].copy_from_slice(data);
        *len += to_write;
        self.total_written += to_write as u64;
        Ok(to_write)
    }

    pub fn mark_valid(&mut self) {
        match self.write_side {
            BufferSide::A => self.a_valid = true,
            BufferSide::B => self.b_valid = true,
        }
    }

    pub fn swap(&mut self) -> Result<(), PongError> {
        let read_side = self.read_side();
        let read_valid = match read_side {
            BufferSide::A => self.a_valid,
            BufferSide::B => self.b_valid,
        };
        let write_valid = match self.write_side {
            BufferSide::A => self.a_valid,
            BufferSide::B => self.b_valid,
        };
        if !write_valid {
            return Err(PongError::NotReady { side: self.write_side });
        }
        self.write_side = match self.write_side {
            BufferSide::A => BufferSide::B,
            BufferSide::B => BufferSide::A,
        };
        self.swap_count += 1;
        let _ = (read_side, read_valid);
        Ok(())
    }

    pub fn read(&mut self) -> Option<&[u8]> {
        let read_side = self.read_side();
        let valid = match read_side {
            BufferSide::A => self.a_valid,
            BufferSide::B => self.b_valid,
        };
        if !valid {
            return None;
        }
        let (buf, len) = match read_side {
            BufferSide::A => (&self.buf_a, self.len_a),
            BufferSide::B => (&self.buf_b, self.len_b),
        };
        self.total_read += len as u64;
        Some(&buf[..len])
    }

    pub fn consume(&mut self) {
        let read_side = self.read_side();
        match read_side {
            BufferSide::A => {
                self.len_a = 0;
                self.a_valid = false;
            }
            BufferSide::B => {
                self.len_b = 0;
                self.b_valid = false;
            }
        }
    }

    pub fn swap_count(&self) -> u64 {
        self.swap_count
    }

    pub fn total_written(&self) -> u64 {
        self.total_written
    }

    pub fn total_read(&self) -> u64 {
        self.total_read
    }

    pub fn is_write_empty(&self) -> bool {
        match self.write_side {
            BufferSide::A => self.len_a == 0,
            BufferSide::B => self.len_b == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer() {
        let pp = PingPongBuffer::new(64);
        assert_eq!(pp.capacity(), 64);
        assert_eq!(pp.write_side(), BufferSide::A);
        assert!(pp.is_write_empty());
    }

    #[test]
    fn side_display() {
        assert_eq!(BufferSide::A.to_string(), "A");
        assert_eq!(BufferSide::B.to_string(), "B");
    }

    #[test]
    fn write_and_read() {
        let mut pp = PingPongBuffer::new(64);
        pp.write(b"hello").unwrap();
        pp.mark_valid();
        pp.swap().unwrap();
        let data = pp.read().unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn write_overflow() {
        let mut pp = PingPongBuffer::new(4);
        let err = pp.write(b"hello").unwrap_err();
        assert!(matches!(err, PongError::BufferOverflow { .. }));
    }

    #[test]
    fn swap_without_valid() {
        let mut pp = PingPongBuffer::new(64);
        let err = pp.swap().unwrap_err();
        assert!(matches!(err, PongError::NotReady { .. }));
    }

    #[test]
    fn double_swap_cycle() {
        let mut pp = PingPongBuffer::new(64);
        pp.write(b"first").unwrap();
        pp.mark_valid();
        pp.swap().unwrap();
        assert_eq!(pp.read().unwrap(), b"first");
        pp.consume();
        pp.write(b"second").unwrap();
        pp.mark_valid();
        pp.swap().unwrap();
        assert_eq!(pp.read().unwrap(), b"second");
        assert_eq!(pp.swap_count(), 2);
    }

    #[test]
    fn consume_clears() {
        let mut pp = PingPongBuffer::new(64);
        pp.write(b"x").unwrap();
        pp.mark_valid();
        pp.swap().unwrap();
        pp.consume();
        assert!(pp.read().is_none());
    }

    #[test]
    fn read_before_swap_none() {
        let mut pp = PingPongBuffer::new(64);
        pp.write(b"data").unwrap();
        assert!(pp.read().is_none());
    }

    #[test]
    fn stats() {
        let mut pp = PingPongBuffer::new(64);
        pp.write(b"abc").unwrap();
        pp.mark_valid();
        pp.swap().unwrap();
        pp.read().unwrap();
        assert_eq!(pp.total_written(), 3);
        assert_eq!(pp.total_read(), 3);
        assert_eq!(pp.swap_count(), 1);
    }

    #[test]
    fn error_display() {
        assert!(PongError::NotReady { side: BufferSide::A }.to_string().contains("not ready"));
        assert!(PongError::BufferOverflow { side: BufferSide::B, capacity: 4, requested: 8 }.to_string().contains("8/4"));
    }
}
