use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeadlineError {
    DuplicateId { id: u64 },
    NotFound { id: u64 },
}

impl std::fmt::Display for DeadlineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeadlineError::DuplicateId { id } => write!(f, "duplicate task {id}"),
            DeadlineError::NotFound { id } => write!(f, "task {id} not found"),
        }
    }
}

impl std::error::Error for DeadlineError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Running,
    Completed,
    Overrun,
}

#[derive(Debug, Clone)]
struct Task {
    id: u64,
    deadline: u64,
    state: TaskState,
    created_at: u64,
}

#[derive(Debug, Clone)]
pub struct DeadlineStats {
    pub total_scheduled: u64,
    pub total_completed: u64,
    pub total_overrun: u64,
    pub pending_count: usize,
}

#[derive(Debug, Clone)]
pub struct DeadlineScheduler {
    tasks: BTreeMap<u64, Task>,
    deadline_index: BTreeMap<(u64, u64), u64>,
    now: u64,
    total_scheduled: u64,
    total_completed: u64,
    total_overrun: u64,
}

impl DeadlineScheduler {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            deadline_index: BTreeMap::new(),
            now: 0,
            total_scheduled: 0,
            total_completed: 0,
            total_overrun: 0,
        }
    }

    pub fn tick(&mut self) -> u64 {
        self.now += 1;
        self.check_overruns();
        self.now
    }

    pub fn set_now(&mut self, now: u64) {
        self.now = now;
        self.check_overruns();
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn schedule(&mut self, id: u64, deadline: u64) -> Result<(), DeadlineError> {
        if self.tasks.contains_key(&id) {
            return Err(DeadlineError::DuplicateId { id });
        }
        let task = Task { id, deadline, state: TaskState::Pending, created_at: self.now };
        self.deadline_index.insert((deadline, id), id);
        self.tasks.insert(id, task);
        self.total_scheduled += 1;
        Ok(())
    }

    pub fn dispatch(&mut self) -> Option<u64> {
        let candidate = self.deadline_index.iter()
            .find(|(_, &id)| {
                self.tasks.get(&id).map(|t| t.state == TaskState::Pending).unwrap_or(false)
            })
            .map(|(&key, _)| key);
        if let Some((deadline, id)) = candidate {
            if let Some(task) = self.tasks.get_mut(&id) {
                task.state = TaskState::Running;
            }
            return Some(id);
        }
        None
    }

    pub fn complete(&mut self, id: u64) -> Result<TaskState, DeadlineError> {
        let task = self.tasks.get_mut(&id).ok_or(DeadlineError::NotFound { id })?;
        if task.state == TaskState::Completed || task.state == TaskState::Overrun {
            return Err(DeadlineError::NotFound { id });
        }
        if self.now > task.deadline {
            task.state = TaskState::Overrun;
            self.total_overrun += 1;
        } else {
            task.state = TaskState::Completed;
        }
        self.total_completed += 1;
        self.deadline_index.remove(&(task.deadline, id));
        Ok(task.state.clone())
    }

    pub fn cancel(&mut self, id: u64) -> Result<(), DeadlineError> {
        let task = self.tasks.remove(&id).ok_or(DeadlineError::NotFound { id })?;
        self.deadline_index.remove(&(task.deadline, id));
        Ok(())
    }

    pub fn state(&self, id: u64) -> Option<&TaskState> {
        self.tasks.get(&id).map(|t| &t.state)
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.values().filter(|t| t.state == TaskState::Pending).count()
    }

    pub fn running_count(&self) -> usize {
        self.tasks.values().filter(|t| t.state == TaskState::Running).count()
    }

    pub fn next_deadline(&self) -> Option<u64> {
        self.deadline_index.iter()
            .find(|(_, &id)| {
                self.tasks.get(&id).map(|t| t.state == TaskState::Pending).unwrap_or(false)
            })
            .map(|((d, _), _)| *d)
    }

    fn check_overruns(&mut self) {
        for task in self.tasks.values_mut() {
            if task.state == TaskState::Pending && self.now > task.deadline {
                task.state = TaskState::Overrun;
            }
        }
    }

    pub fn stats(&self) -> DeadlineStats {
        DeadlineStats {
            total_scheduled: self.total_scheduled,
            total_completed: self.total_completed,
            total_overrun: self.total_overrun,
            pending_count: self.pending_count(),
        }
    }

    pub fn clear_completed(&mut self) -> usize {
        let done: Vec<u64> = self.tasks.iter()
            .filter(|(_, t)| t.state == TaskState::Completed || t.state == TaskState::Overrun)
            .map(|(&id, _)| id)
            .collect();
        let count = done.len();
        for id in &done {
            if let Some(task) = self.tasks.remove(id) {
                self.deadline_index.remove(&(task.deadline, *id));
            }
        }
        count
    }
}

