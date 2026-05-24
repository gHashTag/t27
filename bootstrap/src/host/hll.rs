fn hash_value(data: &[u8]) -> u64 {
    const FNV: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = FNV;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(PRIME); }
    h
}

fn rho(w: u64, bits: u32) -> u8 {
    if w == 0 { return bits as u8; }
    let mut count = 1u32;
    let mut v = w;
    let top_bit = 1u64 << (bits - 1);
    while (v & top_bit) == 0 && count < bits { v <<= 1; count += 1; }
    count as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HllMode {
    Sparse,
    Dense,
}

#[derive(Debug, Clone)]
pub struct HyperLogLog {
    registers: Vec<u8>,
    precision: u8,
    m: usize,
    mode: HllMode,
    sparse_set: Vec<u64>,
    total_adds: u64,
}

impl HyperLogLog {
    pub fn new(precision: u8) -> Self {
        assert!((4..=16).contains(&precision), "precision must be 4..=16");
        let m = 1usize << precision;
        Self { registers: vec![0; m], precision, m, mode: HllMode::Sparse, sparse_set: Vec::new(), total_adds: 0 }
    }

    pub fn add(&mut self, data: &[u8]) {
        self.total_adds += 1;
        let hash = hash_value(data);
        let idx = (hash & ((1u64 << self.precision) - 1)) as usize;
        let w = hash >> self.precision;
        let bits = 64 - self.precision as u32;
        let rho_val = rho(w, bits);
        match self.mode {
            HllMode::Sparse => {
                self.sparse_set.push(hash);
                if self.sparse_set.len() > self.m * 6 { self.flush_sparse(); }
            }
            HllMode::Dense => {
                if rho_val > self.registers[idx] { self.registers[idx] = rho_val; }
            }
        }
    }

    fn flush_sparse(&mut self) {
        for hash in self.sparse_set.drain(..) {
            let idx = (hash & ((1u64 << self.precision) - 1)) as usize;
            let w = hash >> self.precision;
            let bits = 64 - self.precision as u32;
            let rho_val = rho(w, bits);
            if rho_val > self.registers[idx] { self.registers[idx] = rho_val; }
        }
        self.mode = HllMode::Dense;
    }

    pub fn count(&mut self) -> f64 {
        if self.mode == HllMode::Sparse { self.flush_sparse(); }
        let alpha_m = match self.m {
            16 => 0.673,
            32 => 0.697,
            64 => 0.709,
            _ => 0.7213 / (1.0 + 1.079 / self.m as f64),
        };
        let alpha = alpha_m * (self.m as f64) * (self.m as f64);
        let mut sum = 0.0f64;
        let mut zeros = 0usize;
        for &r in &self.registers {
            sum += 2f64.powi(-(r as i32));
            if r == 0 { zeros += 1; }
        }
        let estimate = alpha / sum;
        let small_range = 2.5 * (self.m as f64);
        if zeros == self.m {
            0.0
        } else if estimate <= small_range && zeros > 0 {
            let ratio = (self.m as f64) / (self.m as f64 - zeros as f64);
            (self.m as f64) * ratio.ln()
        } else {
            estimate
        }
    }

    pub fn merge(&mut self, other: &HyperLogLog) {
        assert_eq!(self.precision, other.precision, "precision mismatch");
        if self.mode == HllMode::Sparse { self.flush_sparse(); }
        for i in 0..self.m {
            let ov = if other.mode == HllMode::Dense { other.registers[i] } else { 0 };
            if ov > self.registers[i] { self.registers[i] = ov; }
        }
        self.total_adds += other.total_adds;
    }

    pub fn precision(&self) -> u8 { self.precision }
    pub fn mode(&self) -> HllMode { self.mode }
    pub fn total_adds(&self) -> u64 { self.total_adds }
    pub fn register_count(&self) -> usize { self.m }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_hll() {
        let hll = HyperLogLog::new(10);
        assert_eq!(hll.register_count(), 1024);
        assert_eq!(hll.mode(), HllMode::Sparse);
    }

    #[test]
    fn single_element() {
        let mut hll = HyperLogLog::new(4);
        hll.add(b"hello");
        let c = hll.count();
        assert!(c.is_finite(), "estimate should be finite, got {c}");
        assert!(c > 0.0, "estimate {c} should be positive");
    }

    #[test]
    fn distinct_elements() {
        let mut hll = HyperLogLog::new(10);
        for i in 0..1000u32 { hll.add(&i.to_le_bytes()); }
        let c = hll.count();
        assert!(c > 50.0 && c < 5000.0, "estimate {c} out of range for 1000 items");
    }

    #[test]
    fn monotonicity() {
        let mut h1 = HyperLogLog::new(12);
        let mut h2 = HyperLogLog::new(12);
        for i in 0..100u32 { h1.add(&i.to_le_bytes()); }
        for i in 0..10000u32 { h2.add(&i.to_le_bytes()); }
        assert!(h1.count() < h2.count(), "more distinct should estimate higher");
    }

    #[test]
    fn merge() {
        let mut h1 = HyperLogLog::new(10);
        let mut h2 = HyperLogLog::new(10);
        for i in 0..500u32 { h1.add(&i.to_le_bytes()); }
        for i in 500..1000u32 { h2.add(&i.to_le_bytes()); }
        h1.merge(&h2);
        let c = h1.count();
        assert!(c > 500.0 && c < 2000.0, "merged estimate {c} out of range");
    }

    #[test]
    fn sparse_to_dense() {
        let mut hll = HyperLogLog::new(4);
        assert_eq!(hll.mode(), HllMode::Sparse);
        for i in 0..1000u32 { hll.add(&i.to_le_bytes()); }
        assert_eq!(hll.mode(), HllMode::Dense);
    }

    #[test]
    fn precision_access() {
        let hll = HyperLogLog::new(8);
        assert_eq!(hll.precision(), 8);
    }

    #[test]
    fn total_adds() {
        let mut hll = HyperLogLog::new(8);
        hll.add(b"a"); hll.add(b"b"); hll.add(b"c");
        assert_eq!(hll.total_adds(), 3);
    }

    #[test]
    fn empty_count() {
        let mut hll = HyperLogLog::new(10);
        let c = hll.count();
        assert!(c >= 0.0 && c < 5.0);
    }

    #[test]
    fn large_cardinality() {
        let mut hll = HyperLogLog::new(14);
        for i in 0..50000u32 { hll.add(&i.to_le_bytes()); }
        let c = hll.count();
        assert!(c > 25000.0 && c < 100000.0, "estimate {c} out of range for 50k");
    }

    #[test]
    fn merge_adds() {
        let mut h1 = HyperLogLog::new(8);
        let mut h2 = HyperLogLog::new(8);
        h1.add(b"x"); h2.add(b"y");
        h1.merge(&h2);
        assert_eq!(h1.total_adds(), 2);
    }

    #[test]
    fn register_size() {
        let hll = HyperLogLog::new(4);
        assert_eq!(hll.register_count(), 16);
    }
}
