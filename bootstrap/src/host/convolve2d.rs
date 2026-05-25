pub struct Convolve2D;

impl Convolve2D {
    pub fn apply(matrix: &[Vec<f64>], kernel: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let rows = matrix.len();
        if rows == 0 { return Vec::new(); }
        let cols = matrix[0].len();
        let kr = kernel.len();
        let kc = if kr > 0 { kernel[0].len() } else { 0 };
        let pr = kr / 2;
        let pc = kc / 2;
        let mut out = vec![vec![0.0f64; cols]; rows];
        for i in 0..rows {
            for j in 0..cols {
                let mut sum = 0.0f64;
                for ki in 0..kr {
                    for kj in 0..kc {
                        let mi = (i as isize + ki as isize - pr as isize) as usize;
                        let mj = (j as isize + kj as isize - pc as isize) as usize;
                        if mi < rows && mj < cols {
                            sum += matrix[mi][mj] * kernel[ki][kj];
                        }
                    }
                }
                out[i][j] = sum;
            }
        }
        out
    }

    pub fn sobel_x() -> Vec<Vec<f64>> {
        vec![vec![-1.0, 0.0, 1.0], vec![-2.0, 0.0, 2.0], vec![-1.0, 0.0, 1.0]]
    }

    pub fn sobel_y() -> Vec<Vec<f64>> {
        vec![vec![-1.0, -2.0, -1.0], vec![0.0, 0.0, 0.0], vec![1.0, 2.0, 1.0]]
    }

    pub fn gaussian(size: usize, sigma: f64) -> Vec<Vec<f64>> {
        let mut kernel = vec![vec![0.0f64; size]; size];
        let center = size as f64 / 2.0;
        let mut sum = 0.0f64;
        for i in 0..size {
            for j in 0..size {
                let dx = i as f64 + 0.5 - center;
                let dy = j as f64 + 0.5 - center;
                let val = (-((dx * dx + dy * dy) / (2.0 * sigma * sigma))).exp();
                kernel[i][j] = val;
                sum += val;
            }
        }
        for row in &mut kernel { for v in row.iter_mut() { *v /= sum; } }
        kernel
    }

    pub fn normalize_kernel(kernel: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let sum: f64 = kernel.iter().flat_map(|r| r.iter()).sum();
        if sum == 0.0 { return kernel.to_vec(); }
        kernel.iter().map(|r| r.iter().map(|v| v / sum).collect()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_kernel() {
        let m = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let k = vec![vec![0.0, 0.0, 0.0], vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 0.0]];
        let r = Convolve2D::apply(&m, &k);
        assert!((r[0][0] - 1.0).abs() < 1e-9);
        assert!((r[1][1] - 4.0).abs() < 1e-9);
    }

    #[test]
    fn box_blur() {
        let m = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0], vec![7.0, 8.0, 9.0]];
        let k = Convolve2D::normalize_kernel(&vec![vec![1.0; 3]; 3]);
        let r = Convolve2D::apply(&m, &k);
        assert!((r[1][1] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn gaussian_normalized() {
        let g = Convolve2D::gaussian(5, 1.0);
        let sum: f64 = g.iter().flat_map(|r| r.iter()).sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sobel_shapes() {
        let sx = Convolve2D::sobel_x();
        let sy = Convolve2D::sobel_y();
        assert_eq!(sx.len(), 3);
        assert_eq!(sy[0].len(), 3);
    }

    #[test]
    fn empty() { assert!(Convolve2D::apply(&[], &vec![vec![1.0]]).is_empty()); }
}
