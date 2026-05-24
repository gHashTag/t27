const RING_SIZE: usize = 256;
const ROUNDS: usize = 4;

fn rotl(v: u64, n: u32) -> u64 { v.rotate_left(n) }

fn fmix64(k: u64) -> u64 {
    let mut h = k;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

pub struct RingMixer {
    ring: [u64; RING_SIZE],
    state: u64,
    pos: usize,
    mix_count: u64,
    byte_count: u64,
}

impl RingMixer {
    pub fn new(seed: u64) -> Self {
        let mut ring = [0u64; RING_SIZE];
        let mut s = seed;
        for i in 0..RING_SIZE {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ring[i] = fmix64(s.wrapping_add(i as u64));
        }
        Self { ring, state: fmix64(seed), pos: 0, mix_count: 0, byte_count: 0 }
    }

    pub fn mix(&mut self, data: &[u8]) -> u64 {
        self.byte_count += data.len() as u64;
        self.mix_count += 1;
        for (i, &byte) in data.iter().enumerate() {
            let idx = self.pos;
            let input = (byte as u64)
                .wrapping_mul(0x9e3779b97f4a7c15)
                .wrapping_add(i as u64)
                .wrapping_add(self.state);
            self.ring[idx] = self.ring[idx]
                .wrapping_add(input)
                .wrapping_mul(0x517cc1b727220a95);
            self.state = rotl(self.state ^ self.ring[idx], 7);
            self.pos = (self.pos + 1) % RING_SIZE;
        }
        for _ in 0..ROUNDS {
            for i in 0..RING_SIZE {
                let next = (i + 1) % RING_SIZE;
                self.ring[i] = rotl(self.ring[i], 17)
                    .wrapping_add(self.ring[next])
                    .wrapping_mul(0x6c62272e07bb0142);
            }
        }
        self.finalize()
    }

    fn finalize(&self) -> u64 {
        let mut h = self.state;
        for (i, &v) in self.ring.iter().enumerate() {
            h = h.wrapping_add(rotl(v, (i as u32) % 64));
        }
        fmix64(h)
    }

    pub fn avalanche_score(&self) -> f64 {
        let v = self.finalize();
        let bits = v.count_ones();
        1.0 - ((bits as f64 - 32.0).abs() / 32.0)
    }

    pub fn mix_count(&self) -> u64 { self.mix_count }
    pub fn byte_count(&self) -> u64 { self.byte_count }
    pub fn ring_pos(&self) -> usize { self.pos }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mixer() {
        let rm = RingMixer::new(42);
        assert_eq!(rm.mix_count(), 0);
        assert_eq!(rm.byte_count(), 0);
    }

    #[test]
    fn mix_deterministic() {
        let mut rm1 = RingMixer::new(123);
        let mut rm2 = RingMixer::new(123);
        let h1 = rm1.mix(b"hello");
        let h2 = rm2.mix(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn mix_different_input() {
        let mut rm1 = RingMixer::new(123);
        let mut rm2 = RingMixer::new(123);
        let h1 = rm1.mix(b"hello");
        let h2 = rm2.mix(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn mix_different_seed() {
        let mut rm1 = RingMixer::new(1);
        let mut rm2 = RingMixer::new(2);
        let h1 = rm1.mix(b"test");
        let h2 = rm2.mix(b"test");
        assert_ne!(h1, h2);
    }

    #[test]
    fn incremental_mix() {
        let mut rm1 = RingMixer::new(99);
        let h1 = rm1.mix(b"hello");
        let h2 = rm1.mix(b"world");
        assert_ne!(h1, h2);
        assert_eq!(rm1.mix_count(), 2);
    }

    #[test]
    fn empty_input() {
        let mut rm = RingMixer::new(42);
        let h = rm.mix(b"");
        assert_ne!(h, 0);
    }

    #[test]
    fn large_input() {
        let mut rm = RingMixer::new(42);
        let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let h = rm.mix(&data);
        assert_ne!(h, 0);
        assert_eq!(rm.byte_count(), 10000);
    }

    #[test]
    fn avalanche_quality() {
        let mut rm = RingMixer::new(42);
        rm.mix(b"avalanche test data for quality check");
        let score = rm.avalanche_score();
        assert!(score > 0.5, "avalanche score too low: {score}");
    }

    #[test]
    fn ring_position_advances() {
        let mut rm = RingMixer::new(42);
        let p0 = rm.ring_pos();
        rm.mix(b"abc");
        assert_ne!(rm.ring_pos(), p0);
    }

    #[test]
    fn multiple_mixes_accumulate() {
        let mut rm = RingMixer::new(1);
        let mut hashes = Vec::new();
        for i in 0u8..10 {
            hashes.push(rm.mix(&[i]));
        }
        assert_eq!(hashes.len(), 10);
        assert_eq!(rm.mix_count(), 10);
    }

    #[test]
    fn single_byte() {
        let mut rm = RingMixer::new(42);
        let h = rm.mix(&[0xFF]);
        assert_ne!(h, 0);
    }

    #[test]
    fn null_bytes() {
        let mut rm = RingMixer::new(42);
        let h = rm.mix(&[0u8; 64]);
        assert_ne!(h, 0);
    }
}
