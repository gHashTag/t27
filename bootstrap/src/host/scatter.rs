#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub addr: u32,
    pub data: Vec<u8>,
}

impl Segment {
    pub fn new(addr: u32, data: Vec<u8>) -> Self {
        Self { addr, data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn end(&self) -> u32 {
        self.addr + self.data.len() as u32
    }

    pub fn overlaps(&self, other: &Segment) -> bool {
        self.addr < other.end() && other.addr < self.end()
    }

    pub fn is_contiguous_with(&self, other: &Segment) -> bool {
        self.end() == other.addr || other.end() == self.addr
    }

    pub fn merge(&self, other: &Segment) -> Option<Segment> {
        if self.end() == other.addr {
            let mut data = self.data.clone();
            data.extend_from_slice(&other.data);
            return Some(Segment::new(self.addr, data));
        }
        if other.end() == self.addr {
            let mut data = other.data.clone();
            data.extend_from_slice(&self.data);
            return Some(Segment::new(other.addr, data));
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScatterError {
    EmptySegment { index: usize },
    Overlap { a_addr: u32, b_addr: u32 },
}

impl std::fmt::Display for ScatterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScatterError::EmptySegment { index } => write!(f, "empty segment at index {index}"),
            ScatterError::Overlap { a_addr, b_addr } => {
                write!(f, "overlap: 0x{a_addr:X} and 0x{b_addr:X}")
            }
        }
    }
}

impl std::error::Error for ScatterError {}

#[derive(Debug, Clone)]
pub struct ScatterWriter {
    segments: Vec<Segment>,
    total_bytes: u64,
    total_segments: u64,
}

impl ScatterWriter {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            total_bytes: 0,
            total_segments: 0,
        }
    }

    pub fn add(&mut self, seg: Segment) -> Result<(), ScatterError> {
        if seg.is_empty() {
            return Err(ScatterError::EmptySegment { index: self.segments.len() });
        }
        for existing in &self.segments {
            if seg.overlaps(existing) {
                return Err(ScatterError::Overlap {
                    a_addr: seg.addr,
                    b_addr: existing.addr,
                });
            }
        }
        self.total_bytes += seg.len() as u64;
        self.total_segments += 1;
        self.segments.push(seg);
        self.segments.sort_by_key(|s| s.addr);
        Ok(())
    }

    pub fn coalesce(&self) -> Vec<Segment> {
        if self.segments.is_empty() {
            return Vec::new();
        }
        let mut result = vec![self.segments[0].clone()];
        for seg in &self.segments[1..] {
            let last = result.last().unwrap();
            if let Some(merged) = last.merge(seg) {
                *result.last_mut().unwrap() = merged;
            } else {
                result.push(seg.clone());
            }
        }
        result
    }

    pub fn plan(&self) -> WritePlan {
        let coalesced = self.coalesce();
        let total_bytes: usize = coalesced.iter().map(|s| s.len()).sum();
        WritePlan {
            segments: coalesced,
            total_bytes,
            total_segments: self.segments.len(),
        }
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }
}

impl Default for ScatterWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WritePlan {
    pub segments: Vec<Segment>,
    pub total_bytes: usize,
    pub total_segments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_basic() {
        let s = Segment::new(0x100, vec![1, 2, 3]);
        assert_eq!(s.len(), 3);
        assert_eq!(s.end(), 0x103);
        assert!(!s.is_empty());
    }

    #[test]
    fn segment_overlap() {
        let a = Segment::new(0x100, vec![0; 4]);
        let b = Segment::new(0x102, vec![0; 4]);
        assert!(a.overlaps(&b));
        let c = Segment::new(0x104, vec![0; 4]);
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn segment_contiguous() {
        let a = Segment::new(0x100, vec![1, 2]);
        let b = Segment::new(0x102, vec![3, 4]);
        assert!(a.is_contiguous_with(&b));
    }

    #[test]
    fn segment_merge() {
        let a = Segment::new(0x100, vec![1, 2]);
        let b = Segment::new(0x102, vec![3, 4]);
        let merged = a.merge(&b).unwrap();
        assert_eq!(merged.addr, 0x100);
        assert_eq!(merged.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn segment_merge_non_contiguous() {
        let a = Segment::new(0x100, vec![1]);
        let b = Segment::new(0x200, vec![2]);
        assert!(a.merge(&b).is_none());
    }

    #[test]
    fn add_sorted() {
        let mut sw = ScatterWriter::new();
        sw.add(Segment::new(0x200, vec![1])).unwrap();
        sw.add(Segment::new(0x100, vec![2])).unwrap();
        assert_eq!(sw.segments()[0].addr, 0x100);
        assert_eq!(sw.segments()[1].addr, 0x200);
    }

    #[test]
    fn add_empty_segment() {
        let mut sw = ScatterWriter::new();
        let err = sw.add(Segment::new(0, vec![])).unwrap_err();
        assert!(matches!(err, ScatterError::EmptySegment { .. }));
    }

    #[test]
    fn add_overlap() {
        let mut sw = ScatterWriter::new();
        sw.add(Segment::new(0x100, vec![0; 8])).unwrap();
        let err = sw.add(Segment::new(0x104, vec![0; 8])).unwrap_err();
        assert!(matches!(err, ScatterError::Overlap { .. }));
    }

    #[test]
    fn coalesce_adjacent() {
        let mut sw = ScatterWriter::new();
        sw.add(Segment::new(0x100, vec![1, 2])).unwrap();
        sw.add(Segment::new(0x102, vec![3, 4])).unwrap();
        let coalesced = sw.coalesce();
        assert_eq!(coalesced.len(), 1);
        assert_eq!(coalesced[0].data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn coalesce_gap_keeps_separate() {
        let mut sw = ScatterWriter::new();
        sw.add(Segment::new(0x100, vec![1])).unwrap();
        sw.add(Segment::new(0x200, vec![2])).unwrap();
        assert_eq!(sw.coalesce().len(), 2);
    }

    #[test]
    fn plan() {
        let mut sw = ScatterWriter::new();
        sw.add(Segment::new(0x100, vec![1, 2])).unwrap();
        sw.add(Segment::new(0x102, vec![3, 4])).unwrap();
        sw.add(Segment::new(0x200, vec![5])).unwrap();
        let plan = sw.plan();
        assert_eq!(plan.segments.len(), 2);
        assert_eq!(plan.total_bytes, 5);
        assert_eq!(plan.total_segments, 3);
    }

    #[test]
    fn total_bytes() {
        let mut sw = ScatterWriter::new();
        sw.add(Segment::new(0, vec![1, 2, 3])).unwrap();
        sw.add(Segment::new(0x100, vec![4, 5])).unwrap();
        assert_eq!(sw.total_bytes(), 5);
    }

    #[test]
    fn clear() {
        let mut sw = ScatterWriter::new();
        sw.add(Segment::new(0, vec![1])).unwrap();
        sw.clear();
        assert_eq!(sw.segment_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(ScatterError::EmptySegment { index: 3 }.to_string().contains("3"));
        assert!(ScatterError::Overlap { a_addr: 0x100, b_addr: 0x200 }.to_string().contains("overlap"));
    }
}
