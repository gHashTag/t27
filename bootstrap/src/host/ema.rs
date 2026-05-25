pub struct Ema;

impl Ema {
    pub fn compute(data: &[f64], span: usize) -> Vec<f64> {
        if data.is_empty() || span == 0 { return Vec::new(); }
        let alpha = 2.0 / (span as f64 + 1.0);
        let mut result = Vec::with_capacity(data.len());
        let mut ema = data[0];
        result.push(ema);
        for &v in &data[1..] {
            ema = alpha * v + (1.0 - alpha) * ema;
            result.push(ema);
        }
        result
    }

    pub fn macd(data: &[f64], fast: usize, slow: usize, signal: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let ema_fast = Self::compute(data, fast);
        let ema_slow = Self::compute(data, slow);
        let macd_line: Vec<f64> = ema_fast.iter().zip(ema_slow.iter()).map(|(f, s)| f - s).collect();
        let signal_line = Self::compute(&macd_line, signal);
        let histogram: Vec<f64> = macd_line.iter().zip(signal_line.iter()).map(|(m, s)| m - s).collect();
        (macd_line, signal_line, histogram)
    }

    pub fn crossover_signals(data: &[f64], fast: usize, slow: usize) -> Vec<(usize, bool)> {
        let ema_fast = Self::compute(data, fast);
        let ema_slow = Self::compute(data, slow);
        let mut signals = Vec::new();
        for i in 1..data.len() {
            let prev_diff = ema_fast[i - 1] - ema_slow[i - 1];
            let curr_diff = ema_fast[i] - ema_slow[i];
            if prev_diff <= 0.0 && curr_diff > 0.0 { signals.push((i, true)); }
            else if prev_diff >= 0.0 && curr_diff < 0.0 { signals.push((i, false)); }
        }
        signals
    }

    pub fn smoothing_factor(span: usize) -> f64 { 2.0 / (span as f64 + 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ema = Ema::compute(&data, 3);
        assert_eq!(ema.len(), 5);
        assert_eq!(ema[0], 1.0);
        assert!(ema[4] > ema[0]);
    }

    #[test]
    fn smoothing_factor() {
        let alpha = Ema::smoothing_factor(3);
        assert!((alpha - 0.5).abs() < 1e-9);
    }

    #[test]
    fn macd_length() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (macd, signal, hist) = Ema::macd(&data, 3, 5, 3);
        assert_eq!(macd.len(), 10);
        assert_eq!(signal.len(), 10);
        assert_eq!(hist.len(), 10);
    }

    #[test]
    fn empty() { assert!(Ema::compute(&[], 5).is_empty()); }

    #[test]
    fn crossover() {
        let data = vec![1.0, 2.0, 3.0, 2.0, 1.0, 2.0, 3.0];
        let sigs = Ema::crossover_signals(&data, 2, 5);
        assert!(!sigs.is_empty() || data.len() < 3);
    }
}
