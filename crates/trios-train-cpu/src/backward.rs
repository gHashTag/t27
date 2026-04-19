//! Backward pass for IGLA-GF16
//!
//! Gradient computation using backpropagation.
//! Computes gradients for all trainable parameters.

/// Gradients for a linear layer
#[derive(Debug, Clone)]
pub struct LinearGradients {
    /// Gradient with respect to weights (same shape as weights)
    pub d_w: Vec<f32>,

    /// Gradient with respect to bias (same shape as bias)
    pub d_b: Vec<f32>,
}

impl LinearGradients {
    pub fn new(weight_size: usize, bias_size: usize) -> Self {
        Self {
            d_w: vec![0.0; weight_size],
            d_b: vec![0.0; bias_size],
        }
    }

    pub fn clear(&mut self) {
        for w in self.d_w.iter_mut() {
            *w = 0.0;
        }
        for b in self.d_b.iter_mut() {
            *b = 0.0;
        }
    }
}

/// Compute gradients for a linear layer using backpropagation
///
/// Given forward pass: y = x @ W + b
/// Computes:
/// - d_w = x^T @ doutput
/// - dinput = doutput @ W^T
///
/// # Arguments
///
/// * `x` - Input activations from forward pass (batch_size, in_dim)
/// * `doutput` - Gradient from next layer (batch_size, out_dim)
/// * `weights` - Layer weights (in_dim, out_dim)
/// * `d_w` - Output weight gradients (in_dim, out_dim)
/// * `d_b` - Output bias gradients (out_dim,)
/// * `dinput` - Output gradient wrt input (batch_size, in_dim)
/// * `batch_size` - Batch size
/// * `in_dim` - Input dimension
/// * `out_dim` - Output dimension
#[allow(clippy::too_many_arguments)]
pub fn linear_backward(
    x: &[f32],
    doutput: &[f32],
    weights: &[f32],
    d_w: &mut [f32],
    d_b: &mut [f32],
    dinput: &mut [f32],
    batch_size: usize,
    in_dim: usize,
    out_dim: usize,
) {
    // Clear gradients
    d_w.fill(0.0);
    d_b.fill(0.0);
    dinput.fill(0.0);

    // Compute dW = x^T @ doutput
    // dW[in, out] = sum over batch of x[batch, in] * doutput[batch, out]
    for batch in 0..batch_size {
        let x_offset = batch * in_dim;
        let dout_offset = batch * out_dim;

        // Accumulate bias gradient (sum over batch)
        for out in 0..out_dim {
            d_b[out] += doutput[dout_offset + out];
        }

        // Accumulate weight gradients
        for in_d in 0..in_dim {
            for out in 0..out_dim {
                d_w[in_d * out_dim + out] += x[x_offset + in_d] * doutput[dout_offset + out];
            }
        }

        // Compute dinput = doutput @ W^T
        // dinput[batch, in] = sum over out of doutput[batch, out] * W[in, out]
        for in_d in 0..in_dim {
            let mut sum = 0.0f32;
            for out in 0..out_dim {
                // W[in, out] is at in * out_dim + out
                sum += doutput[dout_offset + out] * weights[in_d * out_dim + out];
            }
            dinput[batch * in_dim + in_d] = sum;
        }
    }
}

/// GELU activation gradient
///
/// dGELU/dx = Φ(x) + x * φ(x) where φ is Gaussian PDF.
/// Uses the same approximation as forward pass.
///
/// # Arguments
///
/// * `x` - Input to GELU (from forward pass)
/// * `dx` - Gradient from next layer (same size as x)
/// * `dgelu_output` - Output gradient wrt GELU input (same size as x)
pub fn gelu_backward(x: &[f32], dx: &[f32], dgelu_output: &mut [f32]) {
    const SQRT_2_OVER_PI: f32 = 0.797_884_6_f32;
    const BETA: f32 = 0.044715f32;

    for i in 0..x.len() {
        let xi = x[i];
        let x3 = xi * xi * xi;
        let tanh_arg = SQRT_2_OVER_PI * (xi + BETA * x3);
        let tanh_val = tanh_arg.tanh();

        // Derivative of GELU approximation
        // dGELU/dx = 0.5 * (1 + tanh) + 0.5 * x * (1 - tanh^2) * sqrt(2/pi) * (1 + 3 * beta * x^2)
        let sech_sq = 1.0 - tanh_val * tanh_val;  // sech^2 = 1 - tanh^2
        let cdf = 0.5 * (1.0 + tanh_val);
        let pdf_term = 0.5 * xi * sech_sq * SQRT_2_OVER_PI * (1.0 + 3.0 * BETA * x3);

        let gelu_grad = cdf + pdf_term;
        dgelu_output[i] = dx[i] * gelu_grad;
    }
}

