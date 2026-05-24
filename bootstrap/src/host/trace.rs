use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TraceCategory {
    Init,
    Dma,
    Irq,
    Mmio,
    Pipeline,
    Error,
}

impl std::fmt::Display for TraceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceCategory::Init => write!(f, "init"),
            TraceCategory::Dma => write!(f, "dma"),
            TraceCategory::Irq => write!(f, "irq"),
            TraceCategory::Mmio => write!(f, "mmio"),
            TraceCategory::Pipeline => write!(f, "pipeline"),
            TraceCategory::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub category: TraceCategory,
    pub timestamp_us: u64,
    pub message: String,
    pub metadata: BTreeMap<String, String>,
}

impl TraceEvent {
    pub fn new(category: TraceCategory, timestamp_us: u64, message: &str) -> Self {
        Self {
            category,
            timestamp_us,
            message: message.to_string(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_meta(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct TraceCollector {
    events: Vec<TraceEvent>,
    max_events: usize,
    dropped: u64,
    filter: Option<Vec<TraceCategory>>,
}

impl TraceCollector {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_events.min(1024)),
            max_events,
            dropped: 0,
            filter: None,
        }
    }

    pub fn set_filter(&mut self, categories: Vec<TraceCategory>) {
        self.filter = Some(categories);
    }

    pub fn clear_filter(&mut self) {
        self.filter = None;
    }

    pub fn record(&mut self, event: TraceEvent) {
        if let Some(ref cats) = self.filter {
            if !cats.contains(&event.category) {
                return;
            }
        }
        if self.events.len() >= self.max_events {
            self.events.remove(0);
            self.dropped += 1;
        }
        self.events.push(event);
    }

    pub fn events(&self) -> &[TraceEvent] {
        &self.events
    }

    pub fn events_by_category(&self, cat: TraceCategory) -> Vec<&TraceEvent> {
        self.events.iter().filter(|e| e.category == cat).collect()
    }

    pub fn events_in_range(&self, start_us: u64, end_us: u64) -> Vec<&TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp_us >= start_us && e.timestamp_us <= end_us)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn category_counts(&self) -> BTreeMap<TraceCategory, usize> {
        let mut counts = BTreeMap::new();
        for e in &self.events {
            *counts.entry(e.category).or_insert(0) += 1;
        }
        counts
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.dropped = 0;
    }

    pub fn summary(&self) -> TraceSummary {
        let mut counts = BTreeMap::new();
        let mut min_ts = u64::MAX;
        let mut max_ts = 0u64;
        for e in &self.events {
            *counts.entry(e.category).or_insert(0usize) += 1;
            if e.timestamp_us < min_ts {
                min_ts = e.timestamp_us;
            }
            if e.timestamp_us > max_ts {
                max_ts = e.timestamp_us;
            }
        }
        TraceSummary {
            total: self.events.len(),
            dropped: self.dropped,
            category_counts: counts,
            span_us: if self.events.is_empty() { 0 } else { max_ts - min_ts },
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceSummary {
    pub total: usize,
    pub dropped: u64,
    pub category_counts: BTreeMap<TraceCategory, usize>,
    pub span_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_display() {
        assert_eq!(TraceCategory::Dma.to_string(), "dma");
        assert_eq!(TraceCategory::Pipeline.to_string(), "pipeline");
    }

    #[test]
    fn trace_event_with_meta() {
        let e = TraceEvent::new(TraceCategory::Irq, 100, "irq fired")
            .with_meta("line", "42");
        assert_eq!(e.metadata.get("line").unwrap(), "42");
    }

    #[test]
    fn record_and_len() {
        let mut tc = TraceCollector::new(100);
        tc.record(TraceEvent::new(TraceCategory::Init, 0, "boot"));
        tc.record(TraceEvent::new(TraceCategory::Dma, 10, "xfer"));
        assert_eq!(tc.len(), 2);
        assert!(!tc.is_empty());
    }

    #[test]
    fn events_by_category() {
        let mut tc = TraceCollector::new(100);
        tc.record(TraceEvent::new(TraceCategory::Init, 0, "a"));
        tc.record(TraceEvent::new(TraceCategory::Dma, 1, "b"));
        tc.record(TraceEvent::new(TraceCategory::Init, 2, "c"));
        assert_eq!(tc.events_by_category(TraceCategory::Init).len(), 2);
    }

    #[test]
    fn events_in_range() {
        let mut tc = TraceCollector::new(100);
        tc.record(TraceEvent::new(TraceCategory::Init, 10, "a"));
        tc.record(TraceEvent::new(TraceCategory::Dma, 20, "b"));
        tc.record(TraceEvent::new(TraceCategory::Irq, 30, "c"));
        let range = tc.events_in_range(15, 25);
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].message, "b");
    }

    #[test]
    fn max_events_evicts_oldest() {
        let mut tc = TraceCollector::new(2);
        tc.record(TraceEvent::new(TraceCategory::Init, 0, "first"));
        tc.record(TraceEvent::new(TraceCategory::Init, 1, "second"));
        tc.record(TraceEvent::new(TraceCategory::Init, 2, "third"));
        assert_eq!(tc.len(), 2);
        assert_eq!(tc.events()[0].message, "second");
        assert_eq!(tc.dropped(), 1);
    }

    #[test]
    fn filter_allows_only_matching() {
        let mut tc = TraceCollector::new(100);
        tc.set_filter(vec![TraceCategory::Dma]);
        tc.record(TraceEvent::new(TraceCategory::Init, 0, "a"));
        tc.record(TraceEvent::new(TraceCategory::Dma, 1, "b"));
        assert_eq!(tc.len(), 1);
        assert_eq!(tc.events()[0].message, "b");
    }

    #[test]
    fn clear_filter() {
        let mut tc = TraceCollector::new(100);
        tc.set_filter(vec![TraceCategory::Dma]);
        tc.clear_filter();
        tc.record(TraceEvent::new(TraceCategory::Init, 0, "a"));
        assert_eq!(tc.len(), 1);
    }

    #[test]
    fn category_counts() {
        let mut tc = TraceCollector::new(100);
        tc.record(TraceEvent::new(TraceCategory::Init, 0, "a"));
        tc.record(TraceEvent::new(TraceCategory::Init, 1, "b"));
        tc.record(TraceEvent::new(TraceCategory::Dma, 2, "c"));
        let counts = tc.category_counts();
        assert_eq!(*counts.get(&TraceCategory::Init).unwrap(), 2);
        assert_eq!(*counts.get(&TraceCategory::Dma).unwrap(), 1);
    }

    #[test]
    fn summary() {
        let mut tc = TraceCollector::new(100);
        tc.record(TraceEvent::new(TraceCategory::Init, 10, "a"));
        tc.record(TraceEvent::new(TraceCategory::Dma, 50, "b"));
        let s = tc.summary();
        assert_eq!(s.total, 2);
        assert_eq!(s.span_us, 40);
    }

    #[test]
    fn clear() {
        let mut tc = TraceCollector::new(100);
        tc.record(TraceEvent::new(TraceCategory::Init, 0, "a"));
        tc.clear();
        assert!(tc.is_empty());
        assert_eq!(tc.dropped(), 0);
    }
}
