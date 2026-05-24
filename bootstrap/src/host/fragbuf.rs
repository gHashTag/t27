use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FbError {
    Overlap { offset: u64 },
    AlreadyComplete,
}

impl std::fmt::Display for FbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FbError::Overlap { offset } => write!(f, "fragment overlap at {offset}"),
            FbError::AlreadyComplete => write!(f, "already complete"),
        }
    }
}

impl std::error::Error for FbError {}

struct Fragment {
    offset: u64,
    data: Vec<u8>,
}

pub struct FragBuf {
    fragments: BTreeMap<u64, Fragment>,
    total_len: u64,
    received: u64,
    total_fragments: u64,
    total_reassembled: u64,
}

impl FragBuf {
    pub fn new(total_len: u64) -> Self { Self { fragments: BTreeMap::new(), total_len, received: 0, total_fragments: 0, total_reassembled: 0 } }

    pub fn insert(&mut self, offset: u64, data: Vec<u8>) -> Result<(), FbError> {
        if self.is_complete() { return Err(FbError::AlreadyComplete); }
        let end = offset + data.len() as u64;
        if end > self.total_len { return Err(FbError::Overlap { offset }); }
        for (_, frag) in &self.fragments {
            let frag_end = frag.offset + frag.data.len() as u64;
            if offset < frag_end && end > frag.offset { return Err(FbError::Overlap { offset }); }
        }
        self.received += data.len() as u64;
        self.fragments.insert(offset, Fragment { offset, data });
        self.total_fragments += 1;
        Ok(())
    }

    pub fn is_complete(&self) -> bool { self.received >= self.total_len }

    pub fn reassemble(&mut self) -> Option<Vec<u8>> {
        if !self.is_complete() { return None; }
        let mut result = vec![0u8; self.total_len as usize];
        for (_, frag) in &self.fragments {
            let start = frag.offset as usize;
            let end = start + frag.data.len();
            result[start..end].copy_from_slice(&frag.data);
        }
        self.total_reassembled += 1;
        Some(result)
    }

    pub fn gaps(&self) -> Vec<(u64, u64)> {
        let mut gaps = Vec::new();
        let mut cursor: u64 = 0;
        for (_, frag) in &self.fragments {
            if frag.offset > cursor { gaps.push((cursor, frag.offset)); }
            let end = frag.offset + frag.data.len() as u64;
            if end > cursor { cursor = end; }
        }
        if cursor < self.total_len { gaps.push((cursor, self.total_len)); }
        gaps
    }

    pub fn gap_count(&self) -> usize { self.gaps().len() }

    pub fn progress(&self) -> f64 {
        if self.total_len == 0 { return 1.0; }
        self.received as f64 / self.total_len as f64
    }

    pub fn fragment_count(&self) -> usize { self.fragments.len() }
    pub fn total_len(&self) -> u64 { self.total_len }
    pub fn received(&self) -> u64 { self.received }
    pub fn total_fragments(&self) -> u64 { self.total_fragments }
    pub fn total_reassembled(&self) -> u64 { self.total_reassembled }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buf() { let fb = FragBuf::new(100); assert_eq!(fb.total_len(), 100); assert!(!fb.is_complete()); }

    #[test]
    fn insert_complete() {
        let mut fb = FragBuf::new(10);
        fb.insert(0, b"hello".to_vec()).unwrap();
        fb.insert(5, b"world".to_vec()).unwrap();
        assert!(fb.is_complete());
    }

    #[test]
    fn reassemble() {
        let mut fb = FragBuf::new(10);
        fb.insert(5, b"world".to_vec()).unwrap();
        fb.insert(0, b"hello".to_vec()).unwrap();
        let data = fb.reassemble().unwrap();
        assert_eq!(data, b"helloworld");
    }

    #[test]
    fn gaps() {
        let mut fb = FragBuf::new(20);
        fb.insert(0, b"aaaa".to_vec()).unwrap();
        fb.insert(10, b"bbbb".to_vec()).unwrap();
        let gaps = fb.gaps();
        assert_eq!(gaps, vec![(4, 10), (14, 20)]);
    }

    #[test]
    fn overlap() {
        let mut fb = FragBuf::new(10);
        fb.insert(0, b"hello".to_vec()).unwrap();
        let err = fb.insert(3, b"xxx".to_vec()).unwrap_err();
        assert!(matches!(err, FbError::Overlap { .. }));
    }

    #[test]
    fn already_complete() {
        let mut fb = FragBuf::new(5);
        fb.insert(0, b"hello".to_vec()).unwrap();
        let err = fb.insert(0, b"x".to_vec()).unwrap_err();
        assert!(matches!(err, FbError::AlreadyComplete));
    }

    #[test]
    fn out_of_bounds() {
        let mut fb = FragBuf::new(5);
        let err = fb.insert(3, b"xxx".to_vec()).unwrap_err();
        assert!(matches!(err, FbError::Overlap { .. }));
    }

    #[test]
    fn progress() {
        let mut fb = FragBuf::new(100);
        fb.insert(0, vec![0; 50]).unwrap();
        assert!((fb.progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn stats() {
        let mut fb = FragBuf::new(10);
        fb.insert(0, b"hello".to_vec()).unwrap();
        fb.insert(5, b"world".to_vec()).unwrap();
        fb.reassemble();
        assert_eq!(fb.total_fragments(), 2);
        assert_eq!(fb.total_reassembled(), 1);
    }

    #[test]
    fn error_display() { assert!(FbError::AlreadyComplete.to_string().contains("complete")); }
}
