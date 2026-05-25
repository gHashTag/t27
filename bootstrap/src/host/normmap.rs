use std::collections::BTreeMap;

pub struct NormMap {
    data: BTreeMap<u64, f64>,
    mean: f64,
    std: f64,
    total_inserts: u64,
    total_lookups: u64,
}

impl NormMap {
    pub fn new() -> Self { Self { data: BTreeMap::new(), mean: 0.0, std: 1.0, total_inserts: 0, total_lookups: 0 } }

    pub fn fit(&mut self) {
        if self.data.len() < 2 { return; }
        let n = self.data.len() as f64;
        self.mean = self.data.values().sum::<f64>() / n;
        let var = self.data.values().map(|&v| (v - self.mean) * (v - self.mean)).sum::<f64>() / n;
        self.std = var.sqrt().max(1e-10);
    }

    pub fn insert(&mut self, key: u64, value: f64) {
        self.total_inserts += 1;
        self.data.insert(key, value);
    }

    pub fn get(&mut self, key: u64) -> Option<f64> {
        self.total_lookups += 1;
        self.data.get(&key).copied()
    }

    pub fn normalized(&mut self, key: u64) -> Option<f64> {
        self.total_lookups += 1;
        self.data.get(&key).map(|&v| (v - self.mean) / self.std)
    }

    pub fn denormalize(&self, z: f64) -> f64 { z * self.std + self.mean }

    pub fn z_score(&mut self, key: u64) -> Option<f64> { self.normalized(key) }

    pub fn cdf_approx(&mut self, key: u64) -> Option<f64> {
        let z = self.normalized(key)?;
        Some(0.5 * (1.0 + erf_approx(z / std::f64::consts::SQRT_2)))
    }

    pub fn contains(&self, key: u64) -> bool { self.data.contains_key(&key) }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn mean(&self) -> f64 { self.mean }
    pub fn std(&self) -> f64 { self.std }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

fn erf_approx(x: f64) -> f64 {
    let a1 = 0.254829592; let a2 = -0.284496736; let a3 = 1.421413741;
    let a4 = -1.453152027; let a5 = 1.061405429; let p = 0.3275911;
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get() {
        let mut nm = NormMap::new();
        nm.insert(1, 10.0);
        assert!((nm.get(1).unwrap() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn fit_normalize() {
        let mut nm = NormMap::new();
        for i in 0..100u64 { nm.insert(i, i as f64); }
        nm.fit();
        let z = nm.normalized(50).unwrap();
        assert!(z.abs() < 2.0);
    }

    #[test]
    fn denormalize() {
        let mut nm = NormMap::new();
        for i in 0..10u64 { nm.insert(i, i as f64 * 10.0); }
        nm.fit();
        let z = nm.normalized(5).unwrap();
        let back = nm.denormalize(z);
        assert!((back - 50.0).abs() < 1e-10);
    }

    #[test]
    fn z_score() {
        let mut nm = NormMap::new();
        nm.insert(1, 100.0); nm.insert(2, 100.0);
        nm.fit();
        assert!(nm.z_score(1).unwrap().abs() < 1e-10);
    }

    #[test]
    fn cdf() {
        let mut nm = NormMap::new();
        for i in 0..100u64 { nm.insert(i, i as f64); }
        nm.fit();
        let cdf = nm.cdf_approx(50).unwrap();
        assert!(cdf > 0.3 && cdf < 0.7);
    }

    #[test]
    fn contains() {
        let mut nm = NormMap::new();
        nm.insert(1, 1.0);
        assert!(nm.contains(1));
        assert!(!nm.contains(2));
    }

    #[test]
    fn stats() {
        let mut nm = NormMap::new();
        nm.insert(1, 1.0); nm.get(1);
        assert_eq!(nm.total_inserts(), 1);
        assert_eq!(nm.total_lookups(), 1);
    }
}
