use std::collections::BTreeMap;

pub struct RingTree {
    data: BTreeMap<u64, Vec<u8>>,
    head: u64,
    tail: u64,
    cap: usize,
    total_pushes: u64,
    total_pops: u64,
    total_lookups: u64,
}

impl RingTree {
    pub fn new(cap: usize) -> Self { Self { data: BTreeMap::new(), head: 0, tail: 0, cap, total_pushes: 0, total_pops: 0, total_lookups: 0 } }

    pub fn push(&mut self, value: Vec<u8>) {
        self.total_pushes += 1;
        if self.len() >= self.cap { self.pop(); }
        self.data.insert(self.tail, value);
        self.tail += 1;
    }

    pub fn pop(&mut self) -> Option<Vec<u8>> {
        self.total_pops += 1;
        if self.head >= self.tail { return None; }
        let v = self.data.remove(&self.head);
        self.head += 1;
        v
    }

    pub fn get(&mut self, index: u64) -> Option<&Vec<u8>> {
        self.total_lookups += 1;
        self.data.get(&(self.head + index))
    }

    pub fn get_abs(&mut self, seq: u64) -> Option<&Vec<u8>> {
        self.total_lookups += 1;
        self.data.get(&seq)
    }

    pub fn range(&mut self, start: u64, end: u64) -> Vec<(u64, &[u8])> {
        self.total_lookups += 1;
        self.data.range(start..end).map(|(&k, v)| (k, v.as_slice())).collect()
    }

    pub fn front(&self) -> Option<&Vec<u8>> { self.data.get(&self.head) }
    pub fn back(&self) -> Option<&Vec<u8>> { if self.tail > self.head { self.data.get(&(self.tail - 1)) } else { None } }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }
    pub fn cap(&self) -> usize { self.cap }
    pub fn head(&self) -> u64 { self.head }
    pub fn tail(&self) -> u64 { self.tail }
    pub fn total_pushes(&self) -> u64 { self.total_pushes }
    pub fn total_pops(&self) -> u64 { self.total_pops }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut rt = RingTree::new(4);
        rt.push(b"a".to_vec()); rt.push(b"b".to_vec());
        assert_eq!(rt.pop().unwrap(), b"a");
        assert_eq!(rt.len(), 1);
    }

    #[test]
    fn cap_overflow() {
        let mut rt = RingTree::new(2);
        rt.push(b"a".to_vec()); rt.push(b"b".to_vec()); rt.push(b"c".to_vec());
        assert_eq!(rt.len(), 2);
        assert_eq!(rt.front().unwrap(), b"b");
    }

    #[test]
    fn get_indexed() {
        let mut rt = RingTree::new(8);
        rt.push(b"a".to_vec()); rt.push(b"b".to_vec()); rt.push(b"c".to_vec());
        assert_eq!(rt.get(1).unwrap(), b"b");
    }

    #[test]
    fn get_abs() {
        let mut rt = RingTree::new(8);
        rt.push(b"x".to_vec());
        let seq = rt.head();
        assert_eq!(rt.get_abs(seq).unwrap(), b"x");
    }

    #[test]
    fn range_query() {
        let mut rt = RingTree::new(8);
        rt.push(b"a".to_vec()); rt.push(b"b".to_vec()); rt.push(b"c".to_vec());
        let r = rt.range(rt.head(), rt.tail());
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn front_back() {
        let mut rt = RingTree::new(8);
        rt.push(b"first".to_vec()); rt.push(b"last".to_vec());
        assert_eq!(rt.front().unwrap(), b"first");
        assert_eq!(rt.back().unwrap(), b"last");
    }

    #[test]
    fn empty_pop() { assert!(RingTree::new(4).pop().is_none()); }

    #[test]
    fn stats() {
        let mut rt = RingTree::new(8);
        rt.push(b"x".to_vec()); rt.pop(); rt.get(0);
        assert_eq!(rt.total_pushes(), 1);
        assert_eq!(rt.total_pops(), 1);
    }
}
