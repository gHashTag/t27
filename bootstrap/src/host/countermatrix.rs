use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CmError {
    OutOfBounds { row: usize, col: usize, rows: usize, cols: usize },
    DimensionMismatch { expected: (usize, usize), found: (usize, usize) },
}

impl std::fmt::Display for CmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmError::OutOfBounds { row, col, rows, cols } => write!(f, "[{row},{col}] out of [{rows},{cols}]"),
            CmError::DimensionMismatch { expected, found } => write!(f, "dim mismatch: expected {:?}, found {:?}", expected, found),
        }
    }
}

impl std::error::Error for CmError {}

pub struct CounterMatrix {
    data: Vec<Vec<u64>>,
    rows: usize,
    cols: usize,
    total_increments: u64,
    total_decrements: u64,
    total_adds: u64,
}

impl CounterMatrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self { data: vec![vec![0; cols]; rows], rows, cols, total_increments: 0, total_decrements: 0, total_adds: 0 }
    }

    pub fn get(&self, row: usize, col: usize) -> Result<u64, CmError> {
        self.data.get(row).and_then(|r| r.get(col)).copied().ok_or(CmError::OutOfBounds { row, col, rows: self.rows, cols: self.cols })
    }

    pub fn set(&mut self, row: usize, col: usize, val: u64) -> Result<(), CmError> {
        *self.data.get_mut(row).and_then(|r| r.get_mut(col)).ok_or(CmError::OutOfBounds { row, col, rows: self.rows, cols: self.cols })? = val;
        Ok(())
    }

    pub fn increment(&mut self, row: usize, col: usize) -> Result<u64, CmError> {
        let cell = self.data.get_mut(row).and_then(|r| r.get_mut(col)).ok_or(CmError::OutOfBounds { row, col, rows: self.rows, cols: self.cols })?;
        *cell = cell.saturating_add(1);
        self.total_increments += 1;
        Ok(*cell)
    }

    pub fn decrement(&mut self, row: usize, col: usize) -> Result<u64, CmError> {
        let cell = self.data.get_mut(row).and_then(|r| r.get_mut(col)).ok_or(CmError::OutOfBounds { row, col, rows: self.rows, cols: self.cols })?;
        *cell = cell.saturating_sub(1);
        self.total_decrements += 1;
        Ok(*cell)
    }

    pub fn add(&mut self, other: &CounterMatrix) -> Result<(), CmError> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err(CmError::DimensionMismatch { expected: (self.rows, self.cols), found: (other.rows, other.cols) });
        }
        for r in 0..self.rows {
            for c in 0..self.cols {
                self.data[r][c] = self.data[r][c].saturating_add(other.data[r][c]);
            }
        }
        self.total_adds += 1;
        Ok(())
    }

    pub fn row_sum(&self, row: usize) -> Option<u64> { self.data.get(row).map(|r| r.iter().sum()) }
    pub fn col_sum(&self, col: usize) -> Option<u64> { if col < self.cols { Some(self.data.iter().map(|r| r[col]).sum()) } else { None } }
    pub fn total_sum(&self) -> u64 { self.data.iter().flat_map(|r| r.iter()).sum() }
    pub fn max(&self) -> u64 { self.data.iter().flat_map(|r| r.iter()).copied().max().unwrap_or(0) }
    pub fn min(&self) -> u64 { self.data.iter().flat_map(|r| r.iter()).copied().min().unwrap_or(0) }

    pub fn transpose(&self) -> CounterMatrix {
        let mut result = CounterMatrix::new(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                result.data[c][r] = self.data[r][c];
            }
        }
        result
    }

    pub fn row(&self, r: usize) -> Option<&[u64]> { self.data.get(r).map(|v| v.as_slice()) }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn total_increments(&self) -> u64 { self.total_increments }
    pub fn total_decrements(&self) -> u64 { self.total_decrements }
    pub fn total_adds(&self) -> u64 { self.total_adds }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_matrix() { let m = CounterMatrix::new(3, 4); assert_eq!(m.rows(), 3); assert_eq!(m.cols(), 4); }

    #[test]
    fn set_get() {
        let mut m = CounterMatrix::new(3, 3);
        m.set(1, 2, 42).unwrap();
        assert_eq!(m.get(1, 2), Ok(42));
    }

    #[test]
    fn increment_decrement() {
        let mut m = CounterMatrix::new(3, 3);
        let v = m.increment(0, 0).unwrap();
        assert_eq!(v, 1);
        m.increment(0, 0).unwrap();
        let v = m.decrement(0, 0).unwrap();
        assert_eq!(v, 1);
    }

    #[test]
    fn add_matrices() {
        let mut a = CounterMatrix::new(2, 2);
        let mut b = CounterMatrix::new(2, 2);
        a.set(0, 0, 1).unwrap(); b.set(0, 0, 2).unwrap();
        a.add(&b).unwrap();
        assert_eq!(a.get(0, 0), Ok(3));
    }

    #[test]
    fn dimension_mismatch() {
        let mut a = CounterMatrix::new(2, 2);
        let b = CounterMatrix::new(3, 3);
        let err = a.add(&b).unwrap_err();
        assert!(matches!(err, CmError::DimensionMismatch { .. }));
    }

    #[test]
    fn row_col_sum() {
        let mut m = CounterMatrix::new(2, 3);
        m.set(0, 0, 1).unwrap(); m.set(0, 1, 2).unwrap(); m.set(0, 2, 3).unwrap();
        assert_eq!(m.row_sum(0), Some(6));
        assert_eq!(m.col_sum(0), Some(1));
    }

    #[test]
    fn transpose() {
        let mut m = CounterMatrix::new(2, 3);
        m.set(0, 1, 5).unwrap();
        let t = m.transpose();
        assert_eq!(t.get(1, 0), Ok(5));
        assert_eq!(t.rows(), 3);
        assert_eq!(t.cols(), 2);
    }

    #[test]
    fn total_sum() {
        let mut m = CounterMatrix::new(2, 2);
        m.set(0, 0, 1).unwrap(); m.set(1, 1, 3).unwrap();
        assert_eq!(m.total_sum(), 4);
    }

    #[test]
    fn out_of_bounds() {
        let m = CounterMatrix::new(2, 2);
        let err = m.get(5, 5).unwrap_err();
        assert!(matches!(err, CmError::OutOfBounds { .. }));
    }

    #[test]
    fn stats() {
        let mut m = CounterMatrix::new(2, 2);
        m.increment(0, 0).unwrap();
        m.decrement(0, 0).unwrap();
        assert_eq!(m.total_increments(), 1);
        assert_eq!(m.total_decrements(), 1);
    }
}
