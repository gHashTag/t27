use std::collections::BTreeMap;

pub type FenceId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceState {
    Pending,
    Signaled,
    TimedOut,
    Error,
}

impl std::fmt::Display for FenceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenceState::Pending => write!(f, "pending"),
            FenceState::Signaled => write!(f, "signaled"),
            FenceState::TimedOut => write!(f, "timed_out"),
            FenceState::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fence {
    pub id: FenceId,
    pub state: FenceState,
    pub created_us: u64,
    pub signaled_us: Option<u64>,
    pub tag: String,
}

impl Fence {
    pub fn new(id: FenceId, created_us: u64, tag: &str) -> Self {
        Self { id, state: FenceState::Pending, created_us, signaled_us: None, tag: tag.to_string() }
    }

    pub fn is_complete(&self) -> bool {
        self.state != FenceState::Pending
    }

    pub fn latency_us(&self) -> Option<u64> {
        self.signaled_us.map(|s| s.saturating_sub(self.created_us))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenceError {
    AlreadySignaled { id: FenceId },
    NotFound { id: FenceId },
}

impl std::fmt::Display for FenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenceError::AlreadySignaled { id } => write!(f, "fence {id} already signaled"),
            FenceError::NotFound { id } => write!(f, "fence {id} not found"),
        }
    }
}

impl std::error::Error for FenceError {}

#[derive(Debug, Clone)]
pub struct DmaFenceManager {
    fences: BTreeMap<FenceId, Fence>,
    next_id: FenceId,
    timeout_us: u64,
    total_created: u64,
    total_signaled: u64,
    total_timeouts: u64,
}

impl DmaFenceManager {
    pub fn new(timeout_us: u64) -> Self {
        Self {
            fences: BTreeMap::new(),
            next_id: 1,
            timeout_us,
            total_created: 0,
            total_signaled: 0,
            total_timeouts: 0,
        }
    }

    pub fn create(&mut self, tag: &str, now_us: u64) -> FenceId {
        let id = self.next_id;
        self.next_id += 1;
        self.fences.insert(id, Fence::new(id, now_us, tag));
        self.total_created += 1;
        id
    }

    pub fn signal(&mut self, id: FenceId, now_us: u64) -> Result<(), FenceError> {
        let fence = self.fences.get_mut(&id).ok_or(FenceError::NotFound { id })?;
        if fence.is_complete() {
            return Err(FenceError::AlreadySignaled { id });
        }
        fence.state = FenceState::Signaled;
        fence.signaled_us = Some(now_us);
        self.total_signaled += 1;
        Ok(())
    }

    pub fn signal_error(&mut self, id: FenceId) -> Result<(), FenceError> {
        let fence = self.fences.get_mut(&id).ok_or(FenceError::NotFound { id })?;
        fence.state = FenceState::Error;
        Ok(())
    }

    pub fn check_timeouts(&mut self, now_us: u64) -> Vec<FenceId> {
        let mut timed_out = Vec::new();
        for fence in self.fences.values_mut() {
            if fence.state == FenceState::Pending {
                let elapsed = now_us.saturating_sub(fence.created_us);
                if elapsed > self.timeout_us {
                    fence.state = FenceState::TimedOut;
                    self.total_timeouts += 1;
                    timed_out.push(fence.id);
                }
            }
        }
        timed_out
    }

    pub fn wait(&mut self, id: FenceId, now_us: u64) -> FenceState {
        if let Some(fence) = self.fences.get(&id) {
            fence.state
        } else {
            FenceState::Error
        }
    }

    pub fn get(&self, id: FenceId) -> Option<&Fence> {
        self.fences.get(&id)
    }

    pub fn pending_count(&self) -> usize {
        self.fences.values().filter(|f| f.state == FenceState::Pending).count()
    }

    pub fn completed_count(&self) -> usize {
        self.fences.values().filter(|f| f.is_complete()).count()
    }

    pub fn fence_count(&self) -> usize {
        self.fences.len()
    }

    pub fn total_created(&self) -> u64 {
        self.total_created
    }

