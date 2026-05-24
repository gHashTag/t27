use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(HistogramSnapshot),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistogramSnapshot {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub p50: f64,
    pub p99: f64,
}

#[derive(Debug, Clone)]
struct HistogramState {
    values: Vec<f64>,
    sorted: bool,
}

impl HistogramState {
    fn new() -> Self { Self { values: Vec::new(), sorted: false } }

    fn record(&mut self, v: f64) {
        self.values.push(v);
        self.sorted = false;
    }

    fn snapshot(&mut self) -> HistogramSnapshot {
        if !self.sorted {
            self.values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            self.sorted = true;
        }
        let count = self.values.len() as u64;
        if count == 0 {
            return HistogramSnapshot { count: 0, sum: 0.0, min: 0.0, max: 0.0, avg: 0.0, p50: 0.0, p99: 0.0 };
        }
        let sum: f64 = self.values.iter().copied().sum();
        let p50 = self.percentile(0.50);
        let p99 = self.percentile(0.99);
        HistogramSnapshot { count, sum, min: self.values[0], max: self.values[count as usize - 1], avg: sum / count as f64, p50, p99 }
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.values.is_empty() { return 0.0; }
        let idx = ((p * self.values.len() as f64).ceil() as usize).saturating_sub(1);
        self.values[idx.min(self.values.len() - 1)]
    }
}

#[derive(Debug, Clone)]
struct MetricEntry {
    kind: MetricKind,
    counter: u64,
    gauge: f64,
    histogram: HistogramState,
}

#[derive(Debug, Clone)]
pub struct MetricSnapshot {
    pub name: String,
    pub kind: MetricKind,
    pub value: MetricValue,
}

#[derive(Debug, Clone)]
pub struct MetricSink {
    entries: BTreeMap<String, MetricEntry>,
}

impl MetricSink {
    pub fn new() -> Self { Self { entries: BTreeMap::new() } }

    fn ensure(&mut self, name: &str, kind: MetricKind) -> Result<(), String> {
        if let Some(e) = self.entries.get(name) {
            if e.kind != kind { return Err(format!("metric '{}' is {:?}, not {:?}", name, e.kind, kind)); }
        } else {
            self.entries.insert(name.to_string(), MetricEntry {
                kind,
                counter: 0,
                gauge: 0.0,
                histogram: HistogramState::new(),
            });
        }
        Ok(())
    }

    pub fn inc_counter(&mut self, name: &str, delta: u64) -> Result<u64, String> {
        self.ensure(name, MetricKind::Counter)?;
        let e = self.entries.get_mut(name).unwrap();
        e.counter = e.counter.saturating_add(delta);
        Ok(e.counter)
    }

    pub fn dec_counter(&mut self, name: &str, delta: u64) -> Result<u64, String> {
        self.ensure(name, MetricKind::Counter)?;
        let e = self.entries.get_mut(name).unwrap();
        e.counter = e.counter.saturating_sub(delta);
        Ok(e.counter)
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) -> Result<(), String> {
        self.ensure(name, MetricKind::Gauge)?;
        self.entries.get_mut(name).unwrap().gauge = value;
        Ok(())
    }

    pub fn observe(&mut self, name: &str, value: f64) -> Result<(), String> {
        self.ensure(name, MetricKind::Histogram)?;
        self.entries.get_mut(name).unwrap().histogram.record(value);
        Ok(())
    }

    pub fn snapshot(&mut self, name: &str) -> Option<MetricSnapshot> {
        let e = self.entries.get_mut(name)?;
        let value = match e.kind {
            MetricKind::Counter => MetricValue::Counter(e.counter),
            MetricKind::Gauge => MetricValue::Gauge(e.gauge),
            MetricKind::Histogram => MetricValue::Histogram(e.histogram.snapshot()),
        };
        Some(MetricSnapshot { name: name.to_string(), kind: e.kind, value })
    }

