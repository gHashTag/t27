use std::collections::BTreeMap;

#[derive(Clone)]
struct Entry {
    value: Vec<u8>,
    timestamp: u64,
    deleted: bool,
}

pub struct CrdtMap {
    data: BTreeMap<Vec<u8>, Entry>,
    node_id: u64,
    clock: u64,
    total_puts: u64,
    total_deletes: u64,
    total_merges: u64,
}

impl CrdtMap {
    pub fn new(node_id: u64) -> Self { Self { data: BTreeMap::new(), node_id, clock: 0, total_puts: 0, total_deletes: 0, total_merges: 0 } }

    pub fn put(&mut self, key: &[u8], value: Vec<u8>) -> u64 {
        self.total_puts += 1;
        self.clock += 1;
        self.data.insert(key.to_vec(), Entry { value, timestamp: self.clock, deleted: false });
        self.clock
    }

    pub fn delete(&mut self, key: &[u8]) -> bool {
        self.total_deletes += 1;
        self.clock += 1;
        if let Some(e) = self.data.get_mut(key) {
            e.deleted = true;
            e.timestamp = self.clock;
            true
        } else { false }
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.data.get(key).and_then(|e| if e.deleted { None } else { Some(e.value.as_slice()) })
    }

    pub fn merge(&mut self, other: &CrdtMap) -> usize {
        self.total_merges += 1;
        let mut conflicts = 0usize;
        for (k, e) in &other.data {
            match self.data.get(k) {
                Some(my) => {
                    if e.timestamp > my.timestamp { self.data.insert(k.clone(), e.clone()); }
                    else if e.timestamp == my.timestamp && other.node_id > self.node_id {
                        conflicts += 1;
                        self.data.insert(k.clone(), e.clone());
                    }
                }
                None => { self.data.insert(k.clone(), e.clone()); }
            }
        }
        self.clock = self.clock.max(other.clock);
        conflicts
    }

    pub fn contains(&self, key: &[u8]) -> bool { self.get(key).is_some() }
    pub fn len(&self) -> usize { self.data.iter().filter(|(_, e)| !e.deleted).count() }
    pub fn is_empty(&self) -> bool { self.len() == 0 }
    pub fn clock(&self) -> u64 { self.clock }
    pub fn node_id(&self) -> u64 { self.node_id }
    pub fn total_puts(&self) -> u64 { self.total_puts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_merges(&self) -> u64 { self.total_merges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get() {
        let mut cm = CrdtMap::new(1);
        cm.put(b"k", b"v".to_vec());
        assert_eq!(cm.get(b"k"), Some(&b"v"[..]));
    }

    #[test]
    fn delete() {
        let mut cm = CrdtMap::new(1);
        cm.put(b"k", b"v".to_vec());
        cm.delete(b"k");
        assert!(cm.get(b"k").is_none());
    }

    #[test]
    fn merge_newer_wins() {
        let mut a = CrdtMap::new(1);
        let mut b = CrdtMap::new(2);
        a.put(b"k", b"old".to_vec());
        b.put(b"k", b"new".to_vec());
        a.merge(&b);
        assert_eq!(a.get(b"k"), Some(&b"new"[..]));
    }

    #[test]
    fn merge_both_add() {
        let mut a = CrdtMap::new(1);
        let mut b = CrdtMap::new(2);
        a.put(b"a", b"1".to_vec());
        b.put(b"b", b"2".to_vec());
        a.merge(&b);
        assert_eq!(a.get(b"a"), Some(&b"1"[..]));
        assert_eq!(a.get(b"b"), Some(&b"2"[..]));
    }

    #[test]
    fn merge_idempotent() {
        let mut a = CrdtMap::new(1);
        a.put(b"k", b"v".to_vec());
        let mut b = CrdtMap::new(2);
        b.merge(&a);
        b.merge(&a);
        assert_eq!(b.get(b"k"), Some(&b"v"[..]));
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn merge_delete_propagates() {
        let mut a = CrdtMap::new(1);
        a.put(b"k", b"v".to_vec());
        let mut b = CrdtMap::new(2);
        b.merge(&a);
        a.delete(b"k");
        b.merge(&a);
        assert!(b.get(b"k").is_none());
    }

    #[test]
    fn clock_advances() {
        let mut cm = CrdtMap::new(1);
        cm.put(b"a", vec![]);
        cm.put(b"b", vec![]);
        assert_eq!(cm.clock(), 2);
    }

    #[test]
    fn stats() {
        let mut cm = CrdtMap::new(1);
        cm.put(b"k", b"v".to_vec());
        cm.delete(b"k");
        let other = CrdtMap::new(2);
        cm.merge(&other);
        assert_eq!(cm.total_puts(), 1);
        assert_eq!(cm.total_deletes(), 1);
        assert_eq!(cm.total_merges(), 1);
    }
}
