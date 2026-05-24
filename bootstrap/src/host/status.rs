use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Ok,
    Warning,
    Critical,
    Fatal,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Ok => write!(f, "ok"),
            Severity::Warning => write!(f, "warning"),
            Severity::Critical => write!(f, "critical"),
            Severity::Fatal => write!(f, "fatal"),
        }
    }
}

impl Severity {
    pub fn is_error(&self) -> bool {
        *self >= Severity::Critical
    }

    pub fn is_ok(&self) -> bool {
        *self == Severity::Ok
    }
}

#[derive(Debug, Clone)]
pub struct StatusEntry {
    pub source: String,
    pub severity: Severity,
    pub message: String,
    pub timestamp_us: u64,
}

impl StatusEntry {
    pub fn new(source: &str, severity: Severity, message: &str, timestamp_us: u64) -> Self {
        Self {
            source: source.to_string(),
            severity,
            message: message.to_string(),
            timestamp_us,
        }
    }

    pub fn ok(source: &str, timestamp_us: u64) -> Self {
        Self::new(source, Severity::Ok, "ok", timestamp_us)
    }
}

#[derive(Debug, Clone)]
pub struct StatusAggregator {
    entries: BTreeMap<String, StatusEntry>,
    worst: Severity,
}

impl StatusAggregator {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            worst: Severity::Ok,
        }
    }

    pub fn update(&mut self, entry: StatusEntry) {
        if entry.severity > self.worst {
            self.worst = entry.severity;
        }
        self.entries.insert(entry.source.clone(), entry);
        self.recompute_worst();
    }

    pub fn remove(&mut self, source: &str) -> bool {
        if self.entries.remove(source).is_some() {
            self.recompute_worst();
            true
        } else {
            false
        }
    }

    fn recompute_worst(&mut self) {
        self.worst = self
            .entries
            .values()
            .map(|e| e.severity)
            .max()
            .unwrap_or(Severity::Ok);
    }

    pub fn get(&self, source: &str) -> Option<&StatusEntry> {
        self.entries.get(source)
    }

    pub fn worst(&self) -> Severity {
        self.worst
    }

    pub fn is_healthy(&self) -> bool {
        self.worst == Severity::Ok
    }

    pub fn has_errors(&self) -> bool {
        self.worst.is_error()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn sources(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    pub fn by_severity(&self, sev: Severity) -> Vec<&StatusEntry> {
        self.entries.values().filter(|e| e.severity == sev).collect()
    }

    pub fn errors(&self) -> Vec<&StatusEntry> {
        self.entries.values().filter(|e| e.severity.is_error()).collect()
    }

    pub fn summary(&self) -> StatusSummary {
        let mut counts = [0usize; 4];
        for entry in self.entries.values() {
            match entry.severity {
                Severity::Ok => counts[0] += 1,
                Severity::Warning => counts[1] += 1,
                Severity::Critical => counts[2] += 1,
                Severity::Fatal => counts[3] += 1,
            }
        }
        StatusSummary {
            total: self.entries.len(),
            ok_count: counts[0],
            warning_count: counts[1],
            critical_count: counts[2],
            fatal_count: counts[3],
            worst: self.worst,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.worst = Severity::Ok;
    }
}

impl Default for StatusAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSummary {
    pub total: usize,
    pub ok_count: usize,
    pub warning_count: usize,
    pub critical_count: usize,
    pub fatal_count: usize,
    pub worst: Severity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Ok < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
        assert!(Severity::Critical < Severity::Fatal);
    }

    #[test]
    fn severity_predicates() {
        assert!(Severity::Ok.is_ok());
        assert!(!Severity::Ok.is_error());
        assert!(Severity::Fatal.is_error());
    }

    #[test]
    fn severity_display() {
        assert_eq!(Severity::Ok.to_string(), "ok");
        assert_eq!(Severity::Fatal.to_string(), "fatal");
    }

    #[test]
    fn status_entry_ok() {
        let e = StatusEntry::ok("clock", 100);
        assert_eq!(e.source, "clock");
        assert_eq!(e.severity, Severity::Ok);
    }

    #[test]
    fn update_single() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::ok("a", 0));
        assert_eq!(sa.worst(), Severity::Ok);
        assert!(sa.is_healthy());
    }

    #[test]
    fn update_escalates_worst() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::ok("a", 0));
        sa.update(StatusEntry::new("b", Severity::Warning, "slow", 1));
        assert_eq!(sa.worst(), Severity::Warning);
        assert!(!sa.is_healthy());
    }

    #[test]
    fn update_deescalates_on_remove() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::new("a", Severity::Fatal, "dead", 0));
        sa.update(StatusEntry::ok("b", 0));
        sa.remove("a");
        assert_eq!(sa.worst(), Severity::Ok);
    }

    #[test]
    fn get_entry() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::new("x", Severity::Critical, "overheat", 42));
        let e = sa.get("x").unwrap();
        assert_eq!(e.message, "overheat");
        assert_eq!(e.timestamp_us, 42);
    }

    #[test]
    fn remove_missing() {
        let mut sa = StatusAggregator::new();
        assert!(!sa.remove("nope"));
    }

    #[test]
    fn sources_sorted() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::ok("bravo", 0));
        sa.update(StatusEntry::ok("alpha", 0));
        assert_eq!(sa.sources(), vec!["alpha", "bravo"]);
    }

    #[test]
    fn by_severity() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::ok("a", 0));
        sa.update(StatusEntry::new("b", Severity::Warning, "w", 0));
        sa.update(StatusEntry::ok("c", 0));
        assert_eq!(sa.by_severity(Severity::Ok).len(), 2);
        assert_eq!(sa.by_severity(Severity::Warning).len(), 1);
    }

    #[test]
    fn errors_filters() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::ok("a", 0));
        sa.update(StatusEntry::new("b", Severity::Critical, "c", 0));
        sa.update(StatusEntry::new("d", Severity::Fatal, "f", 0));
        assert_eq!(sa.errors().len(), 2);
        assert!(sa.has_errors());
    }

    #[test]
    fn summary() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::ok("a", 0));
        sa.update(StatusEntry::new("b", Severity::Warning, "w", 0));
        sa.update(StatusEntry::new("c", Severity::Fatal, "f", 0));
        let s = sa.summary();
        assert_eq!(s.total, 3);
        assert_eq!(s.ok_count, 1);
        assert_eq!(s.warning_count, 1);
        assert_eq!(s.fatal_count, 1);
        assert_eq!(s.worst, Severity::Fatal);
    }

    #[test]
    fn clear() {
        let mut sa = StatusAggregator::new();
        sa.update(StatusEntry::new("a", Severity::Fatal, "x", 0));
        sa.clear();
        assert!(sa.is_empty());
        assert_eq!(sa.worst(), Severity::Ok);
    }

    #[test]
    fn len_and_empty() {
        let sa = StatusAggregator::new();
        assert!(sa.is_empty());
        assert_eq!(sa.len(), 0);
    }
}
