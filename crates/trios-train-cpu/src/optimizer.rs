//! Optimizer for IGLA-GF16
//!
//! AdamW optimizer with phi-based hyperparameters.

/// AdamW optimizer with phi-based hyperparameters
///
/// Uses golden ratio-derived constants:
/// - beta1 = φ^(-1) ≈ 0.618
/// - weight_decay = α_φ ≈ 0.11803
#[derive(Debug, Clone)]
pub struct AdamWCpu {
    /// Learning rate
    pub lr: f64,

    /// First moment decay rate (φ^(-1) ≈ 0.618)
    pub beta1: f64,

    /// Second moment decay rate (typically 0.999)
    pub beta2: f64,

    /// Weight decay coefficient (α_φ ≈ 0.11803)
    pub weight_decay: f64,

    /// Numerical stability constant
    pub eps: f64,

    /// Current step
    step: usize,

    /// First moment estimate (same size as parameters, stored as f64 for precision)
    m: Vec<f64>,

    /// Second moment estimate (same size as parameters, stored as f64 for precision)
    v: Vec<f64>,
}

impl AdamWCpu {
    /// Create a new AdamW optimizer with phi-based defaults
    ///
    /// # Arguments
    ///
    /// * `param_count` - Number of parameters to optimize
    /// * `lr` - Learning rate (default: α_φ ≈ 0.11803)
    ///
    /// # Returns
    ///
    /// A new AdamW optimizer instance
    pub fn new(param_count: usize, lr: f64) -> Self {
        // Phi-based constants
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;  // φ ≈ 1.618
        let beta1 = 1.0 / phi;                   // φ^(-1) ≈ 0.618
        let weight_decay = 1.0 / (phi * phi * phi);  // α_φ ≈ 0.11803

        Self {
            lr,
            beta1,
            beta2: 0.999,
            weight_decay,
            eps: 1e-8,
            step: 0,
            m: vec![0.0; param_count],
            v: vec![0.0; param_count],
        }
    }

    /// Create a new AdamW optimizer with default learning rate (α_φ)
    pub fn with_phi_defaults(param_count: usize) -> Self {
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let lr = 1.0 / (phi * phi * phi);  // α_φ ≈ 0.11803
        Self::new(param_count, lr)
    }

    /// Create a new AdamW optimizer with custom hyperparameters
    pub fn with_params(
        param_count: usize,
        lr: f64,
        beta1: f64,
        beta2: f64,
        weight_decay: f64,
    ) -> Self {
        Self {
            lr,
            beta1,
            beta2,
            weight_decay,
            eps: 1e-8,
            step: 0,
            m: vec![0.0; param_count],
            v: vec![0.0; param_count],
        }
    }

    /// Perform a single optimization step
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters to update (modified in-place)
    /// * `gradients` - Gradients for the parameters
    pub fn step(&mut self, params: &mut [f32], gradients: &[f32]) {
        assert_eq!(
            params.len(),
            gradients.len(),
            "params and gradients must have same length"
        );
        assert_eq!(
            params.len(),
            self.m.len(),
            "parameter count mismatch with optimizer state"
        );

        self.step += 1;

        // Bias-corrected learning rate
        let bias_correction1 = 1.0 - self.beta1.powi(self.step as i32);
        let bias_correction2 = 1.0 - self.beta2.powi(self.step as i32);
        let step_size = self.lr * bias_correction2.sqrt() / bias_correction1;

        // Update each parameter
        for i in 0..params.len() {
            // Apply weight decay (decoupled from gradients in AdamW)
            params[i] -= self.weight_decay as f32 * params[i];

            // Update biased first moment estimate
            self.m[i] = self.beta1 * self.m[i] + (1.0 - self.beta1) * gradients[i] as f64;

            // Update biased second raw moment estimate
            self.v[i] = self.beta2 * self.v[i] + (1.0 - self.beta2) * (gradients[i] * gradients[i]) as f64;

            // Compute bias-corrected estimates
            let m_hat = self.m[i] / bias_correction1;
            let v_hat = self.v[i] / bias_correction2;

            // Update parameter
            params[i] -= step_size as f32 * (m_hat as f32 / ((v_hat.sqrt() as f32) + self.eps as f32));
        }
    }

    /// Reset optimizer state
    pub fn reset(&mut self) {
        self.step = 0;
        self.m.fill(0.0f64);
        self.v.fill(0.0f64);
    }

    /// Get current step number
    pub fn step_count(&self) -> usize {
        self.step
    }
}

/// Simple SGD optimizer with momentum
#[derive(Debug, Clone)]
pub struct SGDMomentum {
    /// Learning rate
    pub lr: f64,

    /// Momentum coefficient
    pub momentum: f64,

    /// Current step
    step: usize,

    /// Velocity buffer
    velocity: Vec<f32>,
}

impl SGDMomentum {
    /// Create a new SGD with momentum optimizer
    pub fn new(param_count: usize, lr: f64, momentum: f64) -> Self {
        Self {
            lr,
            momentum,
            step: 0,
            velocity: vec![0.0; param_count],
        }
    }

    /// Perform a single optimization step
    pub fn step(&mut self, params: &mut [f32], gradients: &[f32]) {
        assert_eq!(params.len(), gradients.len());

        self.step += 1;

        for i in 0..params.len() {
            // Update velocity
            self.velocity[i] =
                self.momentum as f32 * self.velocity[i] - self.lr as f32 * gradients[i];

            // Update parameter
            params[i] += self.velocity[i];
        }
    }

    /// Get current step number
    pub fn step_count(&self) -> usize {
        self.step
    }
}

