use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpStatus {
    Pending,
    Success,
    Failed,
    Timeout,
}

impl std::fmt::Display for OpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpStatus::Pending => write!(f, "pending"),
            OpStatus::Success => write!(f, "success"),
            OpStatus::Failed => write!(f, "failed"),
            OpStatus::Timeout => write!(f, "timeout"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpRecord {
    pub id: u64,
    pub name: String,
    pub start_us: u64,
    pub end_us: Option<u64>,
    pub status: OpStatus,
    pub tags: BTreeMap<String, String>,
}

impl OpRecord {
    pub fn new(id: u64, name: &str, start_us: u64) -> Self {
        Self {
            id,
            name: name.to_string(),
            start_us,
            end_us: None,
            status: OpStatus::Pending,
            tags: BTreeMap::new(),
        }
    }

    pub fn with_tag(mut self, key: &str, value: &str) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }

    pub fn complete(&mut self, end_us: u64) {
        self.end_us = Some(end_us);
        self.status = OpStatus::Success;
    }

    pub fn fail(&mut self, end_us: u64) {
        self.end_us = Some(end_us);
        self.status = OpStatus::Failed;
    }

    pub fn timeout(&mut self, end_us: u64) {
        self.end_us = Some(end_us);
        self.status = OpStatus::Timeout;
    }

    pub fn duration_us(&self) -> Option<u64> {
        self.end_us.map(|e| e.saturating_sub(self.start_us))
    }
}

pub const DEFAULT_MAX_OPS: usize = 128;

#[derive(Debug, Clone)]
pub struct OpLog {
    records: Vec<OpRecord>,
    max_ops: usize,
    next_id: u64,
    dropped: u64,
}

impl OpLog {
    pub fn new(max_ops: usize) -> Self {
        Self {
            records: Vec::with_capacity(max_ops),
            max_ops: max_ops.max(1),
            next_id: 0,
            dropped: 0,
        }
    }

    pub fn begin(&mut self, name: &str, start_us: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let record = OpRecord::new(id, name, start_us);
        if self.records.len() >= self.max_ops {
            self.records.remove(0);
            self.dropped += 1;
        }
        self.records.push(record);
        id
    }

    pub fn begin_with_tag(&mut self, name: &str, start_us: u64, key: &str, value: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let record = OpRecord::new(id, name, start_us).with_tag(key, value);
        if self.records.len() >= self.max_ops {
            self.records.remove(0);
            self.dropped += 1;
        }
        self.records.push(record);
        id
    }

    pub fn complete(&mut self, id: u64, end_us: u64) {
        if let Some(r) = self.records.iter_mut().find(|r| r.id == id) {
            r.complete(end_us);
        }
    }

    pub fn fail(&mut self, id: u64, end_us: u64) {
        if let Some(r) = self.records.iter_mut().find(|r| r.id == id) {
            r.fail(end_us);
        }
    }

    pub fn timeout(&mut self, id: u64, end_us: u64) {
        if let Some(r) = self.records.iter_mut().find(|r| r.id == id) {
            r.timeout(end_us);
        }
    }

    pub fn get(&self, id: u64) -> Option<&OpRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn records(&self) -> &[OpRecord] {
        &self.records
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn by_status(&self, status: OpStatus) -> Vec<&OpRecord> {
        self.records.iter().filter(|r| r.status == status).collect()
    }

    pub fn by_name(&self, name: &str) -> Vec<&OpRecord> {
        self.records.iter().filter(|r| r.name == name).collect()
    }

    pub fn success_count(&self) -> usize {
        self.records.iter().filter(|r| r.status == OpStatus::Success).count()
    }

    pub fn fail_count(&self) -> usize {
        self.records.iter().filter(|r| r.status == OpStatus::Failed).count()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.dropped = 0;
    }

    pub fn stats(&self) -> OpLogStats {
        OpLogStats {
            total: self.records.len(),
            pending: self.by_status(OpStatus::Pending).len(),
            success: self.success_count(),
            failed: self.fail_count(),
            timeout: self.by_status(OpStatus::Timeout).len(),
            dropped: self.dropped,
        }
    }
}

impl Default for OpLog {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_OPS)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpLogStats {
    pub total: usize,
    pub pending: usize,
    pub success: usize,
    pub failed: usize,
    pub timeout: usize,
    pub dropped: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_creates_pending() {
        let mut log = OpLog::new(16);
        let id = log.begin("inference", 100);
        let r = log.get(id).unwrap();
        assert_eq!(r.status, OpStatus::Pending);
        assert_eq!(r.name, "inference");
        assert!(r.duration_us().is_none());
    }

