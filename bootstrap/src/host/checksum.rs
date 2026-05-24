const MOD_ADLER: u64 = 65521;

#[derive(Debug, Clone, PartialEq)]
pub enum CsError {
    WindowEmpty,
    WindowFull { capacity: usize },
}

impl std::fmt::Display for CsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsError::WindowEmpty => write!(f, "window empty"),
            CsError::WindowFull { capacity } => write!(f, "window full ({capacity})"),
        }
    }
}

impl std::error::Error for CsError {}

pub struct RollingChecksum {
    a: u64,
    b: u64,
    window: Vec<u8>,
    capacity: usize,
    total_updates: u64,
    total_roll: u64,
    total_resets: u64,
}

impl RollingChecksum {
    pub fn new(capacity: usize) -> Self {
        Self { a: 1, b: 0, window: Vec::with_capacity(capacity), capacity, total_updates: 0, total_roll: 0, total_resets: 0 }
    }

    pub fn update(&mut self, byte: u8) -> Result<(), CsError> {
        if self.window.len() >= self.capacity { return Err(CsError::WindowFull { capacity: self.capacity }); }
        self.a = (self.a + byte as u64) % MOD_ADLER;
        self.b = (self.b + self.a) % MOD_ADLER;
        self.window.push(byte);
        self.total_updates += 1;
        Ok(())
    }

    pub fn roll(&mut self, new_byte: u8) -> Result<u8, CsError> {
        if self.window.is_empty() { return Err(CsError::WindowEmpty); }
        let old_byte = if self.window.len() >= self.capacity {
            let old = self.window.remove(0);
            old
        } else {
            self.window.remove(0)
        };
        self.a = (self.a + new_byte as u64 - old_byte as u64 + MOD_ADLER) % MOD_ADLER;
        self.b = (self.b + self.a - 1 - (self.window.len() as u64 + 1) * old_byte as u64 + MOD_ADLER * (self.window.len() as u64 + 1)) % MOD_ADLER;
        self.a = (self.a + new_byte as u64) % MOD_ADLER;
        self.b = (self.b + self.a) % MOD_ADLER;
        self.window.push(new_byte);
        self.total_roll += 1;
        Ok(old_byte)
    }

    pub fn checksum(&self) -> u32 { ((self.b << 16) | self.a) as u32 }

    pub fn digest(&self) -> u64 { (self.b << 16) | self.a }

    pub fn reset(&mut self) {
        self.a = 1;
        self.b = 0;
        self.window.clear();
        self.total_resets += 1;
    }

    pub fn from_slice(data: &[u8]) -> Self {
        let mut cs = Self::new(data.len());
        for &b in data { cs.update(b).unwrap(); }
        cs
    }

    pub fn verify(&self, data: &[u8]) -> bool {
        let other = Self::from_slice(data);
        self.digest() == other.digest()
    }

    pub fn window_len(&self) -> usize { self.window.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_roll(&self) -> u64 { self.total_roll }
    pub fn total_resets(&self) -> u64 { self.total_resets }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cs() { let cs = RollingChecksum::new(16); assert_eq!(cs.window_len(), 0); }

    #[test]
    fn update_digest() {
        let mut cs = RollingChecksum::new(16);
        cs.update(b'a').unwrap(); cs.update(b'b').unwrap(); cs.update(b'c').unwrap();
        assert_ne!(cs.digest(), 0);
    }

    #[test]
    fn deterministic() {
        let cs1 = RollingChecksum::from_slice(b"hello");
        let cs2 = RollingChecksum::from_slice(b"hello");
        assert_eq!(cs1.digest(), cs2.digest());
    }

    #[test]
    fn different_data() {
        let cs1 = RollingChecksum::from_slice(b"hello");
        let cs2 = RollingChecksum::from_slice(b"world");
        assert_ne!(cs1.digest(), cs2.digest());
    }

    #[test]
    fn verify_match() {
        let cs = RollingChecksum::from_slice(b"test");
        assert!(cs.verify(b"test"));
    }

    #[test]
    fn verify_mismatch() {
        let cs = RollingChecksum::from_slice(b"test");
        assert!(!cs.verify(b"fail"));
    }

    #[test]
    fn reset() {
        let mut cs = RollingChecksum::from_slice(b"abc");
        cs.reset();
        assert_eq!(cs.window_len(), 0);
    }

    #[test]
    fn window_full() {
        let mut cs = RollingChecksum::new(2);
        cs.update(1).unwrap(); cs.update(2).unwrap();
        let err = cs.update(3).unwrap_err();
        assert!(matches!(err, CsError::WindowFull { .. }));
    }

    #[test]
    fn from_slice() {
        let cs = RollingChecksum::from_slice(b"abc");
        assert_eq!(cs.window_len(), 3);
    }

    #[test]
    fn error_display() { assert!(CsError::WindowEmpty.to_string().contains("empty")); }
}
