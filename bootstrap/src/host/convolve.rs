pub struct Convolve;

impl Convolve {
    pub fn convolve(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
        let n = signal.len();
        let m = kernel.len();
        if n == 0 || m == 0 { return vec![]; }
        let out_len = n + m - 1;
        let mut out = vec![0.0f64; out_len];
        for i in 0..n { for j in 0..m { out[i + j] += signal[i] * kernel[j]; } }
        out
    }

    pub fn correlate(signal: &[f64], kernel: &[f64]) -> Vec<f64> {
        let kr: Vec<f64> = kernel.iter().rev().copied().collect();
        Self::convolve(signal, &kr)
    }

    pub fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
        if window == 0 || data.len() < window { return vec![]; }
        let mut result = Vec::with_capacity(data.len() - window + 1);
        let mut sum: f64 = data[..window].iter().sum();
        result.push(sum / window as f64);
        for i in window..data.len() {
            sum += data[i] - data[i - window];
            result.push(sum / window as f64);
        }
        result
    }

    pub fn gaussian_kernel(size: usize, sigma: f64) -> Vec<f64> {
        let mut k = Vec::with_capacity(size);
        let center = (size as f64 - 1.0) / 2.0;
        let mut sum = 0.0f64;
        for i in 0..size {
            let x = i as f64 - center;
            let v = (-x * x / (2.0 * sigma * sigma)).exp();
            k.push(v);
            sum += v;
        }
        for v in &mut k { *v /= sum; }
        k
    }

    pub fn normalize(data: &[f64]) -> Vec<f64> {
        let sum: f64 = data.iter().sum();
        if sum == 0.0 { return data.to_vec(); }
        data.iter().map(|v| v / sum).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conv_identity() {
        let r = Convolve::convolve(&[1.0, 2.0, 3.0], &[1.0]);
        assert_eq!(r, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn conv_full() {
        let r = Convolve::convolve(&[1.0, 1.0], &[1.0, 1.0]);
        assert_eq!(r, vec![1.0, 2.0, 1.0]);
    }

    #[test]
    fn moving_average() {
        let r = Convolve::moving_average(&[2.0, 4.0, 6.0, 8.0], 2);
        assert!((r[0] - 3.0).abs() < 1e-9);
        assert!((r[1] - 5.0).abs() < 1e-9);
    }

    #[test]
    fn gaussian_sums_to_1() {
        let k = Convolve::gaussian_kernel(5, 1.0);
        let sum: f64 = k.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn correlate() {
        let r = Convolve::correlate(&[1.0, 2.0], &[1.0, 0.5]);
        assert_eq!(r.len(), 3);
        assert!((r[0] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn normalize() {
        let r = Convolve::normalize(&[1.0, 3.0]);
        assert!((r[0] - 0.25).abs() < 1e-9);
        assert!((r[1] - 0.75).abs() < 1e-9);
    }
}
