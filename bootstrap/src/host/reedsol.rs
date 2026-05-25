use std::cell::Cell;

const PRIM: u8 = 0x03;

#[derive(Debug, Clone, PartialEq)]
pub enum RsError {
    TooManyErasures { erasures: usize, parity: usize },
}

impl std::fmt::Display for RsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RsError::TooManyErasures { erasures, parity } => write!(f, "{erasures} erasures > {parity} parity"),
        }
    }
}

impl std::error::Error for RsError {}

struct GfCtx {
    exp_table: [u8; 512],
    log_table: [u8; 256],
    ops: Cell<u64>,
}

impl GfCtx {
    fn new() -> Self {
        let mut exp_table = [0u8; 512];
        let mut log_table = [0u8; 256];
        let mut x: u8 = 1;
        for i in 0..255 {
            exp_table[i] = x; exp_table[i + 255] = x;
            log_table[x as usize] = i as u8;
            x = x.wrapping_mul(PRIM); if x >= 128 { x ^= 0x11B; }
        }
        Self { exp_table, log_table, ops: Cell::new(0) }
    }

    fn mul(&self, a: u8, b: u8) -> u8 {
        self.ops.set(self.ops.get() + 1);
        if a == 0 || b == 0 { return 0; }
        self.exp_table[(self.log_table[a as usize] as usize + self.log_table[b as usize] as usize) % 255]
    }

    fn add(&self, a: u8, b: u8) -> u8 { a ^ b }
    fn sub(&self, a: u8, b: u8) -> u8 { a ^ b }
    fn inv(&self, a: u8) -> u8 { if a == 0 { panic!("inv(0)"); } self.exp_table[255 - self.log_table[a as usize] as usize] }

    fn poly_mul(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut r = vec![0u8; a.len() + b.len() - 1];
        for i in 0..a.len() { for j in 0..b.len() { r[i + j] = self.add(r[i + j], self.mul(a[i], b[j])); } }
        r
    }

    fn poly_eval(&self, p: &[u8], x: u8) -> u8 {
        let mut result = 0u8;
        for &c in p.iter().rev() { result = self.add(self.mul(result, x), c); }
        result
    }
}

pub struct ReedSol {
    gf: GfCtx,
    data_shards: usize,
    parity_shards: usize,
    total_encodes: u64,
    total_decodes: u64,
}

impl ReedSol {
    pub fn new(data_shards: usize, parity_shards: usize) -> Self {
        Self { gf: GfCtx::new(), data_shards, parity_shards, total_encodes: 0, total_decodes: 0 }
    }

    pub fn encode(&mut self, data: &[Vec<u8>]) -> Vec<Vec<u8>> {
        self.total_encodes += 1;
        let block_len = data[0].len();
        let mut parity = vec![vec![0u8; block_len]; self.parity_shards];
        for i in 0..block_len {
            for p in 0..self.parity_shards {
                let x = (self.data_shards + p) as u8;
                let mut val = 0u8;
                for d in 0..self.data_shards {
                    val = self.gf.add(val, self.gf.mul(data[d][i], self.gf.poly_eval(&[1, x], d as u8)));
                }
                parity[p][i] = val;
            }
        }
        parity
    }

    pub fn decode(&mut self, shards: &[Option<Vec<u8>>], erasure_locs: &[usize]) -> Result<Vec<Vec<u8>>, RsError> {
        self.total_decodes += 1;
        if erasure_locs.len() > self.parity_shards { return Err(RsError::TooManyErasures { erasures: erasure_locs.len(), parity: self.parity_shards }); }
        let block_len = shards.iter().find_map(|s| s.as_ref()).map(|s| s.len()).unwrap_or(0);
        let present: Vec<usize> = (0..shards.len()).filter(|&i| shards[i].is_some() && !erasure_locs.contains(&i)).take(self.data_shards).collect();
        if present.len() < self.data_shards { return Err(RsError::TooManyErasures { erasures: erasure_locs.len(), parity: self.parity_shards }); }
        let n = present.len();
        let mut matrix = vec![vec![0u8; n]; n];
        for (row, &col) in present.iter().enumerate() {
            let x = col as u8;
            let mut val = 1u8;
            for p in 0..n { matrix[row][p] = val; val = self.gf.mul(val, x); }
        }
        self.invert_matrix(&mut matrix);
        let mut result = vec![vec![0u8; block_len]; self.data_shards];
        for i in 0..block_len {
            let values: Vec<u8> = present.iter().map(|&j| shards[j].as_ref().unwrap()[i]).collect();
            for d in 0..self.data_shards {
                let mut val = 0u8;
                for j in 0..n { val = self.gf.add(val, self.gf.mul(matrix[d][j], values[j])); }
                result[d][i] = val;
            }
        }
        Ok(result)
    }

