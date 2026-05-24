use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum HmError {
    AddressInUse { addr: u64 },
    BlockNotFound { addr: u64 },
    BlockNotAllocated { addr: u64 },
    DoubleFree { addr: u64 },
}

impl std::fmt::Display for HmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HmError::AddressInUse { addr } => write!(f, "address {addr} in use"),
            HmError::BlockNotFound { addr } => write!(f, "block at {addr} not found"),
            HmError::BlockNotAllocated { addr } => write!(f, "block at {addr} not allocated"),
            HmError::DoubleFree { addr } => write!(f, "double free at {addr}"),
        }
    }
}

impl std::error::Error for HmError {}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockState { Free, Allocated }

struct Block {
    addr: u64,
    size: usize,
    state: BlockState,
    tag: Option<u64>,
}

pub struct HeapMap {
    blocks: BTreeMap<u64, Block>,
    total_size: usize,
    total_allocated: usize,
    total_allocs: u64,
    total_frees: u64,
    total_compactions: u64,
}

impl HeapMap {
    pub fn new(total_size: usize) -> Self {
        let mut hm = Self { blocks: BTreeMap::new(), total_size, total_allocated: 0, total_allocs: 0, total_frees: 0, total_compactions: 0 };
        hm.blocks.insert(0, Block { addr: 0, size: total_size, state: BlockState::Free, tag: None });
        hm
    }

    pub fn allocate(&mut self, size: usize, tag: Option<u64>) -> Option<u64> {
        let mut best_addr: Option<u64> = None;
        let mut best_size: usize = usize::MAX;
        for (&addr, block) in &self.blocks {
            if block.state == BlockState::Free && block.size >= size && block.size < best_size {
                best_addr = Some(addr);
                best_size = block.size;
            }
        }
        let addr = best_addr?;
        let block = self.blocks.get_mut(&addr).unwrap();
        block.state = BlockState::Allocated;
        block.tag = tag;
        self.total_allocated += size;
        self.total_allocs += 1;
        if block.size > size {
            let remaining = block.size - size;
            block.size = size;
            let new_addr = addr + size as u64;
            self.blocks.insert(new_addr, Block { addr: new_addr, size: remaining, state: BlockState::Free, tag: None });
        }
        Some(addr)
    }

    pub fn free(&mut self, addr: u64) -> Result<(), HmError> {
        let block = self.blocks.get_mut(&addr).ok_or(HmError::BlockNotFound { addr })?;
        if block.state == BlockState::Free { return Err(HmError::DoubleFree { addr }); }
        self.total_allocated -= block.size;
        block.state = BlockState::Free;
        block.tag = None;
        self.total_frees += 1;
        self.coalesce(addr);
        Ok(())
    }

    fn coalesce(&mut self, addr: u64) {
        let size = self.blocks.get(&addr).map(|b| b.size).unwrap_or(0);
        let next_addr = addr + size as u64;
        if let Some(next) = self.blocks.remove(&next_addr) {
            if next.state == BlockState::Free {
                if let Some(block) = self.blocks.get_mut(&addr) { block.size += next.size; }
            } else { self.blocks.insert(next_addr, next); }
        }
    }

    pub fn compact(&mut self) -> usize {
        let mut moved = 0;
        let allocated: Vec<(u64, usize, Option<u64>)> = self.blocks.iter()
            .filter(|(_, b)| b.state == BlockState::Allocated)
            .map(|(&a, b)| (a, b.size, b.tag))
            .collect();
        self.blocks.clear();
        self.blocks.insert(0, Block { addr: 0, size: self.total_size, state: BlockState::Free, tag: None });
        self.total_allocated = 0;
        for (_, size, tag) in &allocated {
            self.allocate(*size, *tag);
            moved += 1;
        }
        self.total_compactions += 1;
        moved
    }

    pub fn fragmentation(&self) -> f64 {
        if self.total_size == 0 { return 0.0; }
        let free_blocks: Vec<&Block> = self.blocks.values().filter(|b| b.state == BlockState::Free).collect();
        if free_blocks.is_empty() { return 0.0; }
        let free_total: usize = free_blocks.iter().map(|b| b.size).sum();
        if free_total == 0 { return 0.0; }
        let largest_free = free_blocks.iter().map(|b| b.size).max().unwrap_or(0);
        1.0 - (largest_free as f64 / free_total as f64)
    }

    pub fn block_state(&self, addr: u64) -> Option<&BlockState> { self.blocks.get(&addr).map(|b| &b.state) }
    pub fn block_size(&self, addr: u64) -> Option<usize> { self.blocks.get(&addr).map(|b| b.size) }
    pub fn block_count(&self) -> usize { self.blocks.len() }
    pub fn free_space(&self) -> usize { self.total_size - self.total_allocated }
    pub fn used_space(&self) -> usize { self.total_allocated }
    pub fn total_size(&self) -> usize { self.total_size }
    pub fn total_allocs(&self) -> u64 { self.total_allocs }
    pub fn total_frees(&self) -> u64 { self.total_frees }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_heap() { let h = HeapMap::new(1024); assert_eq!(h.free_space(), 1024); }

    #[test]
    fn allocate_free() {
        let mut h = HeapMap::new(1024);
        let addr = h.allocate(100, None).unwrap();
        assert_eq!(h.used_space(), 100);
        h.free(addr).unwrap();
        assert_eq!(h.free_space(), 1024);
    }

    #[test]
    fn multiple_allocs() {
        let mut h = HeapMap::new(1024);
        let a1 = h.allocate(100, Some(1)).unwrap();
        let a2 = h.allocate(200, Some(2)).unwrap();
        assert_ne!(a1, a2);
        assert_eq!(h.used_space(), 300);
    }

    #[test]
    fn coalesce() {
        let mut h = HeapMap::new(1024);
        let a1 = h.allocate(100, None).unwrap();
        let a2 = h.allocate(100, None).unwrap();
        h.free(a2).unwrap();
        h.free(a1).unwrap();
        assert_eq!(h.free_space(), 1024);
    }

    #[test]
    fn double_free() {
        let mut h = HeapMap::new(1024);
        let a = h.allocate(50, None).unwrap();
        h.free(a).unwrap();
        let err = h.free(a).unwrap_err();
        assert!(matches!(err, HmError::DoubleFree { .. }));
    }

    #[test]
    fn oom() {
        let mut h = HeapMap::new(100);
        assert!(h.allocate(50, None).is_some());
        assert!(h.allocate(60, None).is_none());
    }

    #[test]
    fn fragmentation() {
        let mut h = HeapMap::new(300);
        let a1 = h.allocate(100, None).unwrap();
        let _ = h.allocate(100, None).unwrap();
        h.free(a1).unwrap();
        assert!(h.fragmentation() > 0.0);
    }

    #[test]
    fn compact() {
        let mut h = HeapMap::new(1024);
        h.allocate(100, Some(1)).unwrap();
        h.allocate(100, Some(2)).unwrap();
        h.free(0).unwrap();
        let moved = h.compact();
        assert_eq!(moved, 1);
    }

    #[test]
    fn not_found() {
        let mut h = HeapMap::new(1024);
        let err = h.free(999).unwrap_err();
        assert!(matches!(err, HmError::BlockNotFound { .. }));
    }

    #[test]
    fn error_display() { assert!(HmError::DoubleFree { addr: 1 }.to_string().contains("double")); }
}
