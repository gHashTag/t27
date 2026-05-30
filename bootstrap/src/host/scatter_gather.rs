#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SgError {
    EmptyDescriptor,
    ZeroLength,
    OverlappingRanges { a_start: u64, a_end: u64, b_start: u64, b_end: u64 },
    ExceedsBufferSize,
}

impl std::fmt::Display for SgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SgError::EmptyDescriptor => write!(f, "empty scatter/gather descriptor"),
            SgError::ZeroLength => write!(f, "zero-length segment"),
            SgError::OverlappingRanges { a_start, a_end, b_start, b_end } => {
                write!(f, "overlapping ranges: [{a_start:#x},{a_end:#x}) and [{b_start:#x},{b_end:#x})")
            }
            SgError::ExceedsBufferSize => write!(f, "segment exceeds buffer size"),
        }
    }
}

impl std::error::Error for SgError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SgSegment {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub length: u32,
}

impl SgSegment {
    pub fn new(src: u64, dst: u64, len: u32) -> Self {
        Self { src_offset: src, dst_offset: dst, length: len }
    }

    pub fn src_end(&self) -> u64 {
        self.src_offset + self.length as u64
    }

    pub fn dst_end(&self) -> u64 {
        self.dst_offset + self.length as u64
    }

    pub fn overlaps_src(&self, other: &SgSegment) -> bool {
        self.src_offset < other.src_end() && other.src_offset < self.src_end()
    }

    pub fn overlaps_dst(&self, other: &SgSegment) -> bool {
        self.dst_offset < other.dst_end() && other.dst_offset < self.dst_end()
    }
}

#[derive(Debug, Clone)]
pub struct SgDescriptor {
    pub segments: Vec<SgSegment>,
}

impl SgDescriptor {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    pub fn add(&mut self, src: u64, dst: u64, len: u32) -> Result<(), SgError> {
        if len == 0 {
            return Err(SgError::ZeroLength);
        }
        let seg = SgSegment::new(src, dst, len);
        self.segments.push(seg);
        Ok(())
    }

