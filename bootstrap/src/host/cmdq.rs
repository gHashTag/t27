#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError {
    Full,
    Empty,
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::Full => write!(f, "queue full"),
            QueueError::Empty => write!(f, "queue empty"),
        }
    }
}

impl std::error::Error for QueueError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedCommand {
    pub id: u64,
    pub kind: CmdKind,
    pub priority: Priority,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdKind {
    Configure,
    LoadWeights,
    RunInference,
    ReadStatus,
    SelfTest,
    Reset,
}

#[derive(Debug, Clone)]
struct Slot {
    cmd: Option<QueuedCommand>,
    priority: Priority,
}

pub const DEFAULT_CAPACITY: usize = 32;

#[derive(Debug, Clone)]
pub struct CommandQueue {
    slots: Vec<Slot>,
    capacity: usize,
    next_id: u64,
    total_enqueued: u64,
    total_dequeued: u64,
}

impl CommandQueue {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            slots: Vec::with_capacity(capacity),
            capacity,
            next_id: 0,
            total_enqueued: 0,
            total_dequeued: 0,
        }
    }

    pub fn enqueue(&mut self, kind: CmdKind, priority: Priority, payload: Vec<u8>) -> Result<u64, QueueError> {
        if self.slots.len() >= self.capacity {
            return Err(QueueError::Full);
        }
        let id = self.next_id;
        self.next_id += 1;
        let cmd = QueuedCommand {
            id,
            kind,
            priority,
            payload,
        };
        self.slots.push(Slot {
            cmd: Some(cmd),
            priority,
        });
        self.total_enqueued += 1;
        Ok(id)
    }

    pub fn dequeue(&mut self) -> Result<QueuedCommand, QueueError> {
        if self.slots.is_empty() {
            return Err(QueueError::Empty);
        }
        let mut best_idx = 0;
        let mut best_priority = self.slots[0].priority;
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.priority > best_priority {
                best_idx = i;
                best_priority = slot.priority;
            }
        }
        let slot = self.slots.remove(best_idx);
        let cmd = slot.cmd.unwrap();
        self.total_dequeued += 1;
        Ok(cmd)
    }

    pub fn peek(&self) -> Result<&QueuedCommand, QueueError> {
        if self.slots.is_empty() {
            return Err(QueueError::Empty);
        }
        let mut best_idx = 0;
        let mut best_priority = self.slots[0].priority;
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.priority > best_priority {
                best_idx = i;
                best_priority = slot.priority;
            }
        }
        self.slots[best_idx].cmd.as_ref().ok_or(QueueError::Empty)
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.slots.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.slots.clear();
    }

    pub fn contains(&self, id: u64) -> bool {
        self.slots.iter().any(|s| s.cmd.as_ref().map_or(false, |c| c.id == id))
    }

    pub fn remove(&mut self, id: u64) -> Option<QueuedCommand> {
        let idx = self.slots.iter().position(|s| s.cmd.as_ref().map_or(false, |c| c.id == id))?;
        let slot = self.slots.remove(idx);
        self.total_dequeued += 1;
        slot.cmd
    }

    pub fn stats(&self) -> QueueStats {
        QueueStats {
            len: self.len(),
            capacity: self.capacity,
            total_enqueued: self.total_enqueued,
            total_dequeued: self.total_dequeued,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueStats {
    pub len: usize,
    pub capacity: usize,
    pub total_enqueued: u64,
    pub total_dequeued: u64,
}

impl QueueStats {
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.len as f64 / self.capacity as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let q = CommandQueue::new(8);
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
        assert_eq!(q.capacity(), 8);
    }

    #[test]
    fn enqueue_dequeue() {
        let mut q = CommandQueue::new(8);
        let id = q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        assert_eq!(id, 0);
        assert_eq!(q.len(), 1);
        let cmd = q.dequeue().unwrap();
        assert_eq!(cmd.id, 0);
        assert_eq!(cmd.kind, CmdKind::Reset);
        assert!(q.is_empty());
    }

    #[test]
    fn sequential_ids() {
        let mut q = CommandQueue::new(8);
        let id0 = q.enqueue(CmdKind::Configure, Priority::Low, vec![]).unwrap();
        let id1 = q.enqueue(CmdKind::RunInference, Priority::Normal, vec![]).unwrap();
        let id2 = q.enqueue(CmdKind::ReadStatus, Priority::High, vec![]).unwrap();
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn priority_ordering() {
        let mut q = CommandQueue::new(8);
        q.enqueue(CmdKind::Configure, Priority::Low, vec![]).unwrap();
        q.enqueue(CmdKind::RunInference, Priority::Normal, vec![]).unwrap();
        q.enqueue(CmdKind::SelfTest, Priority::Critical, vec![]).unwrap();
        q.enqueue(CmdKind::ReadStatus, Priority::High, vec![]).unwrap();
        let first = q.dequeue().unwrap();
        assert_eq!(first.kind, CmdKind::SelfTest);
        assert_eq!(first.priority, Priority::Critical);
        let second = q.dequeue().unwrap();
        assert_eq!(second.priority, Priority::High);
        let third = q.dequeue().unwrap();
        assert_eq!(third.priority, Priority::Normal);
        let fourth = q.dequeue().unwrap();
        assert_eq!(fourth.priority, Priority::Low);
    }

    #[test]
    fn full_queue_errors() {
        let mut q = CommandQueue::new(2);
        q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        assert!(q.is_full());
        assert_eq!(q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap_err(), QueueError::Full);
    }

    #[test]
    fn empty_dequeue_errors() {
        let mut q = CommandQueue::new(8);
        assert_eq!(q.dequeue().unwrap_err(), QueueError::Empty);
    }

    #[test]
    fn peek_does_not_remove() {
        let mut q = CommandQueue::new(8);
        q.enqueue(CmdKind::RunInference, Priority::High, vec![1, 2, 3]).unwrap();
        let cmd = q.peek().unwrap();
        assert_eq!(cmd.kind, CmdKind::RunInference);
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn peek_empty_errors() {
        let q = CommandQueue::new(8);
        assert_eq!(q.peek().unwrap_err(), QueueError::Empty);
    }

    #[test]
    fn payload_preserved() {
        let mut q = CommandQueue::new(8);
        q.enqueue(CmdKind::LoadWeights, Priority::Normal, vec![0xDE, 0xAD]).unwrap();
        let cmd = q.dequeue().unwrap();
        assert_eq!(cmd.payload, vec![0xDE, 0xAD]);
    }

    #[test]
    fn contains() {
        let mut q = CommandQueue::new(8);
        let id = q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        assert!(q.contains(id));
        assert!(!q.contains(99));
    }

    #[test]
    fn remove_by_id() {
        let mut q = CommandQueue::new(8);
        let id0 = q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        let id1 = q.enqueue(CmdKind::Configure, Priority::High, vec![]).unwrap();
        let removed = q.remove(id0).unwrap();
        assert_eq!(removed.id, id0);
        assert_eq!(q.len(), 1);
        let remaining = q.peek().unwrap();
        assert_eq!(remaining.id, id1);
    }

    #[test]
    fn remove_nonexistent() {
        let mut q = CommandQueue::new(8);
        assert!(q.remove(99).is_none());
    }

    #[test]
    fn clear() {
        let mut q = CommandQueue::new(8);
        q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        q.clear();
        assert!(q.is_empty());
    }

    #[test]
    fn stats() {
        let mut q = CommandQueue::new(4);
        let stats = q.stats();
        assert_eq!(stats.len, 0);
        assert_eq!(stats.capacity, 4);
        assert_eq!(stats.total_enqueued, 0);
        assert_eq!(stats.utilization(), 0.0);
        q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        q.enqueue(CmdKind::Reset, Priority::Normal, vec![]).unwrap();
        q.dequeue().unwrap();
        let stats = q.stats();
        assert_eq!(stats.total_enqueued, 2);
        assert_eq!(stats.total_dequeued, 1);
        assert!((stats.utilization() - 0.25).abs() < 0.001);
    }

    #[test]
    fn min_capacity_is_one() {
        let q = CommandQueue::new(0);
        assert_eq!(q.capacity(), 1);
    }

    #[test]
    fn priority_ordering_default_is_normal() {
        assert_eq!(Priority::default(), Priority::Normal);
    }

    #[test]
    fn error_display() {
        assert!(QueueError::Full.to_string().contains("full"));
        assert!(QueueError::Empty.to_string().contains("empty"));
    }
}
