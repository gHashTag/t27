pub struct HyperLogLog {
    registers: Vec<u8>,
    precision: usize,
    total_adds: u64,
}

fn rho(w: u64) -> u8 { (w.trailing_zeros() + 1) as u8 }

fn hash64(val: u64) -> u64 {
    let mut h = val.wrapping_mul(0x9e3779b97f4a7c15);
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

impl HyperLogLog {
    pub fn new(precision: usize) -> Self {
        let precision = precision.clamp(4, 16);
        Self { registers: vec![0; 1 << precision], precision, total_adds: 0 }
    }

    pub fn add(&mut self, val: u64) {
        self.total_adds += 1;
        let h = hash64(val);
        let m = 1u64 << self.precision;
        let idx = (h & (m - 1)) as usize;
        let w = (h >> self.precision) | (1u64 << (64 - self.precision));
        let rank = rho(w);
        if rank > self.registers[idx] { self.registers[idx] = rank; }
    }

    pub fn count(&self) -> f64 {
        let m = self.registers.len() as f64;
        let alpha = match self.registers.len() {
            16 => 0.673,
            32 => 0.697,
            64 => 0.709,
            _ => 0.7213 / (1.0 + 1.079 / m),
        };
        let sum: f64 = self.registers.iter().map(|&r| 2f64.powi(-(r as i32))).sum();
        let e = alpha * m * m / sum;
        let zeros = self.registers.iter().filter(|&&r| r == 0).count() as f64;
        if zeros > 0.0 && e < 2.5 * m { (m * (m / zeros).ln()).max(0.0) } else { e }
    }

    pub fn merge(&mut self, other: &HyperLogLog) {
        assert_eq!(self.precision, other.precision);
        for i in 0..self.registers.len() { self.registers[i] = self.registers[i].max(other.registers[i]); }
    }

    pub fn total_adds(&self) -> u64 { self.total_adds }
    pub fn precision(&self) -> usize { self.precision }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() { let hll = HyperLogLog::new(10); assert!(hll.count() < 10.0); }

    #[test]
    fn single() {
        let mut hll = HyperLogLog::new(10);
        hll.add(42);
        assert!(hll.count() > 0.0);
    }

    #[test]
    fn many_distinct() {
        let mut hll = HyperLogLog::new(12);
        for i in 0..10000u64 { hll.add(i); }
        let est = hll.count();
        assert!(est > 5000.0 && est < 20000.0, "estimate was {est}");
    }

    #[test]
    fn duplicates() {
        let mut hll = HyperLogLog::new(10);
        for _ in 0..1000 { hll.add(42); }
        let est = hll.count();
        assert!(est < 10.0, "duplicate estimate was {est}");
    }

    #[test]
    fn merge() {
        let mut h1 = HyperLogLog::new(10);
        let mut h2 = HyperLogLog::new(10);
        for i in 0..500u64 { h1.add(i); }
        for i in 500..1000u64 { h2.add(i); }
        h1.merge(&h2);
        let est = h1.count();
        assert!(est > 100.0 && est < 5000.0, "merge estimate was {est}");
    }

    #[test]
    fn stats() {
        let mut hll = HyperLogLog::new(10);
        hll.add(1); hll.add(2);
        assert_eq!(hll.total_adds(), 2);
    }
}
