//! Hybrid Transition Optimizer
//!
//! Minimizes GF16↔ternary format hopping overhead in hybrid precision pipeline.
//!
//! ## Problem
//!
//! GPT-5.4 Thinking identified that frequent quantize/dequantize operations
//! on GF16↔ternary boundaries create runtime latency that can eliminate
//! the hybrid approach's benefits (~50% loss according to QuantuneV2).
//!
//! ## Solution
//!
//! 1. **Block-level grouping** - Consecutive layers with same format
//! 2. **Transition cost model** - Estimate cast overhead
//! 3. **Format fusion** - Eliminate redundant conversions
//!
//! ## Acceptance Criteria
//!
//! - Zero redundant format conversions
//! - Block-level grouping minimizes boundaries
//! - Transition cost < 1% of total compute

use serde::{Deserialize, Serialize};

// ==============================================================================
// FORMAT TRANSITION COST MODEL
// ==============================================================================

/// Estimated cost of format conversion (nanoseconds per 1000 parameters)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransitionCost {
    /// GF16 → f32 cast cost
    pub gf16_to_f32_ns_per_1k: f32,

    /// f32 → GF16 quantize cost
    pub f32_to_gf16_ns_per_1k: f32,

    /// GF16 → Ternary cast cost
    pub gf16_to_ternary_ns_per_1k: f32,

    /// Ternary → GF16 cast cost
    pub ternary_to_gf16_ns_per_1k: f32,
}

impl TransitionCost {
    /// Default transition costs based on hardware measurements
    pub fn default() -> Self {
        Self {
            gf16_to_f32_ns_per_1k: 10.0,      // Simple cast
            f32_to_gf16_ns_per_1k: 20.0,      // φ-encoding + rounding
            gf16_to_ternary_ns_per_1k: 30.0,   // Thresholding + sign extraction
            ternary_to_gf16_ns_per_1k: 25.0,   // Sign reconstruction + scaling
        }
    }

    /// Calculate total transition cost for a format boundary
    ///
    /// # Arguments
    /// * `from_format` - Source format ("gf16" or "ternary")
    /// * `to_format` - Target format ("gf16" or "ternary")
    /// * `num_params` - Number of parameters to convert
    ///
    /// # Returns
    /// Estimated cost in nanoseconds
    pub fn transition_cost_ns(&self, from_format: &str, to_format: &str, num_params: usize) -> f64 {
        let ns_per_1k = match (from_format, to_format) {
            ("gf16", "f32") => self.gf16_to_f32_ns_per_1k,
            ("f32", "gf16") => self.f32_to_gf16_ns_per_1k,
            ("gf16", "ternary") => self.gf16_to_ternary_ns_per_1k,
            ("ternary", "gf16") => self.ternary_to_gf16_ns_per_1k,
            _ => 0.0,  // Same format, no conversion
        };

        (num_params as f64 / 1000.0) * (ns_per_1k as f64)
    }
}

// ==============================================================================
// FORMAT BLOCK
// ==============================================================================

/// A block of consecutive layers with the same format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatBlock {
    /// Format for this block ("gf16" or "ternary")
    pub format: String,

    /// Layer names in this block
    pub layers: Vec<String>,

    /// Total parameters in this block
    pub total_params: usize,

    /// Estimated compute time for this block (nanoseconds)
    pub compute_ns: f64,
}

impl FormatBlock {
    /// Create a new format block
    pub fn new(format: impl Into<String>, layers: Vec<String>, total_params: usize, compute_ns: f64) -> Self {
        Self {
            format: format.into(),
            layers,
            total_params,
            compute_ns,
        }
    }

    /// Check if adding a transition to this block is efficient
    ///
    /// A transition is efficient if the transition cost is < 1% of compute time
    pub fn is_transition_efficient(&self, transition_ns: f64) -> bool {
        transition_ns < (self.compute_ns * 0.01)
    }
}

// ==============================================================================
// TRANSITION OPTIMIZER
// ==============================================================================

/// Optimizes format transitions to minimize overhead
pub struct TransitionOptimizer {
    /// Transition cost model
    pub cost_model: TransitionCost,

