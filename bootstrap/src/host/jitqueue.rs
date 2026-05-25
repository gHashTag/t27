use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum JitErr {
    NotFound { id: u64 },
    AlreadyScheduled { id: u64 },
    Expired { id: u64, deadline: u64 },
}

impl std::fmt::Display for JitErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitErr::NotFound { id } => write!(f, "item {id} not found"),
            JitErr::AlreadyScheduled { id } => write!(f, "item {id} already scheduled"),
            JitErr::Expired { id, deadline } => write!(f, "item {id} expired at {deadline}"),
        }
    }
}

impl std::error::Error for JitErr {}

#[derive(Clone)]
struct Item {
    id: u64,
    priority: i64,
    deadline: u64,
    payload: Vec<u8>,
    ready_at: u64,
}

impl Eq for Item {}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).then(other.deadline.cmp(&self.deadline))
    }
}

pub struct JitQueue {
    heap: BinaryHeap<Item>,
    deferred: Vec<Item>,
    now: u64,
    total_scheduled: u64,
    total_dispatched: u64,
    total_expired: u64,
}

impl JitQueue {
    pub fn new(now: u64) -> Self { Self { heap: BinaryHeap::new(), deferred: Vec::new(), now, total_scheduled: 0, total_dispatched: 0, total_expired: 0 } }

    pub fn schedule(&mut self, id: u64, priority: i64, deadline: u64, payload: Vec<u8>, ready_at: u64) -> Result<(), JitErr> {
        if self.heap.iter().any(|i| i.id == id) || self.deferred.iter().any(|i| i.id == id) {
            return Err(JitErr::AlreadyScheduled { id });
        }
        self.total_scheduled += 1;
        let item = Item { id, priority, deadline, payload, ready_at };
        if ready_at <= self.now { self.heap.push(item); } else { self.deferred.push(item); }
        Ok(())
    }

    pub fn advance(&mut self, now: u64) -> usize {
        self.now = now;
        let mut activated = 0usize;
        let remaining: Vec<Item> = self.deferred.drain(..).filter(|i| {
            if i.ready_at <= self.now { self.heap.push(i.clone()); activated += 1; false } else { true }
        }).collect();
        self.deferred = remaining;
        activated
    }

    pub fn dispatch(&mut self) -> Option<(u64, Vec<u8>)> {
        loop {
            let item = self.heap.pop()?;
            if item.deadline < self.now {
                self.total_expired += 1;
                continue;
            }
            self.total_dispatched += 1;
            return Some((item.id, item.payload));
        }
    }

    pub fn cancel(&mut self, id: u64) -> Result<Vec<u8>, JitErr> {
        if let Some(idx) = self.deferred.iter().position(|i| i.id == id) {
            return Ok(self.deferred.remove(idx).payload);
        }
        let items: Vec<Item> = self.heap.drain().collect();
        let mut found = None;
        let mut rest = BinaryHeap::new();
        for i in items {
            if i.id == id { found = Some(i.payload); } else { rest.push(i); }
        }
        self.heap = rest;
        found.ok_or(JitErr::NotFound { id })
    }

    pub fn pending_count(&self) -> usize { self.heap.len() }
    pub fn deferred_count(&self) -> usize { self.deferred.len() }
    pub fn now(&self) -> u64 { self.now }
    pub fn total_scheduled(&self) -> u64 { self.total_scheduled }
    pub fn total_dispatched(&self) -> u64 { self.total_dispatched }
    pub fn total_expired(&self) -> u64 { self.total_expired }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_dispatch() {
        let mut q = JitQueue::new(0);
        q.schedule(1, 10, 100, b"a".to_vec(), 0).unwrap();
        let (id, p) = q.dispatch().unwrap();
        assert_eq!(id, 1);
        assert_eq!(p, b"a");
    }

    #[test]
    fn priority_order() {
        let mut q = JitQueue::new(0);
        q.schedule(1, 5, 100, b"low".to_vec(), 0).unwrap();
        q.schedule(2, 10, 100, b"high".to_vec(), 0).unwrap();
        assert_eq!(q.dispatch().unwrap().0, 2);
        assert_eq!(q.dispatch().unwrap().0, 1);
    }

    #[test]
    fn deferred_activate() {
        let mut q = JitQueue::new(0);
        q.schedule(1, 10, 100, b"x".to_vec(), 50).unwrap();
        assert_eq!(q.deferred_count(), 1);
        q.advance(50);
        assert_eq!(q.pending_count(), 1);
        assert_eq!(q.dispatch().unwrap().0, 1);
    }

    #[test]
    fn expiry() {
        let mut q = JitQueue::new(0);
        q.schedule(1, 10, 5, b"old".to_vec(), 0).unwrap();
        q.schedule(2, 10, 100, b"new".to_vec(), 0).unwrap();
        q.advance(10);
        assert_eq!(q.dispatch().unwrap().0, 2);
        assert!(q.dispatch().is_none());
        assert_eq!(q.total_expired(), 1);
    }

    #[test]
    fn cancel() {
        let mut q = JitQueue::new(0);
        q.schedule(1, 10, 100, b"a".to_vec(), 0).unwrap();
        let p = q.cancel(1).unwrap();
        assert_eq!(p, b"a");
        assert!(q.dispatch().is_none());
    }

    #[test]
    fn cancel_not_found() { assert!(JitQueue::new(0).cancel(1).is_err()); }

    #[test]
    fn duplicate() {
        let mut q = JitQueue::new(0);
        q.schedule(1, 10, 100, vec![], 0).unwrap();
        assert!(q.schedule(1, 10, 100, vec![], 0).is_err());
    }

    #[test]
    fn stats() {
        let mut q = JitQueue::new(0);
        q.schedule(1, 10, 100, vec![], 0).unwrap();
        q.dispatch();
        assert_eq!(q.total_scheduled(), 1);
        assert_eq!(q.total_dispatched(), 1);
    }

    #[test]
    fn error_display() { assert!(JitErr::Expired { id: 1, deadline: 5 }.to_string().contains("expired")); }
}
