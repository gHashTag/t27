pub struct Ewma {
    alpha: f64,
    value: Option<f64>,
    total_updates: u64,
}

impl Ewma {
    pub fn new(alpha: f64) -> Self { Self { alpha: alpha.clamp(0.0, 1.0), value: None, total_updates: 0 } }

    pub fn update(&mut self, sample: f64) {
        self.total_updates += 1;
        match self.value {
            None => self.value = Some(sample),
            Some(ref mut v) => *v = self.alpha * sample + (1.0 - self.alpha) * *v,
        }
    }

    pub fn get(&self) -> Option<f64> { self.value }
    pub fn alpha(&self) -> f64 { self.alpha }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn is_empty(&self) -> bool { self.value.is_none() }

    pub fn forecast(&self, steps: usize) -> Option<f64> { self.value.map(|v| v) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial() { let e = Ewma::new(0.3); assert!(e.get().is_none()); }

    #[test]
    fn first_sample() { let mut e = Ewma::new(0.5); e.update(10.0); assert!((e.get().unwrap() - 10.0).abs() < 1e-9); }

    #[test]
    fn smoothing() {
        let mut e = Ewma::new(0.5);
        e.update(0.0); e.update(10.0);
        let v = e.get().unwrap();
        assert!((v - 5.0).abs() < 1e-9);
    }

    #[test]
    fn high_alpha() {
        let mut e = Ewma::new(0.9);
        e.update(0.0); e.update(100.0);
        let v = e.get().unwrap();
        assert!(v > 80.0);
    }

    #[test]
    fn low_alpha() {
        let mut e = Ewma::new(0.1);
        e.update(0.0); e.update(100.0);
        let v = e.get().unwrap();
        assert!(v < 20.0);
    }

    #[test]
    fn many_samples() {
        let mut e = Ewma::new(0.3);
        for _ in 0..100 { e.update(50.0); }
        assert!((e.get().unwrap() - 50.0).abs() < 1.0);
    }

    #[test]
    fn stats() {
        let mut e = Ewma::new(0.5);
        e.update(1.0); e.update(2.0);
        assert_eq!(e.total_updates(), 2);
    }

    #[test]
    fn clamp_alpha() {
        let e = Ewma::new(5.0);
        assert!((e.alpha() - 1.0).abs() < 1e-9);
    }
}
