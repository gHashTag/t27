use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    U64(u64),
    I64(i64),
    F64(f64),
}

impl std::fmt::Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::U64(v) => write!(f, "{v}"),
            MetricValue::I64(v) => write!(f, "{v}"),
            MetricValue::F64(v) => write!(f, "{v:.3}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metric {
    pub name: String,
    pub kind: MetricKind,
    pub value: MetricValue,
    pub labels: BTreeMap<String, String>,
}

impl Metric {
    pub fn counter(name: &str, value: u64) -> Self {
        Self {
            name: name.to_string(),
            kind: MetricKind::Counter,
            value: MetricValue::U64(value),
            labels: BTreeMap::new(),
        }
    }

    pub fn gauge(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            kind: MetricKind::Gauge,
            value: MetricValue::F64(value),
            labels: BTreeMap::new(),
        }
    }

    pub fn with_label(mut self, key: &str, val: &str) -> Self {
        self.labels.insert(key.to_string(), val.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    pub timestamp_us: u64,
    pub metrics: Vec<Metric>,
}

impl TelemetrySnapshot {
    pub fn new(timestamp_us: u64) -> Self {
        Self {
            timestamp_us,
            metrics: Vec::new(),
        }
    }

    pub fn add(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }

    pub fn find(&self, name: &str) -> Option<&Metric> {
        self.metrics.iter().find(|m| m.name == name)
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }

    pub fn to_prometheus(&self) -> String {
        let mut out = String::new();
        for m in &self.metrics {
            let type_str = match m.kind {
                MetricKind::Counter => "counter",
                MetricKind::Gauge => "gauge",
                MetricKind::Histogram => "histogram",
            };
            let labels = if m.labels.is_empty() {
                String::new()
            } else {
                let pairs: Vec<String> = m
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{k}=\"{v}\""))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            };
            out.push_str(&format!("# TYPE {} {}\n", m.name, type_str));
            out.push_str(&format!("{}{} {}\n", m.name, labels, m.value));
        }
        out
    }

    pub fn to_json(&self) -> String {
        let entries: Vec<String> = self
            .metrics
            .iter()
            .map(|m| {
                let labels: Vec<String> = m
                    .labels
                    .iter()
                    .map(|(k, v)| format!("\"{k}\":\"{v}\""))
                    .collect();
                let label_str = if labels.is_empty() {
                    String::new()
                } else {
                    format!(",\"labels\":{{{}}}", labels.join(","))
                };
                format!(
                    "{{\"name\":\"{}\",\"value\":{}{}}}",
                    m.name,
                    m.value,
                    label_str
                )
            })
            .collect();
        format!(
            "{{\"ts\":{},\"metrics\":[{}]}}",
            self.timestamp_us,
            entries.join(",")
        )
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryCollector {
    counters: BTreeMap<String, u64>,
    gauges: BTreeMap<String, f64>,
    labels: BTreeMap<String, BTreeMap<String, String>>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            counters: BTreeMap::new(),
            gauges: BTreeMap::new(),
            labels: BTreeMap::new(),
        }
    }

    pub fn inc_counter(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn inc_counter_by(&mut self, name: &str, delta: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += delta;
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn set_label(&mut self, metric: &str, key: &str, value: &str) {
        self.labels
            .entry(metric.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    pub fn snapshot(&self, timestamp_us: u64) -> TelemetrySnapshot {
        let mut snap = TelemetrySnapshot::new(timestamp_us);
        for (name, &value) in &self.counters {
            let labels = self.labels.get(name).cloned().unwrap_or_default();
            snap.metrics.push(Metric {
                name: name.clone(),
                kind: MetricKind::Counter,
                value: MetricValue::U64(value),
                labels,
            });
        }
        for (name, &value) in &self.gauges {
            let labels = self.labels.get(name).cloned().unwrap_or_default();
            snap.metrics.push(Metric {
                name: name.clone(),
                kind: MetricKind::Gauge,
                value: MetricValue::F64(value),
                labels,
            });
        }
        snap
    }

    pub fn reset(&mut self) {
        self.counters.clear();
        self.gauges.clear();
        self.labels.clear();
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_counter() {
        let m = Metric::counter("requests", 42);
        assert_eq!(m.kind, MetricKind::Counter);
        assert_eq!(m.name, "requests");
    }

    #[test]
    fn metric_gauge() {
        let m = Metric::gauge("temperature", 72.5);
        assert_eq!(m.kind, MetricKind::Gauge);
    }

    #[test]
    fn metric_with_label() {
        let m = Metric::counter("requests", 1).with_label("method", "GET");
        assert_eq!(m.labels.get("method").unwrap(), "GET");
    }

    #[test]
    fn metric_value_display() {
        assert_eq!(MetricValue::U64(42).to_string(), "42");
        assert_eq!(MetricValue::F64(std::f64::consts::PI).to_string(), "3.142");
    }

    #[test]
    fn collector_inc_counter() {
        let mut c = TelemetryCollector::new();
        c.inc_counter("req");
        c.inc_counter("req");
        c.inc_counter_by("bytes", 100);
        let snap = c.snapshot(1000);
        let req = snap.find("req").unwrap();
        assert_eq!(req.value, MetricValue::U64(2));
        let bytes = snap.find("bytes").unwrap();
        assert_eq!(bytes.value, MetricValue::U64(100));
    }

    #[test]
    fn collector_set_gauge() {
        let mut c = TelemetryCollector::new();
        c.set_gauge("cpu", 0.75);
        let snap = c.snapshot(500);
        let cpu = snap.find("cpu").unwrap();
        assert_eq!(cpu.value, MetricValue::F64(0.75));
    }

    #[test]
    fn collector_labels() {
        let mut c = TelemetryCollector::new();
        c.set_label("req", "host", "localhost");
        c.inc_counter("req");
        let snap = c.snapshot(0);
        let req = snap.find("req").unwrap();
        assert_eq!(req.labels.get("host").unwrap(), "localhost");
    }

    #[test]
    fn snapshot_empty() {
        let snap = TelemetrySnapshot::new(0);
        assert!(snap.is_empty());
    }

    #[test]
    fn snapshot_find_missing() {
        let snap = TelemetrySnapshot::new(0);
        assert!(snap.find("nope").is_none());
    }

    #[test]
    fn to_prometheus() {
        let mut snap = TelemetrySnapshot::new(100);
        snap.add(Metric::counter("inferences", 10));
        let s = snap.to_prometheus();
        assert!(s.contains("# TYPE inferences counter"));
        assert!(s.contains("inferences 10"));
    }

    #[test]
    fn to_prometheus_with_labels() {
        let mut snap = TelemetrySnapshot::new(0);
        snap.add(Metric::counter("req", 5).with_label("path", "/api"));
        let s = snap.to_prometheus();
        assert!(s.contains("req{path=\"/api\"} 5"));
    }

    #[test]
    fn to_json() {
        let mut snap = TelemetrySnapshot::new(1000);
        snap.add(Metric::counter("ops", 3));
        let j = snap.to_json();
        assert!(j.contains("\"ts\":1000"));
        assert!(j.contains("\"name\":\"ops\""));
        assert!(j.contains("\"value\":3"));
    }

    #[test]
    fn collector_reset() {
        let mut c = TelemetryCollector::new();
        c.inc_counter("x");
        c.set_gauge("y", 1.0);
        c.reset();
        let snap = c.snapshot(0);
        assert!(snap.is_empty());
    }
}