    pub fn total_signaled(&self) -> u64 {
        self.total_signaled
    }

    pub fn total_timeouts(&self) -> u64 {
        self.total_timeouts
    }

    pub fn avg_latency_us(&self) -> f64 {
        let latencies: Vec<u64> = self.fences.values()
            .filter_map(|f| f.latency_us()).collect();
        if latencies.is_empty() { 0.0 }
        else { latencies.iter().sum::<u64>() as f64 / latencies.len() as f64 }
    }

    pub fn prune_completed(&mut self) -> usize {
        let before = self.fences.len();
        self.fences.retain(|_, f| !f.is_complete());
        before - self.fences.len()
    }

    pub fn clear(&mut self) {
        self.fences.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_display() {
        assert_eq!(FenceState::Signaled.to_string(), "signaled");
    }

    #[test]
    fn create_and_signal() {
        let mut fm = DmaFenceManager::new(1000);
        let id = fm.create("dma0", 100);
        assert_eq!(fm.get(id).unwrap().state, FenceState::Pending);
        fm.signal(id, 150).unwrap();
        assert_eq!(fm.get(id).unwrap().state, FenceState::Signaled);
        assert_eq!(fm.get(id).unwrap().latency_us(), Some(50));
        assert_eq!(fm.total_signaled(), 1);
    }

    #[test]
    fn signal_not_found() {
        let mut fm = DmaFenceManager::new(1000);
        let err = fm.signal(999, 0).unwrap_err();
        assert!(matches!(err, FenceError::NotFound { .. }));
    }

    #[test]
    fn signal_already_signaled() {
        let mut fm = DmaFenceManager::new(1000);
        let id = fm.create("x", 0);
        fm.signal(id, 10).unwrap();
        let err = fm.signal(id, 20).unwrap_err();
        assert!(matches!(err, FenceError::AlreadySignaled { .. }));
    }

    #[test]
    fn signal_error() {
        let mut fm = DmaFenceManager::new(1000);
        let id = fm.create("x", 0);
        fm.signal_error(id).unwrap();
        assert_eq!(fm.get(id).unwrap().state, FenceState::Error);
    }

    #[test]
    fn check_timeouts() {
        let mut fm = DmaFenceManager::new(100);
        let id = fm.create("x", 0);
        let timed_out = fm.check_timeouts(200);
        assert_eq!(timed_out, vec![id]);
        assert_eq!(fm.get(id).unwrap().state, FenceState::TimedOut);
        assert_eq!(fm.total_timeouts(), 1);
    }

    #[test]
    fn check_no_timeout() {
        let mut fm = DmaFenceManager::new(1000);
        fm.create("x", 0);
        let timed_out = fm.check_timeouts(500);
        assert!(timed_out.is_empty());
    }

    #[test]
    fn pending_and_completed() {
        let mut fm = DmaFenceManager::new(1000);
        let id1 = fm.create("a", 0);
        let id2 = fm.create("b", 0);
        fm.signal(id1, 10).unwrap();
        assert_eq!(fm.pending_count(), 1);
        assert_eq!(fm.completed_count(), 1);
    }

    #[test]
    fn avg_latency() {
        let mut fm = DmaFenceManager::new(1000);
        let id1 = fm.create("a", 0);
        let id2 = fm.create("b", 0);
        fm.signal(id1, 100).unwrap();
        fm.signal(id2, 200).unwrap();
        assert!((fm.avg_latency_us() - 150.0).abs() < 0.01);
    }

    #[test]
    fn prune_completed() {
        let mut fm = DmaFenceManager::new(1000);
        let id = fm.create("a", 0);
        fm.signal(id, 10).unwrap();
        assert_eq!(fm.prune_completed(), 1);
        assert_eq!(fm.fence_count(), 0);
    }

    #[test]
    fn clear() {
        let mut fm = DmaFenceManager::new(1000);
        fm.create("a", 0);
        fm.clear();
        assert_eq!(fm.fence_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(FenceError::NotFound { id: 1 }.to_string().contains("1"));
    }
}
