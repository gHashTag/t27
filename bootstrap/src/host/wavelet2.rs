pub struct Wavelet2 {
    total_fwd: u64,
    total_inv: u64,
}

impl Wavelet2 {
    pub fn new() -> Self { Self { total_fwd: 0, total_inv: 0 } }

    pub fn forward(&mut self, data: &mut [Vec<f64>]) {
        self.total_fwd += 1;
        let rows = data.len();
        if rows == 0 { return; }
        let cols = data[0].len();
        let mut temp;
        let s2 = std::f64::consts::FRAC_1_SQRT_2;
        let mut h = rows;
        while h > 1 {
            temp = vec![vec![0.0; cols]; rows];
            for i in 0..h {
                for j in 0..cols/2 {
                    let a = data[i][2*j];
                    let b = data[i][2*j+1];
                    temp[i][j] = s2 * (a + b);
                    temp[i][cols/2 + j] = s2 * (a - b);
                }
            }
            for i in 0..h { for j in 0..cols { data[i][j] = temp[i][j]; } }
            let mut temp2 = vec![vec![0.0; cols]; rows];
            for j in 0..cols/2 {
                for i in 0..h/2 {
                    let a = data[2*i][j];
                    let b = data[2*i+1][j];
                    temp2[i][j] = s2 * (a + b);
                    temp2[h/2 + i][j] = s2 * (a - b);
                }
            }
            for i in 0..h/2 { for j in 0..cols/2 { data[i][j] = temp2[i][j]; data[h/2+i][j] = temp2[h/2+i][j]; } }
            h /= 2;
        }
    }

    pub fn inverse(&mut self, data: &mut [Vec<f64>]) {
        self.total_inv += 1;
        let rows = data.len();
        if rows == 0 { return; }
        let cols = data[0].len();
        let s2 = std::f64::consts::FRAC_1_SQRT_2;
        let mut h = 2;
        while h <= rows {
            let mut temp = vec![vec![0.0; cols]; rows];
            for j in 0..h/2 {
                for i in 0..h/2 {
                    let avg = data[i][j];
                    let diff = data[h/2+i][j];
                    temp[2*i][j] = s2 * (avg + diff);
                    temp[2*i+1][j] = s2 * (avg - diff);
                }
            }
            for i in 0..h { for j in 0..h/2 { data[i][j] = temp[i][j]; } }
            let mut temp2 = vec![vec![0.0; cols]; rows];
            for i in 0..h {
                for j in 0..h/2 {
                    let avg = data[i][j];
                    let diff = data[i][h/2+j];
                    temp2[i][2*j] = s2 * (avg + diff);
                    temp2[i][2*j+1] = s2 * (avg - diff);
                }
            }
            for i in 0..h { for j in 0..h { data[i][j] = temp2[i][j]; } }
            h *= 2;
        }
    }

    pub fn energy(&self, data: &[Vec<f64>]) -> f64 {
        data.iter().flat_map(|r| r.iter()).map(|&v| v * v).sum()
    }

    pub fn energy_ratio(&self, data: &[Vec<f64>], band_size: usize) -> f64 {
        let total = self.energy(data);
        if total == 0.0 { return 0.0; }
        let mut ll = 0.0f64;
        for i in 0..band_size.min(data.len()) {
            for j in 0..band_size.min(data[0].len()) { ll += data[i][j] * data[i][j]; }
        }
        ll / total
    }

    pub fn total_fwd(&self) -> u64 { self.total_fwd }
    pub fn total_inv(&self) -> u64 { self.total_inv }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mat2x2() -> Vec<Vec<f64>> { vec![vec![1.0, 2.0], vec![3.0, 4.0]] }

    #[test]
    fn forward_inverse() {
        let mut w = Wavelet2::new();
        let mut m = mat2x2();
        let orig: Vec<Vec<f64>> = m.clone();
        w.forward(&mut m);
        w.inverse(&mut m);
        for i in 0..2 { for j in 0..2 { assert!((m[i][j] - orig[i][j]).abs() < 1e-10); } }
    }

    #[test]
    fn forward_changes() {
        let mut w = Wavelet2::new();
        let mut m = mat2x2();
        let orig: Vec<Vec<f64>> = m.clone();
        w.forward(&mut m);
        let mut diff = false;
        for i in 0..2 { for j in 0..2 { if (m[i][j] - orig[i][j]).abs() > 1e-10 { diff = true; } } }
        assert!(diff);
    }

    #[test]
    fn energy_conserved() {
        let mut w = Wavelet2::new();
        let mut m = mat2x2();
        let e1 = w.energy(&m);
        w.forward(&mut m);
        let e2 = w.energy(&m);
        assert!((e1 - e2).abs() < 1e-10);
    }

    #[test]
    fn energy_ratio() {
        let mut w = Wavelet2::new();
        let mut m = vec![vec![10.0, 0.0], vec![0.0, 0.0]];
        w.forward(&mut m);
        let r = w.energy_ratio(&m, 1);
        assert!(r > 0.0);
    }

    #[test]
    fn zero_energy() {
        let z = vec![vec![0.0; 4]; 4];
        assert_eq!(Wavelet2::new().energy(&z), 0.0);
    }

    #[test]
    fn stats() {
        let mut w = Wavelet2::new();
        let mut m = mat2x2();
        w.forward(&mut m); w.inverse(&mut m);
        assert_eq!(w.total_fwd(), 1);
        assert_eq!(w.total_inv(), 1);
    }
}
