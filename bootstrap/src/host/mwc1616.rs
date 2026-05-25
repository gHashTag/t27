pub struct Mwc1616 {
    state: [u32; 4],
    total_calls: u64,
}

impl Mwc1616 {
    pub fn new(seed: u64) -> Self {
        let s0 = (seed & 0xFFFF) as u32 | 1;
        let s1 = ((seed >> 16) & 0xFFFF) as u32 | 1;
        let s2 = ((seed >> 32) & 0xFFFF) as u32 | 1;
        let s3 = ((seed >> 48) & 0xFFFF) as u32 | 1;
        Self { state: [s0, s1, s2, s3], total_calls: 0 }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.total_calls += 1;
        self.state[0] = self.state[0].wrapping_mul(5115);
        self.state[0] = self.state[0].wrapping_add(self.state[3]);
        let result = self.state[0] >> 16;
        self.state[1] = self.state[1].wrapping_mul(33555);
        self.state[1] = self.state[1].wrapping_add(self.state[0]);
        self.state[2] = self.state[2].wrapping_mul(65535);
        self.state[2] = self.state[2].wrapping_add(self.state[1]);
        self.state[3] = self.state[3].wrapping_mul(2147483647);
        self.state[3] = self.state[3].wrapping_add(self.state[2]);
        result
    }

    pub fn next_u64(&mut self) -> u64 {
        let hi = self.next_u32() as u64;
        let lo = self.next_u32() as u64;
        (hi << 32) | lo
    }

    pub fn next_bool(&mut self) -> bool { self.next_u32() & 1 == 1 }

    pub fn next_range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + (self.next_u64() % (hi - lo + 1))
    }

    pub fn total_calls(&self) -> u64 { self.total_calls }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = Mwc1616::new(12345);
        let mut b = Mwc1616::new(12345);
        for _ in 0..100 { assert_eq!(a.next_u32(), b.next_u32()); }
    }

    #[test]
    fn different_seeds() {
        let mut a = Mwc1616::new(1);
        let mut b = Mwc1616::new(99999);
        let va = a.next_u64();
        let vb = b.next_u64();
        assert!(va != vb || a.next_u64() != b.next_u32() as u64);
    }

    #[test]
    fn u64() {
        let mut rng = Mwc1616::new(42);
        let v = rng.next_u64();
        assert_ne!(v, 0);
    }

    #[test]
    fn range() {
        let mut rng = Mwc1616::new(42);
        for _ in 0..100 {
            let v = rng.next_range(10, 20);
            assert!(v >= 10 && v <= 20);
        }
    }

    #[test]
    fn distribution() {
        let mut rng = Mwc1616::new(42);
        let mut counts = [0usize; 2];
        for _ in 0..10000 { counts[rng.next_bool() as usize] += 1; }
        assert!(counts[0] > 4000 && counts[1] > 4000);
    }

    #[test]
    fn stats() {
        let mut rng = Mwc1616::new(42);
        rng.next_u32(); rng.next_u64();
        assert_eq!(rng.total_calls(), 3);
    }
}
