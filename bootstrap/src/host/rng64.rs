pub struct Rng64 {
    s: [u64; 4],
}

impl Rng64 {
    pub fn new(seed: u64) -> Self {
        let mut s = [0u64; 4];
        let mut z = seed;
        for i in 0..4 {
            z = z.wrapping_add(0x9e3779b97f4a7c15);
            let mut t = z;
            t = (t ^ (t >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            t = (t ^ (t >> 27)).wrapping_mul(0x94d049bb133111eb);
            s[i] = t ^ (t >> 31);
        }
        Self { s }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5);
        let result = (result ^ (result >> 29)).wrapping_mul(0x9e3779b97f4a7c15);
        let result = result ^ (result >> 29);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    pub fn next_u32(&mut self) -> u32 { self.next_u64() as u32 }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn next_range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo)
    }

    pub fn next_bool(&mut self) -> bool { self.next_u64() & 1 == 1 }

    pub fn shuffle<T>(&mut self, data: &mut [T]) {
        for i in (1..data.len()).rev() {
            let j = self.next_range(0, (i + 1) as u64) as usize;
            data.swap(i, j);
        }
    }

    pub fn normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        let mag = (-2.0 * u1.ln()).sqrt();
        mag * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut r1 = Rng64::new(42);
        let mut r2 = Rng64::new(42);
        for _ in 0..10 { assert_eq!(r1.next_u64(), r2.next_u64()); }
    }

    #[test]
    fn different_seeds() {
        let mut r1 = Rng64::new(1);
        let mut r2 = Rng64::new(2);
        assert_ne!(r1.next_u64(), r2.next_u64());
    }

    #[test]
    fn f64_range() {
        let mut r = Rng64::new(42);
        for _ in 0..100 {
            let v = r.next_f64();
            assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn range() {
        let mut r = Rng64::new(42);
        for _ in 0..100 {
            let v = r.next_range(10, 20);
            assert!(v >= 10 && v < 20);
        }
    }

    #[test]
    fn shuffle() {
        let mut r = Rng64::new(42);
        let mut v: Vec<i32> = (0..20).collect();
        r.shuffle(&mut v);
        assert_ne!(v, (0..20).collect::<Vec<_>>());
        v.sort();
        assert_eq!(v, (0..20).collect::<Vec<_>>());
    }

    #[test]
    fn normal_range() {
        let mut r = Rng64::new(42);
        for _ in 0..100 { let v = r.normal(); assert!(v.abs() < 10.0); }
    }
}
