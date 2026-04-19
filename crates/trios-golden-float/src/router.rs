//! # Static Precision Router for IGLA-GF16 Hybrid Precision Pipeline
//!
//! Implements the consensus policy from 3-model synthesis (GPT-5.4, Claude Opus 4.7, Gemini 3.1):
//! - **GF16** for critical layers (Embedding, Attention, Output)
//! - **Ternary** for bulk quantized layers (FFN, Conv2D)
//!
//! ## Policy Table
//!
//! | Layer Type | Precision | Reason |
//! |------------|-----------|--------|
//! | Embedding | GF16 | Similarity metrics require full floating-point precision |
//! | Attention (QKV) | GF16 | QKV projection requires gradient precision |
//! | Attention Output | GF16 | Context accumulation needs stable scaling |
//! | FFN Gate/Up | Ternary | Mass quantized, can use ternary with QAT+STE |
//! | FFN Down | GF16 | Projection to residual requires precision |
//! | Conv2D (1-3) | Ternary | Early layers highly quantizable |
//! | Conv2D (4+) | GF16 | Deeper layers need gradient flow |
//! | Output Norm/Act | GF16 | Final layer requires stable scaling |
//!
//! ## Usage
//!
//! ```ignore
//! use trios_golden_float::router::{LayerType, PrecisionRouter};
//!
//! let router = PrecisionRouter::new();
//! assert_eq!(router.get_precision("embedding"), Precision::GF16);
//! assert_eq!(router.get_precision("ffn_gate"), Precision::Ternary);
//! ```

use serde::{Deserialize, Serialize};

/// Layer type identifier for precision routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerType {
    /// Input embedding layer
    Embedding,

    /// Multi-head attention QKV projection
    AttentionQKV,

    /// Multi-head attention output projection
    AttentionOutput,

    /// Feed-forward gate projection (first FFN linear)
    FFNGate,

    /// Feed-forward up projection (second FFN linear)
    FFNUp,

    /// Feed-forward down projection (FFN output)
    FFNDown,

    /// 2D convolution (early layers 1-3)
    Conv2DEarly,

    /// 2D convolution (deep layers 4+)
    Conv2DDeep,

    /// Layer normalization
    LayerNorm,

    /// Output classification head
    OutputHead,

    /// Generic dense layer (defaults to GF16 for safety)
    Dense,
}

impl LayerType {
    /// Parse layer type from string name.
    ///
    /// Supports naming conventions: "embedding", "attn_qkv", "ffn_gate", etc.
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            n if n.contains("embed") => Some(Self::Embedding),
            n if n.contains("attn") && n.contains("qkv") => Some(Self::AttentionQKV),
            n if n.contains("attn") && n.contains("out") => Some(Self::AttentionOutput),
            n if n.contains("ffn") && n.contains("gate") => Some(Self::FFNGate),
            n if n.contains("ffn") && n.contains("up") => Some(Self::FFNUp),
            n if n.contains("ffn") && n.contains("down") => Some(Self::FFNDown),
            n if n.contains("conv") || n.contains("2d") => Some(Self::Conv2DEarly),
            n if n.contains("norm") => Some(Self::LayerNorm),
            n if n.contains("output") || n.contains("head") => Some(Self::OutputHead),
            n if n.contains("dense") || n.contains("linear") => Some(Self::Dense),
            _ => None,
        }
    }
}

/// Precision format for layer computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Precision {
    /// Golden Float 16-bit (GF16) — φ-optimized floating-point
    GF16,

    /// Ternary {-1, 0, +1} — 3-state quantized
    Ternary,

    /// IEEE 754 FP32 (baseline for accuracy reference)
    FP32,
}

impl Precision {
    /// Returns the bit-width of this precision.
    pub const fn bit_width(&self) -> u32 {
        match self {
            Self::GF16 => 16,
            Self::Ternary => 2,  // 2 bits sufficient for {-1, 0, +1}
            Self::FP32 => 32,
        }
    }

    /// Returns whether this precision uses floating-point representation.
    pub const fn is_floating_point(&self) -> bool {
        matches!(self, Self::GF16 | Self::FP32)
    }
}

/// Static precision router based on layer type.
///
/// Implements the hybrid precision policy consensus from 3-model analysis.
#[derive(Debug, Clone)]
pub struct PrecisionRouter;

impl PrecisionRouter {
    /// Create a new precision router.
    pub const fn new() -> Self {
        Self
    }