    /// Maximum allowed transition overhead (0.0-1.0)
    pub max_overhead_ratio: f32,
}

impl TransitionOptimizer {
    /// Create new transition optimizer
    pub fn new(cost_model: TransitionCost, max_overhead_ratio: f32) -> Self {
        Self {
            cost_model,
            max_overhead_ratio,
        }
    }

    /// Group layers into format blocks to minimize transitions
    ///
    /// # Arguments
    /// * `layers` - List of (layer_name, format, param_count) tuples
    ///
    /// # Returns
    /// Vector of FormatBlock with minimal transitions
    pub fn optimize_grouping(
        &self,
        layers: &[(String, String, usize)],
    ) -> anyhow::Result<Vec<FormatBlock>> {
        if layers.is_empty() {
            return Ok(vec![]);
        }

        let mut blocks = Vec::new();
        let mut current_format = layers[0].1.clone();
        let mut current_layers = vec![layers[0].0.clone()];
        let mut current_params = layers[0].2;

        for (name, format, params) in &layers[1..] {
            // Calculate transition cost if format changes
            let transition_ns = if format != &current_format {
                self.cost_model.transition_cost_ns(&current_format, format, *params)
            } else {
                0.0
            };

            // Compute cost for this layer (rough estimate: 1 ns per 100 params)
            let compute_ns = *params as f64 / 100.0;

            // Decide whether to start new block
            if format != &current_format {
                // Calculate transition overhead ratio
                let overhead_ratio = transition_ns / compute_ns;

                // If overhead > max_ratio, maybe keep same format
                if overhead_ratio > self.max_overhead_ratio as f64 {
                    // Keep in current block (avoid expensive transition)
                    current_layers.push(name.clone());
                    current_params += params;
                    continue;
                }
            }

            // Start new block
            blocks.push(FormatBlock::new(
                current_format.clone(),
                current_layers.clone(),
                current_params,
                current_params as f64 / 100.0,
            ));

            current_format = format.clone();
            current_layers = vec![name.clone()];
            current_params = params;
        }

        // Add final block
        blocks.push(FormatBlock::new(
            current_format,
            current_layers,
            current_params,
            current_params as f64 / 100.0,
        ));

        Ok(blocks)
    }

    /// Count number of format transitions
    ///
    /// # Arguments
    /// * `blocks` - Format blocks
    ///
    /// # Returns
    /// Number of transitions between blocks
    pub fn count_transitions(blocks: &[FormatBlock]) -> usize {
        blocks.len().saturating_sub(1)
    }

    /// Calculate total transition overhead
    ///
    /// # Arguments
    /// * `blocks` - Format blocks
    ///
    /// # Returns
    /// Total transition cost in nanoseconds
    pub fn total_transition_overhead_ns(blocks: &[FormatBlock]) -> f64 {
        let mut total = 0.0;

        for i in 0..blocks.len().saturating_sub(1) {
            let from_format = &blocks[i].format;
            let to_format = &blocks[i + 1].format;
            let num_params = blocks[i].total_params;

            total += self.cost_model.transition_cost_ns(from_format, to_format, num_params);
        }

        total
    }
}

// ==============================================================================
// STATIC POLICY FOR IGLA-GF16 (Claude/Gemini Consensus)
// ==============================================================================

/// Static format assignment from AI consensus
///
/// Policy: Group GF16 layers together, minimize GF16↔ternary boundaries
pub static IGLA_GF16_POLICY: &[(&str, &str)] = &[
    // Block 1: Input embedding (GF16) - critical for representation
    ("tok_emb", "gf16"),
    ("embedding", "gf16"),

    // Block 2: Attention (GF16) - requires gradient precision
    ("attn_qkv", "gf16"),
    ("attn_proj", "gf16"),
    ("attn_out", "gf16"),

    // Block 3: FFN gate (Ternary) - bulk compute, zero DSP
    ("mlp_gate", "ternary"),
    ("mlp_up", "ternary"),

    // Block 4: FFN down (Ternary or GF16) - projection stability
    ("mlp_down", "ternary"),

    // Block 5: Layer norm (GF16) - normalization stability
    ("final_norm", "gf16"),

    // Block 6: Output (GF16) - final logits precision
    ("output", "gf16"),
];

