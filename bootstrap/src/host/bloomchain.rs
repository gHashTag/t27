use std::collections::BTreeMap;

fn fnv_hash(seed: u64, data: &[u8]) -> u64 {
    let mut h = seed;
    for &b in data { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

#[derive(Debug, Clone, PartialEq)]
pub enum BcError {
    LayerNotFound { layer: usize },
}

impl std::fmt::Display for BcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BcError::LayerNotFound { layer } => write!(f, "layer {layer} not found"),
        }
    }
}

impl std::error::Error for BcError {}

struct BloomLayer {
    bits: Vec<u64>,
    size: usize,
    num_hashes: usize,
    count: usize,
}

impl BloomLayer {
    fn new(size: usize, num_hashes: usize) -> Self {
        let words = (size + 63) / 64;
        Self { bits: vec![0; words], size, num_hashes, count: 0 }
    }

    fn insert(&mut self, item: u64) {
        let bytes = item.to_le_bytes();
        for i in 0..self.num_hashes {
            let h = fnv_hash(0xcbf29ce484222325 + i as u64, &bytes);
            let idx = (h as usize) % self.size;
            self.bits[idx / 64] |= 1 << (idx % 64);
        }
        self.count += 1;
    }

    fn contains(&self, item: u64) -> bool {
        let bytes = item.to_le_bytes();
        for i in 0..self.num_hashes {
            let h = fnv_hash(0xcbf29ce484222325 + i as u64, &bytes);
            let idx = (h as usize) % self.size;
            if self.bits[idx / 64] & (1 << (idx % 64)) == 0 { return false; }
        }
        true
    }

    fn saturation(&self) -> f64 {
        let ones: usize = self.bits.iter().map(|w| w.count_ones() as usize).sum();
        ones as f64 / self.size as f64
    }
}

pub struct BloomChain {
    layers: Vec<BloomLayer>,
    total_inserts: u64,
    total_contains: u64,
    total_hits: u64,
    total_misses: u64,
}

impl BloomChain {
    pub fn new(config: &[(usize, usize)]) -> Self {
        let layers: Vec<_> = config.iter().map(|&(size, hashes)| BloomLayer::new(size, hashes)).collect();
        Self { layers, total_inserts: 0, total_contains: 0, total_hits: 0, total_misses: 0 }
    }

    pub fn insert(&mut self, item: u64) {
        self.total_inserts += 1;
        for layer in &mut self.layers { layer.insert(item); }
    }

    pub fn contains(&mut self, item: u64) -> bool {
        self.total_contains += 1;
        for layer in &self.layers {
            if !layer.contains(item) {
                self.total_misses += 1;
                return false;
            }
        }
        self.total_hits += 1;
        true
    }

    pub fn layer_saturation(&self, layer: usize) -> Result<f64, BcError> {
        let l = self.layers.get(layer).ok_or(BcError::LayerNotFound { layer })?;
        Ok(l.saturation())
    }

    pub fn layer_count(&self, layer: usize) -> Result<usize, BcError> {
        let l = self.layers.get(layer).ok_or(BcError::LayerNotFound { layer })?;
        Ok(l.count)
    }

    pub fn num_layers(&self) -> usize { self.layers.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_contains(&self) -> u64 { self.total_contains }
    pub fn total_hits(&self) -> u64 { self.total_hits }
    pub fn total_misses(&self) -> u64 { self.total_misses }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chain() { let bc = BloomChain::new(&[(1024, 3), (2048, 5)]); assert_eq!(bc.num_layers(), 2); }

    #[test]
    fn insert_contains() {
        let mut bc = BloomChain::new(&[(512, 3)]);
        bc.insert(42);
        assert!(bc.contains(42));
        assert!(!bc.contains(99));
    }

    #[test]
    fn tiered_layers() {
        let mut bc = BloomChain::new(&[(256, 2), (512, 4)]);
        bc.insert(1); bc.insert(2); bc.insert(3);
        assert_eq!(bc.layer_count(0).unwrap(), 3);
        assert_eq!(bc.layer_count(1).unwrap(), 3);
    }

    #[test]
    fn layer_not_found() {
        let bc = BloomChain::new(&[(256, 2)]);
        assert!(bc.layer_count(5).is_err());
    }

    #[test]
    fn many_items() {
        let mut bc = BloomChain::new(&[(4096, 4), (8192, 6)]);
        for i in 0..500u64 { bc.insert(i); }
        let mut tp = 0; let mut fp = 0;
        for i in 0..500u64 { if bc.contains(i) { tp += 1; } }
        for i in 500..1000u64 { if bc.contains(i) { fp += 1; } }
        assert_eq!(tp, 500);
        assert!(fp < 50, "too many false positives: {fp}");
    }

    #[test]
    fn saturation() {
        let mut bc = BloomChain::new(&[(128, 3)]);
        for i in 0..50u64 { bc.insert(i); }
        let sat = bc.layer_saturation(0).unwrap();
        assert!(sat > 0.0 && sat <= 1.0);
    }

    #[test]
    fn stats() {
        let mut bc = BloomChain::new(&[(256, 3)]);
        bc.insert(1); bc.contains(1); bc.contains(2);
        assert_eq!(bc.total_inserts(), 1);
        assert_eq!(bc.total_contains(), 2);
    }

    #[test]
    fn error_display() { assert!(BcError::LayerNotFound { layer: 5 }.to_string().contains("5")); }
}