    fn invert_matrix(&self, mat: &mut [Vec<u8>]) {
        let n = mat.len();
        let mut inv = vec![vec![0u8; n]; n];
        for i in 0..n { inv[i][i] = 1; }
        for col in 0..n {
            let pivot = (col..n).find(|&r| mat[r][col] != 0).unwrap();
            mat.swap(col, pivot); inv.swap(col, pivot);
            let scale = self.gf.inv(mat[col][col]);
            for j in 0..n { mat[col][j] = self.gf.mul(mat[col][j], scale); inv[col][j] = self.gf.mul(inv[col][j], scale); }
            for row in 0..n {
                if row != col && mat[row][col] != 0 {
                    let factor = mat[row][col];
                    for j in 0..n { mat[row][j] = self.gf.sub(mat[row][j], self.gf.mul(factor, mat[col][j])); inv[row][j] = self.gf.sub(inv[row][j], self.gf.mul(factor, inv[col][j])); }
                }
            }
        }
        for i in 0..n { mat[i] = inv[i].clone(); }
    }

    pub fn data_shards(&self) -> usize { self.data_shards }
    pub fn parity_shards(&self) -> usize { self.parity_shards }
    pub fn total_encodes(&self) -> u64 { self.total_encodes }
    pub fn total_decodes(&self) -> u64 { self.total_decodes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rs() { let rs = ReedSol::new(4, 2); assert_eq!(rs.data_shards(), 4); }

    #[test]
    fn encode_decode_no_error() {
        let mut rs = ReedSol::new(4, 2);
        let data = vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]];
        let parity = rs.encode(&data);
        let mut shards: Vec<Option<Vec<u8>>> = data.into_iter().map(Some).chain(parity.into_iter().map(Some)).collect();
        let recovered = rs.decode(&shards, &[]).unwrap();
        assert_eq!(recovered[0], vec![1, 2]);
    }

    #[test]
    fn decode_one_erasure() {
        let mut rs = ReedSol::new(3, 2);
        let data = vec![vec![10, 20], vec![30, 40], vec![50, 60]];
        let parity = rs.encode(&data);
        let shards: Vec<Option<Vec<u8>>> = vec![
            Some(data[0].clone()), None,
            Some(data[2].clone()),
            Some(parity[0].clone()), Some(parity[1].clone()),
        ];
        let recovered = rs.decode(&shards, &[1]).unwrap();
        assert_eq!(recovered[1], vec![30, 40]);
    }

    #[test]
    fn too_many_erasures() {
        let mut rs = ReedSol::new(3, 1);
        let result = rs.decode(&[None, None, None, None], &[0, 1]);
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip() {
        let mut rs = ReedSol::new(4, 3);
        let data: Vec<Vec<u8>> = (0..4).map(|i| vec![i as u8, (i * 2) as u8]).collect();
        let parity = rs.encode(&data);
        assert_eq!(parity.len(), 3);
    }

    #[test]
    fn stats() {
        let mut rs = ReedSol::new(3, 2);
        let data = vec![vec![1], vec![2], vec![3]];
        rs.encode(&data);
        assert_eq!(rs.total_encodes(), 1);
    }

    #[test]
    fn error_display() { assert!(RsError::TooManyErasures { erasures: 5, parity: 2 }.to_string().contains("5")); }
}
