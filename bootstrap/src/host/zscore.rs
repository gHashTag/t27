pub struct ZScore;

impl ZScore {
    pub fn mean(data: &[f64]) -> f64 {
        if data.is_empty() { return 0.0; }
        data.iter().sum::<f64>() / data.len() as f64
    }

    pub fn stddev(data: &[f64]) -> f64 {
        let m = Self::mean(data);
        let variance = if data.is_empty() { 0.0 }
            else { data.iter().map(|x| (x - m).powi(2)).sum::<f64>() / data.len() as f64 };
        variance.sqrt()
    }

    pub fn normalize(data: &[f64]) -> Vec<f64> {
        let m = Self::mean(data);
        let s = Self::stddev(data);
        if s == 0.0 { return vec![0.0; data.len()]; }
        data.iter().map(|x| (x - m) / s).collect()
    }

    pub fn outliers(data: &[f64], threshold: f64) -> Vec<usize> {
        let norm = Self::normalize(data);
        norm.iter().enumerate()
            .filter(|(_, &z)| z.abs() > threshold)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn modified_zscore(data: &[f64]) -> Vec<f64> {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted.is_empty() { 0.0 }
            else if sorted.len() % 2 == 0 { (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0 }
            else { sorted[sorted.len() / 2] };
        let mut abs_dev: Vec<f64> = data.iter().map(|x| (x - median).abs()).collect();
        abs_dev.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mad = if abs_dev.is_empty() { 0.0 }
            else if abs_dev.len() % 2 == 0 { (abs_dev[abs_dev.len() / 2 - 1] + abs_dev[abs_dev.len() / 2]) / 2.0 }
            else { abs_dev[abs_dev.len() / 2] };
        if mad == 0.0 { return vec![0.0; data.len()]; }
        data.iter().map(|x| 0.6745 * (x - median) / mad).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_mean_std() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let norm = ZScore::normalize(&data);
        let m = ZScore::mean(&norm);
        assert!(m.abs() < 1e-9);
        let s = ZScore::stddev(&norm);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn outliers_detected() {
        let data = vec![1.0, 2.0, 2.0, 2.0, 100.0];
        let out = ZScore::outliers(&data, 1.5);
        assert!(out.contains(&4));
    }

    #[test]
    fn constant_data() {
        let norm = ZScore::normalize(&[5.0, 5.0, 5.0]);
        assert!(norm.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn modified_zscore() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        let mz = ZScore::modified_zscore(&data);
        assert!(mz[4].abs() > mz[0].abs());
    }

    #[test]
    fn empty() {
        assert_eq!(ZScore::mean(&[]), 0.0);
        assert!(ZScore::normalize(&[]).is_empty());
    }
}
