#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainError {
    Empty,
    IndexOutOfBounds { index: usize, len: usize },
    SegmentTooLarge { size: usize, max: usize },
    TotalTooLarge { size: usize, max: usize },
}

impl std::fmt::Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainError::Empty => write!(f, "buffer chain is empty"),
            ChainError::IndexOutOfBounds { index, len } => {
                write!(f, "index {index} out of bounds (len={len})")
            }
            ChainError::SegmentTooLarge { size, max } => {
                write!(f, "segment {size}B exceeds max {max}B")
            }
            ChainError::TotalTooLarge { size, max } => {
                write!(f, "total {size}B exceeds max {max}B")
            }
        }
    }
}

impl std::error::Error for ChainError {}

pub const MAX_SEGMENT: usize = 4096;
pub const MAX_CHAIN_BYTES: usize = 65536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufSegment {
    pub data: Vec<u8>,
    pub addr: u64,
}

impl BufSegment {
    pub fn new(data: Vec<u8>, addr: u64) -> Result<Self, ChainError> {
        if data.len() > MAX_SEGMENT {
            return Err(ChainError::SegmentTooLarge {
                size: data.len(),
                max: MAX_SEGMENT,
            });
        }
        Ok(Self { data, addr })
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn end_addr(&self) -> u64 {
        self.addr + self.data.len() as u64
    }
}

#[derive(Debug, Clone)]
pub struct BufChain {
    segments: Vec<BufSegment>,
}

impl BufChain {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push(&mut self, seg: BufSegment) -> Result<(), ChainError> {
        let new_total = self.total_bytes() + seg.len();
        if new_total > MAX_CHAIN_BYTES {
            return Err(ChainError::TotalTooLarge {
                size: new_total,
                max: MAX_CHAIN_BYTES,
            });
        }
        self.segments.push(seg);
        Ok(())
    }

    pub fn get(&self, index: usize) -> Result<&BufSegment, ChainError> {
        self.segments
            .get(index)
            .ok_or(ChainError::IndexOutOfBounds {
                index,
                len: self.segments.len(),
            })
    }

    pub fn segments(&self) -> &[BufSegment] {
        &self.segments
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn total_bytes(&self) -> usize {
        self.segments.iter().map(|s| s.len()).sum()
    }

    pub fn flatten(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.total_bytes());
        for seg in &self.segments {
            out.extend_from_slice(&seg.data);
        }
        out
    }

    pub fn addr_range(&self) -> Option<(u64, u64)> {
        if self.is_empty() {
            return None;
        }
        let start = self.segments.first().unwrap().addr;
        let end = self
            .segments
            .iter()
            .map(|s| s.end_addr())
            .max()
            .unwrap();
        Some((start, end))
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }

    pub fn stats(&self) -> ChainStats {
        ChainStats {
            segment_count: self.segments.len(),
            total_bytes: self.total_bytes(),
            max_segment: self.segments.iter().map(|s| s.len()).max().unwrap_or(0),
            min_segment: self.segments.iter().map(|s| s.len()).min().unwrap_or(0),
        }
    }
}

impl Default for BufChain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainStats {
    pub segment_count: usize,
    pub total_bytes: usize,
    pub max_segment: usize,
    pub min_segment: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(data: &[u8], addr: u64) -> BufSegment {
        BufSegment::new(data.to_vec(), addr).unwrap()
    }

    #[test]
    fn segment_new() {
        let s = seg(b"hello", 0x1000);
        assert_eq!(s.len(), 5);
        assert_eq!(s.addr, 0x1000);
        assert_eq!(s.end_addr(), 0x1005);
    }

    #[test]
    fn segment_too_large() {
        let big = vec![0u8; MAX_SEGMENT + 1];
        let err = BufSegment::new(big, 0).unwrap_err();
        assert!(matches!(err, ChainError::SegmentTooLarge { .. }));
    }

    #[test]
    fn segment_empty() {
        let s = BufSegment::new(vec![], 0).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn chain_push_and_get() {
        let mut c = BufChain::new();
        c.push(seg(b"aaa", 0x1000)).unwrap();
        c.push(seg(b"bbb", 0x2000)).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(0).unwrap().data, b"aaa");
        assert_eq!(c.get(1).unwrap().data, b"bbb");
    }

    #[test]
    fn chain_get_out_of_bounds() {
        let c = BufChain::new();
        let err = c.get(0).unwrap_err();
        assert!(matches!(err, ChainError::IndexOutOfBounds { .. }));
    }

    #[test]
    fn chain_flatten() {
        let mut c = BufChain::new();
        c.push(seg(b"hello", 0x1000)).unwrap();
        c.push(seg(b" world", 0x2000)).unwrap();
        assert_eq!(c.flatten(), b"hello world");
    }

    #[test]
    fn chain_total_bytes() {
        let mut c = BufChain::new();
        c.push(seg(b"abc", 0)).unwrap();
        c.push(seg(b"defgh", 0)).unwrap();
        assert_eq!(c.total_bytes(), 8);
    }

    #[test]
    fn chain_addr_range() {
        let mut c = BufChain::new();
        c.push(seg(b"a", 0x1000)).unwrap();
        c.push(seg(b"bb", 0x2000)).unwrap();
        let (start, end) = c.addr_range().unwrap();
        assert_eq!(start, 0x1000);
        assert_eq!(end, 0x2002);
    }

    #[test]
    fn chain_addr_range_empty() {
        let c = BufChain::new();
        assert!(c.addr_range().is_none());
    }

    #[test]
    fn chain_total_too_large() {
        let mut c = BufChain::new();
        let chunk = vec![0xAAu8; MAX_SEGMENT];
        let segments_needed = MAX_CHAIN_BYTES / MAX_SEGMENT;
        for i in 0..segments_needed {
            c.push(BufSegment::new(chunk.clone(), (i as u64) * MAX_SEGMENT as u64).unwrap()).unwrap();
        }
        let err = c.push(BufSegment::new(chunk.clone(), 0xF0000).unwrap()).unwrap_err();
        assert!(matches!(err, ChainError::TotalTooLarge { .. }));
    }

    #[test]
    fn chain_clear() {
        let mut c = BufChain::new();
        c.push(seg(b"x", 0)).unwrap();
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn chain_stats() {
        let mut c = BufChain::new();
        c.push(seg(b"aaa", 0)).unwrap();
        c.push(seg(b"bbbbb", 0)).unwrap();
        let s = c.stats();
        assert_eq!(s.segment_count, 2);
        assert_eq!(s.total_bytes, 8);
        assert_eq!(s.max_segment, 5);
        assert_eq!(s.min_segment, 3);
    }

    #[test]
    fn default_is_empty() {
        let c = BufChain::default();
        assert!(c.is_empty());
    }

    #[test]
    fn error_display() {
        assert!(ChainError::Empty.to_string().contains("empty"));
        assert!(ChainError::SegmentTooLarge { size: 5000, max: 4096 }.to_string().contains("5000"));
    }
}