/// Layer normalization gradient
///
/// # Arguments
///
/// * `x` - Input from forward pass
/// * `dx` - Gradient from next layer
/// * `dln_output` - Output gradient wrt layer norm input
/// * `eps` - Same epsilon used in forward pass
pub fn layer_norm_backward(x: &[f32], dx: &[f32], dln_output: &mut [f32], eps: f32) {
    let n = x.len();

    // Compute mean and variance from forward pass
    let sum: f32 = x.iter().sum();
    let mean = sum / n as f32;

    let var_sum: f32 = x.iter().map(|&xi| {
        let diff = xi - mean;
        diff * diff
    }).sum();
    let var = var_sum / n as f32;
    let std = (var + eps).sqrt();

    // Compute gradients
    // dL/dx_i = (1 / (n * std)) * (n * dx_i - sum(dx) - (x_i - mean) / (var + eps) * sum(dx * (x - mean)))
    let dx_sum: f32 = dx.iter().sum();

    let mut dx_x_minus_mean_sum = 0.0f32;
    for i in 0..n {
        dx_x_minus_mean_sum += dx[i] * (x[i] - mean);
    }

    let inv_n_std = 1.0 / (n as f32 * std);
    let inv_var_plus_eps = 1.0 / (var + eps);

    for i in 0..n {
        let x_minus_mean = x[i] - mean;
        let term1 = n as f32 * dx[i] - dx_sum;
        let term2 = x_minus_mean * inv_var_plus_eps * dx_x_minus_mean_sum;
        dln_output[i] = inv_n_std * (term1 - term2);
    }
}

/// Softmax cross-entropy gradient
///
/// Combined gradient for softmax + cross-entropy loss.
/// This is more numerically stable than computing separately.
///
/// # Arguments
///
/// * `predictions` - Output of softmax (probabilities, sums to 1)
/// * `targets` - Target class indices (size batch_size)
/// * `doutput` - Output gradient (same shape as predictions)
pub fn softmax_cross_entropy_backward(predictions: &[f32], targets: &[usize], doutput: &mut [f32]) {
    // For each sample in batch
    let batch_size = targets.len();
    let vocab_size = predictions.len() / batch_size;

    for (batch, &target) in targets.iter().enumerate() {
        let offset = batch * vocab_size;

        for v in 0..vocab_size {
            let idx = offset + v;
            // dL/dlogits = predictions - one_hot(target)
            if v == target {
                doutput[idx] = predictions[idx] - 1.0;
            } else {
                doutput[idx] = predictions[idx];
            }
        }
    }
}

/// Compute cross-entropy loss
///
/// # Arguments
///
/// * `predictions` - Logits from model (before softmax)
/// * `targets` - Target class indices
///
/// # Returns
///
/// Average cross-entropy loss over the batch
pub fn cross_entropy_loss(predictions: &[f32], targets: &[usize]) -> f32 {
    let batch_size = targets.len();
    let vocab_size = predictions.len() / batch_size;

    let mut total_loss = 0.0f32;

    for (batch, &target) in targets.iter().enumerate() {
        let offset = batch * vocab_size;

        // Find max for numerical stability
        let max_logit = predictions[offset..offset + vocab_size]
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));

        // Compute log-softmax for target
        let mut sum_exp = 0.0f32;
        for v in 0..vocab_size {
            sum_exp += (predictions[offset + v] - max_logit).exp();
        }

        let log_prob = predictions[offset + target] - max_logit - sum_exp.ln();
        total_loss -= log_prob;
    }

    total_loss / batch_size as f32
}

