use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum Mq2Error {
    QueueNotFound { class: u8 },
    QueueFull { class: u8 },
}

impl std::fmt::Display for Mq2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mq2Error::QueueNotFound { class } => write!(f, "queue class {class} not found"),
            Mq2Error::QueueFull { class } => write!(f, "queue class {class} full"),
        }
    }
}

impl std::error::Error for Mq2Error {}

struct QClass {
    class: u8,
    weight: u8,
    queue: VecDeque<Vec<u8>>,
    capacity: usize,
    total_enqueued: u64,
    total_dequeued: u64,
    total_starved: u64,
}

pub struct MultiQueue {
    classes: BTreeMap<u8, QClass>,
    order: Vec<u8>,
    current: usize,
    credit: u8,
    total_enqueued: u64,
    total_dequeued: u64,
}

impl MultiQueue {
    pub fn new() -> Self { Self { classes: BTreeMap::new(), order: Vec::new(), current: 0, credit: 0, total_enqueued: 0, total_dequeued: 0 } }

    pub fn add_class(&mut self, class: u8, weight: u8, capacity: usize) {
        self.classes.insert(class, QClass { class, weight, queue: VecDeque::new(), capacity, total_enqueued: 0, total_dequeued: 0, total_starved: 0 });
        self.order.push(class);
        self.order.sort();
    }

    pub fn enqueue(&mut self, class: u8, data: Vec<u8>) -> Result<(), Mq2Error> {
        let qc = self.classes.get_mut(&class).ok_or(Mq2Error::QueueNotFound { class })?;
        if qc.queue.len() >= qc.capacity { return Err(Mq2Error::QueueFull { class }); }
        qc.queue.push_back(data);
        qc.total_enqueued += 1;
        self.total_enqueued += 1;
        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<(u8, Vec<u8>)> {
        if self.order.is_empty() { return None; }
        let attempts = self.order.len() * 16;
        for _ in 0..attempts {
            if self.current >= self.order.len() { self.current = 0; }
            let class = self.order[self.current];
            let qc = self.classes.get_mut(&class).unwrap();
            if self.credit < qc.weight {
                if let Some(data) = qc.queue.pop_front() {
                    qc.total_dequeued += 1;
                    self.total_dequeued += 1;
                    self.credit += 1;
                    return Some((class, data));
                }
            }
            self.credit = 0;
            self.current += 1;
        }
        for &class in &self.order {
            if let Some(qc) = self.classes.get_mut(&class) {
                if let Some(data) = qc.queue.pop_front() {
                    qc.total_dequeued += 1;
                    self.total_dequeued += 1;
                    return Some((class, data));
                }
            }
        }
        None
    }

    pub fn queue_len(&self, class: u8) -> Option<usize> { self.classes.get(&class).map(|q| q.queue.len()) }
    pub fn class_count(&self) -> usize { self.classes.len() }
    pub fn total_items(&self) -> usize { self.classes.values().map(|q| q.queue.len()).sum() }
    pub fn total_enqueued(&self) -> u64 { self.total_enqueued }
    pub fn total_dequeued(&self) -> u64 { self.total_dequeued }
}

impl Default for MultiQueue {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mq() { assert_eq!(MultiQueue::new().class_count(), 0); }

    #[test]
    fn enqueue_dequeue() {
        let mut mq = MultiQueue::new();
        mq.add_class(0, 1, 10);
        mq.enqueue(0, b"a".to_vec()).unwrap();
        let (class, data) = mq.dequeue().unwrap();
        assert_eq!(class, 0);
        assert_eq!(data, b"a");
    }

    #[test]
    fn weighted_priority() {
        let mut mq = MultiQueue::new();
        mq.add_class(0, 1, 10); mq.add_class(1, 3, 10);
        for i in 0..3 { mq.enqueue(0, vec![i]).unwrap(); }
        for i in 10..13 { mq.enqueue(1, vec![i]).unwrap(); }
        let mut high_count = 0;
        while let Some((c, _)) = mq.dequeue() { if c == 1 { high_count += 1; } }
        assert_eq!(high_count, 3);
    }

    #[test]
    fn queue_not_found() {
        let mut mq = MultiQueue::new();
        let err = mq.enqueue(5, b"x".to_vec()).unwrap_err();
        assert!(matches!(err, Mq2Error::QueueNotFound { .. }));
    }

    #[test]
    fn queue_full() {
        let mut mq = MultiQueue::new();
        mq.add_class(0, 1, 2);
        mq.enqueue(0, b"a".to_vec()).unwrap();
        mq.enqueue(0, b"b".to_vec()).unwrap();
        let err = mq.enqueue(0, b"c".to_vec()).unwrap_err();
        assert!(matches!(err, Mq2Error::QueueFull { .. }));
    }

    #[test]
    fn empty_dequeue() {
        let mut mq = MultiQueue::new();
        mq.add_class(0, 1, 10);
        assert!(mq.dequeue().is_none());
    }

    #[test]
    fn total_items() {
        let mut mq = MultiQueue::new();
        mq.add_class(0, 1, 10); mq.add_class(1, 1, 10);
        mq.enqueue(0, b"a".to_vec()).unwrap();
        mq.enqueue(1, b"b".to_vec()).unwrap();
        assert_eq!(mq.total_items(), 2);
    }

    #[test]
    fn queue_len() {
        let mut mq = MultiQueue::new();
        mq.add_class(0, 1, 10);
        mq.enqueue(0, b"a".to_vec()).unwrap();
        mq.enqueue(0, b"b".to_vec()).unwrap();
        assert_eq!(mq.queue_len(0), Some(2));
    }

    #[test]
    fn stats() {
        let mut mq = MultiQueue::new();
        mq.add_class(0, 1, 10);
        mq.enqueue(0, b"a".to_vec()).unwrap();
        mq.dequeue();
        assert_eq!(mq.total_enqueued(), 1);
        assert_eq!(mq.total_dequeued(), 1);
    }

    #[test]
    fn error_display() { assert!(Mq2Error::QueueNotFound { class: 1 }.to_string().contains("1")); }
}
