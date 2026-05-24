use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RingCntError {
    RingExists { id: u64 },
    RingNotFound { id: u64 },
}

impl std::fmt::Display for RingCntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RingCntError::RingExists { id } => write!(f, "ring {id} exists"),
            RingCntError::RingNotFound { id } => write!(f, "ring {id} not found"),
        }
    }
}

impl std::error::Error for RingCntError {}

struct Ring {
    id: u64,
    slots: Vec<u64>,
    current_idx: usize,
    total: u64,
}

impl Ring {
    fn new(id: u64, slots: usize) -> Self {
        Self { id, slots: vec![0; slots], current_idx: 0, total: 0 }
    }

    fn advance(&mut self) {
        self.current_idx = (self.current_idx + 1) % self.slots.len();
        self.total = self.total.saturating_sub(self.slots[self.current_idx]);
        self.slots[self.current_idx] = 0;
    }

    fn inc(&mut self, delta: u64) {
        self.slots[self.current_idx] += delta;
        self.total += delta;
    }

    fn dec(&mut self, delta: u64) {
        self.slots[self.current_idx] = self.slots[self.current_idx].saturating_sub(delta);
        self.total = self.total.saturating_sub(delta);
    }
}

pub struct RingCounter {
    rings: BTreeMap<u64, Ring>,
    total_advances: u64,
    total_incs: u64,
    total_decs: u64,
}

impl RingCounter {
    pub fn new() -> Self { Self { rings: BTreeMap::new(), total_advances: 0, total_incs: 0, total_decs: 0 } }

    pub fn register(&mut self, id: u64, slots: usize) -> Result<(), RingCntError> {
        if self.rings.contains_key(&id) { return Err(RingCntError::RingExists { id }); }
        self.rings.insert(id, Ring::new(id, slots));
        Ok(())
    }

    pub fn advance(&mut self, id: u64) -> Result<(), RingCntError> {
        let r = self.rings.get_mut(&id).ok_or(RingCntError::RingNotFound { id })?;
        r.advance();
        self.total_advances += 1;
        Ok(())
    }

    pub fn advance_all(&mut self) {
        for r in self.rings.values_mut() { r.advance(); }
        self.total_advances += 1;
    }

    pub fn inc(&mut self, id: u64, delta: u64) -> Result<(), RingCntError> {
        let r = self.rings.get_mut(&id).ok_or(RingCntError::RingNotFound { id })?;
        r.inc(delta);
        self.total_incs += 1;
        Ok(())
    }

    pub fn dec(&mut self, id: u64, delta: u64) -> Result<(), RingCntError> {
        let r = self.rings.get_mut(&id).ok_or(RingCntError::RingNotFound { id })?;
        r.dec(delta);
        self.total_decs += 1;
        Ok(())
    }

    pub fn total(&self, id: u64) -> Option<u64> { self.rings.get(&id).map(|r| r.total) }
    pub fn slot(&self, id: u64, idx: usize) -> Option<u64> { self.rings.get(&id).and_then(|r| r.slots.get(idx).copied()) }
    pub fn ring_count(&self) -> usize { self.rings.len() }
    pub fn total_advances(&self) -> u64 { self.total_advances }
    pub fn total_incs(&self) -> u64 { self.total_incs }
    pub fn total_decs(&self) -> u64 { self.total_decs }

    pub fn rollup(&self) -> u64 { self.rings.values().map(|r| r.total).sum() }

    pub fn reset(&mut self, id: u64) -> Result<(), RingCntError> {
        let r = self.rings.get_mut(&id).ok_or(RingCntError::RingNotFound { id })?;
        r.total = 0;
        for s in &mut r.slots { *s = 0; }
        Ok(())
    }
}

impl Default for RingCounter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_counter() { assert_eq!(RingCounter::new().ring_count(), 0); }

    #[test]
    fn register_inc() {
        let mut rc = RingCounter::new();
        rc.register(1, 4).unwrap();
        rc.inc(1, 10).unwrap();
        assert_eq!(rc.total(1), Some(10));
    }

    #[test]
    fn advance_wraps() {
        let mut rc = RingCounter::new();
        rc.register(1, 3).unwrap();
        rc.inc(1, 10).unwrap();
        rc.advance(1).unwrap();
        rc.inc(1, 5).unwrap();
        assert_eq!(rc.total(1), Some(15));
        rc.advance(1).unwrap();
        rc.advance(1).unwrap();
        assert_eq!(rc.total(1), Some(5));
    }

    #[test]
    fn dec() {
        let mut rc = RingCounter::new();
        rc.register(1, 4).unwrap();
        rc.inc(1, 10).unwrap();
        rc.dec(1, 3).unwrap();
        assert_eq!(rc.total(1), Some(7));
    }

    #[test]
    fn duplicate() {
        let mut rc = RingCounter::new();
        rc.register(1, 4).unwrap();
        let err = rc.register(1, 4).unwrap_err();
        assert!(matches!(err, RingCntError::RingExists { .. }));
    }

    #[test]
    fn not_found() {
        let mut rc = RingCounter::new();
        let err = rc.inc(99, 1).unwrap_err();
        assert!(matches!(err, RingCntError::RingNotFound { .. }));
    }

    #[test]
    fn advance_all() {
        let mut rc = RingCounter::new();
        rc.register(1, 2).unwrap(); rc.register(2, 2).unwrap();
        rc.inc(1, 10).unwrap(); rc.inc(2, 20).unwrap();
        rc.advance_all();
        rc.advance_all();
        assert_eq!(rc.total(1), Some(0));
        assert_eq!(rc.total(2), Some(0));
    }

    #[test]
    fn rollup() {
        let mut rc = RingCounter::new();
        rc.register(1, 4).unwrap(); rc.register(2, 4).unwrap();
        rc.inc(1, 10).unwrap(); rc.inc(2, 20).unwrap();
        assert_eq!(rc.rollup(), 30);
    }

    #[test]
    fn reset() {
        let mut rc = RingCounter::new();
        rc.register(1, 4).unwrap();
        rc.inc(1, 100).unwrap();
        rc.reset(1).unwrap();
        assert_eq!(rc.total(1), Some(0));
    }

    #[test]
    fn slot_access() {
        let mut rc = RingCounter::new();
        rc.register(1, 4).unwrap();
        rc.inc(1, 42).unwrap();
        assert_eq!(rc.slot(1, 0), Some(42));
    }

    #[test]
    fn stats() {
        let mut rc = RingCounter::new();
        rc.register(1, 4).unwrap();
        rc.inc(1, 1).unwrap(); rc.dec(1, 1).unwrap(); rc.advance(1).unwrap();
        assert_eq!(rc.total_incs(), 1);
        assert_eq!(rc.total_decs(), 1);
        assert_eq!(rc.total_advances(), 1);
    }

    #[test]
    fn error_display() {
        assert!(RingCntError::RingNotFound { id: 3 }.to_string().contains("3"));
    }
}
