use std::collections::BTreeMap;

#[derive(Clone)]
struct Chunk {
    id: u64,
    data: Vec<u8>,
    used: bool,
}

pub struct ChunkMap {
    chunks: BTreeMap<u64, Chunk>,
    chunk_size: usize,
    free_list: Vec<u64>,
    next_id: u64,
    total_allocs: u64,
    total_frees: u64,
    total_reads: u64,
}

impl ChunkMap {
    pub fn new(chunk_size: usize) -> Self { Self { chunks: BTreeMap::new(), chunk_size: chunk_size.max(1), free_list: Vec::new(), next_id: 0, total_allocs: 0, total_frees: 0, total_reads: 0 } }

    pub fn alloc(&mut self) -> u64 {
        self.total_allocs += 1;
        let id = self.free_list.pop().unwrap_or_else(|| { let id = self.next_id; self.next_id += 1; id });
        self.chunks.insert(id, Chunk { id, data: vec![0; self.chunk_size], used: true });
        id
    }

    pub fn write(&mut self, id: u64, offset: usize, data: &[u8]) -> bool {
        let c = match self.chunks.get_mut(&id) {
            Some(c) if c.used => c,
            _ => return false,
        };
        let end = offset + data.len();
        if end > c.data.len() { return false; }
        c.data[offset..end].copy_from_slice(data);
        true
    }

    pub fn read(&mut self, id: u64, offset: usize, len: usize) -> Option<Vec<u8>> {
        self.total_reads += 1;
        let c = self.chunks.get(&id)?;
        if !c.used || offset + len > c.data.len() { return None; }
        Some(c.data[offset..offset + len].to_vec())
    }

    pub fn free(&mut self, id: u64) -> bool {
        self.total_frees += 1;
        if let Some(c) = self.chunks.get_mut(&id) {
            if c.used { c.used = false; self.free_list.push(id); return true; }
        }
        false
    }

    pub fn used_count(&self) -> usize { self.chunks.values().filter(|c| c.used).count() }
    pub fn free_count(&self) -> usize { self.free_list.len() }
    pub fn total_count(&self) -> usize { self.chunks.len() }
    pub fn chunk_size(&self) -> usize { self.chunk_size }
    pub fn utilized_bytes(&self) -> usize { self.used_count() * self.chunk_size }
    pub fn total_allocs(&self) -> u64 { self.total_allocs }
    pub fn total_frees(&self) -> u64 { self.total_frees }
    pub fn total_reads(&self) -> u64 { self.total_reads }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_write_read() {
        let mut cm = ChunkMap::new(64);
        let id = cm.alloc();
        cm.write(id, 0, b"hello");
        let data = cm.read(id, 0, 5).unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn alloc_free_reuse() {
        let mut cm = ChunkMap::new(64);
        let id1 = cm.alloc();
        cm.free(id1);
        let id2 = cm.alloc();
        assert_eq!(id1, id2);
    }

    #[test]
    fn out_of_bounds() {
        let mut cm = ChunkMap::new(8);
        let id = cm.alloc();
        assert!(!cm.write(id, 5, b"aaaaa"));
    }

    #[test]
    fn free_invalid() { assert!(!ChunkMap::new(64).free(999)); }

    #[test]
    fn read_invalid() { let mut cm = ChunkMap::new(64); assert!(cm.read(999, 0, 4).is_none()); }

    #[test]
    fn utilized() {
        let mut cm = ChunkMap::new(32);
        cm.alloc(); cm.alloc();
        assert_eq!(cm.utilized_bytes(), 64);
    }

    #[test]
    fn counts() {
        let mut cm = ChunkMap::new(32);
        let a = cm.alloc(); let _b = cm.alloc();
        cm.free(a);
        assert_eq!(cm.used_count(), 1);
        assert_eq!(cm.free_count(), 1);
    }

    #[test]
    fn stats() {
        let mut cm = ChunkMap::new(64);
        let id = cm.alloc();
        cm.write(id, 0, b"x"); cm.read(id, 0, 1); cm.free(id);
        assert_eq!(cm.total_allocs(), 1);
        assert_eq!(cm.total_reads(), 1);
        assert_eq!(cm.total_frees(), 1);
    }
}
