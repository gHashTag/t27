//! Ternary BitLinear Engine
//!
//! Implements ternary {-1, 0, +1} quantization for bulk compute layers
//! (FFN gate, FFN up, activations). Uses zero-DSP architecture for
//! maximum efficiency on XC7A100T FPGA.
//!
//! ## Usage in Hybrid Pipeline
//!
//! Per STATIC_ROUTING_TABLE, these layers get Ternary format:
//! - FFN gate (first FFN linear)
//! - FFN up (second FFN linear)
//! - FFN down (third FFN linear) - some architectures use GF16
//! - Activations (GELU, ReLU)
//!
//! ## Key Benefits
//!
//! - Zero DSP cost (uses only LUT)
//! - 59× fewer LUT than GF16 at unit level
//! - Compatible with QAT + STE for training-aware quantization

use serde::{Deserialize, Serialize};

// ==============================================================================
// TERNARY VALUE TYPE
// ==============================================================================

/// Ternary value: {-1, 0, +1}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i8)]
pub enum Ternary {
    /// Negative weight (-1)
    NegOne = -1,

    /// Zero weight (0) - enables pruning
    Zero = 0,

    /// Positive weight (+1)
    PosOne = 1,
}

impl Ternary {
    /// Convert f32 to ternary with thresholding
    pub fn from_f32(value: f32) -> Self {
        if value > 0.5 {
            Ternary::PosOne
        } else if value < -0.5 {
            Ternary::NegOne
        } else {
            Ternary::Zero
        }
    }

    /// Convert ternary back to f32
    pub fn to_f32(self) -> f32 {
        self as i8 as f32
    }

    /// Get bit-width per parameter (log2(3) ≈ 1.585)
    pub fn bits_per_param() -> f32 {
        (3.0_f32).log2()
    }
}

// ==============================================================================
// TERNARY QUANTIZATION (Per-Tensor)
// ==============================================================================

/// Quantize f32 weights to ternary
///
/// # Arguments
/// * `weights` - f32 weight tensor
/// * `scale` - Scaling factor for full-range quantization
///
/// # Returns
/// Vector of Ternary values
pub fn quantize(weights: &[f32], scale: f32) -> Vec<Ternary> {
    weights.iter().map(|&w| {
        let scaled = w * scale;
        Ternary::from_f32(scaled)
    }).collect()
}

/// Dequantize ternary weights back to f32
///
/// # Arguments
/// * `ternary_weights` - Ternary quantized weights
/// * `scale` - Scaling factor used during quantization
///
/// # Returns
/// f32 weights
pub fn dequantize(ternary_weights: &[Ternary], scale: f32) -> Vec<f32> {
    ternary_weights.iter().map(|&t| t.to_f32() / scale).collect()
}

/// Compute optimal scaling factor for ternary quantization
///
/// Uses max-abs scaling to preserve dynamic range
///
/// # Arguments
/// * `weights` - f32 weight tensor
///
/// # Returns
/// Optimal scaling factor (1.0 / max_abs_weight)
pub fn compute_scale(weights: &[f32]) -> f32 {
    if weights.is_empty() {
        return 1.0;
    }

    let max_abs = weights.iter().fold(0.0_f32, |acc, &w| acc.abs().max(w.abs()));
    if max_abs > 0.0 {
        1.0 / max_abs
    } else {
        1.0
    }
}

/// Calculate sparsity after ternary quantization
///
/// # Arguments
/// * `ternary_weights` - Ternary quantized weights
///
/// # Returns
/// Sparsity ratio (0.0 = all zero, 1.0 = none zero)
pub fn compute_sparsity(ternary_weights: &[Ternary]) -> f32 {
    let zero_count = ternary_weights.iter().filter(|&&t| t == Ternary::Zero).count();
    zero_count as f32 / ternary_weights.len() as f32
}

// ==============================================================================
// HYBRID API - TERNARY FOR BULK LAYERS
// ==============================================================================