/// Phi-based learning rate schedule
///
/// Returns the learning rate for a given step using the φ-schedule.
///
/// # Arguments
///
/// * `step` - Current training step
/// * `base_lr` - Base learning rate
/// * `warmup_steps` - Number of warmup steps
///
/// # Returns
///
/// Scheduled learning rate for the current step
pub fn phi_lr_schedule(step: usize, base_lr: f64, warmup_steps: usize) -> f64 {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;

    if step < warmup_steps {
        // Linear warmup
        base_lr * (step as f64 / warmup_steps as f64)
    } else {
        // φ-based decay: LR = base_lr * φ^(-(step - warmup) / warmup)
        let decay_steps = (step - warmup_steps) as f64 / warmup_steps as f64;
        base_lr * phi.powf(-decay_steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phi() -> f64 {
        (1.0 + 5.0_f64.sqrt()) / 2.0
    }

    #[test]
    fn test_adamw_phi_defaults() {
        let optimizer = AdamWCpu::with_phi_defaults(100);

        // Verify phi-based constants
        let expected_beta1 = 1.0 / phi();
        let expected_weight_decay = 1.0 / (phi() * phi() * phi());

        assert!((optimizer.beta1 - expected_beta1).abs() < 1e-6);
        assert!((optimizer.weight_decay - expected_weight_decay).abs() < 1e-6);
    }

    #[test]
    fn test_adamw_custom_params() {
        let optimizer = AdamWCpu::with_params(100, 0.001, 0.9, 0.999, 0.01);

        assert_eq!(optimizer.lr, 0.001);
        assert_eq!(optimizer.beta1, 0.9);
        assert_eq!(optimizer.beta2, 0.999);
        assert_eq!(optimizer.weight_decay, 0.01);
    }

    #[test]
    fn test_adamw_step() {
        let mut params = vec![1.0f32; 10];
        let gradients = vec![0.1f32; 10];
        let mut optimizer = AdamWCpu::with_phi_defaults(10);

        let initial_param = params[0];
        optimizer.step(&mut params, &gradients);

        // Parameter should have decreased
        assert!(params[0] < initial_param);
        assert_eq!(optimizer.step_count(), 1);

        // Step again
        optimizer.step(&mut params, &gradients);
        assert_eq!(optimizer.step_count(), 2);
    }

    #[test]
    fn test_adamw_reset() {
        let mut params = vec![1.0f32; 10];
        let gradients = vec![0.1f32; 10];
        let mut optimizer = AdamWCpu::with_phi_defaults(10);

        optimizer.step(&mut params, &gradients);
        assert!(optimizer.m.iter().any(|&m| m != 0.0));

        optimizer.reset();
        assert_eq!(optimizer.step_count(), 0);
        assert!(optimizer.m.iter().all(|&m| m == 0.0));
        assert!(optimizer.v.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_phi_lr_schedule_warmup() {
        let base_lr = 0.1;
        let warmup_steps = 10;

        // At step 0, LR should be 0
        let lr_0 = phi_lr_schedule(0, base_lr, warmup_steps);
        assert_eq!(lr_0, 0.0);

        // At step 5 (half warmup), LR should be half base
        let lr_5 = phi_lr_schedule(5, base_lr, warmup_steps);
        assert!((lr_5 - 0.05).abs() < 1e-6);

        // At step 10 (end of warmup), LR should be base
        let lr_10 = phi_lr_schedule(10, base_lr, warmup_steps);
        assert!((lr_10 - base_lr).abs() < 1e-6);
    }

    #[test]
    fn test_phi_lr_schedule_decay() {
        let base_lr = 0.1;
        let warmup_steps = 10;

        // After warmup, LR should decay
        let lr_10 = phi_lr_schedule(10, base_lr, warmup_steps);
        let lr_20 = phi_lr_schedule(20, base_lr, warmup_steps);
        let lr_30 = phi_lr_schedule(30, base_lr, warmup_steps);

        assert!(lr_20 < lr_10, "LR should decay");
        assert!(lr_30 < lr_20, "LR should continue decaying");
    }

    #[test]
    fn test_phi_lr_schedule_phi_factor() {
        let base_lr = 1.0;
        let warmup_steps = 1;

        let lr_1 = phi_lr_schedule(1, base_lr, warmup_steps);
        let lr_2 = phi_lr_schedule(2, base_lr, warmup_steps);

        // After warmup, LR should decay by factor of φ
        assert!((lr_2 - lr_1 / phi()).abs() < 1e-6);
    }

    #[test]
    fn test_sgd_momentum() {
        let mut params = vec![1.0f32; 10];
        let gradients = vec![0.1f32; 10];
        let mut optimizer = SGDMomentum::new(10, 0.01, 0.9);

        let initial_param = params[0];
        optimizer.step(&mut params, &gradients);

        // Parameter should decrease
        assert!(params[0] < initial_param);
        assert_eq!(optimizer.step_count(), 1);
    }

    #[test]
    fn test_phi_constants_precision() {
        // Verify phi-based constants meet precision requirements from Issue #32
        let optimizer = AdamWCpu::with_phi_defaults(10);

        // beta1 = phi()^-1 = 0.618 (Δ < 1e-6)
        let expected_beta1 = 1.0 / phi();
        assert!((optimizer.beta1 - expected_beta1).abs() < 1e-6);

        // weight_decay = 1/phi^3 = 0.236... (Δ < 1e-6)
        let expected_wd = 1.0 / (phi() * phi() * phi());
        assert!((optimizer.weight_decay - expected_wd).abs() < 1e-6);
        assert!((expected_wd - 0.23607).abs() < 0.0001);
    }
}
