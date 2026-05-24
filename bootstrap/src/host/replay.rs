#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayOp {
    Read,
    Write,
}

impl std::fmt::Display for ReplayOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayOp::Read => write!(f, "rd"),
            ReplayOp::Write => write!(f, "wr"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplayEntry {
    pub op: ReplayOp,
    pub addr: u32,
    pub value: u32,
    pub timestamp_us: u64,
}

impl ReplayEntry {
    pub fn read(addr: u32, value: u32, ts: u64) -> Self {
        Self { op: ReplayOp::Read, addr, value, timestamp_us: ts }
    }

    pub fn write(addr: u32, value: u32, ts: u64) -> Self {
        Self { op: ReplayOp::Write, addr, value, timestamp_us: ts }
    }
}

#[derive(Debug, Clone)]
pub struct ReplayBuffer {
    entries: Vec<ReplayEntry>,
    head: usize,
    len: usize,
    recording: bool,
    total_captured: u64,
}

impl ReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: vec![ReplayEntry::read(0, 0, 0); capacity],
            head: 0,
            len: 0,
            recording: true,
            total_captured: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn start(&mut self) {
        self.recording = true;
    }

    pub fn stop(&mut self) {
        self.recording = false;
    }

    pub fn capture(&mut self, entry: ReplayEntry) {
        if !self.recording {
            return;
        }
        if self.len < self.entries.len() {
            let idx = (self.head + self.len) % self.entries.len();
            self.entries[idx] = entry;
            self.len += 1;
        } else {
            self.entries[self.head] = entry;
            self.head = (self.head + 1) % self.entries.len();
        }
        self.total_captured += 1;
    }

    pub fn capture_read(&mut self, addr: u32, value: u32, ts: u64) {
        self.capture(ReplayEntry::read(addr, value, ts));
    }

    pub fn capture_write(&mut self, addr: u32, value: u32, ts: u64) {
        self.capture(ReplayEntry::write(addr, value, ts));
    }

    pub fn iter(&self) -> ReplayIter<'_> {
        ReplayIter { buf: self, pos: 0 }
    }

    pub fn drain(&mut self) -> Vec<ReplayEntry> {
        let entries: Vec<ReplayEntry> = self.iter().cloned().collect();
        self.head = 0;
        self.len = 0;
        entries
    }

    pub fn filter_by_addr(&self, addr: u32) -> Vec<&ReplayEntry> {
        self.iter().filter(|e| e.addr == addr).collect()
    }

    pub fn filter_by_op(&self, op: ReplayOp) -> Vec<&ReplayEntry> {
        self.iter().filter(|e| e.op == op).collect()
    }

    pub fn filter_by_range(&self, start_us: u64, end_us: u64) -> Vec<&ReplayEntry> {
        self.iter().filter(|e| e.timestamp_us >= start_us && e.timestamp_us <= end_us).collect()
    }

    pub fn total_captured(&self) -> u64 {
        self.total_captured
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }
}

#[derive(Debug)]
pub struct ReplayIter<'a> {
    buf: &'a ReplayBuffer,
    pos: usize,
}

impl<'a> Iterator for ReplayIter<'a> {
    type Item = &'a ReplayEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.buf.len {
            return None;
        }
        let idx = (self.buf.head + self.pos) % self.buf.entries.len();
        self.pos += 1;
        Some(&self.buf.entries[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_display() {
        assert_eq!(ReplayOp::Read.to_string(), "rd");
        assert_eq!(ReplayOp::Write.to_string(), "wr");
    }

    #[test]
    fn new_buffer() {
        let rb = ReplayBuffer::new(8);
        assert_eq!(rb.capacity(), 8);
        assert!(rb.is_empty());
        assert!(rb.is_recording());
    }

    #[test]
    fn capture_and_iter() {
        let mut rb = ReplayBuffer::new(4);
        rb.capture_read(0x100, 0x42, 1);
        rb.capture_write(0x200, 0xFF, 2);
        assert_eq!(rb.len(), 2);
        let entries: Vec<_> = rb.iter().collect();
        assert_eq!(entries[0].addr, 0x100);
        assert_eq!(entries[1].op, ReplayOp::Write);
    }

    #[test]
    fn capture_wraps() {
        let mut rb = ReplayBuffer::new(3);
        rb.capture_read(1, 0, 1);
        rb.capture_read(2, 0, 2);
        rb.capture_read(3, 0, 3);
        rb.capture_read(4, 0, 4);
        assert_eq!(rb.len(), 3);
        let entries: Vec<_> = rb.iter().collect();
        assert_eq!(entries[0].addr, 2);
        assert_eq!(entries[2].addr, 4);
        assert_eq!(rb.total_captured(), 4);
    }

    #[test]
    fn stop_stops_capture() {
        let mut rb = ReplayBuffer::new(4);
        rb.stop();
        rb.capture_read(0, 0, 0);
        assert_eq!(rb.len(), 0);
        assert!(!rb.is_recording());
    }

    #[test]
    fn start_resumes() {
        let mut rb = ReplayBuffer::new(4);
        rb.stop();
        rb.start();
        rb.capture_read(0, 0, 0);
        assert_eq!(rb.len(), 1);
    }

    #[test]
    fn filter_by_addr() {
        let mut rb = ReplayBuffer::new(8);
        rb.capture_read(0x100, 1, 1);
        rb.capture_read(0x200, 2, 2);
        rb.capture_read(0x100, 3, 3);
        assert_eq!(rb.filter_by_addr(0x100).len(), 2);
    }

    #[test]
    fn filter_by_op() {
        let mut rb = ReplayBuffer::new(8);
        rb.capture_read(0, 0, 1);
        rb.capture_write(0, 0, 2);
        rb.capture_read(0, 0, 3);
        assert_eq!(rb.filter_by_op(ReplayOp::Write).len(), 1);
        assert_eq!(rb.filter_by_op(ReplayOp::Read).len(), 2);
    }

    #[test]
    fn filter_by_range() {
        let mut rb = ReplayBuffer::new(8);
        rb.capture_read(0, 0, 10);
        rb.capture_read(0, 0, 20);
        rb.capture_read(0, 0, 30);
        assert_eq!(rb.filter_by_range(15, 25).len(), 1);
    }

    #[test]
    fn drain() {
        let mut rb = ReplayBuffer::new(4);
        rb.capture_read(0, 0, 0);
        rb.capture_read(0, 0, 0);
        let entries = rb.drain();
        assert_eq!(entries.len(), 2);
        assert!(rb.is_empty());
    }

    #[test]
    fn clear() {
        let mut rb = ReplayBuffer::new(4);
        rb.capture_read(0, 0, 0);
        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.total_captured(), 1);
    }
}
