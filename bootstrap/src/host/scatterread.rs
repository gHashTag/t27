use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScatterReadError {
    SegmentOutOfBounds { seg: usize, offset: usize, len: usize, seg_size: usize },
    NoSegments,
    Overlap { seg_a: usize, seg_b: usize },
}

impl std::fmt::Display for ScatterReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScatterReadError::SegmentOutOfBounds { seg, offset, len, seg_size } => {
                write!(f, "seg {seg}: [{offset}..{}] > {seg_size}", offset + len)
            }
            ScatterReadError::NoSegments => write!(f, "no segments"),
            ScatterReadError::Overlap { seg_a, seg_b } => write!(f, "seg {seg_a} overlaps {seg_b}"),
        }
    }
}

impl std::error::Error for ScatterReadError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub id: usize,
    pub base_addr: usize,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct ScatterRead {
    segments: Vec<Segment>,
    memory: BTreeMap<usize, Vec<u8>>,
}

impl ScatterRead {
    pub fn new() -> Self {
        Self { segments: Vec::new(), memory: BTreeMap::new() }
    }

    pub fn add_segment(&mut self, id: usize, base_addr: usize, size: usize) {
        self.memory.insert(id, vec![0u8; size]);
        self.segments.push(Segment { id, base_addr, size });
    }

    pub fn write_memory(&mut self, seg_id: usize, offset: usize, data: &[u8]) -> Result<(), ScatterReadError> {
        let seg = self.segments.iter().find(|s| s.id == seg_id)
            .ok_or(ScatterReadError::SegmentOutOfBounds {
                seg: seg_id, offset, len: data.len(), seg_size: 0,
            })?;
        if offset + data.len() > seg.size {
            return Err(ScatterReadError::SegmentOutOfBounds {
                seg: seg_id, offset, len: data.len(), seg_size: seg.size,
            });
        }
        let mem = self.memory.get_mut(&seg_id).unwrap();
        mem[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    pub fn read_segment(&self, seg_id: usize, offset: usize, len: usize) -> Result<Vec<u8>, ScatterReadError> {
        let seg = self.segments.iter().find(|s| s.id == seg_id)
            .ok_or(ScatterReadError::SegmentOutOfBounds {
                seg: seg_id, offset, len, seg_size: 0,
            })?;
        if offset + len > seg.size {
            return Err(ScatterReadError::SegmentOutOfBounds {
                seg: seg_id, offset, len, seg_size: seg.size,
            });
        }
        let mem = self.memory.get(&seg_id).unwrap();
        Ok(mem[offset..offset + len].to_vec())
    }

    pub fn gather(&self, requests: &[(usize, usize, usize)]) -> Result<Vec<u8>, ScatterReadError> {
        if requests.is_empty() {
            return Err(ScatterReadError::NoSegments);
        }
        let mut result = Vec::new();
        for &(seg_id, offset, len) in requests {
            let data = self.read_segment(seg_id, offset, len)?;
            result.extend_from_slice(&data);
        }
        Ok(result)
    }

    pub fn scatter(&mut self, data: &[u8], destinations: &[(usize, usize)]) -> Result<(), ScatterReadError> {
        if destinations.is_empty() { return Ok(()); }
        let per_seg = (data.len() + destinations.len() - 1) / destinations.len();
        let mut pos = 0;
        for &(seg_id, offset) in destinations {
            if pos >= data.len() { break; }
            let chunk = per_seg.min(data.len() - pos);
            self.write_memory(seg_id, offset, &data[pos..pos + chunk])?;
            pos += chunk;
        }
        Ok(())
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn total_size(&self) -> usize {
        self.segments.iter().map(|s| s.size).sum()
    }

    pub fn segment(&self, id: usize) -> Option<&Segment> {
        self.segments.iter().find(|s| s.id == id)
    }

    pub fn linear_addr(&self, seg_id: usize, offset: usize) -> Option<usize> {
        self.segments.iter().find(|s| s.id == seg_id)
            .map(|s| s.base_addr + offset)
    }
}

impl Default for ScatterRead {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scatter() {
        let sr = ScatterRead::new();
        assert_eq!(sr.segment_count(), 0);
        assert_eq!(sr.total_size(), 0);
    }

    #[test]
    fn add_segment() {
        let mut sr = ScatterRead::new();
        sr.add_segment(0, 0x1000, 256);
        assert_eq!(sr.segment_count(), 1);
        assert_eq!(sr.total_size(), 256);
    }

    #[test]
    fn write_read_roundtrip() {
        let mut sr = ScatterRead::new();
        sr.add_segment(0, 0x1000, 64);
        sr.write_memory(0, 0, &[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
        let data = sr.read_segment(0, 0, 4).unwrap();
        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn write_out_of_bounds() {
        let mut sr = ScatterRead::new();
        sr.add_segment(0, 0x1000, 8);
        let err = sr.write_memory(0, 4, &[1, 2, 3, 4, 5]).unwrap_err();
        assert!(matches!(err, ScatterReadError::SegmentOutOfBounds { .. }));
    }

    #[test]
    fn read_out_of_bounds() {
        let sr = {
            let mut sr = ScatterRead::new();
            sr.add_segment(0, 0x1000, 8);
            sr
        };
        let err = sr.read_segment(0, 0, 16).unwrap_err();
        assert!(matches!(err, ScatterReadError::SegmentOutOfBounds { .. }));
    }

    #[test]
    fn gather_multi_segment() {
        let mut sr = ScatterRead::new();
        sr.add_segment(0, 0x1000, 16);
        sr.add_segment(1, 0x2000, 16);
        sr.write_memory(0, 0, &[1, 2, 3]).unwrap();
        sr.write_memory(1, 0, &[4, 5, 6]).unwrap();
        let data = sr.gather(&[(0, 0, 3), (1, 0, 3)]).unwrap();
        assert_eq!(data, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn gather_empty_requests() {
        let sr = ScatterRead::new();
        let err = sr.gather(&[]).unwrap_err();
        assert!(matches!(err, ScatterReadError::NoSegments));
    }

    #[test]
    fn scatter_write() {
        let mut sr = ScatterRead::new();
        sr.add_segment(0, 0x1000, 16);
        sr.add_segment(1, 0x2000, 16);
        let data: Vec<u8> = (0..8).collect();
        sr.scatter(&data, &[(0, 0), (1, 0)]).unwrap();
        let d0 = sr.read_segment(0, 0, 4).unwrap();
        let d1 = sr.read_segment(1, 0, 4).unwrap();
        assert_eq!(d0, vec![0, 1, 2, 3]);
        assert_eq!(d1, vec![4, 5, 6, 7]);
    }

    #[test]
    fn linear_addr() {
        let mut sr = ScatterRead::new();
        sr.add_segment(5, 0x8000, 256);
        assert_eq!(sr.linear_addr(5, 64), Some(0x8040));
    }

    #[test]
    fn segment_lookup() {
        let mut sr = ScatterRead::new();
        sr.add_segment(3, 0x1000, 128);
        let seg = sr.segment(3).unwrap();
        assert_eq!(seg.base_addr, 0x1000);
        assert_eq!(seg.size, 128);
    }

    #[test]
    fn missing_segment() {
        let sr = ScatterRead::new();
        assert_eq!(sr.segment(99), None);
        assert_eq!(sr.linear_addr(99, 0), None);
    }

    #[test]
    fn error_display() {
        let e = ScatterReadError::NoSegments;
        assert!(e.to_string().contains("no segments"));
    }
}