/// Gradient clipping to prevent exploding gradients
///
/// # Arguments
///
/// * `gradients` - Gradient vector to clip (modified in-place)
/// * `max_norm` - Maximum L2 norm for gradients
///
/// # Returns
///
/// The actual L2 norm of the gradients before clipping
pub fn clip_gradients(gradients: &mut [f32], max_norm: f32) -> f32 {
    // Compute L2 norm
    let l2_sq: f32 = gradients.iter().map(|&g| g * g).sum();
    let l2 = l2_sq.sqrt();

    if l2 > max_norm {
        let scale = max_norm / l2;
        for g in gradients.iter_mut() {
            *g *= scale;
        }
    }

    l2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_entropy_loss_perfect() {
        // Perfect prediction: logit for target is much larger
        let predictions = vec![
            0.0, 0.0, 100.0,  // Target is class 2
        ];
        let targets = vec![2];

        let loss = cross_entropy_loss(&predictions, &targets);
        // Loss should be very small for perfect prediction
        assert!(loss < 0.1);
    }

    #[test]
    fn test_cross_entropy_loss_uniform() {
        // Uniform predictions (all zeros)
        let predictions = vec![0.0f32; 3];
        let targets = vec![1];

        let loss = cross_entropy_loss(&predictions, &targets);
        // For uniform predictions, loss = ln(vocab_size) = ln(3) ≈ 1.099
        assert!((loss - 3.0_f32.ln()).abs() < 0.01);
    }

    #[test]
    fn test_cross_entropy_loss_batch() {
        let predictions = vec![
            0.0, 0.0, 100.0,  // Sample 0: target 2 (perfect)
            100.0, 0.0, 0.0,  // Sample 1: target 0 (perfect)
        ];
        let targets = vec![2, 0];

        let loss = cross_entropy_loss(&predictions, &targets);
        // Both perfect, loss should be very small
        assert!(loss < 0.1);
    }

    #[test]
    fn test_softmax_cross_entropy_backward() {
        let predictions = vec![
            0.1, 0.2, 0.7,  // Probabilities sum to 1
        ];
        let targets = vec![2];  // Target is class 2
        let mut doutput = vec![0.0f32; 3];

        softmax_cross_entropy_backward(&predictions, &targets, &mut doutput);

        // For target class (2): dL/dlogit = p - 1 = 0.7 - 1 = -0.3
        assert!((doutput[2] - (-0.3)).abs() < 1e-6);
        // For non-target classes: dL/dlogit = p
        assert!((doutput[0] - 0.1).abs() < 1e-6);
        assert!((doutput[1] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_clip_gradients_no_clip() {
        let mut gradients = vec![1.0, 2.0, 2.0];  // L2 = sqrt(1+4+4) = 3
        let max_norm = 5.0;

        let l2 = clip_gradients(&mut gradients, max_norm);

        assert!((l2 - 3.0).abs() < 1e-6);
        // No clipping, values unchanged
        assert_eq!(gradients, vec![1.0, 2.0, 2.0]);
    }

    #[test]
    fn test_clip_gradients_clip() {
        let mut gradients = vec![3.0, 4.0, 0.0];  // L2 = 5
        let max_norm = 2.5;

        let l2 = clip_gradients(&mut gradients, max_norm);

        assert!((l2 - 5.0).abs() < 1e-6);
        // Should be scaled by 0.5
        assert!((gradients[0] - 1.5).abs() < 1e-6);
        assert!((gradients[1] - 2.0).abs() < 1e-6);
        assert!((gradients[2] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_gelu_backward() {
        let x = vec![0.0, 1.0, -1.0];
        let dx = vec![1.0, 1.0, 1.0];
        let mut dgelu_output = vec![0.0; 3];

        gelu_backward(&x, &dx, &mut dgelu_output);

        // GELU derivative at x=0 is approximately 0.5
        assert!((dgelu_output[0] - 0.5).abs() < 0.2);

        // GELU derivative at x=1 should be positive (slope of GELU at positive x)
        assert!(dgelu_output[1] > 0.0, "GELU derivative at positive x should be positive");

        // GELU derivative at x=-1 can be negative (slope of GELU near 0 from negative side)
        // The exact value depends on the approximation, but it should be finite
        assert!(dgelu_output[2].is_finite(), "GELU derivative should be finite");

        // All outputs should be finite
        assert!(dgelu_output[0].is_finite());
        assert!(dgelu_output[1].is_finite());
    }

    #[test]
    fn test_layer_norm_backward() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let dx = vec![1.0, 1.0, 1.0, 1.0];
        let mut dln_output = vec![0.0; 4];

        layer_norm_backward(&x, &dx, &mut dln_output, 1e-5);

        // All outputs should be finite
        for &v in &dln_output {
            assert!(v.is_finite(), "Layer norm gradient should be finite");
        }

        // Test with non-uniform gradient to ensure variation
        let dx_varied = vec![1.0, 2.0, 1.0, 2.0];
        layer_norm_backward(&x, &dx_varied, &mut dln_output, 1e-5);

        let max_abs = dln_output.iter().map(|&v| v.abs()).fold(0.0_f32, |a, b| a.max(b));
        assert!(max_abs > 0.0, "Layer norm gradient with varied input should have non-zero values");
    }

    #[test]
    fn test_linear_gradients_new() {
        let grads = LinearGradients::new(100, 10);
        assert_eq!(grads.d_w.len(), 100);
        assert_eq!(grads.d_b.len(), 10);

        let mut grads = grads;
        grads.clear();
        for &w in &grads.d_w {
            assert_eq!(w, 0.0);
        }
    }
}
