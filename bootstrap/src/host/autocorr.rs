pub struct Autocorr;

impl Autocorr {
    pub fn autocorrelation(signal: &[f64], max_lag: usize) -> Vec<f64> {
        let n = signal.len();
        if n == 0 { return Vec::new(); }
        let mean = signal.iter().sum::<f64>() / n as f64;
        let var: f64 = signal.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
        if var == 0.0 { return vec![0.0; max_lag.min(n) + 1]; }
        let mut result = Vec::with_capacity(max_lag + 1);
        for lag in 0..=max_lag.min(n - 1) {
            let mut sum = 0.0f64;
            for i in 0..n - lag {
                sum += (signal[i] - mean) * (signal[i + lag] - mean);
            }
            result.push(sum / (n as f64 * var));
        }
        result
    }

    pub fn peak_period(signal: &[f64], max_lag: usize) -> Option<usize> {
        let acf = Self::autocorrelation(signal, max_lag);
        if acf.len() < 2 { return None; }
        let mut best_lag = 1;
        let mut best_val = f64::NEG_INFINITY;
        for lag in 1..acf.len() {
            if acf[lag] > best_val {
                best_val = acf[lag];
                best_lag = lag;
            }
        }
        if best_val > 0.0 { Some(best_lag) } else { None }
    }

    pub fn energy(signal: &[f64]) -> f64 { signal.iter().map(|x| x * x).sum() }

    pub fn zero_crossings(signal: &[f64]) -> usize {
        if signal.len() < 2 { return 0; }
        signal.windows(2).filter(|w| w[0].signum() != w[1].signum()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lag_zero_unity() {
        let sig = vec![1.0, 2.0, 3.0, 4.0, 3.0, 2.0];
        let acf = Autocorr::autocorrelation(&sig, 3);
        assert!((acf[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn periodic_peak() {
        let sig: Vec<f64> = (0..40).map(|i| (i as f64 * std::f64::consts::PI * 2.0 / 8.0).sin()).collect();
        let period = Autocorr::peak_period(&sig, 16);
        assert_eq!(period, Some(8));
    }

    #[test]
    fn energy() {
        let sig = vec![3.0, 4.0];
        assert!((Autocorr::energy(&sig) - 25.0).abs() < 1e-9);
    }

    #[test]
    fn zero_crossings() {
        let sig = vec![1.0, -1.0, 1.0, -1.0];
        assert_eq!(Autocorr::zero_crossings(&sig), 3);
    }

    #[test]
    fn constant_signal() {
        let acf = Autocorr::autocorrelation(&[5.0, 5.0, 5.0], 2);
        assert!(acf.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn empty() { assert!(Autocorr::autocorrelation(&[], 5).is_empty()); }
}
