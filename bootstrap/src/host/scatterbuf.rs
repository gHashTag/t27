use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Sb2Error {
    SegmentNotFound { id: u64 },
    SegmentExists { id: u64 },
    OffsetOutOfRange { id: u64, offset: usize, len: usize },
}

impl std::fmt::Display for Sb2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sb2Error::SegmentNotFound { id } => write!(f, "segment {id} not found"),
            Sb2Error::SegmentExists { id } => write!(f, "segment {id} exists"),
            Sb2Error::OffsetOutOfRange { id, offset, len } => write!(f, "offset {offset} out of range for segment {id} (len={len})"),
        }
    }
}

impl std::error::Error for Sb2Error {}

struct Segment {
    id: u64,
    data: Vec<u8>,
}

pub struct ScatterBuf {
    segments: BTreeMap<u64, Segment>,
    order: Vec<u64>,
    total_writes: u64,
    total_reads: u64,
    total_gathers: u64,
}

impl ScatterBuf {
    pub fn new() -> Self { Self { segments: BTreeMap::new(), order: Vec::new(), total_writes: 0, total_reads: 0, total_gathers: 0 } }

    pub fn create(&mut self, id: u64, size: usize) -> Result<(), Sb2Error> {
        if self.segments.contains_key(&id) { return Err(Sb2Error::SegmentExists { id }); }
        self.segments.insert(id, Segment { id, data: vec![0u8; size] });
        self.order.push(id);
        Ok(())
    }

    pub fn write(&mut self, id: u64, offset: usize, data: &[u8]) -> Result<(), Sb2Error> {
        let seg = self.segments.get_mut(&id).ok_or(Sb2Error::SegmentNotFound { id })?;
        let end = offset + data.len();
        if end > seg.data.len() { return Err(Sb2Error::OffsetOutOfRange { id, offset, len: seg.data.len() }); }
        seg.data[offset..end].copy_from_slice(data);
        self.total_writes += 1;
        Ok(())
    }

    pub fn read(&mut self, id: u64, offset: usize, len: usize) -> Result<Vec<u8>, Sb2Error> {
        let seg = self.segments.get(&id).ok_or(Sb2Error::SegmentNotFound { id })?;
        let end = offset + len;
        if end > seg.data.len() { return Err(Sb2Error::OffsetOutOfRange { id, offset, len: seg.data.len() }); }
        self.total_reads += 1;
        Ok(seg.data[offset..end].to_vec())
    }

    pub fn gather(&mut self, specs: &[(u64, usize, usize)]) -> Result<Vec<u8>, Sb2Error> {
        self.total_gathers += 1;
        let mut result = Vec::new();
        for &(id, offset, len) in specs {
            let seg = self.segments.get(&id).ok_or(Sb2Error::SegmentNotFound { id })?;
            let end = offset + len;
            if end > seg.data.len() { return Err(Sb2Error::OffsetOutOfRange { id, offset, len: seg.data.len() }); }
            result.extend_from_slice(&seg.data[offset..end]);
        }
        Ok(result)
    }

    pub fn scatter(&mut self, data: &[u8], specs: &[(u64, usize, usize)]) -> Result<(), Sb2Error> {
        let mut pos = 0;
        for &(id, offset, len) in specs {
            let end = offset + len;
            if pos + len > data.len() { break; }
            let seg = self.segments.get_mut(&id).ok_or(Sb2Error::SegmentNotFound { id })?;
            if end > seg.data.len() { return Err(Sb2Error::OffsetOutOfRange { id, offset, len: seg.data.len() }); }
            seg.data[offset..end].copy_from_slice(&data[pos..pos + len]);
            pos += len;
        }
        self.total_writes += 1;
        Ok(())
    }

    pub fn segment_len(&self, id: u64) -> Option<usize> { self.segments.get(&id).map(|s| s.data.len()) }
    pub fn segment_count(&self) -> usize { self.segments.len() }
    pub fn total_size(&self) -> usize { self.segments.values().map(|s| s.data.len()).sum() }
    pub fn total_writes(&self) -> u64 { self.total_writes }
    pub fn total_reads(&self) -> u64 { self.total_reads }
    pub fn total_gathers(&self) -> u64 { self.total_gathers }
}

impl Default for ScatterBuf {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buf() { assert_eq!(ScatterBuf::new().segment_count(), 0); }

    #[test]
    fn write_read() {
        let mut sb = ScatterBuf::new();
        sb.create(1, 16).unwrap();
        sb.write(1, 0, b"hello").unwrap();
        let data = sb.read(1, 0, 5).unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn gather() {
        let mut sb = ScatterBuf::new();
        sb.create(1, 8).unwrap(); sb.create(2, 8).unwrap();
        sb.write(1, 0, b"abcd").unwrap();
        sb.write(2, 0, b"efgh").unwrap();
        let data = sb.gather(&[(1, 0, 4), (2, 0, 4)]).unwrap();
        assert_eq!(data, b"abcdefgh");
    }

    #[test]
    fn scatter() {
        let mut sb = ScatterBuf::new();
        sb.create(1, 4).unwrap(); sb.create(2, 4).unwrap();
        sb.scatter(b"abcdefgh", &[(1, 0, 4), (2, 0, 4)]).unwrap();
        assert_eq!(sb.read(1, 0, 4).unwrap(), b"abcd");
        assert_eq!(sb.read(2, 0, 4).unwrap(), b"efgh");
    }

    #[test]
    fn offset_out_of_range() {
        let mut sb = ScatterBuf::new();
        sb.create(1, 4).unwrap();
        let err = sb.write(1, 0, b"12345").unwrap_err();
        assert!(matches!(err, Sb2Error::OffsetOutOfRange { .. }));
    }

    #[test]
    fn segment_not_found() {
        let mut sb = ScatterBuf::new();
        let err = sb.read(99, 0, 4).unwrap_err();
        assert!(matches!(err, Sb2Error::SegmentNotFound { .. }));
    }

    #[test]
    fn duplicate_segment() {
        let mut sb = ScatterBuf::new();
        sb.create(1, 4).unwrap();
        let err = sb.create(1, 4).unwrap_err();
        assert!(matches!(err, Sb2Error::SegmentExists { .. }));
    }

    #[test]
    fn total_size() {
        let mut sb = ScatterBuf::new();
        sb.create(1, 8).unwrap(); sb.create(2, 16).unwrap();
        assert_eq!(sb.total_size(), 24);
    }

    #[test]
    fn stats() {
        let mut sb = ScatterBuf::new();
        sb.create(1, 8).unwrap();
        sb.write(1, 0, b"ab").unwrap();
        sb.read(1, 0, 2).unwrap();
        sb.gather(&[(1, 0, 2)]).unwrap();
        assert_eq!(sb.total_writes(), 1);
        assert_eq!(sb.total_reads(), 1);
        assert_eq!(sb.total_gathers(), 1);
    }

    #[test]
    fn error_display() { assert!(Sb2Error::SegmentNotFound { id: 1 }.to_string().contains("1")); }
}