    pub fn snapshot_all(&mut self) -> Vec<MetricSnapshot> {
        let names: Vec<String> = self.entries.keys().cloned().collect();
        let mut result = Vec::new();
        for name in names {
            if let Some(s) = self.snapshot(&name) { result.push(s); }
        }
        result
    }

    pub fn metric_count(&self) -> usize { self.entries.len() }

    pub fn reset(&mut self, name: &str) -> bool {
        if let Some(e) = self.entries.get_mut(name) {
            e.counter = 0;
            e.gauge = 0.0;
            e.histogram = HistogramState::new();
            true
        } else { false }
    }

    pub fn clear(&mut self) { self.entries.clear(); }
}

impl Default for MetricSink {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sink() {
        let ms = MetricSink::new();
        assert_eq!(ms.metric_count(), 0);
    }

    #[test]
    fn counter_inc() {
        let mut ms = MetricSink::new();
        let v = ms.inc_counter("reqs", 5).unwrap();
        assert_eq!(v, 5);
        assert_eq!(ms.inc_counter("reqs", 3).unwrap(), 8);
    }

    #[test]
    fn counter_dec() {
        let mut ms = MetricSink::new();
        ms.inc_counter("errs", 10).unwrap();
        assert_eq!(ms.dec_counter("errs", 3).unwrap(), 7);
    }

    #[test]
    fn gauge() {
        let mut ms = MetricSink::new();
        ms.set_gauge("temp", 42.5).unwrap();
        let snap = ms.snapshot("temp").unwrap();
        assert_eq!(snap.value, MetricValue::Gauge(42.5));
    }

    #[test]
    fn histogram() {
        let mut ms = MetricSink::new();
        for v in [1.0, 2.0, 3.0, 4.0, 5.0] { ms.observe("lat", v).unwrap(); }
        let snap = ms.snapshot("lat").unwrap();
        if let MetricValue::Histogram(h) = snap.value {
            assert_eq!(h.count, 5);
            assert_eq!(h.min, 1.0);
            assert_eq!(h.max, 5.0);
        } else { panic!("expected histogram"); }
    }

    #[test]
    fn kind_mismatch() {
        let mut ms = MetricSink::new();
        ms.inc_counter("x", 1).unwrap();
        let err = ms.set_gauge("x", 1.0).unwrap_err();
        assert!(err.contains("not Gauge"));
    }

    #[test]
    fn snapshot_all() {
        let mut ms = MetricSink::new();
        ms.inc_counter("a", 1).unwrap();
        ms.set_gauge("b", 2.0).unwrap();
        let all = ms.snapshot_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn reset() {
        let mut ms = MetricSink::new();
        ms.inc_counter("x", 100).unwrap();
        ms.reset("x");
        let snap = ms.snapshot("x").unwrap();
        assert_eq!(snap.value, MetricValue::Counter(0));
    }

    #[test]
    fn snapshot_missing() {
        let mut ms = MetricSink::new();
        assert!(ms.snapshot("nope").is_none());
    }

    #[test]
    fn histogram_percentiles() {
        let mut ms = MetricSink::new();
        for i in 1..=100 { ms.observe("p", i as f64).unwrap(); }
        let snap = ms.snapshot("p").unwrap();
        if let MetricValue::Histogram(h) = snap.value {
            assert!(h.p50 >= 49.0 && h.p50 <= 51.0);
            assert!(h.p99 >= 98.0);
        } else { panic!("expected histogram"); }
    }

    #[test]
    fn clear() {
        let mut ms = MetricSink::new();
        ms.inc_counter("a", 1).unwrap();
        ms.clear();
        assert_eq!(ms.metric_count(), 0);
    }

    #[test]
    fn saturating_counter() {
        let mut ms = MetricSink::new();
        ms.inc_counter("x", u64::MAX).unwrap();
        assert_eq!(ms.inc_counter("x", 1).unwrap(), u64::MAX);
    }
}
