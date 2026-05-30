#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    NoFreeBlocks,
    BlockNotFound,
    DoubleFree,
    PoolFull,
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::NoFreeBlocks => write!(f, "no free blocks"),
            PoolError::BlockNotFound => write!(f, "block not found"),
            PoolError::DoubleFree => write!(f, "double free"),
            PoolError::PoolFull => write!(f, "pool full"),
        }
    }
}

impl std::error::Error for PoolError {}

pub const BLOCK_SIZE: usize = 64;
pub const MAX_BLOCKS: usize = 64;

#[derive(Debug, Clone)]
pub struct MemBlock {
    pub data: [u8; BLOCK_SIZE],
    pub len: usize,
}

impl MemBlock {
    pub fn new() -> Self {
        Self {
            data: [0u8; BLOCK_SIZE],
            len: 0,
        }
    }

    pub fn with_data(slice: &[u8]) -> Self {
        let mut block = Self::new();
        let copy_len = slice.len().min(BLOCK_SIZE);
        block.data[..copy_len].copy_from_slice(&slice[..copy_len]);
        block.len = copy_len;
        block
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.len]
    }

    pub fn capacity(&self) -> usize {
        BLOCK_SIZE
    }

    pub fn remaining(&self) -> usize {
        BLOCK_SIZE - self.len
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let copy_len = data.len().min(self.remaining());
        self.data[self.len..self.len + copy_len].copy_from_slice(&data[..copy_len]);
        self.len += copy_len;
        copy_len
    }
}

#[derive(Debug, Clone)]
pub struct MemPool {
    blocks: Vec<Option<MemBlock>>,
    free_list: Vec<usize>,
    allocated_count: usize,
}

impl MemPool {
    pub fn new(block_count: usize) -> Self {
        let block_count = block_count.min(MAX_BLOCKS);
        let blocks = (0..block_count).map(|_| None).collect();
        let free_list = (0..block_count).rev().collect();
        Self {
            blocks,
            free_list,
            allocated_count: 0,
        }
    }

    pub fn allocate(&mut self) -> Result<(usize, &mut MemBlock), PoolError> {
        let idx = self.free_list.pop().ok_or(PoolError::NoFreeBlocks)?;
        self.blocks[idx] = Some(MemBlock::new());
        self.allocated_count += 1;
        Ok((idx, self.blocks[idx].as_mut().unwrap()))
    }

    pub fn allocate_with(&mut self, data: &[u8]) -> Result<(usize, &mut MemBlock), PoolError> {
        let idx = self.free_list.pop().ok_or(PoolError::NoFreeBlocks)?;
        self.blocks[idx] = Some(MemBlock::with_data(data));
        self.allocated_count += 1;
        Ok((idx, self.blocks[idx].as_mut().unwrap()))
    }

    pub fn deallocate(&mut self, idx: usize) -> Result<(), PoolError> {
        if idx >= self.blocks.len() {
            return Err(PoolError::BlockNotFound);
        }
        if self.blocks[idx].is_none() {
            return Err(PoolError::DoubleFree);
        }
        self.blocks[idx] = None;
        self.allocated_count -= 1;
        self.free_list.push(idx);
        Ok(())
    }

    pub fn get(&self, idx: usize) -> Result<&MemBlock, PoolError> {
        self.blocks
            .get(idx)
            .and_then(|b| b.as_ref())
            .ok_or(PoolError::BlockNotFound)
    }

    pub fn get_mut(&mut self, idx: usize) -> Result<&mut MemBlock, PoolError> {
        self.blocks
            .get_mut(idx)
            .and_then(|b| b.as_mut())
            .ok_or(PoolError::BlockNotFound)
    }

    pub fn free_count(&self) -> usize {
        self.free_list.len()
    }

    pub fn allocated_count(&self) -> usize {
        self.allocated_count
    }

