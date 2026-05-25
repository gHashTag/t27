const PRIME: u64 = 0x9e3779b97f4a7c15;

fn mulh(a: u64, b: u64) -> u64 { ((a as u128 * b as u128) >> 64) as u64 }

fn round(h: u64, input: u64) -> u64 {
    let a = mulh(h ^ input, PRIME);
    let b = h.wrapping_add(input).wrapping_mul(PRIME);
    a ^ b
}

fn final_mix(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

pub struct GxHash {
    state: u64,
    len: usize,
    total_hashed: u64,
}

impl GxHash {
    pub fn new(seed: u64) -> Self { Self { state: seed, len: 0, total_hashed: 0 } }

    pub fn write_u64(&mut self, value: u64) {
        self.total_hashed += 1;
        self.state = round(self.state, value);
        self.len += 8;
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.total_hashed += 1;
        let mut i = 0;
        while i + 8 <= data.len() {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&data[i..i + 8]);
            self.state = round(self.state, u64::from_le_bytes(buf));
            i += 8;
        }
        if i < data.len() {
            let mut buf = [0u8; 8];
            buf[..data.len() - i].copy_from_slice(&data[i..]);
            buf[7] = (data.len() - i) as u8;
            self.state = round(self.state, u64::from_le_bytes(buf));
        }
        self.len += data.len();
    }

    pub fn finish(&self) -> u64 { final_mix(self.state ^ (self.len as u64).wrapping_mul(PRIME)) }

    pub fn avalanche_score(&self) -> f64 {
        let base = self.finish();
        let mut flipped = 0usize;
        let mut total = 0usize;
        for bit in 0..64 {
            let mut h2 = GxHash { state: self.state, len: self.len, total_hashed: self.total_hashed };
            h2.state ^= 1u64 << bit;
            let other = h2.finish();
            for b in 0..64 { if ((base >> b) & 1) != ((other >> b) & 1) { flipped += 1; } }
            total += 64;
        }
        flipped as f64 / total as f64
    }

    pub fn len(&self) -> usize { self.len }
    pub fn total_hashed(&self) -> u64 { self.total_hashed }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_hash() { let h = GxHash::new(0); assert_eq!(h.len(), 0); }

    #[test]
    fn single_u64() {
        let mut h = GxHash::new(42);
        h.write_u64(100);
        let v = h.finish();
        assert_ne!(v, 0);
    }

    #[test]
    fn deterministic() {
        let mut h1 = GxHash::new(0); h1.write_u64(1); h1.write_u64(2);
        let mut h2 = GxHash::new(0); h2.write_u64(1); h2.write_u64(2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn different_inputs() {
        let mut h1 = GxHash::new(0); h1.write_u64(1);
        let mut h2 = GxHash::new(0); h2.write_u64(2);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn bytes() {
        let mut h = GxHash::new(0);
        h.write_bytes(b"hello world");
        assert_ne!(h.finish(), 0);
        assert_eq!(h.len(), 11);
    }

    #[test]
    fn empty_bytes() {
        let mut h = GxHash::new(1);
        h.write_bytes(b"");
        let v = h.finish();
        assert_ne!(v, 0);
    }

    #[test]
    fn seed_difference() {
        let mut h1 = GxHash::new(0); h1.write_u64(1);
        let mut h2 = GxHash::new(99); h2.write_u64(1);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn avalanche() {
        let mut h = GxHash::new(0);
        h.write_u64(12345);
        let score = h.avalanche_score();
        assert!(score > 0.4);
    }

    #[test]
    fn stats() {
        let mut h = GxHash::new(0);
        h.write_u64(1); h.write_bytes(b"ab");
        assert_eq!(h.total_hashed(), 2);
    }
}