    pub fn total_bytes(&self) -> u64 {
        self.segments.iter().map(|s| s.length as u64).sum()
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn validate_no_overlap(&self) -> Result<(), SgError> {
        for i in 0..self.segments.len() {
            for j in (i + 1)..self.segments.len() {
                let a = &self.segments[i];
                let b = &self.segments[j];
                if a.overlaps_src(b) {
                    return Err(SgError::OverlappingRanges {
                        a_start: a.src_offset,
                        a_end: a.src_end(),
                        b_start: b.src_offset,
                        b_end: b.src_end(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn execute(&self, src_buf: &[u8], dst_buf: &mut [u8]) -> Result<u64, SgError> {
        if self.segments.is_empty() {
            return Err(SgError::EmptyDescriptor);
        }
        let mut copied: u64 = 0;
        for seg in &self.segments {
            let src_start = seg.src_offset as usize;
            let src_end = src_start + seg.length as usize;
            let dst_start = seg.dst_offset as usize;
            let dst_end = dst_start + seg.length as usize;
            if src_end > src_buf.len() || dst_end > dst_buf.len() {
                return Err(SgError::ExceedsBufferSize);
            }
            dst_buf[dst_start..dst_end].copy_from_slice(&src_buf[src_start..src_end]);
            copied += seg.length as u64;
        }
        Ok(copied)
    }
}

impl Default for SgDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_descriptor_is_empty() {
        let d = SgDescriptor::new();
        assert!(d.is_empty());
        assert_eq!(d.segment_count(), 0);
        assert_eq!(d.total_bytes(), 0);
    }

    #[test]
    fn add_segment() {
        let mut d = SgDescriptor::new();
        d.add(0, 100, 64).unwrap();
        assert_eq!(d.segment_count(), 1);
        assert_eq!(d.total_bytes(), 64);
    }

    #[test]
    fn add_zero_length_errors() {
        let mut d = SgDescriptor::new();
        assert!(matches!(d.add(0, 0, 0), Err(SgError::ZeroLength)));
    }

    #[test]
    fn segment_src_end() {
        let s = SgSegment::new(100, 200, 50);
        assert_eq!(s.src_end(), 150);
        assert_eq!(s.dst_end(), 250);
    }

    #[test]
    fn segment_no_overlap() {
        let a = SgSegment::new(0, 100, 50);
        let b = SgSegment::new(50, 200, 50);
        assert!(!a.overlaps_src(&b));
        assert!(!a.overlaps_dst(&b));
    }

    #[test]
    fn segment_overlaps_src() {
        let a = SgSegment::new(0, 100, 100);
        let b = SgSegment::new(50, 200, 100);
        assert!(a.overlaps_src(&b));
    }

    #[test]
    fn segment_overlaps_dst() {
        let a = SgSegment::new(0, 0, 100);
        let b = SgSegment::new(200, 50, 100);
        assert!(a.overlaps_dst(&b));
    }

    #[test]
    fn validate_no_overlap_pass() {
        let mut d = SgDescriptor::new();
        d.add(0, 0, 64).unwrap();
        d.add(64, 128, 64).unwrap();
        assert!(d.validate_no_overlap().is_ok());
    }

    #[test]
    fn validate_overlap_fails() {
        let mut d = SgDescriptor::new();
        d.add(0, 0, 128).unwrap();
        d.add(64, 256, 64).unwrap();
        assert!(matches!(d.validate_no_overlap(), Err(SgError::OverlappingRanges { .. })));
    }

    #[test]
    fn execute_single_segment() {
        let mut d = SgDescriptor::new();
        d.add(0, 0, 8).unwrap();
        let src = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22];
        let mut dst = vec![0u8; 8];
        let copied = d.execute(&src, &mut dst).unwrap();
        assert_eq!(copied, 8);
        assert_eq!(dst, src);
    }

    #[test]
    fn execute_multi_segment() {
        let mut d = SgDescriptor::new();
        d.add(0, 8, 4).unwrap();
        d.add(4, 0, 4).unwrap();
        let src = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let mut dst = vec![0u8; 12];
        let copied = d.execute(&src, &mut dst).unwrap();
        assert_eq!(copied, 8);
        assert_eq!(&dst[0..4], &[5, 6, 7, 8]);
        assert_eq!(&dst[8..12], &[1, 2, 3, 4]);
    }

    #[test]
    fn execute_exceeds_src_errors() {
        let mut d = SgDescriptor::new();
        d.add(0, 0, 100).unwrap();
        let src = vec![0u8; 10];
        let mut dst = vec![0u8; 200];
        assert!(matches!(d.execute(&src, &mut dst), Err(SgError::ExceedsBufferSize)));
    }

    #[test]
    fn execute_exceeds_dst_errors() {
        let mut d = SgDescriptor::new();
        d.add(0, 0, 10).unwrap();
        let src = vec![0u8; 100];
        let mut dst = vec![0u8; 5];
        assert!(matches!(d.execute(&src, &mut dst), Err(SgError::ExceedsBufferSize)));
    }

    #[test]
    fn execute_empty_descriptor_errors() {
        let d = SgDescriptor::new();
        let src = vec![0u8; 10];
        let mut dst = vec![0u8; 10];
        assert!(matches!(d.execute(&src, &mut dst), Err(SgError::EmptyDescriptor)));
    }

    #[test]
    fn total_bytes_multi() {
        let mut d = SgDescriptor::new();
        d.add(0, 0, 64).unwrap();
        d.add(64, 128, 32).unwrap();
        d.add(96, 256, 16).unwrap();
        assert_eq!(d.total_bytes(), 112);
    }

    #[test]
    fn error_display() {
        let e = SgError::ZeroLength;
        assert!(e.to_string().contains("zero"));
        let e = SgError::OverlappingRanges { a_start: 0, a_end: 10, b_start: 5, b_end: 15 };
        assert!(e.to_string().contains("overlapping"));
        let e = SgError::EmptyDescriptor;
        assert!(e.to_string().contains("empty"));
    }
}
