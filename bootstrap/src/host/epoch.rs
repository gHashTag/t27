use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpochError {
    NotPinned { guard_id: u64 },
    AlreadyPinned { guard_id: u64 },
    NothingToRetire,
}

impl std::fmt::Display for EpochError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpochError::NotPinned { guard_id } => write!(f, "guard {guard_id} not pinned"),
            EpochError::AlreadyPinned { guard_id } => write!(f, "guard {guard_id} already pinned"),
            EpochError::NothingToRetire => write!(f, "nothing to retire"),
        }
    }
}

impl std::error::Error for EpochError {}

#[derive(Debug, Clone)]
struct PendingRetire {
    epoch: u64,
    data: u64,
}

#[derive(Debug, Clone)]
pub struct EpochStats {
    pub current_epoch: u64,
    pub active_guards: usize,
    pub min_guard_epoch: Option<u64>,
    pub pending_count: usize,
    pub total_retired: u64,
    pub total_epochs: u64,
}

#[derive(Debug, Clone)]
pub struct EpochCounter {
    current_epoch: u64,
    guards: BTreeMap<u64, u64>,
    pending: Vec<PendingRetire>,
    next_guard: u64,
    total_retired: u64,
    total_epochs: u64,
    epoch_advance_threshold: usize,
}

impl EpochCounter {
    pub fn new(advance_threshold: usize) -> Self {
        Self {
            current_epoch: 0,
            guards: BTreeMap::new(),
            pending: Vec::new(),
            next_guard: 1,
            total_retired: 0,
            total_epochs: 1,
            epoch_advance_threshold: advance_threshold.max(1),
        }
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    pub fn pin(&mut self) -> u64 {
        let id = self.next_guard;
        self.next_guard += 1;
        self.guards.insert(id, self.current_epoch);
        id
    }

    pub fn unpin(&mut self, guard_id: u64) -> Result<u64, EpochError> {
        let epoch = self.guards.remove(&guard_id)
            .ok_or(EpochError::NotPinned { guard_id })?;
        if self.guards.is_empty() || self.safe_epoch() > self.current_epoch {
            if self.pending.len() >= self.epoch_advance_threshold {
                self.try_advance();
            }
        }
        Ok(epoch)
    }

    pub fn is_pinned(&self, guard_id: u64) -> bool {
        self.guards.contains_key(&guard_id)
    }

    pub fn guard_epoch(&self, guard_id: u64) -> Option<u64> {
        self.guards.get(&guard_id).copied()
    }

    pub fn active_guards(&self) -> usize {
        self.guards.len()
    }

    pub fn defer_retire(&mut self, data: u64) {
        self.pending.push(PendingRetire { epoch: self.current_epoch, data });
    }

    fn safe_epoch(&self) -> u64 {
        self.guards.values().copied().min().unwrap_or(self.current_epoch)
    }

    pub fn try_advance(&mut self) -> usize {
        let before = self.pending.len();
        if self.guards.is_empty() {
            let retired = before;
            self.total_retired += retired as u64;
            self.pending.clear();
            self.current_epoch += 1;
            self.total_epochs += 1;
            return retired;
        }
        let safe = self.safe_epoch();
        let to_keep: Vec<PendingRetire> = self.pending.drain(..)
            .filter(|p| p.epoch >= safe)
            .collect();
        let retired = before - to_keep.len();
        self.total_retired += retired as u64;
        self.pending = to_keep;
        self.current_epoch += 1;
        self.total_epochs += 1;
        retired
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn total_retired(&self) -> u64 {
        self.total_retired
    }

    pub fn stats(&self) -> EpochStats {
        EpochStats {
            current_epoch: self.current_epoch,
            active_guards: self.guards.len(),
            min_guard_epoch: self.guards.values().copied().min(),
            pending_count: self.pending.len(),
            total_retired: self.total_retired,
            total_epochs: self.total_epochs,
        }
    }

    pub fn reset(&mut self) {
        self.current_epoch = 0;
        self.guards.clear();
        self.pending.clear();
        self.next_guard = 1;
        self.total_retired = 0;
        self.total_epochs = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_counter() {
        let ec = EpochCounter::new(10);
        assert_eq!(ec.current_epoch(), 0);
        assert_eq!(ec.active_guards(), 0);
    }

    #[test]
    fn pin_unpin() {
        let mut ec = EpochCounter::new(10);
        let g = ec.pin();
        assert!(ec.is_pinned(g));
        assert_eq!(ec.active_guards(), 1);
        ec.unpin(g).unwrap();
        assert!(!ec.is_pinned(g));
        assert_eq!(ec.active_guards(), 0);
    }

    #[test]
    fn unpin_not_pinned() {
        let mut ec = EpochCounter::new(10);
        let err = ec.unpin(99).unwrap_err();
        assert!(matches!(err, EpochError::NotPinned { .. }));
    }

    #[test]
    fn guard_epoch() {
        let mut ec = EpochCounter::new(10);
        let g = ec.pin();
        assert_eq!(ec.guard_epoch(g), Some(0));
    }

    #[test]
    fn defer_retire() {
        let mut ec = EpochCounter::new(10);
        ec.defer_retire(42);
        ec.defer_retire(99);
        assert_eq!(ec.pending_count(), 2);
    }

    #[test]
    fn advance_retires_safe() {
        let mut ec = EpochCounter::new(10);
        ec.defer_retire(1);
        ec.defer_retire(2);
        let retired = ec.try_advance();
        assert_eq!(retired, 2);
        assert_eq!(ec.pending_count(), 0);
        assert_eq!(ec.total_retired(), 2);
    }

    #[test]
    fn advance_keeps_pinned_epoch() {
        let mut ec = EpochCounter::new(10);
        let g = ec.pin();
        ec.defer_retire(1);
        ec.try_advance();
        assert_eq!(ec.pending_count(), 1);
        ec.unpin(g).unwrap();
        ec.try_advance();
        assert_eq!(ec.pending_count(), 0);
    }

    #[test]
    fn multiple_guards() {
        let mut ec = EpochCounter::new(10);
        let g1 = ec.pin();
        ec.defer_retire(1);
        ec.try_advance();
        let g2 = ec.pin();
        assert_eq!(ec.active_guards(), 2);
        ec.unpin(g1).unwrap();
        ec.unpin(g2).unwrap();
        assert_eq!(ec.active_guards(), 0);
    }

    #[test]
    fn stats_snapshot() {
        let mut ec = EpochCounter::new(10);
        ec.pin();
        ec.defer_retire(1);
        let s = ec.stats();
        assert_eq!(s.active_guards, 1);
        assert_eq!(s.pending_count, 1);
    }

    #[test]
    fn reset() {
        let mut ec = EpochCounter::new(10);
        ec.pin();
        ec.defer_retire(1);
        ec.reset();
        assert_eq!(ec.active_guards(), 0);
        assert_eq!(ec.pending_count(), 0);
        assert_eq!(ec.current_epoch(), 0);
    }

    #[test]
    fn epoch_increments() {
        let mut ec = EpochCounter::new(10);
        ec.try_advance();
        ec.try_advance();
        assert_eq!(ec.current_epoch(), 2);
        assert_eq!(ec.total_epochs, 3);
    }

    #[test]
    fn error_display() {
        assert!(EpochError::NotPinned { guard_id: 5 }.to_string().contains("5"));
    }
}
