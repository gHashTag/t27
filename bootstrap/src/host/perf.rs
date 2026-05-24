pub const TRITS_PER_WORD: u32 = 27;
pub const BITS_PER_TRIT: u32 = 2;
pub const DATA_WIDTH: u32 = TRITS_PER_WORD * BITS_PER_TRIT;
pub const BRAM_DEPTH: u32 = 4096;
pub const DDR_BEAT_BITS: u32 = 64;
#[allow(dead_code)]
pub const DDR_BEAT_BYTES: u32 = DDR_BEAT_BITS / 8;
pub const WORDS_PER_DDR_BEAT: u32 = DDR_BEAT_BITS / DATA_WIDTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineConfig {
    pub num_layers: u32,
    pub neurons: u32,
    pub chunks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerEstimate {
    pub layer_index: u32,
    pub weight_words: u32,
    pub weight_bytes: u32,
    pub dma_prefetch_beats: u32,
    pub compute_cycles: u32,
    pub dma_drain_beats: u32,
    pub total_cycles: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceEstimate {
    pub config: EngineConfig,
    pub total_weight_words: u32,
    pub total_weight_bytes: u32,
    pub bram_utilization_pct: f64,
    pub total_dma_beats: u32,
    pub total_inference_cycles: u32,
    pub layers: Vec<LayerEstimate>,
}

impl EngineConfig {
    pub fn new(num_layers: u32, neurons: u32, chunks: u32) -> Option<Self> {
        if num_layers == 0 || neurons == 0 || chunks == 0 {
            return None;
        }
        Some(Self {
            num_layers,
            neurons,
            chunks,
        })
    }

    pub fn weight_words_per_layer(&self) -> u32 {
        self.neurons * self.chunks
    }

    pub fn total_weight_words(&self) -> u32 {
        self.weight_words_per_layer() * self.num_layers
    }

    pub fn weight_bytes_per_layer(&self) -> u32 {
        self.weight_words_per_layer() * (DATA_WIDTH / 8)
    }

    pub fn total_weight_bytes(&self) -> u32 {
        self.total_weight_words() * (DATA_WIDTH / 8)
    }

    pub fn bram_utilization_pct(&self) -> f64 {
        let words = self.weight_words_per_layer() as f64;
        (words / BRAM_DEPTH as f64) * 100.0
    }

    pub fn dma_beats_per_layer(&self) -> u32 {
        let words = self.weight_words_per_layer();
        (words + WORDS_PER_DDR_BEAT - 1) / WORDS_PER_DDR_BEAT
    }

    pub fn compute_cycles_per_layer(&self) -> u32 {
        self.neurons * self.chunks
    }

    pub fn cycles_per_layer(&self) -> u32 {
        let dma_beats = self.dma_beats_per_layer();
        let compute = self.compute_cycles_per_layer();
        dma_beats + compute + dma_beats
    }

    pub fn total_inference_cycles(&self) -> u32 {
        self.cycles_per_layer() * self.num_layers
    }

    pub fn throughput_inf_per_sec(&self, clock_mhz: f64) -> f64 {
        if clock_mhz <= 0.0 {
            return 0.0;
        }
        let cycles_per_sec = clock_mhz * 1e6;
        let cycles_per_inf = self.total_inference_cycles() as f64;
        if cycles_per_inf == 0.0 {
            return 0.0;
        }
        cycles_per_sec / cycles_per_inf
    }

    pub fn estimate(&self) -> PerformanceEstimate {
        let layers: Vec<LayerEstimate> = (0..self.num_layers)
            .map(|i| {
                let weight_words = self.weight_words_per_layer();
                let dma_beats = self.dma_beats_per_layer();
                let compute = self.compute_cycles_per_layer();
                LayerEstimate {
                    layer_index: i,
                    weight_words,
                    weight_bytes: self.weight_bytes_per_layer(),
                    dma_prefetch_beats: dma_beats,
                    compute_cycles: compute,
                    dma_drain_beats: dma_beats,
                    total_cycles: dma_beats + compute + dma_beats,
                }
            })
            .collect();

        PerformanceEstimate {
            config: *self,
            total_weight_words: self.total_weight_words(),
            total_weight_bytes: self.total_weight_bytes(),
            bram_utilization_pct: self.bram_utilization_pct(),
            total_dma_beats: layers.iter().map(|l| l.dma_prefetch_beats + l.dma_drain_beats).sum(),
            total_inference_cycles: self.total_inference_cycles(),
            layers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> EngineConfig {
        EngineConfig::new(2, 16, 4).unwrap()
    }

    #[test]
    fn rejects_zero_layers() {
        assert!(EngineConfig::new(0, 1, 1).is_none());
    }

    #[test]
    fn rejects_zero_neurons() {
        assert!(EngineConfig::new(1, 0, 1).is_none());
    }

    #[test]
    fn rejects_zero_chunks() {
        assert!(EngineConfig::new(1, 1, 0).is_none());
    }

    #[test]
    fn weight_words_per_layer() {
        assert_eq!(cfg().weight_words_per_layer(), 64);
    }

    #[test]
    fn total_weight_words() {
        assert_eq!(cfg().total_weight_words(), 128);
    }

    #[test]
    fn weight_bytes_per_layer() {
        assert_eq!(cfg().weight_bytes_per_layer(), 64 * (DATA_WIDTH / 8));
    }

    #[test]
    fn bram_utilization_under_100() {
        assert!(cfg().bram_utilization_pct() < 100.0);
    }

    #[test]
    fn bram_utilization_exact() {
        let c = cfg();
        let expected = (c.weight_words_per_layer() as f64 / BRAM_DEPTH as f64) * 100.0;
        assert!((c.bram_utilization_pct() - expected).abs() < 0.001);
    }

    #[test]
    fn compute_cycles_per_layer() {
        assert_eq!(cfg().compute_cycles_per_layer(), 64);
    }

    #[test]
    fn cycles_per_layer_is_three_stages() {
        let c = cfg();
        let dma = c.dma_beats_per_layer();
        let compute = c.compute_cycles_per_layer();
        assert_eq!(c.cycles_per_layer(), dma + compute + dma);
    }

    #[test]
    fn total_inference_cycles_scales_linearly() {
        let c = cfg();
        assert_eq!(
            c.total_inference_cycles(),
            c.cycles_per_layer() * c.num_layers
        );
    }

    #[test]
    fn throughput_at_66_mhz() {
        let t = cfg().throughput_inf_per_sec(66.0);
        assert!(t > 0.0, "throughput should be positive: {t}");
    }

    #[test]
    fn throughput_zero_clock_is_zero() {
        assert_eq!(cfg().throughput_inf_per_sec(0.0), 0.0);
    }

    #[test]
    fn estimate_has_correct_layer_count() {
        let e = cfg().estimate();
        assert_eq!(e.layers.len(), 2);
    }

    #[test]
    fn estimate_layer_indices_sequential() {
        let e = cfg().estimate();
        for (i, l) in e.layers.iter().enumerate() {
            assert_eq!(l.layer_index, i as u32);
        }
    }

    #[test]
    fn estimate_total_weight_words_matches() {
        let e = cfg().estimate();
        assert_eq!(e.total_weight_words, cfg().total_weight_words());
    }

    #[test]
    fn large_config_does_not_overflow() {
        let c = EngineConfig::new(100, 4096, 64).unwrap();
        let e = c.estimate();
        assert!(e.total_weight_words > 0);
        assert!(e.total_inference_cycles > 0);
    }

    #[test]
    fn data_width_is_54() {
        assert_eq!(DATA_WIDTH, 54);
    }

    #[test]
    fn bram_depth_is_4096() {
        assert_eq!(BRAM_DEPTH, 4096);
    }
}