    /// Get the recommended precision for a given layer type.
    pub const fn get_precision(&self, layer: LayerType) -> Precision {
        match layer {
            // === GF16 Critical Layers ===
            LayerType::Embedding => Precision::GF16,
            LayerType::AttentionQKV => Precision::GF16,
            LayerType::AttentionOutput => Precision::GF16,
            LayerType::FFNDown => Precision::GF16,
            LayerType::Conv2DDeep => Precision::GF16,
            LayerType::LayerNorm => Precision::GF16,
            LayerType::OutputHead => Precision::GF16,
            LayerType::Dense => Precision::GF16,

            // === Ternary Bulk Layers ===
            LayerType::FFNGate => Precision::Ternary,
            LayerType::FFNUp => Precision::Ternary,
            LayerType::Conv2DEarly => Precision::Ternary,
        }
    }

    /// Get precision by layer name string.
    pub fn get_precision_by_name(&self, name: &str) -> Precision {
        LayerType::from_name(name)
            .map(|layer| self.get_precision(layer))
            .unwrap_or(Precision::GF16) // Default to GF16 for unknown layers
    }

    /// Check if a transition between layer precisions requires conversion.
    pub fn needs_conversion(from: Precision, to: Precision) -> bool {
        from != to
    }

    /// Get the hardware cost (LUT) for a single MAC operation in this precision.
    pub const fn mac_lut_cost(precision: Precision) -> u32 {
        match precision {
            Precision::GF16 => 71,    // From BENCH-006
            Precision::Ternary => 52, // From BENCH-006
            Precision::FP32 => 94,    // GF16 mul cost (estimate)
        }
    }

    /// Get the hardware cost (DSP) for a single MAC operation in this precision.
    pub const fn mac_dsp_cost(precision: Precision) -> u32 {
        match precision {
            Precision::GF16 => 16,    // From BENCH-006
            Precision::Ternary => 0,  // No DSP needed
            Precision::FP32 => 1,     // Minimal DSP usage
        }
    }
}

impl Default for PrecisionRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_is_gf16() {
        let router = PrecisionRouter::new();
        assert_eq!(router.get_precision(LayerType::Embedding), Precision::GF16);
    }

    #[test]
    fn test_attention_is_gf16() {
        let router = PrecisionRouter::new();
        assert_eq!(router.get_precision(LayerType::AttentionQKV), Precision::GF16);
        assert_eq!(router.get_precision(LayerType::AttentionOutput), Precision::GF16);
    }

    #[test]
    fn test_ffn_bulk_is_ternary() {
        let router = PrecisionRouter::new();
        assert_eq!(router.get_precision(LayerType::FFNGate), Precision::Ternary);
        assert_eq!(router.get_precision(LayerType::FFNUp), Precision::Ternary);
    }

    #[test]
    fn test_ffn_down_is_gf16() {
        let router = PrecisionRouter::new();
        assert_eq!(router.get_precision(LayerType::FFNDown), Precision::GF16);
    }

    #[test]
    fn test_output_is_gf16() {
        let router = PrecisionRouter::new();
        assert_eq!(router.get_precision(LayerType::OutputHead), Precision::GF16);
    }

    #[test]
    fn test_layer_name_parsing() {
        let router = PrecisionRouter::new();
        assert_eq!(router.get_precision_by_name("embedding"), Precision::GF16);
        assert_eq!(router.get_precision_by_name("attn_qkv"), Precision::GF16);
        assert_eq!(router.get_precision_by_name("ffn_gate"), Precision::Ternary);
        assert_eq!(router.get_precision_by_name("ffn_up"), Precision::Ternary);
        assert_eq!(router.get_precision_by_name("ffn_down"), Precision::GF16);
        assert_eq!(router.get_precision_by_name("output"), Precision::GF16);
    }

    #[test]
    fn test_hardware_cost() {
        assert_eq!(PrecisionRouter::mac_lut_cost(Precision::GF16), 71);
        assert_eq!(PrecisionRouter::mac_lut_cost(Precision::Ternary), 52);
        assert_eq!(PrecisionRouter::mac_dsp_cost(Precision::GF16), 16);
        assert_eq!(PrecisionRouter::mac_dsp_cost(Precision::Ternary), 0);
    }

    #[test]
    fn test_conversion_needed() {
        assert!(PrecisionRouter::needs_conversion(Precision::GF16, Precision::Ternary));
        assert!(PrecisionRouter::needs_conversion(Precision::Ternary, Precision::GF16));
        assert!(!PrecisionRouter::needs_conversion(Precision::GF16, Precision::GF16));
    }

    #[test]
    fn test_bit_width() {
        assert_eq!(Precision::GF16.bit_width(), 16);
        assert_eq!(Precision::Ternary.bit_width(), 2);
        assert_eq!(Precision::FP32.bit_width(), 32);
    }
}
