pub struct BloomRing {
    bits: Vec<u64>,
    size: usize,
    k: usize,
    epoch: u64,
    epoch_bits: usize,
    total_inserts: u64,
    total_queries: u64,
    total_fp: u64,
}

fn hash_pair(key: u64) -> (u64, u64) {
    let h1 = key.wrapping_mul(0x9e3779b97f4a7c15);
    let h2 = key.wrapping_mul(0xff51afd7ed558ccd);
    (h1, h2)
}

impl BloomRing {
    pub fn new(size: usize, k: usize, epoch_bits: usize) -> Self {
        Self { bits: vec![0; (size + 63) / 64], size: size.max(64), k: k.max(1), epoch: 0, epoch_bits: epoch_bits.max(1),
               total_inserts: 0, total_queries: 0, total_fp: 0 }
    }

    fn idx(&self, h: u64) -> usize { (h as usize) % self.size }

    pub fn insert(&mut self, key: u64) {
        self.total_inserts += 1;
        let (h1, h2) = hash_pair(key);
        for i in 0..self.k {
            let h = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let idx = self.idx(h);
            self.bits[idx / 64] |= 1 << (idx % 64);
        }
    }

    pub fn contains(&mut self, key: u64) -> bool {
        self.total_queries += 1;
        let (h1, h2) = hash_pair(key);
        for i in 0..self.k {
            let h = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let idx = self.idx(h);
            if self.bits[idx / 64] & (1 << (idx % 64)) == 0 { return false; }
        }
        true
    }

    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
        if self.epoch % (1 << self.epoch_bits) == 0 { self.clear(); }
    }

    pub fn clear(&mut self) { for b in &mut self.bits { *b = 0; } }

    pub fn fill_ratio(&self) -> f64 {
        let set = self.bits.iter().map(|w| w.count_ones() as usize).sum::<usize>();
        set as f64 / self.size as f64
    }

    pub fn epoch(&self) -> u64 { self.epoch }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_contains() {
        let mut br = BloomRing::new(256, 3, 4);
        br.insert(42);
        assert!(br.contains(42));
    }

    #[test]
    fn not_present() {
        let mut br = BloomRing::new(256, 3, 4);
        let mut fp = 0;
        for i in 1000..2000u64 { if br.contains(i) { fp += 1; } }
        assert!(fp < 50);
    }

    #[test]
    fn clear() {
        let mut br = BloomRing::new(256, 3, 4);
        br.insert(1); br.clear();
        assert!(!br.contains(1));
    }

    #[test]
    fn advance_epoch() {
        let mut br = BloomRing::new(256, 3, 2);
        br.insert(1);
        for _ in 0..3 { br.advance_epoch(); }
        assert_eq!(br.epoch(), 3);
    }

    #[test]
    fn fill_ratio() {
        let mut br = BloomRing::new(256, 3, 4);
        assert!(br.fill_ratio() < 0.01);
        for i in 0..50u64 { br.insert(i); }
        assert!(br.fill_ratio() > 0.1);
    }

    #[test]
    fn stats() {
        let mut br = BloomRing::new(256, 3, 4);
        br.insert(1); br.contains(1);
        assert_eq!(br.total_inserts(), 1);
        assert_eq!(br.total_queries(), 1);
    }
}