    pub fn total_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_full(&self) -> bool {
        self.free_list.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.allocated_count == 0
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total: self.total_count(),
            allocated: self.allocated_count,
            free: self.free_count(),
            block_size: BLOCK_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolStats {
    pub total: usize,
    pub allocated: usize,
    pub free: usize,
    pub block_size: usize,
}

impl PoolStats {
    pub fn utilization(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.allocated as f64 / self.total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_block_new() {
        let b = MemBlock::new();
        assert_eq!(b.len, 0);
        assert_eq!(b.capacity(), BLOCK_SIZE);
        assert_eq!(b.remaining(), BLOCK_SIZE);
    }

    #[test]
    fn mem_block_with_data() {
        let b = MemBlock::with_data(b"hello");
        assert_eq!(b.len, 5);
        assert_eq!(b.as_slice(), b"hello");
        assert_eq!(b.remaining(), BLOCK_SIZE - 5);
    }

    #[test]
    fn mem_block_write() {
        let mut b = MemBlock::new();
        let n = b.write(b"abc");
        assert_eq!(n, 3);
        assert_eq!(b.len, 3);
        let n2 = b.write(b"defgh");
        assert_eq!(n2, 5);
        assert_eq!(b.as_slice(), b"abcdefgh");
    }

    #[test]
    fn mem_block_write_overflow_truncates() {
        let mut b = MemBlock::with_data(&[0u8; 60]);
        let n = b.write(&[1u8; 10]);
        assert_eq!(n, 4);
        assert_eq!(b.len, BLOCK_SIZE);
    }

    #[test]
    fn mem_block_clear() {
        let mut b = MemBlock::with_data(b"data");
        b.clear();
        assert_eq!(b.len, 0);
    }

    #[test]
    fn pool_allocate_deallocate() {
        let mut pool = MemPool::new(4);
        let (idx, block) = pool.allocate().unwrap();
        block.write(b"test");
        assert_eq!(pool.allocated_count(), 1);
        assert_eq!(pool.free_count(), 3);
        pool.deallocate(idx).unwrap();
        assert_eq!(pool.allocated_count(), 0);
        assert_eq!(pool.free_count(), 4);
    }

    #[test]
    fn pool_allocate_with_data() {
        let mut pool = MemPool::new(4);
        let (idx, block) = pool.allocate_with(b"payload").unwrap();
        assert_eq!(block.as_slice(), b"payload");
        let fetched = pool.get(idx).unwrap();
        assert_eq!(fetched.as_slice(), b"payload");
    }

    #[test]
    fn pool_deallocate_double_free() {
        let mut pool = MemPool::new(4);
        let (idx, _) = pool.allocate().unwrap();
        pool.deallocate(idx).unwrap();
        let err = pool.deallocate(idx).unwrap_err();
        assert_eq!(err, PoolError::DoubleFree);
    }

    #[test]
    fn pool_deallocate_out_of_range() {
        let mut pool = MemPool::new(4);
        let err = pool.deallocate(99).unwrap_err();
        assert_eq!(err, PoolError::BlockNotFound);
    }

    #[test]
    fn pool_exhaustion() {
        let mut pool = MemPool::new(2);
        pool.allocate().unwrap();
        pool.allocate().unwrap();
        assert!(pool.is_full());
        let err = pool.allocate().unwrap_err();
        assert_eq!(err, PoolError::NoFreeBlocks);
    }

    #[test]
    fn pool_reuse_after_free() {
        let mut pool = MemPool::new(2);
        let (idx1, _) = pool.allocate().unwrap();
        let (_, _) = pool.allocate().unwrap();
        pool.deallocate(idx1).unwrap();
        let (idx2, _) = pool.allocate().unwrap();
        assert_eq!(idx1, idx2);
    }

    #[test]
    fn pool_stats() {
        let mut pool = MemPool::new(8);
        let stats = pool.stats();
        assert_eq!(stats.total, 8);
        assert_eq!(stats.allocated, 0);
        assert_eq!(stats.free, 8);
        assert_eq!(stats.block_size, BLOCK_SIZE);
        assert_eq!(stats.utilization(), 0.0);
        pool.allocate().unwrap();
        pool.allocate().unwrap();
        let stats = pool.stats();
        assert_eq!(stats.allocated, 2);
        assert!((stats.utilization() - 0.25).abs() < 0.001);
    }

    #[test]
    fn pool_is_empty() {
        let pool = MemPool::new(4);
        assert!(pool.is_empty());
        assert!(!pool.is_full());
    }

    #[test]
    fn pool_get_mut() {
        let mut pool = MemPool::new(4);
        let (idx, _) = pool.allocate().unwrap();
        {
            let block = pool.get_mut(idx).unwrap();
            block.write(b"modified");
        }
        assert_eq!(pool.get(idx).unwrap().as_slice(), b"modified");
    }

    #[test]
    fn pool_capped_to_max() {
        let pool = MemPool::new(MAX_BLOCKS + 100);
        assert_eq!(pool.total_count(), MAX_BLOCKS);
    }

    #[test]
    fn error_display() {
        assert!(PoolError::NoFreeBlocks.to_string().contains("free"));
        assert!(PoolError::DoubleFree.to_string().contains("double"));
    }
}
