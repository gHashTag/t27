use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CmdPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmdError {
    QueueFull { capacity: usize },
    Empty,
    UnknownCmd { cmd_type: String },
}

impl std::fmt::Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdError::QueueFull { capacity } => write!(f, "queue full (cap {capacity})"),
            CmdError::Empty => write!(f, "queue empty"),
            CmdError::UnknownCmd { cmd_type } => write!(f, "unknown cmd: {cmd_type}"),
        }
    }
}

impl std::error::Error for CmdError {}

#[derive(Debug, Clone)]
pub struct Command {
    pub id: u64,
    pub cmd_type: String,
    pub payload: Vec<u8>,
    pub priority: CmdPriority,
}

#[derive(Debug, Clone, Default)]
pub struct DispatchStats {
    pub dispatched: BTreeMap<String, u64>,
    pub dropped: u64,
    pub batched: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub struct CmdDispatcher {
    queues: BTreeMap<CmdPriority, Vec<Command>>,
    capacity: usize,
    next_id: u64,
    stats: DispatchStats,
    handlers: Vec<String>,
}

impl CmdDispatcher {
    pub fn new(capacity: usize) -> Self {
        let mut queues = BTreeMap::new();
        queues.insert(CmdPriority::Critical, Vec::new());
        queues.insert(CmdPriority::High, Vec::new());
        queues.insert(CmdPriority::Normal, Vec::new());
        queues.insert(CmdPriority::Low, Vec::new());
        Self { queues, capacity, next_id: 1, stats: DispatchStats::default(), handlers: Vec::new() }
    }

    pub fn register_handler(&mut self, cmd_type: &str) {
        if !self.handlers.contains(&cmd_type.to_string()) {
            self.handlers.push(cmd_type.to_string());
        }
    }

    fn total_queued(&self) -> usize {
        self.queues.values().map(|q| q.len()).sum()
    }

    pub fn enqueue(&mut self, cmd_type: &str, payload: Vec<u8>, priority: CmdPriority) -> Result<u64, CmdError> {
        if !self.handlers.is_empty() && !self.handlers.contains(&cmd_type.to_string()) {
            return Err(CmdError::UnknownCmd { cmd_type: cmd_type.to_string() });
        }
        if self.total_queued() >= self.capacity {
            self.stats.dropped += 1;
            return Err(CmdError::QueueFull { capacity: self.capacity });
        }
        let id = self.next_id;
        self.next_id += 1;
        self.queues.get_mut(&priority).unwrap().push(Command { id, cmd_type: cmd_type.to_string(), payload, priority });
        self.stats.total += 1;
        Ok(id)
    }

    pub fn dispatch(&mut self) -> Option<Command> {
        for pri in [CmdPriority::Critical, CmdPriority::High, CmdPriority::Normal, CmdPriority::Low] {
            if let Some(queue) = self.queues.get_mut(&pri) {
                if let Some(cmd) = queue.pop() {
                    *self.stats.dispatched.entry(cmd.cmd_type.clone()).or_insert(0) += 1;
                    return Some(cmd);
                }
            }
        }
        None
    }

    pub fn dispatch_batch(&mut self, max: usize) -> Vec<Command> {
        let mut batch = Vec::new();
        while batch.len() < max {
            match self.dispatch() {
                Some(cmd) => batch.push(cmd),
                None => break,
            }
        }
        self.stats.batched += batch.len() as u64;
        batch
    }

    pub fn queued_count(&self) -> usize { self.total_queued() }
    pub fn stats(&self) -> &DispatchStats { &self.stats }
    pub fn handlers(&self) -> &[String] { &self.handlers }
    pub fn capacity(&self) -> usize { self.capacity }

