use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MetricType { Counter, Gauge, Histogram }

#[derive(Debug, Clone, PartialEq)]
pub enum MetricsError {
    MetricExists { name: String },
    MetricNotFound { name: String },
    TypeMismatch { name: String, expected: MetricType, found: MetricType },
}

impl std::fmt::Display for MetricsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricsError::MetricExists { name } => write!(f, "metric {name} exists"),
            MetricsError::MetricNotFound { name } => write!(f, "metric {name} not found"),
            MetricsError::TypeMismatch { name, expected, found } => write!(f, "{name}: expected {:?}, found {:?}", expected, found),
        }
    }
}

impl std::error::Error for MetricsError {}

enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram { samples: Vec<f64>, sum: f64, count: u64 },
}

struct Metric {
    name: String,
    mtype: MetricType,
    value: MetricValue,
    labels: Vec<(String, String)>,
}

pub struct MetricsRegistry {
    metrics: BTreeMap<String, Metric>,
    total_updates: u64,
    total_reads: u64,
}

impl MetricsRegistry {
    pub fn new() -> Self { Self { metrics: BTreeMap::new(), total_updates: 0, total_reads: 0 } }

    pub fn counter(&mut self, name: &str, labels: Vec<(String, String)>) -> Result<(), MetricsError> {
        if self.metrics.contains_key(name) { return Err(MetricsError::MetricExists { name: name.to_string() }); }
        self.metrics.insert(name.to_string(), Metric { name: name.to_string(), mtype: MetricType::Counter, value: MetricValue::Counter(0), labels });
        Ok(())
    }

    pub fn gauge(&mut self, name: &str, initial: f64, labels: Vec<(String, String)>) -> Result<(), MetricsError> {
        if self.metrics.contains_key(name) { return Err(MetricsError::MetricExists { name: name.to_string() }); }
        self.metrics.insert(name.to_string(), Metric { name: name.to_string(), mtype: MetricType::Gauge, value: MetricValue::Gauge(initial), labels });
        Ok(())
    }

    pub fn histogram(&mut self, name: &str, labels: Vec<(String, String)>) -> Result<(), MetricsError> {
        if self.metrics.contains_key(name) { return Err(MetricsError::MetricExists { name: name.to_string() }); }
        self.metrics.insert(name.to_string(), Metric { name: name.to_string(), mtype: MetricType::Histogram, value: MetricValue::Histogram { samples: Vec::new(), sum: 0.0, count: 0 }, labels });
        Ok(())
    }

    pub fn inc(&mut self, name: &str, delta: u64) -> Result<u64, MetricsError> {
        let m = self.metrics.get_mut(name).ok_or(MetricsError::MetricNotFound { name: name.to_string() })?;
        match &mut m.value {
            MetricValue::Counter(v) => { *v = v.saturating_add(delta); self.total_updates += 1; Ok(*v) }
            _ => Err(MetricsError::TypeMismatch { name: name.to_string(), expected: MetricType::Counter, found: m.mtype.clone() }),
        }
    }

    pub fn dec(&mut self, name: &str, delta: u64) -> Result<u64, MetricsError> {
        let m = self.metrics.get_mut(name).ok_or(MetricsError::MetricNotFound { name: name.to_string() })?;
        match &mut m.value {
            MetricValue::Counter(v) => { *v = v.saturating_sub(delta); self.total_updates += 1; Ok(*v) }
            _ => Err(MetricsError::TypeMismatch { name: name.to_string(), expected: MetricType::Counter, found: m.mtype.clone() }),
        }
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) -> Result<(), MetricsError> {
        let m = self.metrics.get_mut(name).ok_or(MetricsError::MetricNotFound { name: name.to_string() })?;
        match &mut m.value {
            MetricValue::Gauge(v) => { *v = value; self.total_updates += 1; Ok(()) }
            _ => Err(MetricsError::TypeMismatch { name: name.to_string(), expected: MetricType::Gauge, found: m.mtype.clone() }),
        }
    }

    pub fn observe(&mut self, name: &str, value: f64) -> Result<(), MetricsError> {
        let m = self.metrics.get_mut(name).ok_or(MetricsError::MetricNotFound { name: name.to_string() })?;
        match &mut m.value {
            MetricValue::Histogram { samples, sum, count } => {
                samples.push(value);
                *sum += value;
                *count += 1;
                self.total_updates += 1;
                Ok(())
            }
            _ => Err(MetricsError::TypeMismatch { name: name.to_string(), expected: MetricType::Histogram, found: m.mtype.clone() }),
        }
    }

