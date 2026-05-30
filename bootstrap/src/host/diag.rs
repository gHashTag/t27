use std::collections::BTreeMap;

pub const COUNTER_INFERENCE_COUNT: &str = "inference_count";
pub const COUNTER_INFERENCE_ERRORS: &str = "inference_errors";
pub const COUNTER_DMA_TRANSFERS: &str = "dma_transfers";
pub const COUNTER_DMA_BYTES: &str = "dma_bytes";
pub const COUNTER_WEIGHT_LOADS: &str = "weight_loads";
pub const COUNTER_RETRIES: &str = "retries";
pub const COUNTER_TIMEOUTS: &str = "timeouts";
pub const COUNTER_WATCHDOG_FEEDS: &str = "watchdog_feeds";

#[derive(Debug, Clone)]
pub struct DiagCounters {
    counters: BTreeMap<String, u64>,
}

impl DiagCounters {
    pub fn new() -> Self {
        Self {
            counters: BTreeMap::new(),
        }
    }

    pub fn inc(&mut self, name: &str) -> u64 {
        let v = self.counters.entry(name.to_string()).or_insert(0);
        *v += 1;
        *v
    }

    pub fn inc_by(&mut self, name: &str, delta: u64) -> u64 {
        let v = self.counters.entry(name.to_string()).or_insert(0);
        *v += delta;
        *v
    }

    pub fn dec(&mut self, name: &str) -> u64 {
        let v = self.counters.entry(name.to_string()).or_insert(0);
        *v = v.saturating_sub(1);
        *v
    }

    pub fn set(&mut self, name: &str, value: u64) {
        self.counters.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> u64 {
        *self.counters.get(name).unwrap_or(&0)
    }

    pub fn names(&self) -> Vec<&str> {
        self.counters.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.counters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.counters.is_empty()
    }

    pub fn snapshot(&self) -> Vec<(&str, u64)> {
        self.counters
            .iter()
            .map(|(k, &v)| (k.as_str(), v))
            .collect()
    }

    pub fn reset(&mut self) {
        for v in self.counters.values_mut() {
            *v = 0;
        }
    }

    pub fn clear(&mut self) {
        self.counters.clear();
    }

    pub fn summary(&self) -> String {
        let mut parts: Vec<String> = self
            .counters
            .iter()
            .map(|(k, &v)| format!("{k}={v}"))
            .collect();
        parts.sort();
        parts.join(", ")
    }
}

impl Default for DiagCounters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let d = DiagCounters::new();
        assert!(d.is_empty());
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn inc_from_zero() {
        let mut d = DiagCounters::new();
        assert_eq!(d.inc("foo"), 1);
        assert_eq!(d.inc("foo"), 2);
        assert_eq!(d.get("foo"), 2);
    }

    #[test]
    fn inc_by_delta() {
        let mut d = DiagCounters::new();
        assert_eq!(d.inc_by("bytes", 100), 100);
        assert_eq!(d.inc_by("bytes", 50), 150);
    }

    #[test]
    fn dec_saturating() {
        let mut d = DiagCounters::new();
        d.set("x", 2);
        assert_eq!(d.dec("x"), 1);
        assert_eq!(d.dec("x"), 0);
        assert_eq!(d.dec("x"), 0);
    }

    #[test]
    fn set_and_get() {
        let mut d = DiagCounters::new();
        d.set("val", 42);
        assert_eq!(d.get("val"), 42);
    }

    #[test]
    fn get_missing_is_zero() {
        let d = DiagCounters::new();
        assert_eq!(d.get("nonexistent"), 0);
    }

    #[test]
    fn names_sorted() {
        let mut d = DiagCounters::new();
        d.inc("charlie");
        d.inc("alpha");
        d.inc("bravo");
        assert_eq!(d.names(), vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn snapshot_sorted() {
        let mut d = DiagCounters::new();
        d.inc_by("b", 2);
        d.inc_by("a", 1);
        let snap = d.snapshot();
        assert_eq!(snap, vec![("a", 1), ("b", 2)]);
    }

    #[test]
    fn reset_zeros_all() {
        let mut d = DiagCounters::new();
        d.inc("x");
        d.inc("y");
        d.reset();
        assert_eq!(d.get("x"), 0);
        assert_eq!(d.get("y"), 0);
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn clear_removes_all() {
        let mut d = DiagCounters::new();
        d.inc("x");
        d.clear();
        assert!(d.is_empty());
    }

    #[test]
    fn predefined_counter_names() {
        let mut d = DiagCounters::new();
        d.inc(COUNTER_INFERENCE_COUNT);
        d.inc(COUNTER_DMA_BYTES);
        assert_eq!(d.get(COUNTER_INFERENCE_COUNT), 1);
        assert_eq!(d.get(COUNTER_DMA_BYTES), 1);
    }

    #[test]
    fn summary_format() {
        let mut d = DiagCounters::new();
        d.inc_by("alpha", 10);
        d.inc_by("beta", 20);
        let s = d.summary();
        assert!(s.contains("alpha=10"));
        assert!(s.contains("beta=20"));
    }

    #[test]
    fn default_trait() {
        let d = DiagCounters::default();
        assert!(d.is_empty());
    }
}
