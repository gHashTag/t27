use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaveFrontError {
    DuplicateStage { stage: usize },
    UnknownStage { stage: usize },
    AlreadyComplete { stage: usize },
    NotAllComplete,
}

impl std::fmt::Display for WaveFrontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaveFrontError::DuplicateStage { stage } => write!(f, "stage {stage} already exists"),
            WaveFrontError::UnknownStage { stage } => write!(f, "unknown stage {stage}"),
            WaveFrontError::AlreadyComplete { stage } => write!(f, "stage {stage} already complete"),
            WaveFrontError::NotAllComplete => write!(f, "not all stages complete"),
        }
    }
}

impl std::error::Error for WaveFrontError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageState {
    Pending,
    InProgress,
    Complete,
}

#[derive(Debug, Clone)]
struct Stage {
    id: usize,
    state: StageState,
    epoch: u64,
}

#[derive(Debug, Clone)]
pub struct WaveFrontStats {
    pub total_stages: usize,
    pub pending: usize,
    pub in_progress: usize,
    pub complete: usize,
    pub current_epoch: u64,
    pub min_epoch: u64,
    pub max_epoch: u64,
}

#[derive(Debug, Clone)]
pub struct WaveFront {
    stages: BTreeMap<usize, Stage>,
    current_epoch: u64,
    total_advances: u64,
    total_completed: u64,
}

impl WaveFront {
    pub fn new() -> Self {
        Self { stages: BTreeMap::new(), current_epoch: 0, total_advances: 0, total_completed: 0 }
    }

    pub fn add_stage(&mut self, id: usize) -> Result<(), WaveFrontError> {
        if self.stages.contains_key(&id) {
            return Err(WaveFrontError::DuplicateStage { stage: id });
        }
        self.stages.insert(id, Stage { id, state: StageState::Pending, epoch: 0 });
        Ok(())
    }

    pub fn start(&mut self, stage: usize) -> Result<(), WaveFrontError> {
        let s = self.stages.get_mut(&stage).ok_or(WaveFrontError::UnknownStage { stage })?;
        if s.state == StageState::Complete {
            return Err(WaveFrontError::AlreadyComplete { stage });
        }
        s.state = StageState::InProgress;
        Ok(())
    }

    pub fn complete(&mut self, stage: usize) -> Result<u64, WaveFrontError> {
        if !self.stages.contains_key(&stage) {
            return Err(WaveFrontError::UnknownStage { stage });
        }
        let s = self.stages.get_mut(&stage).unwrap();
        if s.state == StageState::Complete {
            return Err(WaveFrontError::AlreadyComplete { stage });
        }
        s.state = StageState::Complete;
        s.epoch = self.current_epoch + 1;
        self.total_completed += 1;
        let ep = s.epoch;
        if self.stages.values().all(|s| s.state == StageState::Complete) {
            self.current_epoch += 1;
            self.total_advances += 1;
            for s in self.stages.values_mut() {
                s.state = StageState::Pending;
                s.epoch = self.current_epoch;
            }
        }
        Ok(ep)
    }

    fn try_advance(&mut self) {
        if self.stages.values().all(|s| s.state == StageState::Complete) {
            self.current_epoch += 1;
            self.total_advances += 1;
            for s in self.stages.values_mut() {
                s.state = StageState::Pending;
                s.epoch = self.current_epoch;
            }
        }
    }

    pub fn state(&self, stage: usize) -> Option<StageState> {
        self.stages.get(&stage).map(|s| s.state)
    }

    pub fn epoch(&self, stage: usize) -> Option<u64> {
        self.stages.get(&stage).map(|s| s.epoch)
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    pub fn all_complete(&self) -> bool {
        self.stages.values().all(|s| s.state == StageState::Complete)
    }

    pub fn pending_stages(&self) -> Vec<usize> {
        self.stages.values().filter(|s| s.state == StageState::Pending).map(|s| s.id).collect()
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    pub fn stats(&self) -> WaveFrontStats {
        WaveFrontStats {
            total_stages: self.stages.len(),
            pending: self.stages.values().filter(|s| s.state == StageState::Pending).count(),
            in_progress: self.stages.values().filter(|s| s.state == StageState::InProgress).count(),
            complete: self.stages.values().filter(|s| s.state == StageState::Complete).count(),
            current_epoch: self.current_epoch,
            min_epoch: self.stages.values().map(|s| s.epoch).min().unwrap_or(0),
            max_epoch: self.stages.values().map(|s| s.epoch).max().unwrap_or(0),
        }
    }

    pub fn total_advances(&self) -> u64 {
        self.total_advances
    }

    pub fn total_completed(&self) -> u64 {
        self.total_completed
    }

    pub fn reset(&mut self) {
        for s in self.stages.values_mut() {
            s.state = StageState::Pending;
            s.epoch = self.current_epoch;
        }
    }
}

impl Default for WaveFront {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wavefront() {
        let wf = WaveFront::new();
        assert_eq!(wf.stage_count(), 0);
        assert_eq!(wf.current_epoch(), 0);
    }

    #[test]
    fn add_stage() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        assert_eq!(wf.stage_count(), 2);
    }

    #[test]
    fn duplicate_stage() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        let err = wf.add_stage(0).unwrap_err();
        assert!(matches!(err, WaveFrontError::DuplicateStage { .. }));
    }

    #[test]
    fn start_and_complete() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        wf.start(0).unwrap();
        assert_eq!(wf.state(0), Some(StageState::InProgress));
        let ep = wf.complete(0).unwrap();
        assert_eq!(ep, 1);
        assert_eq!(wf.state(0), Some(StageState::Complete));
    }

    #[test]
    fn epoch_advances_on_all_complete() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        wf.complete(0).unwrap();
        assert_eq!(wf.current_epoch(), 0);
        wf.complete(1).unwrap();
        assert_eq!(wf.current_epoch(), 1);
        assert_eq!(wf.total_advances(), 1);
        assert_eq!(wf.state(0), Some(StageState::Pending));
    }

    #[test]
    fn already_complete() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        wf.complete(0).unwrap();
        let err = wf.complete(0).unwrap_err();
        assert!(matches!(err, WaveFrontError::AlreadyComplete { .. }));
    }

    #[test]
    fn unknown_stage() {
        let mut wf = WaveFront::new();
        let err = wf.start(99).unwrap_err();
        assert!(matches!(err, WaveFrontError::UnknownStage { .. }));
    }

    #[test]
    fn multi_epoch() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        for _ in 0..3 {
            wf.complete(0).unwrap();
            wf.complete(1).unwrap();
        }
        assert_eq!(wf.current_epoch(), 3);
        assert_eq!(wf.total_completed(), 6);
    }

    #[test]
    fn pending_stages() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        wf.start(0).unwrap();
        let p = wf.pending_stages();
        assert_eq!(p, vec![1]);
    }

    #[test]
    fn stats() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        wf.add_stage(2).unwrap();
        wf.complete(0).unwrap();
        wf.start(1).unwrap();
        let s = wf.stats();
        assert_eq!(s.complete, 1);
        assert_eq!(s.pending, 1);
        assert_eq!(s.in_progress, 1);
    }

    #[test]
    fn reset() {
        let mut wf = WaveFront::new();
        wf.add_stage(0).unwrap();
        wf.add_stage(1).unwrap();
        wf.complete(0).unwrap();
        wf.reset();
        assert_eq!(wf.state(0), Some(StageState::Pending));
        assert_eq!(wf.state(1), Some(StageState::Pending));
    }

    #[test]
    fn error_display() {
        assert!(WaveFrontError::UnknownStage { stage: 3 }.to_string().contains("3"));
    }
}