    pub fn get_counter(&mut self, name: &str) -> Result<u64, MetricsError> {
        self.total_reads += 1;
        let m = self.metrics.get(name).ok_or(MetricsError::MetricNotFound { name: name.to_string() })?;
        match &m.value {
            MetricValue::Counter(v) => Ok(*v),
            _ => Err(MetricsError::TypeMismatch { name: name.to_string(), expected: MetricType::Counter, found: m.mtype.clone() }),
        }
    }

    pub fn get_gauge(&mut self, name: &str) -> Result<f64, MetricsError> {
        self.total_reads += 1;
        let m = self.metrics.get(name).ok_or(MetricsError::MetricNotFound { name: name.to_string() })?;
        match &m.value {
            MetricValue::Gauge(v) => Ok(*v),
            _ => Err(MetricsError::TypeMismatch { name: name.to_string(), expected: MetricType::Gauge, found: m.mtype.clone() }),
        }
    }

    pub fn percentile(&mut self, name: &str, p: f64) -> Result<f64, MetricsError> {
        self.total_reads += 1;
        let m = self.metrics.get(name).ok_or(MetricsError::MetricNotFound { name: name.to_string() })?;
        match &m.value {
            MetricValue::Histogram { samples, .. } => {
                if samples.is_empty() { return Ok(0.0); }
                let mut s = samples.clone();
                s.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let idx = ((p / 100.0) * (s.len() as f64 - 1.0)).round() as usize;
                Ok(s[idx.min(s.len() - 1)])
            }
            _ => Err(MetricsError::TypeMismatch { name: name.to_string(), expected: MetricType::Histogram, found: m.mtype.clone() }),
        }
    }

    pub fn metric_type(&self, name: &str) -> Option<&MetricType> { self.metrics.get(name).map(|m| &m.mtype) }
    pub fn metric_count(&self) -> usize { self.metrics.len() }
    pub fn total_updates(&self) -> u64 { self.total_updates }
    pub fn total_reads(&self) -> u64 { self.total_reads }
}

impl Default for MetricsRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry() { assert_eq!(MetricsRegistry::new().metric_count(), 0); }

    #[test]
    fn counter_inc_dec() {
        let mut mr = MetricsRegistry::new();
        mr.counter("requests", vec![]).unwrap();
        mr.inc("requests", 10).unwrap();
        mr.inc("requests", 5).unwrap();
        assert_eq!(mr.get_counter("requests"), Ok(15));
        mr.dec("requests", 3).unwrap();
        assert_eq!(mr.get_counter("requests"), Ok(12));
    }

    #[test]
    fn gauge_set() {
        let mut mr = MetricsRegistry::new();
        mr.gauge("temp", 20.0, vec![]).unwrap();
        mr.set_gauge("temp", 25.5).unwrap();
        assert_eq!(mr.get_gauge("temp"), Ok(25.5));
    }

    #[test]
    fn histogram_percentile() {
        let mut mr = MetricsRegistry::new();
        mr.histogram("latency", vec![]).unwrap();
        for i in 1..=100 { mr.observe("latency", i as f64).unwrap(); }
        let p50 = mr.percentile("latency", 50.0).unwrap();
        assert!(p50 > 40.0 && p50 < 60.0);
        let p99 = mr.percentile("latency", 99.0).unwrap();
        assert!(p99 >= 90.0);
    }

    #[test]
    fn type_mismatch() {
        let mut mr = MetricsRegistry::new();
        mr.counter("x", vec![]).unwrap();
        let err = mr.set_gauge("x", 1.0).unwrap_err();
        assert!(matches!(err, MetricsError::TypeMismatch { .. }));
    }

    #[test]
    fn duplicate() {
        let mut mr = MetricsRegistry::new();
        mr.counter("x", vec![]).unwrap();
        let err = mr.counter("x", vec![]).unwrap_err();
        assert!(matches!(err, MetricsError::MetricExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut mr = MetricsRegistry::new();
        let err = mr.inc("z", 1).unwrap_err();
        assert!(matches!(err, MetricsError::MetricNotFound { .. }));
    }

    #[test]
    fn metric_type() {
        let mut mr = MetricsRegistry::new();
        mr.counter("x", vec![]).unwrap();
        assert_eq!(mr.metric_type("x"), Some(&MetricType::Counter));
    }

    #[test]
    fn stats() {
        let mut mr = MetricsRegistry::new();
        mr.counter("x", vec![]).unwrap();
        mr.inc("x", 1).unwrap();
        mr.get_counter("x").unwrap();
        assert_eq!(mr.total_updates(), 1);
        assert_eq!(mr.total_reads(), 1);
    }

    #[test]
    fn error_display() { assert!(MetricsError::MetricNotFound { name: "x".into() }.to_string().contains("x")); }
}
