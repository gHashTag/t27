const WINDOW: usize = 48;
const MASK_13BIT: u64 = (1 << 13) - 1;
const BUILTIN_PRIME: u64 = 0x9e3779b97f4a7c15;

fn gear_hash(byte: u8, hash: u64) -> u64 {
    (hash << 1).wrapping_add(BUILTIN_PRIME).wrapping_add(byte as u64)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkError {
    MinSize { min: usize, got: usize },
}

impl std::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkError::MinSize { min, got } => write!(f, "min size {min}, got {got}"),
        }
    }
}

impl std::error::Error for ChunkError {}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub offset: usize,
    pub len: usize,
    pub hash: u64,
}

pub struct Chunker {
    min_chunk: usize,
    max_chunk: usize,
    mask: u64,
    total_bytes: u64,
    total_chunks: u64,
}

impl Chunker {
    pub fn new(min_chunk: usize, max_chunk: usize, mask_bits: u32) -> Self {
        Self { min_chunk, max_chunk, mask: (1u64 << mask_bits) - 1, total_bytes: 0, total_chunks: 0 }
    }

    pub fn chunk(&mut self, data: &[u8]) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut offset = 0;
        let mut hash: u64 = 0;
        while offset < data.len() {
            let start = offset;
            let mut rolled = 0;
            while offset < data.len() {
                hash = gear_hash(data[offset], hash);
                rolled += 1;
                offset += 1;
                if rolled >= self.min_chunk {
                    if rolled >= self.max_chunk { break; }
                    if (hash & self.mask) == 0 { break; }
                }
            }
            chunks.push(Chunk { offset: start, len: offset - start, hash });
            self.total_chunks += 1;
        }
        self.total_bytes += data.len() as u64;
        chunks
    }

    pub fn chunk_with_dedupe(&mut self, data: &[u8], seen: &mut std::collections::BTreeSet<u64>) -> Vec<(Chunk, bool)> {
        let chunks = self.chunk(data);
        chunks.into_iter().map(|c| {
            let is_new = seen.insert(c.hash);
            (c, is_new)
        }).collect()
    }

    pub fn total_bytes(&self) -> u64 { self.total_bytes }
    pub fn total_chunks(&self) -> u64 { self.total_chunks }
    pub fn avg_chunk_size(&self) -> f64 {
        if self.total_chunks == 0 { 0.0 } else { self.total_bytes as f64 / self.total_chunks as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chunker() {
        let c = Chunker::new(128, 4096, 13);
        assert_eq!(c.total_chunks(), 0);
    }

    #[test]
    fn chunk_data() {
        let mut c = Chunker::new(16, 1024, 8);
        let data = vec![0u8; 500];
        let chunks = c.chunk(&data);
        assert!(!chunks.is_empty());
        assert_eq!(chunks.iter().map(|ch| ch.len).sum::<usize>(), 500);
    }

    #[test]
    fn chunk_respects_min() {
        let mut c = Chunker::new(64, 4096, 13);
        let data = vec![0xABu8; 200];
        let chunks = c.chunk(&data);
        for ch in &chunks { assert!(ch.len >= 64 || ch.offset + ch.len == data.len()); }
    }

    #[test]
    fn chunk_respects_max() {
        let mut c = Chunker::new(16, 64, 20);
        let data = vec![0xFFu8; 1000];
        let chunks = c.chunk(&data);
        for ch in &chunks { assert!(ch.len <= 64); }
    }

    #[test]
    fn deterministic() {
        let mut c1 = Chunker::new(16, 1024, 8);
        let mut c2 = Chunker::new(16, 1024, 8);
        let data: Vec<u8> = (0..500).map(|i| (i % 256) as u8).collect();
        let ch1 = c1.chunk(&data);
        let ch2 = c2.chunk(&data);
        assert_eq!(ch1.len(), ch2.len());
    }

    #[test]
    fn dedupe() {
        let mut c = Chunker::new(16, 256, 8);
        let mut seen = std::collections::BTreeSet::new();
        let data = vec![0u8; 300];
        let result = c.chunk_with_dedupe(&data, &mut seen);
        assert!(!result.is_empty());
    }

    #[test]
    fn empty_data() {
        let mut c = Chunker::new(16, 1024, 8);
        let chunks = c.chunk(&[]);
        assert!(chunks.is_empty());
    }

    #[test]
    fn small_data() {
        let mut c = Chunker::new(16, 1024, 13);
        let chunks = c.chunk(&[1, 2, 3]);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len, 3);
    }

    #[test]
    fn stats() {
        let mut c = Chunker::new(16, 1024, 8);
        c.chunk(&vec![0u8; 500]);
        assert_eq!(c.total_bytes(), 500);
        assert!(c.total_chunks() > 0);
    }

    #[test]
    fn avg_size() {
        let mut c = Chunker::new(16, 1024, 8);
        c.chunk(&vec![0u8; 500]);
        assert!(c.avg_chunk_size() > 0.0);
    }

    #[test]
    fn offset_tracking() {
        let mut c = Chunker::new(16, 64, 20);
        let data = vec![0xAAu8; 300];
        let chunks = c.chunk(&data);
        let mut expected_offset = 0;
        for ch in &chunks {
            assert_eq!(ch.offset, expected_offset);
            expected_offset += ch.len;
        }
    }
}
