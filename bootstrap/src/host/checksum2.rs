const MOD: u32 = 65521;

pub struct Checksum2 {
    a: u32,
    b: u32,
    total_bytes: u64,
    total_rollbacks: u64,
}

impl Checksum2 {
    pub fn new() -> Self { Self { a: 1, b: 0, total_bytes: 0, total_rollbacks: 0 } }

    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.a = (self.a + byte as u32) % MOD;
            self.b = (self.b + self.a) % MOD;
            self.total_bytes += 1;
        }
    }

    pub fn finalize(&self) -> u32 { (self.b << 16) | self.a }

    pub fn reset(&mut self) { self.a = 1; self.b = 0; self.total_bytes = 0; self.total_rollbacks = 0; }

    pub fn roll(&mut self, old_byte: u8, new_byte: u8, window_len: usize) {
        self.total_rollbacks += 1;
        let old = old_byte as u32;
        let nw = new_byte as u32;
        let n = window_len as u32;
        self.a = (self.a + MOD - old % MOD + nw) % MOD;
        self.b = (self.b + MOD - n * old % MOD + self.a + MOD - 1) % MOD;
        self.total_bytes += 1;
    }

    pub fn checksum(data: &[u8]) -> u32 {
        let mut cs = Self::new();
        cs.update(data);
        cs.finalize()
    }

    pub fn verify(data: &[u8], expected: u32) -> bool { Self::checksum(data) == expected }

    pub fn total_bytes(&self) -> u64 { self.total_bytes }
    pub fn total_rollbacks(&self) -> u64 { self.total_rollbacks }
    pub fn a(&self) -> u32 { self.a }
    pub fn b(&self) -> u32 { self.b }
}

impl Default for Checksum2 {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() { assert_eq!(Checksum2::checksum(b""), 1); }

    #[test]
    fn known_value() {
        let cs = Checksum2::checksum(b"Adler-32");
        assert_ne!(cs, 0);
        assert_eq!(cs, {
            let mut cs2 = Checksum2::new();
            cs2.update(b"Adler-32");
            cs2.finalize()
        });
    }

    #[test]
    fn incremental() {
        let mut cs = Checksum2::new();
        cs.update(b"Hello"); cs.update(b" ");
        cs.update(b"World");
        assert_eq!(cs.finalize(), Checksum2::checksum(b"Hello World"));
    }

    #[test]
    fn reset() {
        let mut cs = Checksum2::new();
        cs.update(b"data"); cs.reset();
        assert_eq!(cs.finalize(), 1);
    }

    #[test]
    fn roll() {
        let window = b"ABCDE";
        let mut cs = Checksum2::new();
        cs.update(window);
        let c1 = cs.finalize();
        cs.roll(b'A', b'F', 5);
        let c2 = cs.finalize();
        assert_ne!(c1, c2);
        assert_eq!(c2, Checksum2::checksum(b"BCDEF"));
    }

    #[test]
    fn verify_ok() { assert!(Checksum2::verify(b"test", Checksum2::checksum(b"test"))); }
    #[test]
    fn verify_fail() { assert!(!Checksum2::verify(b"test", 0)); }

    #[test]
    fn large_data() {
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let cs = Checksum2::checksum(&data);
        assert_ne!(cs, 0);
    }

    #[test]
    fn stats() {
        let mut cs = Checksum2::new();
        cs.update(b"abc");
        assert_eq!(cs.total_bytes(), 3);
    }
}
