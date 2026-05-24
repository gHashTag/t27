use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum BatchError {
    JobExists { id: u64 },
    JobNotFound { id: u64 },
    JobNotPending { id: u64 },
    QueueFull { capacity: usize },
}

impl std::fmt::Display for BatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchError::JobExists { id } => write!(f, "job {id} exists"),
            BatchError::JobNotFound { id } => write!(f, "job {id} not found"),
            BatchError::JobNotPending { id } => write!(f, "job {id} not pending"),
            BatchError::QueueFull { capacity } => write!(f, "queue full ({capacity})"),
        }
    }
}

impl std::error::Error for BatchError {}

#[derive(Debug, Clone, PartialEq)]
pub enum JobState { Pending, Running, Completed, Cancelled }

struct Job {
    id: u64,
    priority: u32,
    group: u64,
    state: JobState,
    weight: u32,
    runtime_ticks: u64,
}

pub struct BatchScheduler {
    jobs: BTreeMap<u64, Job>,
    capacity: usize,
    quantum: u32,
    group_quanta: BTreeMap<u64, u32>,
    group_used: BTreeMap<u64, u32>,
    next_id: u64,
    total_scheduled: u64,
    total_completed: u64,
    total_cancelled: u64,
    total_preemptions: u64,
}

impl BatchScheduler {
    pub fn new(capacity: usize, quantum: u32) -> Self {
        Self { jobs: BTreeMap::new(), capacity, quantum, group_quanta: BTreeMap::new(), group_used: BTreeMap::new(), next_id: 1, total_scheduled: 0, total_completed: 0, total_cancelled: 0, total_preemptions: 0 }
    }

    pub fn set_group_quantum(&mut self, group: u64, quantum: u32) { self.group_quanta.insert(group, quantum); }

    pub fn submit(&mut self, priority: u32, group: u64, weight: u32) -> Result<u64, BatchError> {
        let pending = self.jobs.values().filter(|j| j.state == JobState::Pending).count();
        if pending >= self.capacity { return Err(BatchError::QueueFull { capacity: self.capacity }); }
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.insert(id, Job { id, priority, group, state: JobState::Pending, weight, runtime_ticks: 0 });
        self.total_scheduled += 1;
        Ok(id)
    }

    pub fn schedule(&mut self) -> Option<u64> {
        let running = self.jobs.values().filter(|j| j.state == JobState::Running).count();
        if running > 0 {
            let running_ids: Vec<u64> = self.jobs.iter().filter(|(_, j)| j.state == JobState::Running).map(|(&id, _)| id).collect();
            for &rid in &running_ids {
                let j = self.jobs.get_mut(&rid).unwrap();
                j.runtime_ticks += 1;
                let gq = self.group_quanta.get(&j.group).copied().unwrap_or(self.quantum);
                let used = self.group_used.entry(j.group).or_insert(0);
                *used += 1;
                if *used >= gq {
                    *used = 0;
                    j.state = JobState::Pending;
                    self.total_preemptions += 1;
                } else if j.runtime_ticks >= j.weight as u64 {
                    j.state = JobState::Completed;
                    self.total_completed += 1;
                    self.group_used.entry(j.group).and_modify(|u| *u = 0);
                }
            }
        }
        let has_running = self.jobs.values().any(|j| j.state == JobState::Running);
        if has_running { return self.jobs.iter().filter(|(_, j)| j.state == JobState::Running).map(|(&id, _)| id).next(); }
        let best = self.jobs.iter()
            .filter(|(_, j)| j.state == JobState::Pending)
            .max_by(|a, b| a.1.priority.cmp(&b.1.priority).then_with(|| a.1.id.cmp(&b.1.id)))
            .map(|(&id, _)| id);
        if let Some(id) = best { self.jobs.get_mut(&id).unwrap().state = JobState::Running; }
        best
    }

    pub fn cancel(&mut self, id: u64) -> Result<(), BatchError> {
        let j = self.jobs.get_mut(&id).ok_or(BatchError::JobNotFound { id })?;
        if j.state != JobState::Pending && j.state != JobState::Running { return Err(BatchError::JobNotPending { id }); }
        j.state = JobState::Cancelled;
        self.total_cancelled += 1;
        Ok(())
    }

