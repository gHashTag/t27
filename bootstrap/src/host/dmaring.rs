#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDir {
    Tx,
    Rx,
}

impl std::fmt::Display for TransferDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferDir::Tx => write!(f, "tx"),
            TransferDir::Rx => write!(f, "rx"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DmaSegment {
    pub src: u32,
    pub dst: u32,
    pub len: u32,
    pub dir: TransferDir,
}

impl DmaSegment {
    pub fn tx(src: u32, dst: u32, len: u32) -> Self {
        Self { src, dst, len, dir: TransferDir::Tx }
    }

    pub fn rx(src: u32, dst: u32, len: u32) -> Self {
        Self { src, dst, len, dir: TransferDir::Rx }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingBuildError {
    Empty,
    Overlap { a_src: u32, b_src: u32 },
    ZeroLength { index: usize },
    MaxSegmentsExceeded { max: usize, got: usize },
}

impl std::fmt::Display for RingBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingBuildError::Empty => write!(f, "no segments"),
            RingBuildError::Overlap { a_src, b_src } => {
                write!(f, "overlap: 0x{a_src:X} and 0x{b_src:X}")
            }
            RingBuildError::ZeroLength { index } => write!(f, "segment {index} has zero length"),
            RingBuildError::MaxSegmentsExceeded { max, got } => {
                write!(f, "too many segments: {got}/{max}")
            }
        }
    }
}

impl std::error::Error for RingBuildError {}

#[derive(Debug, Clone)]
pub struct DmaRingConfig {
    pub segments: Vec<DmaSegment>,
    pub total_bytes: u64,
    pub tx_count: usize,
    pub rx_count: usize,
}

#[derive(Debug, Clone)]
pub struct DmaRingBuilder {
    segments: Vec<DmaSegment>,
    max_segments: usize,
}

impl DmaRingBuilder {
    pub fn new(max_segments: usize) -> Self {
        Self {
            segments: Vec::new(),
            max_segments,
        }
    }

    pub fn add_tx(&mut self, src: u32, dst: u32, len: u32) -> &mut Self {
        self.segments.push(DmaSegment::tx(src, dst, len));
        self
    }

    pub fn add_rx(&mut self, src: u32, dst: u32, len: u32) -> &mut Self {
        self.segments.push(DmaSegment::rx(src, dst, len));
        self
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn build(&self) -> Result<DmaRingConfig, RingBuildError> {
        if self.segments.is_empty() {
            return Err(RingBuildError::Empty);
        }
        if self.segments.len() > self.max_segments {
            return Err(RingBuildError::MaxSegmentsExceeded {
                max: self.max_segments,
                got: self.segments.len(),
            });
        }
        for (i, seg) in self.segments.iter().enumerate() {
            if seg.len == 0 {
                return Err(RingBuildError::ZeroLength { index: i });
            }
        }
        let total_bytes: u64 = self.segments.iter().map(|s| s.len as u64).sum();
        let tx_count = self.segments.iter().filter(|s| s.dir == TransferDir::Tx).count();
        let rx_count = self.segments.iter().filter(|s| s.dir == TransferDir::Rx).count();
        Ok(DmaRingConfig {
            segments: self.segments.clone(),
            total_bytes,
            tx_count,
            rx_count,
        })
    }

    pub fn build_checked(&self) -> Result<DmaRingConfig, RingBuildError> {
        let config = self.build()?;
        let mut sorted: Vec<&DmaSegment> = config.segments.iter().collect();
        sorted.sort_by_key(|s| s.src);
        for window in sorted.windows(2) {
            let a = window[0];
            let b = window[1];
            let a_end = a.src.checked_add(a.len);
            if a_end.map_or(false, |e| e > b.src) {
                return Err(RingBuildError::Overlap { a_src: a.src, b_src: b.src });
            }
        }
        Ok(config)
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_display() {
        assert_eq!(TransferDir::Tx.to_string(), "tx");
        assert_eq!(TransferDir::Rx.to_string(), "rx");
    }

    #[test]
    fn build_simple() {
        let cfg = DmaRingBuilder::new(16)
            .add_tx(0x1000, 0x2000, 64)
            .add_rx(0x3000, 0x4000, 128)
            .build().unwrap();
        assert_eq!(cfg.segments.len(), 2);
        assert_eq!(cfg.total_bytes, 192);
        assert_eq!(cfg.tx_count, 1);
        assert_eq!(cfg.rx_count, 1);
    }

    #[test]
    fn build_empty() {
        let err = DmaRingBuilder::new(16).build().unwrap_err();
        assert!(matches!(err, RingBuildError::Empty));
    }

    #[test]
    fn build_zero_length() {
        let err = DmaRingBuilder::new(16)
            .add_tx(0, 0, 0)
            .build().unwrap_err();
        assert!(matches!(err, RingBuildError::ZeroLength { index: 0 }));
    }

    #[test]
    fn build_max_exceeded() {
        let err = DmaRingBuilder::new(1)
            .add_tx(0, 0, 1)
            .add_tx(0, 0, 1)
            .build().unwrap_err();
        assert!(matches!(err, RingBuildError::MaxSegmentsExceeded { .. }));
    }

    #[test]
    fn build_checked_no_overlap() {
        let cfg = DmaRingBuilder::new(16)
            .add_tx(0x1000, 0, 64)
            .add_tx(0x2000, 0, 64)
            .build_checked().unwrap();
        assert_eq!(cfg.segments.len(), 2);
    }

    #[test]
    fn build_checked_overlap_detected() {
        let err = DmaRingBuilder::new(16)
            .add_tx(0x1000, 0, 0x100)
            .add_tx(0x1050, 0, 0x100)
            .build_checked().unwrap_err();
        assert!(matches!(err, RingBuildError::Overlap { .. }));
    }

    #[test]
    fn segment_tx_rx() {
        let tx = DmaSegment::tx(1, 2, 3);
        assert_eq!(tx.dir, TransferDir::Tx);
        let rx = DmaSegment::rx(4, 5, 6);
        assert_eq!(rx.dir, TransferDir::Rx);
    }

    #[test]
    fn multiple_segments() {
        let cfg = DmaRingBuilder::new(16)
            .add_tx(0x1000, 0x2000, 32)
            .add_tx(0x1040, 0x2040, 32)
            .add_rx(0x3000, 0x4000, 64)
            .build().unwrap();
        assert_eq!(cfg.segments.len(), 3);
        assert_eq!(cfg.tx_count, 2);
        assert_eq!(cfg.rx_count, 1);
        assert_eq!(cfg.total_bytes, 128);
    }

    #[test]
    fn segment_count() {
        let mut b = DmaRingBuilder::new(16);
        b.add_tx(0, 0, 1);
        b.add_rx(0, 0, 1);
        assert_eq!(b.segment_count(), 2);
    }

    #[test]
    fn clear() {
        let mut b = DmaRingBuilder::new(16);
        b.add_tx(0, 0, 1);
        b.clear();
        assert_eq!(b.segment_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(RingBuildError::Empty.to_string().contains("no segments"));
        assert!(RingBuildError::Overlap { a_src: 1, b_src: 2 }.to_string().contains("overlap"));
    }
}