    pub fn clear(&mut self) {
        for q in self.queues.values_mut() { q.clear(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dispatcher() {
        let d = CmdDispatcher::new(100);
        assert_eq!(d.queued_count(), 0);
        assert_eq!(d.capacity(), 100);
    }

    #[test]
    fn enqueue_dispatch() {
        let mut d = CmdDispatcher::new(10);
        d.register_handler("read");
        let id = d.enqueue("read", vec![1, 2, 3], CmdPriority::Normal).unwrap();
        assert_eq!(d.queued_count(), 1);
        let cmd = d.dispatch().unwrap();
        assert_eq!(cmd.id, id);
        assert_eq!(cmd.cmd_type, "read");
        assert_eq!(d.queued_count(), 0);
    }

    #[test]
    fn priority_order() {
        let mut d = CmdDispatcher::new(10);
        d.register_handler("a");
        d.enqueue("a", vec![], CmdPriority::Low).unwrap();
        d.enqueue("a", vec![], CmdPriority::Critical).unwrap();
        d.enqueue("a", vec![], CmdPriority::Normal).unwrap();
        let c1 = d.dispatch().unwrap();
        assert_eq!(c1.priority, CmdPriority::Critical);
        let c2 = d.dispatch().unwrap();
        assert_eq!(c2.priority, CmdPriority::Normal);
        let c3 = d.dispatch().unwrap();
        assert_eq!(c3.priority, CmdPriority::Low);
    }

    #[test]
    fn queue_full() {
        let mut d = CmdDispatcher::new(2);
        d.register_handler("x");
        d.enqueue("x", vec![], CmdPriority::Normal).unwrap();
        d.enqueue("x", vec![], CmdPriority::Normal).unwrap();
        let err = d.enqueue("x", vec![], CmdPriority::Normal).unwrap_err();
        assert!(matches!(err, CmdError::QueueFull { .. }));
    }

    #[test]
    fn unknown_cmd() {
        let mut d = CmdDispatcher::new(10);
        d.register_handler("a");
        let err = d.enqueue("b", vec![], CmdPriority::Normal).unwrap_err();
        assert!(matches!(err, CmdError::UnknownCmd { .. }));
    }

    #[test]
    fn dispatch_empty() {
        let mut d = CmdDispatcher::new(10);
        assert!(d.dispatch().is_none());
    }

    #[test]
    fn dispatch_batch() {
        let mut d = CmdDispatcher::new(10);
        d.register_handler("x");
        d.enqueue("x", vec![], CmdPriority::Normal).unwrap();
        d.enqueue("x", vec![], CmdPriority::Normal).unwrap();
        d.enqueue("x", vec![], CmdPriority::Normal).unwrap();
        let batch = d.dispatch_batch(2);
        assert_eq!(batch.len(), 2);
        assert_eq!(d.queued_count(), 1);
    }

    #[test]
    fn stats() {
        let mut d = CmdDispatcher::new(10);
        d.register_handler("read");
        d.register_handler("write");
        d.enqueue("read", vec![], CmdPriority::Normal).unwrap();
        d.enqueue("write", vec![], CmdPriority::Normal).unwrap();
        d.dispatch().unwrap();
        d.dispatch().unwrap();
        let s = d.stats();
        assert_eq!(*s.dispatched.get("read").unwrap(), 1);
        assert_eq!(*s.dispatched.get("write").unwrap(), 1);
    }

    #[test]
    fn clear() {
        let mut d = CmdDispatcher::new(10);
        d.register_handler("x");
        d.enqueue("x", vec![], CmdPriority::Normal).unwrap();
        d.clear();
        assert_eq!(d.queued_count(), 0);
    }

    #[test]
    fn open_handlers() {
        let mut d = CmdDispatcher::new(10);
        assert!(d.handlers().is_empty());
        d.enqueue("any", vec![], CmdPriority::Normal).unwrap();
    }

    #[test]
    fn dropped_count() {
        let mut d = CmdDispatcher::new(1);
        d.register_handler("x");
        d.enqueue("x", vec![], CmdPriority::Normal).unwrap();
        let _ = d.enqueue("x", vec![], CmdPriority::Normal);
        assert_eq!(d.stats().dropped, 1);
    }

    #[test]
    fn error_display() {
        assert!(CmdError::Empty.to_string().contains("empty"));
    }
}
