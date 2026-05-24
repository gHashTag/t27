use std::collections::BTreeMap;

fn fnv_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hll2Error {
    MergePrecisionMismatch { self_p: u8, other_p: u8 },
}

impl std::fmt::Display for Hll2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Hll2Error::MergePrecisionMismatch { self_p, other_p } => write!(f, "precision mismatch: {self_p} vs {other_p}"),
        }
    }
}

impl std::error::Error for Hll2Error {}

#[derive(Clone)]
pub struct HyperLogLog2 {
    precision: u8,
    registers: Vec<u8>,
    sparse: BTreeMap<u32, u8>,
    is_sparse: bool,
    total_adds: u64,
    total_merges: u64,
    total_estimates: u64,
}

impl HyperLogLog2 {
    pub fn new(precision: u8) -> Self {
        let m = 1 << precision;
        Self { precision, registers: vec![0; m], sparse: BTreeMap::new(), is_sparse: true, total_adds: 0, total_merges: 0, total_estimates: 0 }
    }

    fn to_dense(&mut self) {
        if !self.is_sparse { return; }
        let m = 1 << self.precision;
        self.registers = vec![0; m];
        for (&idx, &val) in &self.sparse { self.registers[idx as usize] = val; }
        self.sparse.clear();
        self.is_sparse = false;
    }

    pub fn add(&mut self, value: u64) {
        self.total_adds += 1;
        let hash = fnv_hash(&value.to_le_bytes());
        let idx = (hash & ((1u64 << self.precision) - 1)) as u32;
        let w = hash >> self.precision;
        let rho = w.trailing_zeros() as u8 + 1;
        if self.is_sparse {
            let entry = self.sparse.entry(idx).or_insert(0);
            if rho > *entry { *entry = rho; }
            if self.sparse.len() > (1 << self.precision) / 4 { self.to_dense(); }
        } else {
            if rho > self.registers[idx as usize] { self.registers[idx as usize] = rho; }
        }
    }

    pub fn estimate(&mut self) -> f64 {
        self.total_estimates += 1;
        let regs = if self.is_sparse {
            self.to_dense();
            &self.registers
        } else { &self.registers };
        let m = regs.len() as f64;
        let alpha = match regs.len() {
            16 => 0.673,
            32 => 0.697,
            64 => 0.709,
            _ => 0.7213 / (1.0 + 1.079 / m),
        };
        let z: f64 = regs.iter().map(|&r| 2.0f64.powi(-(r as i32))).sum();
        let e = alpha * m * m / z;
        let e = if e <= 2.5 * m {
            let zeros = regs.iter().filter(|&&r| r == 0).count() as f64;
            if zeros > 0.0 { m * (m / zeros).ln() } else { e }
        } else { e };
        e
    }

    pub fn merge(&mut self, other: &HyperLogLog2) -> Result<(), Hll2Error> {
        if self.precision != other.precision { return Err(Hll2Error::MergePrecisionMismatch { self_p: self.precision, other_p: other.precision }); }
        self.to_dense();
        for i in 0..self.registers.len() {
            let other_reg = if other.is_sparse { *other.sparse.get(&(i as u32)).unwrap_or(&0) } else { other.registers[i] };
            if other_reg > self.registers[i] { self.registers[i] = other_reg; }
        }
        self.total_merges += 1;
        Ok(())
    }

    pub fn precision(&self) -> u8 { self.precision }
    pub fn is_sparse(&self) -> bool { self.is_sparse }
    pub fn register_count(&self) -> usize { 1 << self.precision }
    pub fn total_adds(&self) -> u64 { self.total_adds }
    pub fn total_merges(&self) -> u64 { self.total_merges }
    pub fn total_estimates(&self) -> u64 { self.total_estimates }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_hll() { let h = HyperLogLog2::new(12); assert!(h.is_sparse()); assert_eq!(h.register_count(), 4096); }

    #[test]
    fn single_element() {
        let mut h = HyperLogLog2::new(12);
        h.add(1);
        let est = h.estimate();
        assert!(est >= 0.5 && est <= 4.0);
    }

    #[test]
    fn many_distinct() {
        let mut h = HyperLogLog2::new(14);
        for i in 0..10000 { h.add(i); }
        let est = h.estimate();
        assert!(est > 5000.0 && est < 20000.0);
    }

    #[test]
    fn duplicates() {
        let mut h = HyperLogLog2::new(12);
        for _ in 0..1000 { h.add(42); }
        let est = h.estimate();
        assert!(est < 3.0);
    }

    #[test]
    fn merge() {
        let mut h1 = HyperLogLog2::new(12);
        let mut h2 = HyperLogLog2::new(12);
        for i in 0..500 { h1.add(i); }
        for i in 500..1000 { h2.add(i); }
        h1.merge(&h2).unwrap();
        let est = h1.estimate();
        assert!(est > 500.0 && est < 2000.0);
    }

    #[test]
    fn precision_mismatch() {
        let mut h1 = HyperLogLog2::new(10);
        let h2 = HyperLogLog2::new(12);
        let err = h1.merge(&h2).unwrap_err();
        assert!(matches!(err, Hll2Error::MergePrecisionMismatch { .. }));
    }

    #[test]
    fn sparse_to_dense() {
        let mut h = HyperLogLog2::new(4);
        for i in 0..1000 { h.add(i); }
        assert!(!h.is_sparse());
    }

    #[test]
    fn commutative() {
        let mut h1 = HyperLogLog2::new(10);
        let mut h2 = HyperLogLog2::new(10);
        for i in 0..100 { h1.add(i); h2.add(i + 100); }
        let mut merged1 = h1.clone(); merged1.merge(&h2).unwrap();
        let mut merged2 = h2.clone(); merged2.merge(&h1).unwrap();
        let e1 = merged1.estimate();
        let e2 = merged2.estimate();
        assert!((e1 - e2).abs() / e1.max(e2) < 0.1);
    }

    #[test]
    fn stats() {
        let mut h = HyperLogLog2::new(12);
        h.add(1); h.estimate();
        assert_eq!(h.total_adds(), 1);
        assert_eq!(h.total_estimates(), 1);
    }

    #[test]
    fn error_display() { assert!(Hll2Error::MergePrecisionMismatch { self_p: 4, other_p: 8 }.to_string().contains("mismatch")); }
}
