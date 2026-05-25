use std::collections::BTreeMap;

pub struct SpMatMul {
    row_ptrs: Vec<usize>,
    col_idx: Vec<usize>,
    vals: Vec<f64>,
    rows: usize,
    cols: usize,
    total_muls: u64,
    total_flops: u64,
}

impl SpMatMul {
    pub fn identity(n: usize) -> Self {
        let mut row_ptrs = vec![0; n + 1];
        let col_idx: Vec<usize> = (0..n).collect();
        let vals = vec![1.0; n];
        for i in 0..n { row_ptrs[i + 1] = i + 1; }
        Self { row_ptrs, col_idx, vals, rows: n, cols: n, total_muls: 0, total_flops: 0 }
    }

    pub fn from_dense(data: &[Vec<f64>]) -> Self {
        let rows = data.len();
        let cols = if rows > 0 { data[0].len() } else { 0 };
        let mut row_ptrs = vec![0; rows + 1];
        let mut col_idx = Vec::new();
        let mut vals = Vec::new();
        for (i, row) in data.iter().enumerate() {
            for (j, &v) in row.iter().enumerate() {
                if v != 0.0 { col_idx.push(j); vals.push(v); }
            }
            row_ptrs[i + 1] = vals.len();
        }
        Self { row_ptrs, col_idx, vals, rows, cols, total_muls: 0, total_flops: 0 }
    }

    pub fn multiply(&mut self, other: &SpMatMul) -> SpMatMul {
        self.total_muls += 1;
        let m = self.rows;
        let n = other.cols;
        let mut result_row_ptrs = vec![0; m + 1];
        let mut result_col_idx = Vec::new();
        let mut result_vals = Vec::new();
        for i in 0..m {
            let mut row_map: BTreeMap<usize, f64> = BTreeMap::new();
            for k_idx in self.row_ptrs[i]..self.row_ptrs[i + 1] {
                let k = self.col_idx[k_idx];
                let a_ik = self.vals[k_idx];
                for j_idx in other.row_ptrs[k]..other.row_ptrs[k + 1] {
                    let j = other.col_idx[j_idx];
                    let b_kj = other.vals[j_idx];
                    *row_map.entry(j).or_insert(0.0) += a_ik * b_kj;
                    self.total_flops += 2;
                }
            }
            for (j, v) in &row_map { result_col_idx.push(*j); result_vals.push(*v); }
            result_row_ptrs[i + 1] = result_vals.len();
        }
        SpMatMul { row_ptrs: result_row_ptrs, col_idx: result_col_idx, vals: result_vals, rows: m, cols: n, total_muls: 0, total_flops: 0 }
    }

    pub fn get(&self, row: usize, col: usize) -> f64 {
        for idx in self.row_ptrs[row]..self.row_ptrs[row + 1] {
            if self.col_idx[idx] == col { return self.vals[idx]; }
        }
        0.0
    }

    pub fn to_dense(&self) -> Vec<Vec<f64>> {
        let mut out = vec![vec![0.0; self.cols]; self.rows];
        for i in 0..self.rows {
            for idx in self.row_ptrs[i]..self.row_ptrs[i + 1] {
                out[i][self.col_idx[idx]] = self.vals[idx];
            }
        }
        out
    }

    pub fn nnz(&self) -> usize { self.vals.len() }
    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn total_muls(&self) -> u64 { self.total_muls }
    pub fn total_flops(&self) -> u64 { self.total_flops }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_mul() {
        let i = SpMatMul::identity(3);
        let mut ii = SpMatMul::identity(3);
        let r = ii.multiply(&i);
        assert!((r.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((r.get(1, 1) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn from_dense() {
        let m = SpMatMul::from_dense(&[vec![1.0, 0.0], vec![0.0, 2.0]]);
        assert_eq!(m.nnz(), 2);
        assert!((m.get(1, 1) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn multiply_result() {
        let a = SpMatMul::from_dense(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let b = SpMatMul::from_dense(&[vec![2.0, 0.0], vec![1.0, 2.0]]);
        let mut am = a;
        let c = am.multiply(&b);
        assert!((c.get(0, 0) - 4.0).abs() < 1e-10);
        assert!((c.get(0, 1) - 4.0).abs() < 1e-10);
        assert!((c.get(1, 0) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn to_dense() {
        let m = SpMatMul::from_dense(&[vec![1.0, 2.0], vec![3.0, 4.0]]);
        let d = m.to_dense();
        assert!((d[0][1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn nnz() {
        let m = SpMatMul::from_dense(&[vec![1.0, 0.0, 3.0]]);
        assert_eq!(m.nnz(), 2);
    }

    #[test]
    fn flops() {
        let a = SpMatMul::identity(2);
        let mut am = SpMatMul::identity(2);
        am.multiply(&a);
        assert!(am.total_flops() > 0);
    }

    #[test]
    fn stats() {
        let mut m = SpMatMul::identity(2);
        m.multiply(&SpMatMul::identity(2));
        assert_eq!(m.total_muls(), 1);
    }
}
