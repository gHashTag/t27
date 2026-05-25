#[derive(Debug, Clone, PartialEq)]
pub enum CsrErr {
    DimensionMismatch { expected: usize, got: usize },
}

impl std::fmt::Display for CsrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsrErr::DimensionMismatch { expected, got } => write!(f, "dim mismatch {expected} vs {got}"),
        }
    }
}

impl std::error::Error for CsrErr {}

pub struct CsrMatrix {
    row_ptrs: Vec<usize>,
    col_indices: Vec<usize>,
    values: Vec<f64>,
    rows: usize,
    cols: usize,
    total_muls: u64,
    total_norms: u64,
}

impl CsrMatrix {
    pub fn from_triplets(rows: usize, cols: usize, triplets: &[(usize, usize, f64)]) -> Self {
        let mut sorted = triplets.to_vec();
        sorted.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut row_ptrs = vec![0; rows + 1];
        let mut col_indices = Vec::new();
        let mut values = Vec::new();
        for (r, c, v) in &sorted { row_ptrs[r + 1] += 1; col_indices.push(*c); values.push(*v); }
        for i in 1..=rows { row_ptrs[i] += row_ptrs[i - 1]; }
        Self { row_ptrs, col_indices, values, rows, cols, total_muls: 0, total_norms: 0 }
    }

    pub fn get(&self, row: usize, col: usize) -> f64 {
        let start = self.row_ptrs[row];
        let end = self.row_ptrs[row + 1];
        for i in start..end {
            if self.col_indices[i] == col { return self.values[i]; }
        }
        0.0
    }

    pub fn mul_vec(&mut self, x: &[f64]) -> Result<Vec<f64>, CsrErr> {
        self.total_muls += 1;
        if x.len() != self.cols { return Err(CsrErr::DimensionMismatch { expected: self.cols, got: x.len() }); }
        let mut y = vec![0.0; self.rows];
        for r in 0..self.rows {
            let mut sum = 0.0;
            for i in self.row_ptrs[r]..self.row_ptrs[r + 1] {
                sum += self.values[i] * x[self.col_indices[i]];
            }
            y[r] = sum;
        }
        Ok(y)
    }

    pub fn transpose(&self) -> CsrMatrix {
        let mut triplets = Vec::new();
        for r in 0..self.rows {
            for i in self.row_ptrs[r]..self.row_ptrs[r + 1] {
                triplets.push((self.col_indices[i], r, self.values[i]));
            }
        }
        CsrMatrix::from_triplets(self.cols, self.rows, &triplets)
    }

    pub fn frobenius_norm(&self) -> f64 {
        self.values.iter().map(|v| v * v).sum::<f64>().sqrt()
    }

    pub fn nnz(&self) -> usize { self.values.len() }
    pub fn density(&self) -> f64 { self.values.len() as f64 / (self.rows * self.cols).max(1) as f64 }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn total_muls(&self) -> u64 { self.total_muls }
    pub fn total_norms(&self) -> u64 { self.total_norms }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_triplets() {
        let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 2.0), (1, 1, 3.0)]);
        assert_eq!(m.nnz(), 3);
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(1, 0), 0.0);
    }

    #[test]
    fn mul_vec() {
        let mut m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 2.0), (0, 1, 3.0), (1, 0, 1.0), (1, 1, 4.0)]);
        let y = m.mul_vec(&[1.0, 2.0]).unwrap();
        assert!((y[0] - 8.0).abs() < 1e-10);
        assert!((y[1] - 9.0).abs() < 1e-10);
    }

    #[test]
    fn mul_dim_err() {
        let mut m = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]);
        assert!(m.mul_vec(&[1.0, 2.0]).is_err());
    }

    #[test]
    fn transpose() {
        let m = CsrMatrix::from_triplets(2, 3, &[(0, 2, 7.0), (1, 0, 5.0)]);
        let t = m.transpose();
        assert_eq!(t.rows(), 3);
        assert_eq!(t.get(2, 0), 7.0);
        assert_eq!(t.get(0, 1), 5.0);
    }

    #[test]
    fn frobenius() {
        let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 3.0), (0, 1, 4.0)]);
        let norm = m.frobenius_norm();
        assert!((norm - 5.0).abs() < 1e-10);
    }

    #[test]
    fn density() {
        let m = CsrMatrix::from_triplets(10, 10, &[(0, 0, 1.0), (5, 5, 2.0)]);
        assert!(m.density() < 0.05);
    }

    #[test]
    fn zero_get() {
        let m = CsrMatrix::from_triplets(3, 3, &[(1, 1, 5.0)]);
        assert_eq!(m.get(0, 0), 0.0);
    }

    #[test]
    fn stats() {
        let mut m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]);
        m.mul_vec(&[1.0, 0.0]).unwrap();
        assert_eq!(m.total_muls(), 1);
    }

    #[test]
    fn error_display() { assert!(CsrErr::DimensionMismatch { expected: 3, got: 2 }.to_string().contains("mismatch")); }
}
