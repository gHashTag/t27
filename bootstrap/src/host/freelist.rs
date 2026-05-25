use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FlError {
    OutOfMemory { requested: usize, available: usize },
}

impl std::fmt::Display for FlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlError::OutOfMemory { requested, available } => write!(f, "oom: need {requested}, have {available}"),
        }
    }
}

impl std::error::Error for FlError {}

#[derive(Clone)]
struct Block {
    offset: usize,
    size: usize,
    free: bool,
}

pub struct FreeList {
    blocks: BTreeMap<usize, Block>,
    total_size: usize,
    allocated: usize,
    total_allocs: u64,
    total_frees: u64,
    total_splits: u64,
    total_coalesces: u64,
}

impl FreeList {
    pub fn new(size: usize) -> Self {
        let mut blocks = BTreeMap::new();
        blocks.insert(0, Block { offset: 0, size, free: true });
        Self { blocks, total_size: size, allocated: 0, total_allocs: 0, total_frees: 0, total_splits: 0, total_coalesces: 0 }
    }

    pub fn alloc(&mut self, size: usize, align: usize) -> Result<usize, FlError> {
        self.total_allocs += 1;
        let align = align.max(1);
        for (&offset, block) in &self.blocks {
            if !block.free { continue; }
            let aligned = (offset + align - 1) / align * align;
            let padding = aligned - offset;
            if block.size >= size + padding {
                let alloc_offset = aligned;
                let alloc_end = alloc_offset + size;
                self.total_splits += 1;
                let old_offset = offset;
                let old_size = block.size;
                self.blocks.remove(&old_offset);
                if old_offset < alloc_offset {
                    self.blocks.insert(old_offset, Block { offset: old_offset, size: alloc_offset - old_offset, free: true });
                }
                self.blocks.insert(alloc_offset, Block { offset: alloc_offset, size, free: false });
                let end = alloc_offset + size;
                if end < old_offset + old_size {
                    self.blocks.insert(end, Block { offset: end, size: old_offset + old_size - end, free: true });
                }
                self.allocated += size;
                return Ok(alloc_offset);
            }
        }
        Err(FlError::OutOfMemory { requested: size, available: self.total_size - self.allocated })
    }

    pub fn free(&mut self, offset: usize) -> Result<(), FlError> {
        self.total_frees += 1;
        let block = self.blocks.get(&offset).cloned().ok_or(FlError::OutOfMemory { requested: 0, available: 0 })?;
        if block.free { return Err(FlError::OutOfMemory { requested: 0, available: 0 }); }
        self.allocated -= block.size;
        let mut start = offset;
        let mut end = offset + block.size;
        if let Some((&prev_off, prev)) = self.blocks.range(..offset).next_back() {
            if prev.free && prev_off + prev.size == offset {
                start = prev_off;
                self.blocks.remove(&prev_off);
                self.total_coalesces += 1;
            }
        }
        if let Some(next) = self.blocks.get(&(offset + block.size)).cloned() {
            if next.free {
                end = next.offset + next.size;
                self.blocks.remove(&next.offset);
                self.total_coalesces += 1;
            }
        }
        self.blocks.remove(&offset);
        self.blocks.insert(start, Block { offset: start, size: end - start, free: true });
        Ok(())
    }

    pub fn is_allocated(&self, offset: usize) -> bool {
        self.blocks.get(&offset).map(|b| !b.free).unwrap_or(false)
    }

    pub fn fragmentation(&self) -> f64 {
        let free_count = self.blocks.values().filter(|b| b.free).count();
        if free_count <= 1 { return 0.0; }
        let largest_free: usize = self.blocks.values().filter(|b| b.free).map(|b| b.size).max().unwrap_or(0);
        let total_free = self.total_size - self.allocated;
        if total_free == 0 { return 0.0; }
        1.0 - (largest_free as f64 / total_free as f64)
    }

    pub fn total_size(&self) -> usize { self.total_size }
    pub fn allocated(&self) -> usize { self.allocated }
    pub fn available(&self) -> usize { self.total_size - self.allocated }
    pub fn total_allocs(&self) -> u64 { self.total_allocs }
    pub fn total_frees(&self) -> u64 { self.total_frees }
    pub fn total_splits(&self) -> u64 { self.total_splits }
    pub fn total_coalesces(&self) -> u64 { self.total_coalesces }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fl() { let fl = FreeList::new(1024); assert_eq!(fl.available(), 1024); }

    #[test]
    fn alloc_free() {
        let mut fl = FreeList::new(1024);
        let a = fl.alloc(100, 1).unwrap();
        assert_eq!(a, 0);
        assert_eq!(fl.allocated(), 100);
        fl.free(a).unwrap();
        assert_eq!(fl.allocated(), 0);
    }

    #[test]
    fn aligned_alloc() {
        let mut fl = FreeList::new(1024);
        let a = fl.alloc(10, 1).unwrap();
        let b = fl.alloc(10, 64).unwrap();
        assert_eq!(b % 64, 0);
    }

    #[test]
    fn coalesce() {
        let mut fl = FreeList::new(1024);
        let a = fl.alloc(100, 1).unwrap();
        let b = fl.alloc(100, 1).unwrap();
        fl.free(a).unwrap(); fl.free(b).unwrap();
        assert_eq!(fl.available(), 1024);
        assert!(fl.total_coalesces() > 0);
    }

    #[test]
    fn oom() {
        let mut fl = FreeList::new(100);
        fl.alloc(100, 1).unwrap();
        assert!(fl.alloc(1, 1).is_err());
    }

    #[test]
    fn reuse() {
        let mut fl = FreeList::new(1024);
        let a = fl.alloc(100, 1).unwrap();
        fl.free(a).unwrap();
        let b = fl.alloc(50, 1).unwrap();
        assert_eq!(b, 0);
    }

    #[test]
    fn fragmentation() {
        let mut fl = FreeList::new(1000);
        let a = fl.alloc(10, 1).unwrap();
        let b = fl.alloc(10, 1).unwrap();
        let c = fl.alloc(10, 1).unwrap();
        fl.free(b).unwrap();
        assert!(fl.fragmentation() > 0.0);
    }

    #[test]
    fn is_allocated() {
        let mut fl = FreeList::new(1024);
        let a = fl.alloc(50, 1).unwrap();
        assert!(fl.is_allocated(a));
        fl.free(a).unwrap();
        assert!(!fl.is_allocated(a));
    }

    #[test]
    fn stats() {
        let mut fl = FreeList::new(1024);
        let a = fl.alloc(100, 1).unwrap(); fl.free(a).unwrap();
        assert_eq!(fl.total_allocs(), 1);
        assert_eq!(fl.total_frees(), 1);
    }

    #[test]
    fn error_display() { assert!(FlError::OutOfMemory { requested: 100, available: 50 }.to_string().contains("100")); }
}