    #[test]
    fn complete_marks_success() {
        let mut log = OpLog::new(16);
        let id = log.begin("inference", 100);
        log.complete(id, 350);
        let r = log.get(id).unwrap();
        assert_eq!(r.status, OpStatus::Success);
        assert_eq!(r.duration_us(), Some(250));
    }

    #[test]
    fn fail_marks_failed() {
        let mut log = OpLog::new(16);
        let id = log.begin("inference", 100);
        log.fail(id, 200);
        let r = log.get(id).unwrap();
        assert_eq!(r.status, OpStatus::Failed);
    }

    #[test]
    fn timeout_marks_timeout() {
        let mut log = OpLog::new(16);
        let id = log.begin("inference", 100);
        log.timeout(id, 5000);
        assert_eq!(log.get(id).unwrap().status, OpStatus::Timeout);
    }

    #[test]
    fn with_tag() {
        let mut log = OpLog::new(16);
        let id = log.begin_with_tag("inference", 0, "layer", "3");
        let r = log.get(id).unwrap();
        assert_eq!(r.tags.get("layer").unwrap(), "3");
    }

    #[test]
    fn overflow_drops_oldest() {
        let mut log = OpLog::new(3);
        log.begin("a", 1);
        log.begin("b", 2);
        log.begin("c", 3);
        assert_eq!(log.dropped(), 0);
        log.begin("d", 4);
        assert_eq!(log.dropped(), 1);
        assert_eq!(log.len(), 3);
        assert!(log.get(0).is_none());
        assert!(log.get(3).is_some());
    }

    #[test]
    fn by_status_filter() {
        let mut log = OpLog::new(16);
        let id0 = log.begin("a", 0);
        let id1 = log.begin("b", 0);
        log.complete(id0, 10);
        log.fail(id1, 20);
        assert_eq!(log.by_status(OpStatus::Success).len(), 1);
        assert_eq!(log.by_status(OpStatus::Failed).len(), 1);
    }

    #[test]
    fn by_name_filter() {
        let mut log = OpLog::new(16);
        log.begin("infer", 0);
        log.begin("infer", 0);
        log.begin("load", 0);
        assert_eq!(log.by_name("infer").len(), 2);
    }

    #[test]
    fn success_fail_counts() {
        let mut log = OpLog::new(16);
        let a = log.begin("a", 0);
        let b = log.begin("b", 0);
        let c = log.begin("c", 0);
        log.complete(a, 10);
        log.complete(b, 20);
        log.fail(c, 30);
        assert_eq!(log.success_count(), 2);
        assert_eq!(log.fail_count(), 1);
    }

    #[test]
    fn stats() {
        let mut log = OpLog::new(16);
        let a = log.begin("a", 0);
        log.begin("b", 0);
        log.complete(a, 10);
        let s = log.stats();
        assert_eq!(s.total, 2);
        assert_eq!(s.success, 1);
        assert_eq!(s.pending, 1);
    }

    #[test]
    fn clear() {
        let mut log = OpLog::new(16);
        log.begin("a", 0);
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.dropped(), 0);
    }

    #[test]
    fn status_display() {
        assert_eq!(OpStatus::Success.to_string(), "success");
        assert_eq!(OpStatus::Timeout.to_string(), "timeout");
    }

    #[test]
    fn default_capacity() {
        let log = OpLog::default();
        assert_eq!(log.max_ops, DEFAULT_MAX_OPS);
    }
}
