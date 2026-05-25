pub struct InterpG {
    data: Vec<Vec<f64>>,
    rows: usize,
    cols: usize,
    total_queries: u64,
}

impl InterpG {
    pub fn new(data: Vec<Vec<f64>>) -> Self {
        let rows = data.len();
        let cols = if rows > 0 { data[0].len() } else { 0 };
        Self { data, rows, cols, total_queries: 0 }
    }

    pub fn bilinear(&mut self, x: f64, y: f64) -> f64 {
        self.total_queries += 1;
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.cols - 1);
        let y1 = (y0 + 1).min(self.rows - 1);
        let x0 = x0.min(self.cols - 1);
        let y0 = y0.min(self.rows - 1);
        let xf = x - x.floor();
        let yf = y - y.floor();
        let top = self.data[y0][x0] * (1.0 - xf) + self.data[y0][x1] * xf;
        let bot = self.data[y1][x0] * (1.0 - xf) + self.data[y1][x1] * xf;
        top * (1.0 - yf) + bot * yf
    }

    pub fn nearest(&mut self, x: f64, y: f64) -> f64 {
        self.total_queries += 1;
        let xi = (x.round() as usize).min(self.cols - 1);
        let yi = (y.round() as usize).min(self.rows - 1);
        self.data[yi][xi]
    }

    pub fn bicubic(&mut self, x: f64, y: f64) -> f64 {
        self.total_queries += 1;
        let x0 = x.floor() as i64;
        let y0 = y.floor() as i64;
        let xf = x - x.floor();
        let yf = y - y.floor();
        let mut result = 0.0f64;
        for j in -1i64..=2 {
            for i in -1i64..=2 {
                let xi = (x0 + i).clamp(0, self.cols as i64 - 1) as usize;
                let yi = (y0 + j).clamp(0, self.rows as i64 - 1) as usize;
                let wx = Self::cubic_weight(xf - i as f64);
                let wy = Self::cubic_weight(yf - j as f64);
                result += self.data[yi][xi] * wx * wy;
            }
        }
        result
    }

    fn cubic_weight(t: f64) -> f64 {
        let t = t.abs();
        if t <= 1.0 { (1.5 * t - 2.5) * t * t + 1.0 }
        else if t <= 2.0 { ((-0.5 * t + 2.5) * t - 4.0) * t + 2.0 }
        else { 0.0 }
    }

    pub fn rows(&self) -> usize { self.rows }
    pub fn cols(&self) -> usize { self.cols }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> Vec<Vec<f64>> { vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0], vec![7.0, 8.0, 9.0]] }

    #[test]
    fn nearest_exact() {
        let mut ig = InterpG::new(grid());
        assert!((ig.nearest(0.0, 0.0) - 1.0).abs() < 1e-10);
        assert!((ig.nearest(1.0, 1.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn bilinear_corner() {
        let mut ig = InterpG::new(grid());
        assert!((ig.bilinear(0.0, 0.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn bilinear_midpoint() {
        let mut ig = InterpG::new(grid());
        let v = ig.bilinear(0.5, 0.5);
        assert!((v - 3.0).abs() < 1e-10);
    }

    #[test]
    fn bicubic_midpoint() {
        let mut ig = InterpG::new(grid());
        let v = ig.bicubic(1.0, 1.0);
        assert!((v - 5.0).abs() < 1e-10);
    }

    #[test]
    fn out_of_bounds_clamps() {
        let mut ig = InterpG::new(grid());
        let v = ig.bilinear(10.0, 10.0);
        assert!(v.is_finite());
    }

    #[test]
    fn dims() { let ig = InterpG::new(grid()); assert_eq!(ig.rows(), 3); assert_eq!(ig.cols(), 3); }

    #[test]
    fn stats() {
        let mut ig = InterpG::new(grid());
        ig.nearest(0.0, 0.0); ig.bilinear(0.5, 0.5);
        assert_eq!(ig.total_queries(), 2);
    }
}
