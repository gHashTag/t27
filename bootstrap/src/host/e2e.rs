use super::engine::InferenceReport;
use super::perf::{EngineConfig, PerformanceEstimate};
use super::weights::{WeightConfig, WeightPattern};

#[derive(Debug, Clone)]
pub struct E2eResult {
    pub layers: u32,
    pub neurons: u32,
    pub chunks: u32,
    pub pattern: String,
    pub weight_words: usize,
    pub inference: InferenceReport,
    pub estimate: PerformanceEstimate,
    pub weight_gen_ok: bool,
}

pub struct E2eConfig {
    pub num_layers: u32,
    pub neurons: u32,
    pub chunks: u32,
    pub threshold: u32,
    pub weight_addr: u64,
    pub pattern: WeightPattern,
    pub max_rounds: u32,
}

impl E2eConfig {
    pub fn perf_config(&self) -> Option<EngineConfig> {
        EngineConfig::new(self.num_layers, self.neurons, self.chunks)
    }
}

pub fn run_e2e(config: &E2eConfig) -> anyhow::Result<E2eResult> {
    let weight_config = WeightConfig {
        neurons: config.neurons,
        chunks: config.chunks,
        pattern: config.pattern,
    };
    let weight_words = super::weights::generate_weights(&weight_config);
    let weight_gen_ok = !weight_words.is_empty();

    let mut engine = super::engine::InferenceEngine::new(
        super::driver::BitnetDriver::new(super::mmio::MockMmio::with_csrs_zeroed()),
    );
    engine
        .configure(config.num_layers, config.neurons, config.chunks, config.threshold, config.weight_addr)
        .map_err(|e| anyhow::anyhow!("configure: {:?}", e))?;
    let inference = engine
        .run(config.max_rounds)
        .map_err(|e| anyhow::anyhow!("inference: {:?}", e))?;

    let perf_config = config.perf_config()
        .ok_or_else(|| anyhow::anyhow!("invalid perf config"))?;
    let estimate = perf_config.estimate();

    Ok(E2eResult {
        layers: config.num_layers,
        neurons: config.neurons,
        chunks: config.chunks,
        pattern: super::weights::pattern_name(config.pattern).to_string(),
        weight_words: weight_words.len(),
        inference,
        estimate,
        weight_gen_ok,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct E2eJson {
    pub ok: bool,
    pub layers: u32,
    pub neurons: u32,
    pub chunks: u32,
    pub pattern: String,
    pub weight_words: usize,
    pub layers_completed: u32,
    pub total_writes: usize,
    pub total_reads: usize,
    pub estimated_cycles: u32,
    pub estimated_dma_beats: u32,
    pub bram_pct: f64,
    pub weight_gen_ok: bool,
}

impl E2eJson {
    pub fn from_result(r: &E2eResult) -> Self {
        Self {
            ok: r.inference.error_layer.is_none(),
            layers: r.layers,
            neurons: r.neurons,
            chunks: r.chunks,
            pattern: r.pattern.clone(),
            weight_words: r.weight_words,
            layers_completed: r.inference.layers_completed,
            total_writes: r.inference.total_writes,
            total_reads: r.inference.total_reads,
            estimated_cycles: r.estimate.total_inference_cycles,
            estimated_dma_beats: r.estimate.total_dma_beats,
            bram_pct: r.estimate.bram_utilization_pct,
            weight_gen_ok: r.weight_gen_ok,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> E2eConfig {
        E2eConfig {
            num_layers: 2,
            neurons: 16,
            chunks: 4,
            threshold: 1,
            weight_addr: 0,
            pattern: WeightPattern::Alternating,
            max_rounds: 4,
        }
    }

    #[test]
    fn e2e_default_succeeds() {
        let result = run_e2e(&default_config()).unwrap();
        assert!(result.weight_gen_ok);
    }

    #[test]
    fn e2e_all_layers_complete() {
        let result = run_e2e(&default_config()).unwrap();
        assert_eq!(result.inference.layers_completed, result.layers);
    }

    #[test]
    fn e2e_no_error_layer() {
        let result = run_e2e(&default_config()).unwrap();
        assert!(result.inference.error_layer.is_none());
    }

    #[test]
    fn e2e_weight_words_match_config() {
        let result = run_e2e(&default_config()).unwrap();
        assert_eq!(result.weight_words, 16 * 4);
    }

    #[test]
    fn e2e_has_mmio_transactions() {
        let result = run_e2e(&default_config()).unwrap();
        assert!(result.inference.total_writes > 0);
        assert!(result.inference.total_reads > 0);
    }

    #[test]
    fn e2e_estimate_has_cycles() {
        let result = run_e2e(&default_config()).unwrap();
        assert!(result.estimate.total_inference_cycles > 0);
    }

    #[test]
    fn e2e_single_layer() {
        let mut cfg = default_config();
        cfg.num_layers = 1;
        let result = run_e2e(&cfg).unwrap();
        assert_eq!(result.inference.layers_completed, 1);
    }

    #[test]
    fn e2e_five_layers() {
        let mut cfg = default_config();
        cfg.num_layers = 5;
        let result = run_e2e(&cfg).unwrap();
        assert_eq!(result.inference.layers_completed, 5);
    }

    #[test]
    fn e2e_all_n_pattern() {
        let mut cfg = default_config();
        cfg.pattern = WeightPattern::AllN;
        let result = run_e2e(&cfg).unwrap();
        assert!(result.weight_gen_ok);
    }

    #[test]
    fn e2e_seeded_random_pattern() {
        let mut cfg = default_config();
        cfg.pattern = WeightPattern::SeededRandom(123);
        let result = run_e2e(&cfg).unwrap();
        assert!(result.weight_gen_ok);
    }

    #[test]
    fn e2e_json_serializable() {
        let result = run_e2e(&default_config()).unwrap();
        let json = E2eJson::from_result(&result);
        let s = serde_json::to_string(&json).unwrap();
        assert!(s.contains("\"ok\":true"));
    }

    #[test]
    fn e2e_json_has_all_fields() {
        let result = run_e2e(&default_config()).unwrap();
        let json = E2eJson::from_result(&result);
        let v: serde_json::Value = serde_json::to_string(&json).unwrap().parse().unwrap();
        assert!(v["layers"].is_number());
        assert!(v["weight_words"].is_number());
        assert!(v["estimated_cycles"].is_number());
        assert!(v["bram_pct"].is_number());
    }

    #[test]
    fn e2e_writes_increase_with_layers() {
        let mut cfg1 = default_config();
        cfg1.num_layers = 1;
        let r1 = run_e2e(&cfg1).unwrap();

        let mut cfg2 = default_config();
        cfg2.num_layers = 4;
        let r2 = run_e2e(&cfg2).unwrap();

        assert!(r2.inference.total_writes > r1.inference.total_writes);
    }
}