impl Default for DeadlineScheduler {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scheduler() {
        let ds = DeadlineScheduler::new();
        assert_eq!(ds.now(), 0);
        assert_eq!(ds.pending_count(), 0);
    }

    #[test]
    fn schedule_and_dispatch() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 100).unwrap();
        ds.schedule(2, 50).unwrap();
        let first = ds.dispatch().unwrap();
        assert_eq!(first, 2);
        let second = ds.dispatch().unwrap();
        assert_eq!(second, 1);
    }

    #[test]
    fn duplicate_id() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 100).unwrap();
        let err = ds.schedule(1, 200).unwrap_err();
        assert!(matches!(err, DeadlineError::DuplicateId { .. }));
    }

    #[test]
    fn complete_on_time() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 100).unwrap();
        ds.dispatch().unwrap();
        ds.set_now(80);
        let state = ds.complete(1).unwrap();
        assert_eq!(state, TaskState::Completed);
    }

    #[test]
    fn complete_overrun() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 100).unwrap();
        ds.dispatch().unwrap();
        ds.set_now(150);
        let state = ds.complete(1).unwrap();
        assert_eq!(state, TaskState::Overrun);
    }

    #[test]
    fn cancel() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 100).unwrap();
        ds.cancel(1).unwrap();
        assert_eq!(ds.pending_count(), 0);
    }

    #[test]
    fn cancel_not_found() {
        let mut ds = DeadlineScheduler::new();
        let err = ds.cancel(99).unwrap_err();
        assert!(matches!(err, DeadlineError::NotFound { .. }));
    }

    #[test]
    fn tick_advances_time() {
        let mut ds = DeadlineScheduler::new();
        assert_eq!(ds.tick(), 1);
        assert_eq!(ds.tick(), 2);
    }

    #[test]
    fn next_deadline() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 200).unwrap();
        ds.schedule(2, 50).unwrap();
        assert_eq!(ds.next_deadline(), Some(50));
    }

    #[test]
    fn stats() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 100).unwrap();
        ds.schedule(2, 200).unwrap();
        ds.dispatch().unwrap();
        ds.set_now(80);
        ds.complete(1).unwrap();
        let s = ds.stats();
        assert_eq!(s.total_scheduled, 2);
        assert_eq!(s.total_completed, 1);
        assert_eq!(s.pending_count, 1);
    }

    #[test]
    fn clear_completed() {
        let mut ds = DeadlineScheduler::new();
        ds.schedule(1, 10).unwrap();
        ds.dispatch().unwrap();
        ds.set_now(5);
        ds.complete(1).unwrap();
        assert_eq!(ds.clear_completed(), 1);
        assert_eq!(ds.pending_count(), 0);
    }

    #[test]
    fn dispatch_none_when_empty() {
        let mut ds = DeadlineScheduler::new();
        assert_eq!(ds.dispatch(), None);
    }

    #[test]
    fn error_display() {
        assert!(DeadlineError::DuplicateId { id: 7 }.to_string().contains("7"));
    }
}
