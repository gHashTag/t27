use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

impl std::fmt::Display for MetricKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricKind::Counter => write!(f, "counter"),
            MetricKind::Gauge => write!(f, "gauge"),
            MetricKind::Histogram => write!(f, "histogram"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricEntry {
    pub name: String,
    pub kind: MetricKind,
    pub value: f64,
    pub count: u64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

impl MetricEntry {
    pub fn new(name: &str, kind: MetricKind) -> Self {
        Self {
            name: name.to_string(),
            kind,
            value: 0.0,
            count: 0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            sum: 0.0,
        }
    }

    pub fn observe(&mut self, v: f64) {
        self.value = v;
        self.count += 1;
        self.sum += v;
        if v < self.min {
            self.min = v;
        }
        if v > self.max {
            self.max = v;
        }
    }

    pub fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricError {
    Duplicate { name: String },
    NotFound { name: String },
    KindMismatch { name: String, expected: MetricKind, got: MetricKind },
}

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricError::Duplicate { name } => write!(f, "duplicate metric: {name}"),
            MetricError::NotFound { name } => write!(f, "metric not found: {name}"),
            MetricError::KindMismatch { name, expected, got } => {
                write!(f, "{name}: expected {expected}, got {got}")
            }
        }
    }
}

impl std::error::Error for MetricError {}

#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    pub name: String,
    pub kind: MetricKind,
    pub value: f64,
    pub count: u64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
}

impl From<&MetricEntry> for MetricSnapshot {
    fn from(e: &MetricEntry) -> Self {
        Self {
            name: e.name.clone(),
            kind: e.kind,
            value: e.value,
            count: e.count,
            average: e.average(),
            min: if e.count == 0 { 0.0 } else { e.min },
            max: if e.count == 0 { 0.0 } else { e.max },
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricSink {
    entries: BTreeMap<String, MetricEntry>,
}

impl MetricSink {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, kind: MetricKind) -> Result<(), MetricError> {
        if self.entries.contains_key(name) {
            return Err(MetricError::Duplicate { name: name.to_string() });
        }
        self.entries.insert(name.to_string(), MetricEntry::new(name, kind));
        Ok(())
    }

    pub fn increment(&mut self, name: &str, delta: f64) -> Result<(), MetricError> {
        let entry = self.entries.get_mut(name).ok_or_else(|| MetricError::NotFound { name: name.to_string() })?;
        if entry.kind != MetricKind::Counter {
            return Err(MetricError::KindMismatch {
                name: name.to_string(),
                expected: MetricKind::Counter,
                got: entry.kind,
            });
        }
        entry.observe(entry.sum + delta);
        Ok(())
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) -> Result<(), MetricError> {
        let entry = self.entries.get_mut(name).ok_or_else(|| MetricError::NotFound { name: name.to_string() })?;
        if entry.kind != MetricKind::Gauge {
            return Err(MetricError::KindMismatch {
                name: name.to_string(),
                expected: MetricKind::Gauge,
                got: entry.kind,
            });
        }
        entry.observe(value);
        Ok(())
    }

    pub fn observe(&mut self, name: &str, value: f64) -> Result<(), MetricError> {
        let entry = self.entries.get_mut(name).ok_or_else(|| MetricError::NotFound { name: name.to_string() })?;
        if entry.kind != MetricKind::Histogram {
            return Err(MetricError::KindMismatch {
                name: name.to_string(),
                expected: MetricKind::Histogram,
                got: entry.kind,
            });
        }
        entry.observe(value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&MetricEntry> {
        self.entries.get(name)
    }

    pub fn snapshot(&self, name: &str) -> Option<MetricSnapshot> {
        self.entries.get(name).map(MetricSnapshot::from)
    }

    pub fn snapshot_all(&self) -> Vec<MetricSnapshot> {
        self.entries.values().map(MetricSnapshot::from).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn names(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for MetricSink {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_display() {
        assert_eq!(MetricKind::Counter.to_string(), "counter");
        assert_eq!(MetricKind::Gauge.to_string(), "gauge");
    }

    #[test]
    fn register_and_get() {
        let mut sink = MetricSink::new();
        sink.register("packets", MetricKind::Counter).unwrap();
        assert!(sink.get("packets").is_some());
    }

    #[test]
    fn register_duplicate() {
        let mut sink = MetricSink::new();
        sink.register("x", MetricKind::Counter).unwrap();
        let err = sink.register("x", MetricKind::Gauge).unwrap_err();
        assert!(matches!(err, MetricError::Duplicate { .. }));
    }

    #[test]
    fn increment_counter() {
        let mut sink = MetricSink::new();
        sink.register("cnt", MetricKind::Counter).unwrap();
        sink.increment("cnt", 1.0).unwrap();
        sink.increment("cnt", 5.0).unwrap();
        let e = sink.get("cnt").unwrap();
        assert_eq!(e.count, 2);
    }

    #[test]
    fn increment_wrong_kind() {
        let mut sink = MetricSink::new();
        sink.register("g", MetricKind::Gauge).unwrap();
        let err = sink.increment("g", 1.0).unwrap_err();
        assert!(matches!(err, MetricError::KindMismatch { .. }));
    }

    #[test]
    fn set_gauge() {
        let mut sink = MetricSink::new();
        sink.register("temp", MetricKind::Gauge).unwrap();
        sink.set_gauge("temp", 42.5).unwrap();
        sink.set_gauge("temp", 43.0).unwrap();
        let e = sink.get("temp").unwrap();
        assert_eq!(e.count, 2);
        assert!((e.value - 43.0).abs() < 0.01);
    }

    #[test]
    fn observe_histogram() {
        let mut sink = MetricSink::new();
        sink.register("lat", MetricKind::Histogram).unwrap();
        sink.observe("lat", 1.0).unwrap();
        sink.observe("lat", 3.0).unwrap();
        sink.observe("lat", 5.0).unwrap();
        let e = sink.get("lat").unwrap();
        assert_eq!(e.count, 3);
        assert!((e.min - 1.0).abs() < 0.01);
        assert!((e.max - 5.0).abs() < 0.01);
        assert!((e.average() - 3.0).abs() < 0.01);
    }

    #[test]
    fn snapshot_single() {
        let mut sink = MetricSink::new();
        sink.register("cnt", MetricKind::Counter).unwrap();
        sink.increment("cnt", 1.0).unwrap();
        let snap = sink.snapshot("cnt").unwrap();
        assert_eq!(snap.count, 1);
    }

    #[test]
    fn snapshot_all() {
        let mut sink = MetricSink::new();
        sink.register("a", MetricKind::Counter).unwrap();
        sink.register("b", MetricKind::Gauge).unwrap();
        assert_eq!(sink.snapshot_all().len(), 2);
    }

    #[test]
    fn not_found() {
        let mut sink = MetricSink::new();
        let err = sink.increment("missing", 1.0).unwrap_err();
        assert!(matches!(err, MetricError::NotFound { .. }));
    }

    #[test]
    fn names_sorted() {
        let mut sink = MetricSink::new();
        sink.register("bravo", MetricKind::Counter).unwrap();
        sink.register("alpha", MetricKind::Gauge).unwrap();
        assert_eq!(sink.names(), vec!["alpha", "bravo"]);
    }

    #[test]
    fn error_display() {
        assert!(MetricError::Duplicate { name: "x".into() }.to_string().contains("x"));
        assert!(MetricError::KindMismatch { name: "x".into(), expected: MetricKind::Counter, got: MetricKind::Gauge }.to_string().contains("expected"));
    }
}