/// Ternary quantization for FFN layers (bulk compute)
///
/// Used in hybrid precision pipeline where FFN gate/up use Ternary
/// for zero-DSP efficiency.
pub mod ffn {
    use super::*;

    /// Quantize FFN gate weights to ternary
    ///
    /// FFN gate determines activation routing - can use ternary
    /// because it's followed by GELU nonlinearity which handles quantization noise.
    pub fn quantize_gate(weights: &[f32], scale: Option<f32>) -> Vec<Ternary> {
        let scale = scale.unwrap_or_else(|| compute_scale(weights));
        quantize(weights, scale)
    }

    /// Quantize FFN up weights to ternary
    ///
    /// FFN up expands dimensionality - massive compute, ternary ideal.
    pub fn quantize_up(weights: &[f32], scale: Option<f32>) -> Vec<Ternary> {
        let scale = scale.unwrap_or_else(|| compute_scale(weights));
        quantize(weights, scale)
    }

    /// Quantize FFN down weights to ternary
    ///
    /// FFN down projects back to d_model. Some architectures use GF16
    /// here for precision, but ternary is possible with QAT.
    pub fn quantize_down(weights: &[f32], scale: Option<f32>) -> Vec<Ternary> {
        let scale = scale.unwrap_or_else(|| compute_scale(weights));
        quantize(weights, scale)
    }

    /// Calculate memory savings from ternary FFN layers
    ///
    /// # Arguments
    /// * `num_params` - Number of parameters in FFN layer
    ///
    /// # Returns
    /// Memory in bytes (1.58 bits/param = 0.2 bytes/param)
    pub fn ternary_size_bytes(num_params: usize) -> usize {
        // 1.58 bits/param = 0.2 bytes/param (approximately)
        num_params / 5  // Integer division for conservative estimate
    }

    /// Calculate compression ratio vs f32
    ///
    /// # Arguments
    /// * `num_params` - Number of parameters
    ///
    /// # Returns
    /// Compression ratio (32.0 / 1.58 ≈ 20.25x)
    pub fn compression_ratio(_num_params: usize) -> f32 {
        32.0 / Ternary::bits_per_param()
    }
}

// ==============================================================================
// TESTS
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_from_f32() {
        assert_eq!(Ternary::from_f32(1.0), Ternary::PosOne);
        assert_eq!(Ternary::from_f32(-1.0), Ternary::NegOne);
        assert_eq!(Ternary::from_f32(0.0), Ternary::Zero);
    }

    #[test]
    fn test_quantize_dequantize() {
        let weights = vec![1.5, -0.8, 0.2, 2.0];
        let scale = compute_scale(&weights);
        let ternary = quantize(&weights, scale);
        let dequant = dequantize(&ternary, scale);

        for (orig, got) in weights.iter().zip(dequant.iter()) {
            assert!((orig - got).abs() < 1.0, "roundtrip error");
        }
    }

    #[test]
    fn test_compute_scale() {
        let weights = vec![0.1, 0.5, 1.0, 1.5];
        let scale = compute_scale(&weights);
        assert_eq!(scale, 1.0 / 1.5);
    }

    #[test]
    fn test_sparsity() {
        let ternary = vec![Ternary::PosOne, Ternary::Zero, Ternary::NegOne, Ternary::Zero];
        let sparsity = compute_sparsity(&ternary);
        assert_eq!(sparsity, 0.5);
    }

    #[test]
    fn test_ffn_quantization() {
        let gate_weights = vec![0.2, 0.8, -0.3, 0.6, -0.1, 0.9];
        let ternary_gate = ffn::quantize_gate(&gate_weights, None);
        assert_eq!(ternary_gate.len(), 6);

        let sparsity = compute_sparsity(&ternary_gate);
        assert!(sparsity > 0.0 && sparsity < 1.0);
    }

    #[test]
    fn test_bits_per_param() {
        assert!((Ternary::bits_per_param() - 1.585).abs() < 0.01);
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = ffn::compression_ratio(1000);
        assert!(ratio > 20.0 && ratio < 21.0);
    }
}
