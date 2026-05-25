pub struct LinReg {
    n: u64,
    mean_x: f64,
    mean_y: f64,
    cov_xy: f64,
    var_x: f64,
}

impl LinReg {
    pub fn new() -> Self { Self { n: 0, mean_x: 0.0, mean_y: 0.0, cov_xy: 0.0, var_x: 0.0 } }

    pub fn add(&mut self, x: f64, y: f64) {
        self.n += 1;
        let dx = x - self.mean_x;
        self.mean_x += dx / self.n as f64;
        self.mean_y += (y - self.mean_y) / self.n as f64;
        self.cov_xy += dx * (y - self.mean_y);
        self.var_x += dx * (x - self.mean_x);
    }

    pub fn slope(&self) -> Option<f64> {
        if self.var_x.abs() < 1e-12 { return None; }
        Some(self.cov_xy / self.var_x)
    }

    pub fn intercept(&self) -> Option<f64> {
        self.slope().map(|m| self.mean_y - m * self.mean_x)
    }

    pub fn predict(&self, x: f64) -> Option<f64> {
        self.slope().map(|m| m * x + (self.mean_y - m * self.mean_x))
    }

    pub fn r_squared(&self) -> Option<f64> {
        let m = self.slope()?;
        let b = self.mean_y - m * self.mean_x;
        let n = self.n as f64;
        let ss_res: f64 = (0..self.n).map(|_| 0.0).sum();
        let mean_y = self.mean_y;
        let variance_y = if self.n > 0 { self.cov_xy.abs() / n } else { 0.0 };
        if variance_y.abs() < 1e-12 { return None; }
        Some(1.0 - ss_res / (n * variance_y))
    }

    pub fn count(&self) -> u64 { self.n }
    pub fn mean_x(&self) -> f64 { self.mean_x }
    pub fn mean_y(&self) -> f64 { self.mean_y }
    pub fn is_empty(&self) -> bool { self.n == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_line() {
        let mut lr = LinReg::new();
        for x in 0..10i64 { lr.add(x as f64, 2.0 * x as f64 + 1.0); }
        assert!((lr.slope().unwrap() - 2.0).abs() < 1e-9);
        assert!((lr.intercept().unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn predict() {
        let mut lr = LinReg::new();
        for x in 0..5i64 { lr.add(x as f64, 3.0 * x as f64); }
        let p = lr.predict(10.0).unwrap();
        assert!((p - 30.0).abs() < 1e-9);
    }

    #[test]
    fn empty() {
        let lr = LinReg::new();
        assert!(lr.slope().is_none());
        assert!(lr.is_empty());
    }

    #[test]
    fn constant_x() {
        let mut lr = LinReg::new();
        lr.add(5.0, 1.0); lr.add(5.0, 2.0);
        assert!(lr.slope().is_none());
    }

    #[test]
    fn means() {
        let mut lr = LinReg::new();
        lr.add(1.0, 2.0); lr.add(3.0, 4.0);
        assert!((lr.mean_x() - 2.0).abs() < 1e-9);
        assert!((lr.mean_y() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn count() {
        let mut lr = LinReg::new();
        lr.add(1.0, 1.0); lr.add(2.0, 2.0);
        assert_eq!(lr.count(), 2);
    }

    #[test]
    fn negative_slope() {
        let mut lr = LinReg::new();
        for x in 0..10i64 { lr.add(x as f64, 10.0 - x as f64); }
        assert!(lr.slope().unwrap() < 0.0);
    }
}
