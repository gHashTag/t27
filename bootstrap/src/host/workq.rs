use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum WqError {
    QueueFull { capacity: usize },
    ItemNotFound { id: u64 },
    ItemAlreadyDone { id: u64 },
}

impl std::fmt::Display for WqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WqError::QueueFull { capacity } => write!(f, "queue full ({capacity})"),
            WqError::ItemNotFound { id } => write!(f, "item {id} not found"),
            WqError::ItemAlreadyDone { id } => write!(f, "item {id} already done"),
        }
    }
}

impl std::error::Error for WqError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemState { Pending, Running, Done }

struct WorkItem {
    id: u64,
    priority: i32,
    deadline: u64,
    payload: Vec<u8>,
    state: ItemState,
    enqueued_at: u64,
    started_at: Option<u64>,
    completed_at: Option<u64>,
}

pub struct WorkQ {
    items: BTreeMap<u64, WorkItem>,
    capacity: usize,
    next_id: u64,
    total_enqueued: u64,
    total_completed: u64,
    total_expired: u64,
}

impl WorkQ {
    pub fn new(capacity: usize) -> Self {
        Self { items: BTreeMap::new(), capacity, next_id: 1, total_enqueued: 0, total_completed: 0, total_expired: 0 }
    }

    pub fn enqueue(&mut self, priority: i32, deadline: u64, payload: Vec<u8>, now: u64) -> Result<u64, WqError> {
        if self.items.len() >= self.capacity { return Err(WqError::QueueFull { capacity: self.capacity }); }
        let id = self.next_id;
        self.next_id += 1;
        self.items.insert(id, WorkItem { id, priority, deadline, payload, state: ItemState::Pending, enqueued_at: now, started_at: None, completed_at: None });
        self.total_enqueued += 1;
        Ok(id)
    }

    pub fn dequeue(&mut self, now: u64) -> Option<(u64, Vec<u8>)> {
        let best_id = self.items.values()
            .filter(|i| i.state == ItemState::Pending)
            .max_by(|a, b| a.priority.cmp(&b.priority))
            .map(|i| i.id)?;
        let item = self.items.get_mut(&best_id).unwrap();
        item.state = ItemState::Running;
        item.started_at = Some(now);
        Some((item.id, item.payload.clone()))
    }

    pub fn complete(&mut self, id: u64, now: u64) -> Result<(), WqError> {
        let item = self.items.get_mut(&id).ok_or(WqError::ItemNotFound { id })?;
        if item.state == ItemState::Done { return Err(WqError::ItemAlreadyDone { id }); }
        item.state = ItemState::Done;
        item.completed_at = Some(now);
        self.total_completed += 1;
        Ok(())
    }

    pub fn expire(&mut self, now: u64) -> Vec<u64> {
        let expired: Vec<u64> = self.items.values()
            .filter(|i| i.state == ItemState::Pending && now > i.deadline)
            .map(|i| i.id)
            .collect();
        for &id in &expired {
            self.items.remove(&id);
            self.total_expired += 1;
        }
        expired
    }

    pub fn drain(&mut self) -> Vec<(u64, Vec<u8>)> {
        let pending: Vec<_> = self.items.values()
            .filter(|i| i.state == ItemState::Pending)
            .map(|i| (i.id, i.payload.clone()))
            .collect();
        for &(id, _) in &pending { self.items.remove(&id); }
        pending
    }

    pub fn state(&self, id: u64) -> Option<&ItemState> { self.items.get(&id).map(|i| &i.state) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
    pub fn pending_count(&self) -> usize { self.items.values().filter(|i| i.state == ItemState::Pending).count() }
    pub fn running_count(&self) -> usize { self.items.values().filter(|i| i.state == ItemState::Running).count() }
    pub fn total_enqueued(&self) -> u64 { self.total_enqueued }
    pub fn total_completed(&self) -> u64 { self.total_completed }
    pub fn total_expired(&self) -> u64 { self.total_expired }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() { assert!(WorkQ::new(10).is_empty()); }

    #[test]
    fn enqueue_dequeue() {
        let mut q = WorkQ::new(10);
        let id = q.enqueue(1, 100, b"work".to_vec(), 0).unwrap();
        let (did, data) = q.dequeue(0).unwrap();
        assert_eq!(did, id);
        assert_eq!(data, b"work");
        assert_eq!(q.state(id), Some(&ItemState::Running));
    }

    #[test]
    fn priority_order() {
        let mut q = WorkQ::new(10);
        q.enqueue(1, 100, b"low".to_vec(), 0).unwrap();
        q.enqueue(10, 100, b"high".to_vec(), 0).unwrap();
        let (_, data) = q.dequeue(0).unwrap();
        assert_eq!(data, b"high");
    }

    #[test]
    fn complete() {
        let mut q = WorkQ::new(10);
        let id = q.enqueue(1, 100, b"x".to_vec(), 0).unwrap();
        q.dequeue(0);
        q.complete(id, 5).unwrap();
        assert_eq!(q.state(id), Some(&ItemState::Done));
    }

    #[test]
    fn expire() {
        let mut q = WorkQ::new(10);
        q.enqueue(1, 10, b"x".to_vec(), 0).unwrap();
        let expired = q.expire(20);
        assert_eq!(expired.len(), 1);
        assert!(q.is_empty());
    }

    #[test]
    fn queue_full() {
        let mut q = WorkQ::new(2);
        q.enqueue(1, 100, vec![], 0).unwrap();
        q.enqueue(1, 100, vec![], 0).unwrap();
        let err = q.enqueue(1, 100, vec![], 0).unwrap_err();
        assert!(matches!(err, WqError::QueueFull { .. }));
    }

    #[test]
    fn drain() {
        let mut q = WorkQ::new(10);
        q.enqueue(1, 100, b"a".to_vec(), 0).unwrap();
        q.enqueue(1, 100, b"b".to_vec(), 0).unwrap();
        let drained = q.drain();
        assert_eq!(drained.len(), 2);
        assert!(q.is_empty());
    }

    #[test]
    fn double_complete() {
        let mut q = WorkQ::new(10);
        let id = q.enqueue(1, 100, vec![], 0).unwrap();
        q.dequeue(0);
        q.complete(id, 5).unwrap();
        let err = q.complete(id, 10).unwrap_err();
        assert!(matches!(err, WqError::ItemAlreadyDone { .. }));
    }

    #[test]
    fn not_found() {
        let mut q = WorkQ::new(10);
        let err = q.complete(99, 0).unwrap_err();
        assert!(matches!(err, WqError::ItemNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut q = WorkQ::new(10);
        q.enqueue(1, 100, vec![], 0).unwrap();
        q.dequeue(0).unwrap();
        q.complete(1, 5).unwrap();
        assert_eq!(q.total_enqueued(), 1);
        assert_eq!(q.total_completed(), 1);
    }

    #[test]
    fn error_display() { assert!(WqError::QueueFull { capacity: 3 }.to_string().contains("3")); }
}
