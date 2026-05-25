#[derive(Debug, Clone, PartialEq)]
pub enum BmxError {
    IndexOutOfRange { row: usize, col: usize, rows: usize, cols: usize },
    DimensionMismatch,
}

impl std::fmt::Display for BmxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BmxError::IndexOutOfRange { row, col, rows, cols } => write!(f, "({row},{col}) out of ({rows},{cols})"),
            BmxError::DimensionMismatch => write!(f, "dimension mismatch"),
        }
    }
}

impl std::error::Error for BmxError {}

pub struct BitMatrix {
    rows: Vec<u64>,
    nrows: usize,
    ncols: usize,
    words_per_row: usize,
    total_ops: u64,
}

impl BitMatrix {
    pub fn new(nrows: usize, ncols: usize) -> Self {
        let words_per_row = (ncols + 63) / 64;
        Self { rows: vec![0; nrows * words_per_row], nrows, ncols, words_per_row, total_ops: 0 }
    }

    pub fn identity(n: usize) -> Self {
        let mut bm = Self::new(n, n);
        for i in 0..n { bm.set(i, i, true); }
        bm
    }

    fn row_ptr(&self, row: usize) -> usize { row * self.words_per_row }

    pub fn get(&self, row: usize, col: usize) -> bool {
        if row >= self.nrows || col >= self.ncols { return false; }
        let word = col / 64;
        let bit = col % 64;
        (self.rows[self.row_ptr(row) + word] >> bit) & 1 == 1
    }

    pub fn set(&mut self, row: usize, col: usize, val: bool) {
        if row >= self.nrows || col >= self.ncols { return; }
        let word = col / 64;
        let bit = col % 64;
        let ptr = self.row_ptr(row) + word;
        if val { self.rows[ptr] |= 1 << bit; } else { self.rows[ptr] &= !(1 << bit); }
        self.total_ops += 1;
    }

    pub fn get_row(&self, row: usize) -> u64 {
        if row >= self.nrows || self.words_per_row == 0 { return 0; }
        self.rows[self.row_ptr(row)]
    }

    pub fn set_row(&mut self, row: usize, val: u64) {
        if row >= self.nrows { return; }
        self.rows[self.row_ptr(row)] = val & ((1u64 << self.ncols) - 1);
        self.total_ops += 1;
    }

    pub fn xor_row(&mut self, dst: usize, src: usize) {
        if dst >= self.nrows || src >= self.nrows { return; }
        for w in 0..self.words_per_row {
            self.rows[self.row_ptr(dst) + w] ^= self.rows[self.row_ptr(src) + w];
        }
        self.total_ops += 1;
    }

    pub fn and_row(&mut self, dst: usize, src: usize) {
        if dst >= self.nrows || src >= self.nrows { return; }
        for w in 0..self.words_per_row {
            self.rows[self.row_ptr(dst) + w] &= self.rows[self.row_ptr(src) + w];
        }
        self.total_ops += 1;
    }

    pub fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b || a >= self.nrows || b >= self.nrows { return; }
        for w in 0..self.words_per_row {
            let pa = self.row_ptr(a) + w;
            let pb = self.row_ptr(b) + w;
            let tmp = self.rows[pa]; self.rows[pa] = self.rows[pb]; self.rows[pb] = tmp;
        }
    }

    pub fn transpose(&self) -> BitMatrix {
        let mut bm = BitMatrix::new(self.ncols, self.nrows);
        for r in 0..self.nrows {
            for c in 0..self.ncols {
                bm.set(c, r, self.get(r, c));
            }
        }
        bm
    }

    pub fn rank(&mut self) -> usize {
        let mut mat = self.rows.clone();
        let mut rank = 0;
        for col in 0..self.ncols {
            let word = col / 64;
            let bit = col % 64;
            let mut pivot = None;
            for r in rank..self.nrows {
                if (mat[r * self.words_per_row + word] >> bit) & 1 == 1 { pivot = Some(r); break; }
            }
            match pivot {
                Some(p) => {
                    if p != rank {
                        for w in 0..self.words_per_row {
                            let tmp = mat[rank * self.words_per_row + w];
                            mat[rank * self.words_per_row + w] = mat[p * self.words_per_row + w];
                            mat[p * self.words_per_row + w] = tmp;
                        }
                    }
                    for r in 0..self.nrows {
                        if r != rank && (mat[r * self.words_per_row + word] >> bit) & 1 == 1 {
                            for w in 0..self.words_per_row { mat[r * self.words_per_row + w] ^= mat[rank * self.words_per_row + w]; }
                        }
                    }
                    rank += 1;
                }
                None => {}
            }
        }
        rank
    }

    pub fn popcount(&self) -> usize { self.rows.iter().map(|w| w.count_ones() as usize).sum() }
    pub fn nrows(&self) -> usize { self.nrows }
    pub fn ncols(&self) -> usize { self.ncols }
    pub fn total_ops(&self) -> u64 { self.total_ops }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bm() { let bm = BitMatrix::new(4, 4); assert_eq!(bm.popcount(), 0); }

    #[test]
    fn set_get() {
        let mut bm = BitMatrix::new(4, 64);
        bm.set(0, 0, true); bm.set(1, 63, true);
        assert!(bm.get(0, 0)); assert!(bm.get(1, 63));
        assert!(!bm.get(0, 1));
    }

    #[test]
    fn identity() {
        let bm = BitMatrix::identity(4);
        assert!(bm.get(0, 0)); assert!(!bm.get(0, 1));
        assert!(bm.get(3, 3));
        assert_eq!(bm.popcount(), 4);
    }

    #[test]
    fn xor_row() {
        let mut bm = BitMatrix::identity(4);
        bm.xor_row(0, 1);
        assert!(bm.get(0, 0)); assert!(bm.get(0, 1));
    }

    #[test]
    fn swap_rows() {
        let mut bm = BitMatrix::new(2, 64);
        bm.set_row(0, 0b01); bm.set_row(1, 0b10);
        bm.swap_rows(0, 1);
        assert_eq!(bm.get_row(0), 0b10);
    }

    #[test]
    fn transpose() {
        let mut bm = BitMatrix::new(2, 3);
        bm.set(0, 1, true); bm.set(1, 0, true);
        let t = bm.transpose();
        assert!(t.get(1, 0)); assert!(t.get(0, 1));
        assert_eq!(t.nrows(), 3);
    }

    #[test]
    fn rank() {
        let mut bm = BitMatrix::identity(4);
        assert_eq!(bm.rank(), 4);
    }

    #[test]
    fn rank_dependent() {
        let mut bm = BitMatrix::new(3, 64);
        bm.set_row(0, 0b001); bm.set_row(1, 0b010); bm.set_row(2, 0b011);
        assert_eq!(bm.rank(), 2);
    }

    #[test]
    fn stats() {
        let mut bm = BitMatrix::new(4, 4);
        bm.set(0, 0, true); bm.set(1, 1, true);
        assert!(bm.total_ops() > 0);
    }
}
