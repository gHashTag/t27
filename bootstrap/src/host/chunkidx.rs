use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ChunkError {
    ChunkNotFound { id: u64 },
    Overlap { id: u64, offset: u64, len: u64 },
}

impl std::fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkError::ChunkNotFound { id } => write!(f, "chunk {id} not found"),
            ChunkError::Overlap { id, offset, len } => write!(f, "chunk {id} overlaps at {offset}+{len}"),
        }
    }
}

impl std::error::Error for ChunkError {}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub id: u64,
    pub offset: u64,
    pub length: u64,
    pub checksum: u64,
}

pub struct ChunkIndex {
    chunks: BTreeMap<u64, Chunk>,
    by_offset: BTreeMap<u64, u64>,
    next_id: u64,
    total_bytes: u64,
    total_compactions: u64,
    total_holes: u64,
}

impl ChunkIndex {
    pub fn new() -> Self { Self { chunks: BTreeMap::new(), by_offset: BTreeMap::new(), next_id: 1, total_bytes: 0, total_compactions: 0, total_holes: 0 } }

    pub fn append(&mut self, length: u64, checksum: u64) -> u64 {
        let offset = self.total_bytes;
        let id = self.next_id;
        self.next_id += 1;
        self.chunks.insert(id, Chunk { id, offset, length, checksum });
        self.by_offset.insert(offset, id);
        self.total_bytes += length;
        id
    }

    pub fn insert_at(&mut self, offset: u64, length: u64, checksum: u64) -> Result<u64, ChunkError> {
        for (_, &cid) in self.by_offset.range(..=offset) {
            if let Some(c) = self.chunks.get(&cid) {
                if c.offset < offset + length && c.offset + c.length > offset {
                    return Err(ChunkError::Overlap { id: cid, offset, len: length });
                }
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        self.chunks.insert(id, Chunk { id, offset, length, checksum });
        self.by_offset.insert(offset, id);
        self.total_bytes += length;
        Ok(id)
    }

    pub fn remove(&mut self, id: u64) -> Result<Chunk, ChunkError> {
        let chunk = self.chunks.remove(&id).ok_or(ChunkError::ChunkNotFound { id })?;
        self.by_offset.remove(&chunk.offset);
        self.total_holes += 1;
        Ok(chunk)
    }

    pub fn get(&self, id: u64) -> Option<&Chunk> { self.chunks.get(&id) }

    pub fn find_at(&self, offset: u64) -> Option<&Chunk> {
        let (_, &id) = self.by_offset.range(..=offset).next_back()?;
        let chunk = self.chunks.get(&id)?;
        if offset >= chunk.offset && offset < chunk.offset + chunk.length { Some(chunk) } else { None }
    }

    pub fn find_range(&self, start: u64, end: u64) -> Vec<&Chunk> {
        self.chunks.values()
            .filter(|c| c.offset < end && c.offset + c.length > start)
            .collect()
    }

    pub fn compact(&mut self) -> usize {
        let chunks: Vec<Chunk> = self.chunks.values().cloned().collect();
        self.chunks.clear();
        self.by_offset.clear();
        let mut offset = 0u64;
        let count = chunks.len();
        let mut new_id = 1u64;
        for mut c in chunks {
            c.offset = offset;
            c.id = new_id;
            new_id += 1;
            self.chunks.insert(c.id, c.clone());
            self.by_offset.insert(offset, c.id);
            offset += c.length;
        }
        self.next_id = new_id;
        self.total_bytes = offset;
        self.total_holes = 0;
        self.total_compactions += 1;
        count
    }

    pub fn chunk_count(&self) -> usize { self.chunks.len() }
    pub fn total_bytes(&self) -> u64 { self.total_bytes }
    pub fn total_holes(&self) -> u64 { self.total_holes }
    pub fn total_compactions(&self) -> u64 { self.total_compactions }
}

impl Default for ChunkIndex {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_idx() { assert_eq!(ChunkIndex::new().chunk_count(), 0); }

    #[test]
    fn append_find() {
        let mut ci = ChunkIndex::new();
        let id = ci.append(100, 0xABCD);
        let c = ci.get(id).unwrap();
        assert_eq!(c.offset, 0);
        assert_eq!(c.length, 100);
    }

    #[test]
    fn sequential_append() {
        let mut ci = ChunkIndex::new();
        ci.append(50, 1); ci.append(100, 2);
        assert_eq!(ci.get(2).unwrap().offset, 50);
        assert_eq!(ci.total_bytes(), 150);
    }

    #[test]
    fn find_at_offset() {
        let mut ci = ChunkIndex::new();
        ci.append(50, 1); ci.append(100, 2);
        let c = ci.find_at(75).unwrap();
        assert_eq!(c.id, 2);
    }

    #[test]
    fn find_range() {
        let mut ci = ChunkIndex::new();
        ci.append(50, 1); ci.append(100, 2); ci.append(50, 3);
        let found = ci.find_range(25, 125);
        assert!(found.len() >= 2);
    }

    #[test]
    fn remove_creates_hole() {
        let mut ci = ChunkIndex::new();
        ci.append(50, 1); ci.append(100, 2);
        ci.remove(1).unwrap();
        assert_eq!(ci.total_holes(), 1);
        assert_eq!(ci.chunk_count(), 1);
    }

    #[test]
    fn compact() {
        let mut ci = ChunkIndex::new();
        ci.append(50, 1); ci.append(100, 2);
        ci.remove(1).unwrap();
        let count = ci.compact();
        assert_eq!(count, 1);
        assert_eq!(ci.total_holes(), 0);
        assert_eq!(ci.get(1).unwrap().offset, 0);
    }

    #[test]
    fn not_found() {
        let ci = ChunkIndex::new();
        assert!(ci.get(99).is_none());
    }

    #[test]
    fn find_at_missing() {
        let mut ci = ChunkIndex::new();
        ci.append(50, 1);
        assert!(ci.find_at(100).is_none());
    }

    #[test]
    fn stats() {
        let mut ci = ChunkIndex::new();
        ci.append(100, 1);
        assert_eq!(ci.total_bytes(), 100);
        assert_eq!(ci.chunk_count(), 1);
    }

    #[test]
    fn error_display() { assert!(ChunkError::ChunkNotFound { id: 3 }.to_string().contains("3")); }
}
