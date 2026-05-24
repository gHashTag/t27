use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum WfError {
    StageExists { name: String },
    StageNotFound { name: String },
    DependencyCycle { name: String },
    AlreadyComplete { name: String },
    NotReady { name: String },
}

impl std::fmt::Display for WfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WfError::StageExists { name } => write!(f, "stage {name} exists"),
            WfError::StageNotFound { name } => write!(f, "stage {name} not found"),
            WfError::DependencyCycle { name } => write!(f, "cycle: {name}"),
            WfError::AlreadyComplete { name } => write!(f, "stage {name} complete"),
            WfError::NotReady { name } => write!(f, "stage {name} not ready"),
        }
    }
}

impl std::error::Error for WfError {}

#[derive(Debug, Clone, PartialEq)]
pub enum StageState { Pending, Ready, Running, Complete }

struct Stage {
    name: String,
    deps: Vec<String>,
    state: StageState,
    phase: u32,
    items: Vec<u64>,
    processed: usize,
}

pub struct WavefrontEngine {
    stages: BTreeMap<String, Stage>,
    total_stages_run: u64,
    total_items_processed: u64,
    total_barrier_waits: u64,
}

impl WavefrontEngine {
    pub fn new() -> Self { Self { stages: BTreeMap::new(), total_stages_run: 0, total_items_processed: 0, total_barrier_waits: 0 } }

    pub fn add_stage(&mut self, name: &str, phase: u32, deps: Vec<String>) -> Result<(), WfError> {
        if self.stages.contains_key(name) { return Err(WfError::StageExists { name: name.to_string() }); }
        for d in &deps {
            if !self.stages.contains_key(d) { return Err(WfError::StageNotFound { name: d.clone() }); }
        }
        if self.has_cycle(name, &deps) { return Err(WfError::DependencyCycle { name: name.to_string() }); }
        self.stages.insert(name.to_string(), Stage { name: name.to_string(), deps, state: StageState::Pending, phase, items: Vec::new(), processed: 0 });
        Ok(())
    }

    fn has_cycle(&self, name: &str, deps: &[String]) -> bool {
        for d in deps {
            if d == name { return true; }
            if let Some(s) = self.stages.get(d) {
                if self.has_cycle(name, &s.deps) { return true; }
            }
        }
        false
    }

    pub fn enqueue(&mut self, stage: &str, item: u64) -> Result<(), WfError> {
        let s = self.stages.get_mut(stage).ok_or(WfError::StageNotFound { name: stage.to_string() })?;
        s.items.push(item);
        Ok(())
    }

    pub fn ready_stages(&mut self) -> Vec<String> {
        self.total_barrier_waits += 1;
        let mut ready = Vec::new();
        for (name, stage) in &self.stages {
            if stage.state != StageState::Pending { continue; }
            let all_deps_done = stage.deps.iter().all(|d| {
                self.stages.get(d).map(|s| s.state == StageState::Complete).unwrap_or(false)
            });
            if all_deps_done && !stage.items.is_empty() {
                ready.push(name.clone());
            }
        }
        for name in &ready {
            self.stages.get_mut(name).unwrap().state = StageState::Ready;
        }
        ready
    }

    pub fn execute(&mut self, stage: &str) -> Result<Vec<u64>, WfError> {
        let s = self.stages.get_mut(stage).ok_or(WfError::StageNotFound { name: stage.to_string() })?;
        if s.state == StageState::Complete { return Err(WfError::AlreadyComplete { name: stage.to_string() }); }
        if s.state == StageState::Pending { return Err(WfError::NotReady { name: stage.to_string() }); }
        s.state = StageState::Running;
        let items: Vec<u64> = s.items.drain(..).collect();
        let count = items.len();
        s.processed += count;
        s.state = StageState::Complete;
        self.total_stages_run += 1;
        self.total_items_processed += count as u64;
        Ok(items)
    }

    pub fn stage_state(&self, name: &str) -> Option<&StageState> { self.stages.get(name).map(|s| &s.state) }
    pub fn stage_phase(&self, name: &str) -> Option<u32> { self.stages.get(name).map(|s| s.phase) }
    pub fn stage_count(&self) -> usize { self.stages.len() }
    pub fn complete_count(&self) -> usize { self.stages.values().filter(|s| s.state == StageState::Complete).count() }
    pub fn total_stages_run(&self) -> u64 { self.total_stages_run }
    pub fn total_items_processed(&self) -> u64 { self.total_items_processed }
    pub fn total_barrier_waits(&self) -> u64 { self.total_barrier_waits }
}

impl Default for WavefrontEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_engine() { assert_eq!(WavefrontEngine::new().stage_count(), 0); }

    #[test]
    fn add_stages() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("fetch", 0, vec![]).unwrap();
        wf.add_stage("parse", 1, vec!["fetch".to_string()]).unwrap();
        assert_eq!(wf.stage_count(), 2);
    }

    #[test]
    fn execute_linear() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("a", 0, vec![]).unwrap();
        wf.add_stage("b", 1, vec!["a".to_string()]).unwrap();
        wf.enqueue("a", 1).unwrap();
        let ready = wf.ready_stages();
        assert!(ready.contains(&"a".to_string()));
        let items = wf.execute("a").unwrap();
        assert_eq!(items, vec![1]);
        wf.enqueue("b", 2).unwrap();
        let ready = wf.ready_stages();
        assert!(ready.contains(&"b".to_string()));
        wf.execute("b").unwrap();
        assert_eq!(wf.complete_count(), 2);
    }

    #[test]
    fn dependency_not_ready() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("a", 0, vec![]).unwrap();
        wf.add_stage("b", 1, vec!["a".to_string()]).unwrap();
        wf.enqueue("b", 1).unwrap();
        let ready = wf.ready_stages();
        assert!(!ready.contains(&"b".to_string()));
    }

    #[test]
    fn self_dependency() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("a", 0, vec![]).unwrap();
        wf.add_stage("b", 0, vec!["a".to_string()]).unwrap();
        let err = wf.add_stage("a", 0, vec!["b".to_string()]).unwrap_err();
        assert!(matches!(err, WfError::StageExists { .. }));
    }

    #[test]
    fn already_complete() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("a", 0, vec![]).unwrap();
        wf.enqueue("a", 1).unwrap();
        wf.ready_stages();
        wf.execute("a").unwrap();
        let err = wf.execute("a").unwrap_err();
        assert!(matches!(err, WfError::AlreadyComplete { .. }));
    }

    #[test]
    fn not_ready() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("a", 0, vec![]).unwrap();
        wf.enqueue("a", 1).unwrap();
        let err = wf.execute("a").unwrap_err();
        assert!(matches!(err, WfError::NotReady { .. }));
    }

    #[test]
    fn duplicate_stage() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("x", 0, vec![]).unwrap();
        let err = wf.add_stage("x", 0, vec![]).unwrap_err();
        assert!(matches!(err, WfError::StageExists { .. }));
    }

    #[test]
    fn stats() {
        let mut wf = WavefrontEngine::new();
        wf.add_stage("a", 0, vec![]).unwrap();
        wf.enqueue("a", 1).unwrap(); wf.enqueue("a", 2).unwrap();
        wf.ready_stages();
        wf.execute("a").unwrap();
        assert_eq!(wf.total_stages_run(), 1);
        assert_eq!(wf.total_items_processed(), 2);
    }

    #[test]
    fn error_display() { assert!(WfError::StageNotFound { name: "x".into() }.to_string().contains("x")); }
}
