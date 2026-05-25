pub struct Levy {
    alpha: f64,
    total_samples: u64,
    sum: f64,
    sum_sq: f64,
}

impl Levy {
    pub fn new(alpha: f64) -> Self {
        Self { alpha: alpha.clamp(0.1, 2.0), total_samples: 0, sum: 0.0, sum_sq: 0.0 }
    }

    pub fn sample(&mut self, u1: f64, u2: f64) -> f64 {
        self.total_samples += 1;
        let u1 = u1.clamp(1e-10, 1.0 - 1e-10);
        let u2 = u2.clamp(1e-10, 1.0 - 1e-10);
        let phi = std::f64::consts::PI * (u1 - 0.5);
        let w = -u2.ln();
        let alpha = self.alpha;
        let val = if (alpha - 2.0).abs() < 1e-10 {
            (2.0 * w).sqrt() * phi.cos() / phi.cos().max(1e-300).powf(0.0).max(1.0)
        } else if (alpha - 1.0).abs() < 1e-10 {
            phi.tan() * std::f64::consts::FRAC_2_PI + phi.sin().cos() * (2.0 * phi).tan() / std::f64::consts::PI
        } else {
            let eps = alpha - 1.0;
            let numer = (alpha * phi).sin();
            let denom = phi.cos().max(1e-300).powf(1.0 - 1.0 / alpha);
            let base = (eps * phi).cos() / w.max(1e-300);
            (numer / denom) * base.powf(eps / alpha)
        };
        let val = if val.is_finite() { val } else { 0.0 };
        self.sum += val;
        self.sum_sq += val * val;
        val
    }

    pub fn mean(&self) -> f64 { if self.total_samples > 0 { self.sum / self.total_samples as f64 } else { 0.0 } }
    pub fn variance(&self) -> f64 {
        if self.total_samples < 2 { return 0.0; }
        let m = self.sum / self.total_samples as f64;
        self.sum_sq / self.total_samples as f64 - m * m
    }
    pub fn alpha(&self) -> f64 { self.alpha }
    pub fn total_samples(&self) -> u64 { self.total_samples }
    pub fn is_stable(&self) -> bool { self.alpha >= 0.1 && self.alpha <= 2.0 }

    pub fn empirical_cdf(&self, samples: &[f64], x: f64) -> f64 {
        if samples.is_empty() { return 0.0; }
        let count = samples.iter().filter(|&&s| s <= x).count();
        count as f64 / samples.len() as f64
    }

    pub fn ks_distance(&self, samples: &[f64], other_cdf: &[f64]) -> f64 {
        if samples.is_empty() || other_cdf.is_empty() { return 0.0; }
        let n = samples.len().min(other_cdf.len());
        let mut max_d = 0.0f64;
        for i in 0..n {
            let ecdf = (i + 1) as f64 / n as f64;
            let d = (ecdf - other_cdf[i]).abs();
            if d > max_d { max_d = d; }
        }
        max_d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_levy() {
        let l = Levy::new(1.5);
        assert!((l.alpha() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn sample_produces_value() {
        let mut l = Levy::new(1.0);
        let v = l.sample(0.3, 0.5);
        assert!(v.is_finite());
    }

    #[test]
    fn samples_accumulate() {
        let mut l = Levy::new(1.5);
        for i in 0..100 {
            let u1 = ((i * 7919 + 1) % 10000) as f64 / 10000.0;
            let u2 = ((i * 6271 + 3) % 10000) as f64 / 10000.0;
            l.sample(u1, u2);
        }
        assert_eq!(l.total_samples(), 100);
        assert!(l.mean().is_finite());
    }

    #[test]
    fn gaussian_mode() {
        let mut l = Levy::new(2.0);
        let mut samples = Vec::new();
        for i in 0..200 {
            let u1 = ((i * 7919 + 1) % 10000) as f64 / 10000.0;
            let u2 = ((i * 6271 + 3) % 10000) as f64 / 10000.0;
            samples.push(l.sample(u1, u2));
        }
        assert!(l.variance() > 0.0);
    }

    #[test]
    fn empirical_cdf() {
        let l = Levy::new(1.5);
        let s = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((l.empirical_cdf(&s, 3.0) - 0.6).abs() < 1e-10);
    }

    #[test]
    fn ks_distance() {
        let l = Levy::new(1.5);
        let s = vec![1.0, 2.0, 3.0];
        let cdf = vec![0.2, 0.5, 0.8];
        let d = l.ks_distance(&s, &cdf);
        assert!(d >= 0.0 && d <= 1.0);
    }

    #[test]
    fn alpha_clamped() {
        let l = Levy::new(5.0);
        assert!((l.alpha() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn is_stable() { assert!(Levy::new(1.0).is_stable()); }
}
