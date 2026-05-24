use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DqError {
    QueueEmpty,
    ItemNotFound { id: u64 },
}

impl std::fmt::Display for DqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DqError::QueueEmpty => write!(f, "queue empty"),
            DqError::ItemNotFound { id } => write!(f, "item {id} not found"),
        }
    }
}

impl std::error::Error for DqError {}

struct Item {
    id: u64,
    value: i64,
    delta: i64,
    data: Vec<u8>,
}

pub struct DeltaQ {
    items: BTreeMap<u64, Item>,
    base_value: i64,
    next_id: u64,
    total_pushes: u64,
    total_pops: u64,
    total_updates: u64,
    total_merges: u64,
}

impl DeltaQ {
    pub fn new(base_value: i64) -> Self { Self { items: BTreeMap::new(), base_value, next_id: 1, total_pushes: 0, total_pops: 0, total_updates: 0, total_merges: 0 } }

    pub fn push(&mut self, value: i64, data: Vec<u8>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let delta = value - self.base_value;
        self.items.insert(id, Item { id, value, delta, data });
        self.total_pushes += 1;
        id
    }

    pub fn pop_min(&mut self) -> Option<(u64, i64, Vec<u8>)> {
        let min_id = self.items.values().min_by_key(|i| i.value)?.id;
        let item = self.items.remove(&min_id)?;
        self.base_value = item.value;
        self.total_pops += 1;
        Some((item.id, item.value, item.data))
    }

    pub fn update(&mut self, id: u64, new_value: i64) -> Result<i64, DqError> {
        let item = self.items.get_mut(&id).ok_or(DqError::ItemNotFound { id })?;
        let old_delta = item.delta;
        item.value = new_value;
        item.delta = new_value - self.base_value;
        self.total_updates += 1;
        Ok(old_delta)
    }

    pub fn delta(&self, id: u64) -> Option<i64> { self.items.get(&id).map(|i| i.delta) }

    pub fn value(&self, id: u64) -> Option<i64> { self.items.get(&id).map(|i| i.value) }

    pub fn merge(&mut self, other: &DeltaQ) -> u64 {
        let mut count = 0;
        for item in other.items.values() {
            let new_id = self.next_id;
            self.next_id += 1;
            self.items.insert(new_id, Item { id: new_id, value: item.value, delta: item.value - self.base_value, data: item.data.clone() });
            count += 1;
        }
        self.total_merges += 1;
        self.total_pushes += count;
        count
    }

    pub fn rebalance(&mut self) {
        self.base_value = self.items.values().map(|i| i.value).min().unwrap_or(0);
        for item in self.items.values_mut() { item.delta = item.value - self.base_value; }
    }

    pub fn peek_min(&self) -> Option<(u64, i64)> { self.items.values().min_by_key(|i| i.value).map(|i| (i.id, i.value)) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
    pub fn base_value(&self) -> i64 { self.base_value }
    pub fn total_pushes(&self) -> u64 { self.total_pushes }
    pub fn total_pops(&self) -> u64 { self.total_pops }
    pub fn total_updates(&self) -> u64 { self.total_updates }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() { let q = DeltaQ::new(0); assert!(q.is_empty()); }

    #[test]
    fn push_pop() {
        let mut q = DeltaQ::new(0);
        q.push(10, b"a".to_vec());
        q.push(5, b"b".to_vec());
        let (id, val, _) = q.pop_min().unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn delta_tracking() {
        let mut q = DeltaQ::new(0);
        let id = q.push(10, b"a".to_vec());
        assert_eq!(q.delta(id), Some(10));
    }

    #[test]
    fn update() {
        let mut q = DeltaQ::new(0);
        let id = q.push(10, b"a".to_vec());
        q.update(id, 5).unwrap();
        assert_eq!(q.value(id), Some(5));
        assert_eq!(q.delta(id), Some(5));
    }

    #[test]
    fn merge() {
        let mut q1 = DeltaQ::new(0);
        let mut q2 = DeltaQ::new(0);
        q1.push(10, b"a".to_vec());
        q2.push(5, b"b".to_vec());
        q1.merge(&q2);
        assert_eq!(q1.len(), 2);
        let (_, val, _) = q1.pop_min().unwrap();
        assert_eq!(val, 5);
    }

    #[test]
    fn rebalance() {
        let mut q = DeltaQ::new(100);
        q.push(110, b"a".to_vec());
        q.push(120, b"b".to_vec());
        q.rebalance();
        assert_eq!(q.base_value(), 110);
    }

    #[test]
    fn peek() {
        let mut q = DeltaQ::new(0);
        q.push(10, b"a".to_vec()); q.push(3, b"b".to_vec());
        assert_eq!(q.peek_min(), Some((2, 3)));
    }

    #[test]
    fn empty_pop() { assert!(DeltaQ::new(0).pop_min().is_none()); }

    #[test]
    fn stats() {
        let mut q = DeltaQ::new(0);
        q.push(1, vec![]);
        q.pop_min();
        assert_eq!(q.total_pushes(), 1);
        assert_eq!(q.total_pops(), 1);
    }

    #[test]
    fn error_display() { assert!(DqError::QueueEmpty.to_string().contains("empty")); }
}
