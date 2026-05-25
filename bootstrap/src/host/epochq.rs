use std::collections::BTreeMap;

pub struct EpochQ {
    epoch: u64,
    pinned: BTreeMap<u64, u64>,
    pending: Vec<(u64, Vec<u8>)>,
    retired: Vec<Vec<u8>>,
    total_pins: u64,
    total_unpins: u64,
    total_retires: u64,
}

impl EpochQ {
    pub fn new() -> Self { Self { epoch: 0, pinned: BTreeMap::new(), pending: Vec::new(), retired: Vec::new(), total_pins: 0, total_unpins: 0, total_retires: 0 } }

    pub fn pin(&mut self, guard_id: u64) -> u64 {
        self.total_pins += 1;
        self.pinned.insert(guard_id, self.epoch);
        self.epoch
    }

    pub fn unpin(&mut self, guard_id: u64) -> bool {
        self.total_unpins += 1;
        self.pinned.remove(&guard_id).is_some()
    }

    pub fn defer(&mut self, data: Vec<u8>) {
        self.pending.push((self.epoch, data));
    }

    pub fn try_retire(&mut self) -> usize {
        if self.pinned.is_empty() {
            let count = self.pending.len();
            for (_, data) in self.pending.drain(..) { self.retired.push(data); }
            self.total_retires += count as u64;
            return count;
        }
        let min_pinned = *self.pinned.values().min().unwrap();
        let to_retire: Vec<usize> = self.pending.iter().enumerate()
            .filter(|(_, (ep, _))| *ep < min_pinned)
            .map(|(i, _)| i)
            .rev()
            .collect();
        for i in to_retire {
            let (_, data) = self.pending.remove(i);
            self.retired.push(data);
            self.total_retires += 1;
        }
        self.total_retires as usize
    }

    pub fn advance(&mut self) {
        self.epoch += 1;
        if self.pinned.is_empty() { self.try_retire(); }
    }

    pub fn epoch(&self) -> u64 { self.epoch }
    pub fn pending_count(&self) -> usize { self.pending.len() }
    pub fn retired_count(&self) -> usize { self.retired.len() }
    pub fn pinned_count(&self) -> usize { self.pinned.len() }
    pub fn total_pins(&self) -> u64 { self.total_pins }
    pub fn total_unpins(&self) -> u64 { self.total_unpins }
    pub fn total_retires(&self) -> u64 { self.total_retires }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_unpin() {
        let mut eq = EpochQ::new();
        eq.pin(1);
        assert_eq!(eq.pinned_count(), 1);
        eq.unpin(1);
        assert_eq!(eq.pinned_count(), 0);
    }

    #[test]
    fn defer_retire() {
        let mut eq = EpochQ::new();
        eq.defer(b"a".to_vec());
        eq.try_retire();
        assert_eq!(eq.retired_count(), 1);
        assert_eq!(eq.pending_count(), 0);
    }

    #[test]
    fn pinned_blocks_retire() {
        let mut eq = EpochQ::new();
        eq.pin(1);
        eq.defer(b"a".to_vec());
        eq.try_retire();
        assert_eq!(eq.retired_count(), 0);
        assert_eq!(eq.pending_count(), 1);
    }

    #[test]
    fn unpin_allows_retire() {
        let mut eq = EpochQ::new();
        eq.pin(1);
        eq.defer(b"a".to_vec());
        eq.unpin(1);
        eq.try_retire();
        assert_eq!(eq.retired_count(), 1);
    }

    #[test]
    fn advance() {
        let mut eq = EpochQ::new();
        eq.advance();
        eq.advance();
        assert_eq!(eq.epoch(), 2);
    }

    #[test]
    fn advance_auto_retires() {
        let mut eq = EpochQ::new();
        eq.defer(b"x".to_vec());
        eq.advance();
        assert_eq!(eq.retired_count(), 1);
    }

    #[test]
    fn stats() {
        let mut eq = EpochQ::new();
        eq.pin(1); eq.unpin(1); eq.defer(b"x".to_vec()); eq.try_retire();
        assert_eq!(eq.total_pins(), 1);
        assert_eq!(eq.total_unpins(), 1);
        assert!(eq.total_retires() > 0);
    }
}