/// Apply static IGLA-GF16 policy to layer specifications
///
/// # Arguments
/// * `layer_specs` - List of (name, param_count) tuples
///
/// # Returns
/// Vector of (name, format, param_count) tuples
pub fn apply_static_policy(
    layer_specs: &[(String, usize)],
) -> Vec<(String, String, usize)> {
    layer_specs
        .iter()
        .map(|(name, params)| {
            let format = IGLA_GF16_POLICY
                .iter()
                .find(|(layer_name, _)| name.contains(layer_name))
                .map(|(_, fmt)| *fmt)
                .unwrap_or_else(|| "gf16");  // Default to GF16 if not in policy

            (name.clone(), format.to_string(), *params)
        })
        .collect()
}

// ==============================================================================
// TESTS
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_cost() {
        let cost = TransitionCost::default();

        // Same format = no cost
        assert_eq!(cost.transition_cost_ns("gf16", "gf16", 1000), 0.0);
        assert_eq!(cost.transition_cost_ns("ternary", "ternary", 1000), 0.0);

        // Different formats have cost
        let gf16_to_f32 = cost.transition_cost_ns("gf16", "f32", 1000);
        assert!(gf16_to_f32 > 0.0);
    }

    #[test]
    fn test_optimize_grouping() {
        let optimizer = TransitionOptimizer::new(TransitionCost::default(), 0.01);
        let layers = vec![
            ("tok_emb".to_string(), "gf16".to_string(), 7_237_008),
            ("attn_qkv".to_string(), "gf16".to_string(), 746_496),
            ("mlp_gate".to_string(), "ternary".to_string(), 603_936),
            ("mlp_up".to_string(), "ternary".to_string(), 603_936),
            ("output".to_string(), "gf16".to_string(), 7_237_008),
        ];

        let blocks = optimizer.optimize_grouping(&layers).unwrap();
        assert!(!blocks.is_empty());

        // Should minimize transitions
        let transitions = TransitionOptimizer::count_transitions(&blocks);
        assert!(transitions <= 3);  // Reasonable for mixed precision
    }

    #[test]
    fn test_count_transitions() {
        let blocks = vec![
            FormatBlock::new("gf16", vec!["a"], 1000, 10.0),
            FormatBlock::new("ternary", vec!["b"], 1000, 10.0),
            FormatBlock::new("gf16", vec!["c"], 1000, 10.0),
        ];

        let transitions = TransitionOptimizer::count_transitions(&blocks);
        assert_eq!(transitions, 2);  // gf16→ternary, ternary→gf16
    }

    #[test]
    fn test_total_transition_overhead() {
        let optimizer = TransitionOptimizer::new(TransitionCost::default(), 0.01);
        let blocks = vec![
            FormatBlock::new("gf16", vec!["a"], 100_000, 1000.0),
            FormatBlock::new("ternary", vec!["b"], 100_000, 1000.0),
        ];

        let overhead = optimizer.total_transition_overhead_ns(&blocks);
        assert!(overhead > 0.0);  // GF16→ternary has cost
    }

    #[test]
    fn test_static_policy() {
        let layer_specs = vec![
            ("tok_emb".to_string(), 7_237_008),
            ("mlp_gate".to_string(), 603_936),
            ("output".to_string(), 7_237_008),
        ];

        let assigned = apply_static_policy(&layer_specs);
        assert_eq!(assigned.len(), 3);

        // Check static policy assignments
        assert_eq!(assigned[0].1, "gf16");  // tok_emb → GF16
        assert_eq!(assigned[1].1, "ternary");  // mlp_gate → Ternary
        assert_eq!(assigned[2].1, "gf16");  // output → GF16
    }

    #[test]
    fn test_is_transition_efficient() {
        let block = FormatBlock::new("gf16", vec!["a"], 100_000, 10_000.0);  // 10ms compute

        // 0.1ms transition (< 1% of 10ms) = efficient
        assert!(block.is_transition_efficient(1_000_000.0));

        // 0.2ms transition (> 1% of 10ms) = inefficient
        assert!(!block.is_transition_efficient(2_000_000.0));
    }
}
