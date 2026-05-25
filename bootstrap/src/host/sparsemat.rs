#[derive(Debug, Clone, PartialEq)]
pub enum SmError {
    IndexOutOfRange { row: usize, col: usize, rows: usize, cols: usize },
    DimensionMismatch { a: (usize, usize), b: (usize, usize) },
}

impl std::fmt::Display for SmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmError::IndexOutOfRange { row, col, rows, cols } => write!(f, "({row},{col}) out of ({rows},{cols})"),
            SmError::DimensionMismatch { a, b } => write!(f, "dim mismatch {:?} vs {:?}", a, b),
        }
    }
}

impl std::error::Error for SmError {}

pub struct SparseMat {
    rows: usize,
    cols: usize,
    row_ptrs: Vec<usize>,
    col_indices: Vec<usize>,
    values: Vec<i64>,
    total_gets: u64,
    total_sets: u64,
    total_multiplies: u64,
}

impl SparseMat {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols, row_ptrs: vec![0; rows + 1], col_indices: Vec::new(), values: Vec::new(), total_gets: 0, total_sets: 0, total_multiplies: 0 }
    }

    pub fn from_triplets(rows: usize, cols: usize, triplets: &[(usize, usize, i64)]) -> Self {
        let mut sorted: Vec<_> = triplets.to_vec();
        sorted.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut row_ptrs = vec![0; rows + 1];
        let mut col_indices = Vec::new();
        let mut values = Vec::new();
        for (r, c, v) in &sorted {
            row_ptrs[r + 1] += 1;
            col_indices.push(*c);
            values.push(*v);
        }
        for i in 1..=rows { row_ptrs[i] += row_ptrs[i - 1]; }
        Self { rows, cols, row_ptrs, col_indices, values, total_gets: 0, total_sets: 0, total_multiplies: 0 }
    }

    pub fn get(&mut self, row: usize, col: usize) -> i64 {
        self.total_gets += 1;
        if row >= self.rows || col >= self.cols { return 0; }
        let start = self.row_ptrs[row];
        let end = self.row_ptrs[row + 1];
        for i in start..end {
            if self.col_indices[i] == col { return self.values[i]; }
        }
        0
    }

    pub fn set(&mut self, row: usize, col: usize, val: i64) -> Result<(), SmError> {
        if row >= self.rows || col >= self.cols { return Err(SmError::IndexOutOfRange { row, col, rows: self.rows, cols: self.cols }); }
        self.total_sets += 1;
        let start = self.row_ptrs[row];
        let end = self.row_ptrs[row + 1];
        for i in start..end {
            if self.col_indices[i] == col {
                self.values[i] = val;
                return Ok(());
            }
        }
        let insert_pos = self.row_ptrs[row + 1];
        self.col_indices.insert(insert_pos, col);
        self.values.insert(insert_pos, val);
        for r in (row + 1)..=self.rows { self.row_ptrs[r] += 1; }
        Ok(())
    }

    pub fn transpose(&self) -> SparseMat {
        let mut triplets = Vec::new();
        for r in 0..self.rows {
            for i in self.row_ptrs[r]..self.row_ptrs[r + 1] {
                triplets.push((self.col_indices[i], r, self.values[i]));
            }
        }
        SparseMat::from_triplets(self.cols, self.rows, &triplets)
    }

    pub fn multiply(&self, other: &SparseMat) -> Result<SparseMat, SmError> {
        if self.cols != other.rows { return Err(SmError::DimensionMismatch { a: (self.rows, self.cols), b: (other.rows, other.cols) }); }
        let mut triplets = Vec::new();
        for r in 0..self.rows {
            for i in self.row_ptrs[r]..self.row_ptrs[r + 1] {
                let k = self.col_indices[i];
                let a_val = self.values[i];
                for j in other.row_ptrs[k]..other.row_ptrs[k + 1] {
                    let c = other.col_indices[j];
                    let b_val = other.values[j];
                    triplets.push((r, c, a_val * b_val));
                }
            }
        }
        let mut combined: std::collections::BTreeMap<(usize, usize), i64> = std::collections::BTreeMap::new();
        for (r, c, v) in &triplets { *combined.entry((*r, *c)).or_insert(0) += v; }
        let final_triplets: Vec<_> = combined.into_iter().filter(|(_, v)| *v != 0).map(|((r, c), v)| (r, c, v)).collect();
        Ok(SparseMat::from_triplets(self.rows, other.cols, &final_triplets))
    }

    pub fn nnz(&self) -> usize { self.values.len() }
    pub fn density(&self) -> f64 { self.values.len() as f64 / (self.rows * self.cols).max(1) as f64 }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_sets(&self) -> u64 { self.total_sets }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sm() { let m = SparseMat::new(3, 3); assert_eq!(m.nnz(), 0); }

    #[test]
    fn from_triplets() {
        let m = SparseMat::from_triplets(2, 2, &[(0, 0, 1), (0, 1, 2), (1, 1, 3)]);
        let mut m = m;
        assert_eq!(m.get(0, 0), 1);
        assert_eq!(m.get(0, 1), 2);
        assert_eq!(m.get(1, 0), 0);
        assert_eq!(m.get(1, 1), 3);
    }

    #[test]
    fn set_get() {
        let mut m = SparseMat::new(3, 3);
        m.set(1, 2, 42).unwrap();
        assert_eq!(m.get(1, 2), 42);
        assert_eq!(m.get(0, 0), 0);
    }

    #[test]
    fn set_update() {
        let mut m = SparseMat::new(2, 2);
        m.set(0, 0, 1).unwrap(); m.set(0, 0, 99).unwrap();
        assert_eq!(m.get(0, 0), 99);
        assert_eq!(m.nnz(), 1);
    }

    #[test]
    fn transpose() {
        let m = SparseMat::from_triplets(2, 3, &[(0, 2, 7), (1, 0, 5)]);
        let t = m.transpose();
        let mut t = t;
        assert_eq!(t.rows(), 3);
        assert_eq!(t.cols(), 2);
        assert_eq!(t.get(2, 0), 7);
        assert_eq!(t.get(0, 1), 5);
    }

    #[test]
    fn multiply() {
        let a = SparseMat::from_triplets(2, 2, &[(0, 0, 1), (0, 1, 2), (1, 1, 3)]);
        let b = SparseMat::from_triplets(2, 2, &[(0, 0, 4), (1, 0, 5), (1, 1, 6)]);
        let c = a.multiply(&b).unwrap();
        let mut c = c;
        assert_eq!(c.get(0, 0), 14);
        assert_eq!(c.get(0, 1), 12);
        assert_eq!(c.get(1, 0), 15);
        assert_eq!(c.get(1, 1), 18);
    }

    #[test]
    fn dim_mismatch() {
        let a = SparseMat::from_triplets(2, 3, &[(0, 0, 1)]);
        let b = SparseMat::from_triplets(2, 2, &[(0, 0, 1)]);
        assert!(a.multiply(&b).is_err());
    }

    #[test]
    fn index_err() {
        let mut m = SparseMat::new(2, 2);
        assert!(m.set(5, 5, 1).is_err());
    }

    #[test]
    fn density() {
        let m = SparseMat::from_triplets(10, 10, &[(0, 0, 1), (5, 5, 2)]);
        assert!(m.density() < 0.05);
    }

    #[test]
    fn error_display() { assert!(SmError::DimensionMismatch { a: (1, 2), b: (3, 4) }.to_string().contains("mismatch")); }
}
