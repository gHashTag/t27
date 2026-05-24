use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum VebError {
    Empty,
    NotFound { key: u64 },
    AlreadyExists { key: u64 },
}

impl std::fmt::Display for VebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VebError::Empty => write!(f, "tree empty"),
            VebError::NotFound { key } => write!(f, "key {key} not found"),
            VebError::AlreadyExists { key } => write!(f, "key {key} exists"),
        }
    }
}

impl std::error::Error for VebError {}

pub struct VebTree {
    u: u64,
    min: Option<u64>,
    max: Option<u64>,
    summary: Option<Box<VebTree>>,
    clusters: BTreeMap<u64, Box<VebTree>>,
    total_inserts: u64,
    total_deletes: u64,
    total_queries: u64,
}

impl VebTree {
    pub fn new(bits: u8) -> Self {
        let u = 1u64 << bits;
        Self { u, min: None, max: None, summary: None, clusters: BTreeMap::new(), total_inserts: 0, total_deletes: 0, total_queries: 0 }
    }

    fn high(&self, x: u64) -> u64 { x >> (self.bits() / 2) }
    fn low(&self, x: u64) -> u64 { x & ((1u64 << (self.bits() / 2)) - 1) }
    fn index(&self, high: u64, low: u64) -> u64 { (high << (self.bits() / 2)) | low }
    fn bits(&self) -> u8 { 64 - self.u.leading_zeros() as u8 - 1 }
    fn sub_bits(&self) -> u8 { (self.bits() + 1) / 2 }

    fn cluster(&mut self, high: u64) -> &mut VebTree {
        let sub = self.sub_bits();
        self.clusters.entry(high).or_insert_with(|| Box::new(VebTree::new(sub)))
    }

    fn summary(&mut self) -> &mut VebTree {
        let sub = self.sub_bits();
        self.summary.get_or_insert_with(|| Box::new(VebTree::new(sub)))
    }

    pub fn insert(&mut self, x: u64) -> Result<(), VebError> {
        if x >= self.u { return Err(VebError::NotFound { key: x }); }
        self.total_inserts += 1;
        match self.min {
            None => { self.min = Some(x); self.max = Some(x); return Ok(()); }
            Some(m) if x == m => return Err(VebError::AlreadyExists { key: x }),
            _ => {}
        }
        let mut val = x;
        if val < self.min.unwrap() { std::mem::swap(&mut val, self.min.as_mut().unwrap()); }
        let hi = self.high(val);
        let lo = self.low(val);
        {
            let c = self.cluster(hi);
            if c.min.is_none() {
                self.summary().insert(hi).ok();
            }
            self.cluster(hi).insert(lo).ok();
        }
        if val > self.max.unwrap() { self.max = Some(val); }
        Ok(())
    }

    pub fn contains(&mut self, x: u64) -> bool {
        self.total_queries += 1;
        if x >= self.u { return false; }
        if self.min == Some(x) || self.max == Some(x) { return true; }
        if self.min.is_none() { return false; }
        let hi = self.high(x);
        let lo = self.low(x);
        match self.clusters.get_mut(&hi) {
            Some(c) => c.contains(lo),
            None => false,
        }
    }

    pub fn predecessor(&mut self, x: u64) -> Option<u64> {
        self.total_queries += 1;
        if self.min.is_none() { return None; }
        if x > self.max.unwrap() { return self.max; }
        if x <= self.min.unwrap() { return None; }
        let hi = self.high(x);
        let lo = self.low(x);
        let lo_min = self.clusters.get(&hi).and_then(|c| c.min);
        if let Some(lm) = lo_min {
            if lo > lm {
                if let Some(c) = self.clusters.get_mut(&hi) {
                    if let Some(pred) = c.predecessor(lo) { return Some(self.index(hi, pred)); }
                }
            }
        }
        let pred_cluster = self.summary.as_mut().and_then(|s| s.predecessor(hi));
        match pred_cluster {
            Some(pc) => Some(self.index(pc, self.clusters.get(&pc).unwrap().max.unwrap())),
            None => {
                if self.min.unwrap() < x { Some(self.min.unwrap()) } else { None }
            }
        }
    }

    pub fn successor(&mut self, x: u64) -> Option<u64> {
        self.total_queries += 1;
        if self.min.is_none() { return None; }
        if x < self.min.unwrap() { return self.min; }
        if x >= self.max.unwrap() { return None; }
        let hi = self.high(x);
        let lo = self.low(x);
        let lo_max = self.clusters.get(&hi).and_then(|c| c.max);
        if let Some(lm) = lo_max {
            if lo < lm {
                if let Some(c) = self.clusters.get_mut(&hi) {
                    if let Some(succ) = c.successor(lo) { return Some(self.index(hi, succ)); }
                }
            }
        }
        let succ_cluster = self.summary.as_mut().and_then(|s| s.successor(hi));
        succ_cluster.map(|sc| self.index(sc, self.clusters.get(&sc).unwrap().min.unwrap()))
    }

    pub fn min(&self) -> Option<u64> { self.min }
    pub fn max(&self) -> Option<u64> { self.max }
    pub fn universe(&self) -> u64 { self.u }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_veb() { let v = VebTree::new(4); assert_eq!(v.universe(), 16); assert!(v.min().is_none()); }

    #[test]
    fn insert_min_max() {
        let mut v = VebTree::new(8);
        v.insert(5).unwrap(); v.insert(200).unwrap(); v.insert(42).unwrap();
        assert_eq!(v.min(), Some(5));
        assert_eq!(v.max(), Some(200));
    }

    #[test]
    fn contains() {
        let mut v = VebTree::new(8);
        v.insert(10).unwrap(); v.insert(20).unwrap();
        assert!(v.contains(10));
        assert!(v.contains(20));
        assert!(!v.contains(15));
    }

    #[test]
    fn duplicate() {
        let mut v = VebTree::new(4);
        v.insert(5).unwrap();
        assert!(v.insert(5).is_err());
    }

    #[test]
    fn predecessor() {
        let mut v = VebTree::new(8);
        for x in [10, 20, 50, 100] { v.insert(x).unwrap(); }
        assert_eq!(v.predecessor(30), Some(20));
        assert_eq!(v.predecessor(50), Some(20));
        assert_eq!(v.predecessor(10), None);
    }

    #[test]
    fn successor() {
        let mut v = VebTree::new(8);
        for x in [10, 20, 50, 100] { v.insert(x).unwrap(); }
        assert_eq!(v.successor(30), Some(50));
        assert_eq!(v.successor(10), Some(20));
        assert_eq!(v.successor(200), None);
    }

    #[test]
    fn many_inserts() {
        let mut v = VebTree::new(10);
        for i in (0..1024).step_by(7) { v.insert(i).unwrap(); }
        assert_eq!(v.min(), Some(0));
        assert!(v.contains(0));
        assert!(v.contains(21));
        assert!(!v.contains(1));
    }

    #[test]
    fn edge_pred_succ() {
        let mut v = VebTree::new(4);
        v.insert(0).unwrap(); v.insert(15).unwrap();
        assert_eq!(v.predecessor(15), Some(0));
        assert_eq!(v.successor(0), Some(15));
    }

    #[test]
    fn stats() {
        let mut v = VebTree::new(4);
        v.insert(1).unwrap(); v.contains(1); v.predecessor(5);
        assert_eq!(v.total_inserts(), 1);
        assert_eq!(v.total_queries(), 2);
    }

    #[test]
    fn error_display() { assert!(VebError::Empty.to_string().contains("empty")); }
}
