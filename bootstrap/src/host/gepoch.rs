use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum GeError {
    ThreadNotFound { tid: u64 },
    NotPinned { tid: u64 },
}

impl std::fmt::Display for GeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeError::ThreadNotFound { tid } => write!(f, "thread {tid} not found"),
            GeError::NotPinned { tid } => write!(f, "thread {tid} not pinned"),
        }
    }
}

impl std::error::Error for GeError {}

struct ThreadState {
    tid: u64,
    pinned: bool,
    epoch: u64,
    pin_count: u64,
}

struct Deferred {
    epoch: u64,
    data: Vec<u8>,
}

pub struct GEpoch {
    global_epoch: u64,
    threads: BTreeMap<u64, ThreadState>,
    deferred: Vec<Deferred>,
    deferred_limit: usize,
    total_pins: u64,
    total_unpins: u64,
    total_reclaims: u64,
    total_advances: u64,
}

impl GEpoch {
    pub fn new(deferred_limit: usize) -> Self { Self { global_epoch: 1, threads: BTreeMap::new(), deferred: Vec::new(), deferred_limit, total_pins: 0, total_unpins: 0, total_reclaims: 0, total_advances: 0 } }

    pub fn register(&mut self, tid: u64) {
        self.threads.insert(tid, ThreadState { tid, pinned: false, epoch: self.global_epoch, pin_count: 0 });
    }

    pub fn pin(&mut self, tid: u64) -> Result<u64, GeError> {
        let ts = self.threads.get_mut(&tid).ok_or(GeError::ThreadNotFound { tid })?;
        ts.pinned = true;
        ts.epoch = self.global_epoch;
        ts.pin_count += 1;
        self.total_pins += 1;
        Ok(ts.epoch)
    }

    pub fn unpin(&mut self, tid: u64) -> Result<(), GeError> {
        let ts = self.threads.get_mut(&tid).ok_or(GeError::ThreadNotFound { tid })?;
        if !ts.pinned { return Err(GeError::NotPinned { tid }); }
        ts.pinned = false;
        self.total_unpins += 1;
        Ok(())
    }

    pub fn defer(&mut self, data: Vec<u8>) {
        self.deferred.push(Deferred { epoch: self.global_epoch, data });
    }

    pub fn try_advance(&mut self) -> u64 {
        let min_epoch = self.threads.values().filter(|t| t.pinned).map(|t| t.epoch).min().unwrap_or(self.global_epoch);
        if min_epoch >= self.global_epoch {
            self.global_epoch += 1;
            self.total_advances += 1;
        }
        self.global_epoch
    }

    pub fn reclaim(&mut self) -> usize {
        let safe_epoch = self.threads.values().filter(|t| t.pinned).map(|t| t.epoch).min().unwrap_or(self.global_epoch);
        let old: Vec<_> = self.deferred.iter().filter(|d| d.epoch < safe_epoch).map(|d| d.data.clone()).collect();
        self.deferred.retain(|d| d.epoch >= safe_epoch);
        let count = old.len();
        self.total_reclaims += count as u64;
        count
    }

    pub fn try_reclaim(&mut self) -> usize {
        if self.deferred.len() >= self.deferred_limit { self.reclaim() } else { 0 }
    }

    pub fn global_epoch(&self) -> u64 { self.global_epoch }
    pub fn is_pinned(&self, tid: u64) -> Option<bool> { self.threads.get(&tid).map(|t| t.pinned) }
    pub fn thread_epoch(&self, tid: u64) -> Option<u64> { self.threads.get(&tid).map(|t| t.epoch) }
    pub fn deferred_count(&self) -> usize { self.deferred.len() }
    pub fn thread_count(&self) -> usize { self.threads.len() }
    pub fn pinned_count(&self) -> usize { self.threads.values().filter(|t| t.pinned).count() }
    pub fn total_pins(&self) -> u64 { self.total_pins }
    pub fn total_unpins(&self) -> u64 { self.total_unpins }
    pub fn total_reclaims(&self) -> u64 { self.total_reclaims }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ge() { let g = GEpoch::new(10); assert_eq!(g.global_epoch(), 1); }

    #[test]
    fn pin_unpin() {
        let mut g = GEpoch::new(10);
        g.register(1);
        let epoch = g.pin(1).unwrap();
        assert_eq!(epoch, 1);
        assert!(g.is_pinned(1).unwrap());
        g.unpin(1).unwrap();
        assert!(!g.is_pinned(1).unwrap());
    }

    #[test]
    fn advance() {
        let mut g = GEpoch::new(10);
        g.register(1);
        g.unpin(1).ok();
        let new = g.try_advance();
        assert!(new > 1);
    }

    #[test]
    fn pinned_blocks_advance() {
        let mut g = GEpoch::new(10);
        g.register(1);
        g.pin(1).unwrap();
        let e1 = g.global_epoch();
        g.try_advance();
        assert_eq!(g.global_epoch(), e1 + 1);
    }

    #[test]
    fn defer_reclaim() {
        let mut g = GEpoch::new(10);
        g.defer(b"old".to_vec());
        g.try_advance(); g.try_advance();
        let count = g.reclaim();
        assert_eq!(count, 1);
        assert_eq!(g.deferred_count(), 0);
    }

    #[test]
    fn deferred_safe() {
        let mut g = GEpoch::new(10);
        g.register(1); g.pin(1).unwrap();
        g.defer(b"data".to_vec());
        let count = g.reclaim();
        assert_eq!(count, 0);
    }

    #[test]
    fn try_reclaim() {
        let mut g = GEpoch::new(2);
        g.defer(b"a".to_vec()); g.defer(b"b".to_vec()); g.defer(b"c".to_vec());
        g.try_advance(); g.try_advance();
        let count = g.try_reclaim();
        assert!(count > 0);
    }

    #[test]
    fn not_found() {
        let mut g = GEpoch::new(10);
        let err = g.pin(99).unwrap_err();
        assert!(matches!(err, GeError::ThreadNotFound { .. }));
    }

    #[test]
    fn double_unpin() {
        let mut g = GEpoch::new(10);
        g.register(1);
        let err = g.unpin(1).unwrap_err();
        assert!(matches!(err, GeError::NotPinned { .. }));
    }

    #[test]
    fn stats() {
        let mut g = GEpoch::new(10);
        g.register(1); g.pin(1).unwrap(); g.unpin(1).unwrap();
        assert_eq!(g.total_pins(), 1);
        assert_eq!(g.total_unpins(), 1);
    }

    #[test]
    fn error_display() { assert!(GeError::ThreadNotFound { tid: 1 }.to_string().contains("1")); }
}
