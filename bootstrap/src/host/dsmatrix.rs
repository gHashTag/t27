use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DsErr {
    DimensionMismatch { expected: (usize, usize), got: (usize, usize) },
    OutOfBounds { row: usize, col: usize, rows: usize, cols: usize },
}

impl std::fmt::Display for DsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DsErr::DimensionMismatch { expected, got } => write!(f, "dim mismatch {:?} vs {:?}", expected, got),
            DsErr::OutOfBounds { row, col, rows, cols } => write!(f, "({row},{col}) out of ({rows},{cols})"),
        }
    }
}

impl std::error::Error for DsErr {}

pub struct DsMatrix {
    data: BTreeMap<(usize, usize), f64>,
    rows: usize,
    cols: usize,
    total_muls: u64,
    total_adds: u64,
}

impl DsMatrix {
    pub fn new(rows: usize, cols: usize) -> Self { Self { data: BTreeMap::new(), rows, cols, total_muls: 0, total_adds: 0 } }

    pub fn set(&mut self, row: usize, col: usize, val: f64) -> Result<(), DsErr> {
        if row >= self.rows || col >= self.cols { return Err(DsErr::OutOfBounds { row, col, rows: self.rows, cols: self.cols }); }
        if val != 0.0 { self.data.insert((row, col), val); } else { self.data.remove(&(row, col)); }
        Ok(())
    }

    pub fn get(&self, row: usize, col: usize) -> f64 { self.data.get(&(row, col)).copied().unwrap_or(0.0) }

    pub fn add(&self, other: &DsMatrix) -> Result<DsMatrix, DsErr> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err(DsErr::DimensionMismatch { expected: (self.rows, self.cols), got: (other.rows, other.cols) });
        }
        let mut result = DsMatrix::new(self.rows, self.cols);
        result.total_adds += 1;
        for (&(r, c), &v) in &self.data { result.data.insert((r, c), v); }
        for (&(r, c), &v) in &other.data {
            let sum = result.data.get(&(r, c)).copied().unwrap_or(0.0) + v;
            if sum != 0.0 { result.data.insert((r, c), sum); } else { result.data.remove(&(r, c)); }
        }
        Ok(result)
    }

    pub fn mul(&self, other: &DsMatrix) -> Result<DsMatrix, DsErr> {
        if self.cols != other.rows {
            return Err(DsErr::DimensionMismatch { expected: (self.cols, self.cols), got: (other.rows, other.rows) });
        }
        let mut result = DsMatrix::new(self.rows, other.cols);
        result.total_muls += 1;
        let mut row_data: BTreeMap<usize, Vec<(usize, f64)>> = BTreeMap::new();
        for (&(r, c), &v) in &other.data { row_data.entry(r).or_default().push((c, v)); }
        for (&(i, k), &a_ik) in &self.data {
            if let Some(row) = row_data.get(&k) {
                for (j, b_kj) in row {
                    let val = result.data.get(&(i, *j)).copied().unwrap_or(0.0) + a_ik * b_kj;
                    if val != 0.0 { result.data.insert((i, *j), val); } else { result.data.remove(&(i, *j)); }
                }
            }
        }
        Ok(result)
    }

    pub fn scale(&self, s: f64) -> DsMatrix {
        let mut result = DsMatrix::new(self.rows, self.cols);
        for (&k, &v) in &self.data { result.data.insert(k, v * s); }
        result
    }

    pub fn transpose(&self) -> DsMatrix {
        let mut result = DsMatrix::new(self.cols, self.rows);
        for (&(r, c), &v) in &self.data { result.data.insert((c, r), v); }
        result
    }

    pub fn nnz(&self) -> usize { self.data.len() }
    pub fn density(&self) -> f64 { self.data.len() as f64 / (self.rows * self.cols).max(1) as f64 }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn total_muls(&self) -> u64 { self.total_muls }
    pub fn total_adds(&self) -> u64 { self.total_adds }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        let mut m = DsMatrix::new(3, 3);
        m.set(0, 0, 5.0).unwrap();
        assert!((m.get(0, 0) - 5.0).abs() < 1e-10);
        assert!((m.get(1, 1)).abs() < 1e-10);
    }

    #[test]
    fn add() {
        let mut a = DsMatrix::new(2, 2);
        a.set(0, 0, 1.0).unwrap(); a.set(0, 1, 2.0).unwrap();
        let mut b = DsMatrix::new(2, 2);
        b.set(0, 0, 3.0).unwrap(); b.set(1, 1, 4.0).unwrap();
        let c = a.add(&b).unwrap();
        assert!((c.get(0, 0) - 4.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn mul() {
        let mut a = DsMatrix::new(2, 3);
        a.set(0, 0, 1.0).unwrap(); a.set(0, 1, 2.0).unwrap(); a.set(0, 2, 3.0).unwrap();
        let mut b = DsMatrix::new(3, 2);
        b.set(0, 0, 1.0).unwrap(); b.set(1, 0, 1.0).unwrap(); b.set(2, 0, 1.0).unwrap();
        let c = a.mul(&b).unwrap();
        assert!((c.get(0, 0) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn transpose() {
        let mut m = DsMatrix::new(2, 3);
        m.set(0, 2, 7.0).unwrap();
        let t = m.transpose();
        assert_eq!(t.rows(), 3);
        assert!((t.get(2, 0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn scale() {
        let mut m = DsMatrix::new(2, 2);
        m.set(0, 0, 3.0).unwrap();
        let s = m.scale(2.0);
        assert!((s.get(0, 0) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn zero_removes() {
        let mut m = DsMatrix::new(2, 2);
        m.set(0, 0, 5.0).unwrap();
        m.set(0, 0, 0.0).unwrap();
        assert_eq!(m.nnz(), 0);
    }

    #[test]
    fn out_of_bounds() { assert!(DsMatrix::new(2, 2).set(5, 0, 1.0).is_err()); }

    #[test]
    fn dim_mismatch() {
        let a = DsMatrix::new(2, 3);
        let b = DsMatrix::new(4, 2);
        assert!(a.mul(&b).is_err());
    }

    #[test]
    fn density() {
        let mut m = DsMatrix::new(10, 10);
        m.set(0, 0, 1.0).unwrap();
        assert!(m.density() < 0.05);
    }

    #[test]
    fn stats() {
        let mut m = DsMatrix::new(2, 2);
        m.set(0, 0, 1.0).unwrap();
        let n = m.add(&m);
        assert!(n.is_ok());
    }
}