    pub fn complete(&mut self, id: u64) -> Result<(), BatchError> {
        let j = self.jobs.get_mut(&id).ok_or(BatchError::JobNotFound { id })?;
        if j.state != JobState::Running { return Err(BatchError::JobNotPending { id }); }
        j.state = JobState::Completed;
        self.total_completed += 1;
        Ok(())
    }

    pub fn state(&self, id: u64) -> Option<JobState> { self.jobs.get(&id).map(|j| j.state.clone()) }
    pub fn job_count(&self) -> usize { self.jobs.len() }
    pub fn pending_count(&self) -> usize { self.jobs.values().filter(|j| j.state == JobState::Pending).count() }
    pub fn running_count(&self) -> usize { self.jobs.values().filter(|j| j.state == JobState::Running).count() }
    pub fn total_scheduled(&self) -> u64 { self.total_scheduled }
    pub fn total_completed(&self) -> u64 { self.total_completed }
    pub fn total_cancelled(&self) -> u64 { self.total_cancelled }
    pub fn total_preemptions(&self) -> u64 { self.total_preemptions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sched() { let s = BatchScheduler::new(10, 4); assert_eq!(s.job_count(), 0); }

    #[test]
    fn submit_schedule() {
        let mut s = BatchScheduler::new(10, 4);
        let id = s.submit(5, 1, 1).unwrap();
        let sid = s.schedule().unwrap();
        assert_eq!(sid, id);
        assert_eq!(s.state(id), Some(JobState::Running));
    }

    #[test]
    fn priority_order() {
        let mut s = BatchScheduler::new(10, 4);
        let lo = s.submit(1, 1, 1).unwrap();
        let hi = s.submit(10, 1, 1).unwrap();
        let first = s.schedule().unwrap();
        assert_eq!(first, hi);
    }

    #[test]
    fn complete_job() {
        let mut s = BatchScheduler::new(10, 4);
        let id = s.submit(5, 1, 1).unwrap();
        s.schedule().unwrap();
        s.complete(id).unwrap();
        assert_eq!(s.state(id), Some(JobState::Completed));
        assert_eq!(s.total_completed(), 1);
    }

    #[test]
    fn cancel_job() {
        let mut s = BatchScheduler::new(10, 4);
        let id = s.submit(5, 1, 1).unwrap();
        s.cancel(id).unwrap();
        assert_eq!(s.state(id), Some(JobState::Cancelled));
    }

    #[test]
    fn queue_full() {
        let mut s = BatchScheduler::new(2, 4);
        s.submit(1, 1, 1).unwrap();
        s.submit(1, 1, 1).unwrap();
        let err = s.submit(1, 1, 1).unwrap_err();
        assert!(matches!(err, BatchError::QueueFull { .. }));
    }

    #[test]
    fn not_found() {
        let mut s = BatchScheduler::new(10, 4);
        let err = s.cancel(99).unwrap_err();
        assert!(matches!(err, BatchError::JobNotFound { .. }));
    }

    #[test]
    fn preemption() {
        let mut s = BatchScheduler::new(10, 2);
        let id = s.submit(5, 1, 10).unwrap();
        s.schedule().unwrap();
        s.schedule();
        s.schedule();
        assert!(s.total_preemptions() > 0);
    }

    #[test]
    fn group_quantum() {
        let mut s = BatchScheduler::new(10, 100);
        s.set_group_quantum(1, 1);
        let a = s.submit(5, 1, 10).unwrap();
        let b = s.submit(5, 2, 10).unwrap();
        s.schedule().unwrap();
        let next = s.schedule().unwrap();
        assert_ne!(next, a);
    }

    #[test]
    fn stats() {
        let mut s = BatchScheduler::new(10, 4);
        s.submit(5, 1, 1).unwrap();
        assert_eq!(s.total_scheduled(), 1);
    }

    #[test]
    fn error_display() { assert!(BatchError::JobNotFound { id: 3 }.to_string().contains("3")); }
}
