use std::collections::BTreeMap;

pub struct MetricMap {
    metrics: BTreeMap<String, Vec<f64>>,
    total_records: u64,
    total_queries: u64,
}

impl MetricMap {
    pub fn new() -> Self { Self { metrics: BTreeMap::new(), total_records: 0, total_queries: 0 } }

    pub fn record(&mut self, name: &str, value: f64) {
        self.total_records += 1;
        self.metrics.entry(name.to_string()).or_default().push(value);
    }

    pub fn mean(&mut self, name: &str) -> Option<f64> {
        self.total_queries += 1;
        let v = self.metrics.get(name)?;
        if v.is_empty() { return None; }
        Some(v.iter().sum::<f64>() / v.len() as f64)
    }

    pub fn percentile(&mut self, name: &str, p: f64) -> Option<f64> {
        self.total_queries += 1;
        let v = self.metrics.get(name)?;
        if v.is_empty() { return None; }
        let mut sorted = v.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    pub fn count(&self, name: &str) -> usize { self.metrics.get(name).map(|v| v.len()).unwrap_or(0) }

    pub fn sum(&mut self, name: &str) -> f64 {
        self.total_queries += 1;
        self.metrics.get(name).map(|v| v.iter().sum()).unwrap_or(0.0)
    }

    pub fn tag_record(&mut self, prefix: &str, tag: &str, value: f64) {
        let key = format!("{prefix}{{{tag}}}");
        self.record(&key, value);
    }

    pub fn tag_aggregate(&mut self, prefix: &str) -> BTreeMap<String, f64> {
        self.total_queries += 1;
        let mut result = BTreeMap::new();
        for (k, v) in &self.metrics {
            if k.starts_with(prefix) && v.iter().sum::<f64>() != 0.0 {
                result.insert(k.clone(), v.iter().sum::<f64>() / v.len() as f64);
            }
        }
        result
    }

    pub fn metric_names(&self) -> Vec<&str> { self.metrics.keys().map(|s| s.as_str()).collect() }
    pub fn total_records(&self) -> u64 { self.total_records }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_mean() {
        let mut mm = MetricMap::new();
        mm.record("latency", 10.0); mm.record("latency", 20.0);
        assert!((mm.mean("latency").unwrap() - 15.0).abs() < 1e-10);
    }

    #[test]
    fn percentile() {
        let mut mm = MetricMap::new();
        for i in 1..=100 { mm.record("req", i as f64); }
        assert!((mm.percentile("req", 50.0).unwrap() - 50.0).abs() < 2.0);
    }

    #[test]
    fn count() {
        let mut mm = MetricMap::new();
        mm.record("x", 1.0); mm.record("x", 2.0);
        assert_eq!(mm.count("x"), 2);
    }

    #[test]
    fn sum() {
        let mut mm = MetricMap::new();
        mm.record("bytes", 100.0); mm.record("bytes", 200.0);
        assert!((mm.sum("bytes") - 300.0).abs() < 1e-10);
    }

    #[test]
    fn tags() {
        let mut mm = MetricMap::new();
        mm.tag_record("http", "GET", 1.0);
        mm.tag_record("http", "POST", 2.0);
        assert_eq!(mm.count("http{GET}"), 1);
    }

    #[test]
    fn tag_aggregate() {
        let mut mm = MetricMap::new();
        mm.tag_record("cpu", "host1", 50.0);
        mm.tag_record("cpu", "host2", 70.0);
        let agg = mm.tag_aggregate("cpu");
        assert_eq!(agg.len(), 2);
    }

    #[test]
    fn missing_metric() { assert!(MetricMap::new().mean("x").is_none()); }

    #[test]
    fn stats() {
        let mut mm = MetricMap::new();
        mm.record("x", 1.0); mm.mean("x");
        assert_eq!(mm.total_records(), 1);
        assert_eq!(mm.total_queries(), 1);
    }
}
