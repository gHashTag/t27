#[derive(Debug, Clone, PartialEq)]
pub enum BmError {
    BlockOutOfRange { brow: usize, bcol: usize },
    IndexOutOfRange { row: usize, col: usize },
    DimensionMismatch { expected: usize, got: usize },
}

impl std::fmt::Display for BmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BmError::BlockOutOfRange { brow, bcol } => write!(f, "block ({brow},{bcol}) out of range"),
            BmError::IndexOutOfRange { row, col } => write!(f, "index ({row},{col}) out of range"),
            BmError::DimensionMismatch { expected, got } => write!(f, "dim mismatch expected {expected} got {got}"),
        }
    }
}

impl std::error::Error for BmError {}

pub struct BlockMat {
    data: Vec<Vec<i64>>,
    dim: usize,
    block_size: usize,
    blocks_per_side: usize,
    total_gets: u64,
    total_sets: u64,
    total_block_gets: u64,
}

impl BlockMat {
    pub fn new(dim: usize, block_size: usize) -> Self {
        let blocks_per_side = (dim + block_size - 1) / block_size;
        let total_blocks = blocks_per_side * blocks_per_side;
        let actual_dim = blocks_per_side * block_size;
        Self { data: vec![vec![0; actual_dim * actual_dim]; total_blocks], dim: actual_dim, block_size, blocks_per_side, total_gets: 0, total_sets: 0, total_block_gets: 0 }
    }

    fn block_index(&self, brow: usize, bcol: usize) -> usize { brow * self.blocks_per_side + bcol }

    pub fn get(&mut self, row: usize, col: usize) -> i64 {
        self.total_gets += 1;
        let brow = row / self.block_size;
        let bcol = col / self.block_size;
        let lr = row % self.block_size;
        let lc = col % self.block_size;
        let bi = self.block_index(brow, bcol);
        self.data[bi][lr * self.block_size + lc]
    }

    pub fn set(&mut self, row: usize, col: usize, val: i64) -> Result<(), BmError> {
        if row >= self.dim || col >= self.dim { return Err(BmError::IndexOutOfRange { row, col }); }
        self.total_sets += 1;
        let brow = row / self.block_size;
        let bcol = col / self.block_size;
        let lr = row % self.block_size;
        let lc = col % self.block_size;
        let bi = self.block_index(brow, bcol);
        self.data[bi][lr * self.block_size + lc] = val;
        Ok(())
    }

    pub fn get_block(&mut self, brow: usize, bcol: usize) -> Option<Vec<Vec<i64>>> {
        self.total_block_gets += 1;
        if brow >= self.blocks_per_side || bcol >= self.blocks_per_side { return None; }
        let bi = self.block_index(brow, bcol);
        let mut result = vec![vec![0; self.block_size]; self.block_size];
        for r in 0..self.block_size {
            for c in 0..self.block_size {
                result[r][c] = self.data[bi][r * self.block_size + c];
            }
        }
        Some(result)
    }

    pub fn set_block(&mut self, brow: usize, bcol: usize, block: &[Vec<i64>]) -> Result<(), BmError> {
        if brow >= self.blocks_per_side || bcol >= self.blocks_per_side { return Err(BmError::BlockOutOfRange { brow, bcol }); }
        let bi = self.block_index(brow, bcol);
        for r in 0..self.block_size.min(block.len()) {
            for c in 0..self.block_size.min(block[r].len()) {
                self.data[bi][r * self.block_size + c] = block[r][c];
            }
        }
        Ok(())
    }

    pub fn add_block(&mut self, brow: usize, bcol: usize, block: &[Vec<i64>]) -> Result<(), BmError> {
        if brow >= self.blocks_per_side || bcol >= self.blocks_per_side { return Err(BmError::BlockOutOfRange { brow, bcol }); }
        let bi = self.block_index(brow, bcol);
        for r in 0..self.block_size.min(block.len()) {
            for c in 0..self.block_size.min(block[r].len()) {
                self.data[bi][r * self.block_size + c] += block[r][c];
            }
        }
        Ok(())
    }

    pub fn zeros(&mut self) {
        for block in &mut self.data { block.fill(0); }
    }

    pub fn dim(&self) -> usize { self.dim }
    pub fn block_size(&self) -> usize { self.block_size }
    pub fn blocks_per_side(&self) -> usize { self.blocks_per_side }
    pub fn total_blocks(&self) -> usize { self.data.len() }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_sets(&self) -> u64 { self.total_sets }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bm() { let bm = BlockMat::new(8, 4); assert_eq!(bm.dim(), 8); assert_eq!(bm.blocks_per_side(), 2); }

    #[test]
    fn get_set() {
        let mut bm = BlockMat::new(8, 4);
        bm.set(0, 0, 1).unwrap(); bm.set(3, 7, 42).unwrap();
        assert_eq!(bm.get(0, 0), 1);
        assert_eq!(bm.get(3, 7), 42);
        assert_eq!(bm.get(1, 1), 0);
    }

    #[test]
    fn block_ops() {
        let mut bm = BlockMat::new(8, 4);
        let block = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12], vec![13, 14, 15, 16]];
        bm.set_block(0, 0, &block).unwrap();
        assert_eq!(bm.get(0, 0), 1);
        assert_eq!(bm.get(1, 1), 6);
        assert_eq!(bm.get(3, 3), 16);
    }

    #[test]
    fn get_block() {
        let mut bm = BlockMat::new(8, 4);
        bm.set(0, 0, 99).unwrap();
        let b = bm.get_block(0, 0).unwrap();
        assert_eq!(b[0][0], 99);
    }

    #[test]
    fn add_block() {
        let mut bm = BlockMat::new(8, 4);
        let block = vec![vec![1, 0, 0, 0], vec![0; 4], vec![0; 4], vec![0; 4]];
        bm.add_block(0, 0, &block).unwrap();
        bm.add_block(0, 0, &block).unwrap();
        assert_eq!(bm.get(0, 0), 2);
    }

    #[test]
    fn zeros() {
        let mut bm = BlockMat::new(4, 2);
        bm.set(0, 0, 42).unwrap(); bm.zeros();
        assert_eq!(bm.get(0, 0), 0);
    }

    #[test]
    fn out_of_range() {
        let mut bm = BlockMat::new(4, 2);
        assert!(bm.set(10, 10, 1).is_err());
        assert!(bm.get_block(5, 0).is_none());
    }

    #[test]
    fn non_square_dim() {
        let bm = BlockMat::new(6, 4);
        assert_eq!(bm.dim(), 8);
        assert_eq!(bm.blocks_per_side(), 2);
    }

    #[test]
    fn stats() {
        let mut bm = BlockMat::new(4, 2);
        bm.set(0, 0, 1).unwrap(); bm.get(0, 0); bm.get_block(0, 0);
        assert_eq!(bm.total_sets(), 1);
        assert_eq!(bm.total_gets(), 1);
    }

    #[test]
    fn error_display() { assert!(BmError::BlockOutOfRange { brow: 5, bcol: 5 }.to_string().contains("block")); }
}
