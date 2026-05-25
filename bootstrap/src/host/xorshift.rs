pub struct XorShift {
    state: u64,
    total_calls: u64,
}

impl XorShift {
    pub fn new(seed: u64) -> Self { Self { state: if seed == 0 { 0xdeadbeefcafe1234 } else { seed }, total_calls: 0 } }

    pub fn next_u64(&mut self) -> u64 {
        self.total_calls += 1;
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545F4914F6CDD1D)
    }

    pub fn next_u32(&mut self) -> u32 { self.next_u64() as u32 }

    pub fn next_bool(&mut self) -> bool { self.next_u64() & 1 == 1 }

    pub fn next_range(&mut self, lo: u64, hi: u64) -> u64 { lo + (self.next_u64() % (hi - lo + 1)) }

    pub fn next_float(&mut self) -> f64 { (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64 }

    pub fn total_calls(&self) -> u64 { self.total_calls }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = XorShift::new(42);
        let mut b = XorShift::new(42);
        for _ in 0..100 { assert_eq!(a.next_u64(), b.next_u64()); }
    }

    #[test]
    fn nonzero_seed() {
        let mut rng = XorShift::new(0);
        assert_ne!(rng.next_u64(), 0);
    }

    #[test]
    fn range() {
        let mut rng = XorShift::new(42);
        for _ in 0..1000 { let v = rng.next_range(5, 10); assert!(v >= 5 && v <= 10); }
    }

    #[test]
    fn float_range() {
        let mut rng = XorShift::new(42);
        for _ in 0..1000 {
            let f = rng.next_float();
            assert!(f >= 0.0 && f < 1.0);
        }
    }

    #[test]
    fn distribution() {
        let mut rng = XorShift::new(42);
        let mut counts = [0usize; 2];
        for _ in 0..10000 { counts[rng.next_bool() as usize] += 1; }
        assert!(counts[0] > 4000 && counts[1] > 4000);
    }

    #[test]
    fn stats() {
        let mut rng = XorShift::new(42);
        rng.next_u64(); rng.next_u32();
        assert_eq!(rng.total_calls(), 2);
    }
}
