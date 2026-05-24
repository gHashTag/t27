use std::collections::BTreeMap;

pub type LaneId = u8;
pub type JobId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneState {
    Idle,
    Busy,
    Stalled,
}

impl std::fmt::Display for LaneState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LaneState::Idle => write!(f, "idle"),
            LaneState::Busy => write!(f, "busy"),
            LaneState::Stalled => write!(f, "stalled"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: JobId,
    pub name: String,
    pub cost: u32,
    pub affinity: Option<LaneId>,
}

impl Job {
    pub fn new(name: &str, cost: u32) -> Self {
        static mut NEXT: JobId = 1;
        let id = unsafe { let i = NEXT; NEXT += 1; i };
        Self { id, name: name.to_string(), cost, affinity: None }
    }

    pub fn with_affinity(mut self, lane: LaneId) -> Self {
        self.affinity = Some(lane);
        self
    }
}

#[derive(Debug, Clone)]
pub struct LaneStats {
    pub id: LaneId,
    pub jobs_completed: u64,
    pub total_cost: u64,
    pub utilization: f64,
}

#[derive(Debug, Clone)]
struct Lane {
    id: LaneId,
    state: LaneState,
    current_job: Option<JobId>,
    queue: Vec<Job>,
    depth: usize,
    jobs_completed: u64,
    total_cost: u64,
}

#[derive(Debug, Clone)]
pub struct LaneScheduler {
    lanes: BTreeMap<LaneId, Lane>,
    total_dispatched: u64,
    total_ticks: u64,
}

impl LaneScheduler {
    pub fn new() -> Self {
        Self {
            lanes: BTreeMap::new(),
            total_dispatched: 0,
            total_ticks: 0,
        }
    }

    pub fn add_lane(&mut self, id: LaneId, depth: usize) {
        self.lanes.insert(id, Lane {
            id, state: LaneState::Idle, current_job: None,
            queue: Vec::with_capacity(depth), depth,
            jobs_completed: 0, total_cost: 0,
        });
    }

    pub fn remove_lane(&mut self, id: LaneId) -> bool {
        self.lanes.remove(&id).is_some()
    }

    pub fn submit(&mut self, job: Job) -> Result<LaneId, String> {
        if let Some(lane_id) = job.affinity {
            let lane = self.lanes.get_mut(&lane_id)
                .ok_or_else(|| format!("lane {lane_id} not found"))?;
            if lane.queue.len() >= lane.depth {
                return Err(format!("lane {lane_id} full"));
            }
            lane.queue.push(job);
            return Ok(lane_id);
        }
        let best = self.lanes.values()
            .filter(|l| l.queue.len() < l.depth)
            .min_by_key(|l| l.queue.len());
        match best {
            Some(l) => {
                let lane_id = l.id;
                self.lanes.get_mut(&lane_id).unwrap().queue.push(job);
                Ok(lane_id)
            }
            None => Err("all lanes full".into()),
        }
    }

    pub fn dispatch(&mut self) -> Vec<(LaneId, Job)> {
        let mut dispatched = Vec::new();
        for lane in self.lanes.values_mut() {
            if lane.state != LaneState::Idle {
                continue;
            }
            if let Some(job) = lane.queue.first() {
                let j = job.clone();
                lane.state = LaneState::Busy;
                lane.current_job = Some(j.id);
                lane.queue.remove(0);
                dispatched.push((lane.id, j));
                self.total_dispatched += 1;
            }
        }
        dispatched
    }

    pub fn complete(&mut self, lane_id: LaneId) -> Option<JobId> {
        let lane = self.lanes.get_mut(&lane_id)?;
        if lane.state != LaneState::Busy {
            return None;
        }
        let job_id = lane.current_job.take();
        lane.state = LaneState::Idle;
        if let Some(jid) = job_id {
            lane.jobs_completed += 1;
        }
        job_id
    }

    pub fn complete_with_cost(&mut self, lane_id: LaneId, cost: u32) -> Option<JobId> {
        let job_id = self.complete(lane_id);
        if job_id.is_some() {
            self.lanes.get_mut(&lane_id).unwrap().total_cost += cost as u64;
        }
        job_id
    }

    pub fn tick(&mut self) {
        self.total_ticks += 1;
    }

    pub fn lane_state(&self, id: LaneId) -> Option<LaneState> {
        self.lanes.get(&id).map(|l| l.state)
    }

    pub fn lane_queue_depth(&self, id: LaneId) -> Option<usize> {
        self.lanes.get(&id).map(|l| l.queue.len())
    }

    pub fn lane_count(&self) -> usize {
        self.lanes.len()
    }

    pub fn idle_count(&self) -> usize {
        self.lanes.values().filter(|l| l.state == LaneState::Idle).count()
    }

    pub fn busy_count(&self) -> usize {
        self.lanes.values().filter(|l| l.state == LaneState::Busy).count()
    }

    pub fn lane_stats(&self, id: LaneId) -> Option<LaneStats> {
        self.lanes.get(&id).map(|l| {
            let util = if self.total_ticks == 0 { 0.0 }
            else { l.jobs_completed as f64 / self.total_ticks as f64 };
            LaneStats { id: l.id, jobs_completed: l.jobs_completed, total_cost: l.total_cost, utilization: util }
        })
    }

    pub fn all_lane_stats(&self) -> Vec<LaneStats> {
        self.lanes.keys().filter_map(|&id| self.lane_stats(id)).collect()
    }

    pub fn total_dispatched(&self) -> u64 {
        self.total_dispatched
    }
}

impl Default for LaneScheduler {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_state_display() {
        assert_eq!(LaneState::Busy.to_string(), "busy");
    }

    #[test]
    fn add_lane() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        ls.add_lane(1, 4);
        assert_eq!(ls.lane_count(), 2);
        assert_eq!(ls.idle_count(), 2);
    }

    #[test]
    fn remove_lane() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        assert!(ls.remove_lane(0));
        assert_eq!(ls.lane_count(), 0);
    }

    #[test]
    fn submit_least_loaded() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        ls.add_lane(1, 4);
        ls.submit(Job::new("a", 10)).unwrap();
        ls.submit(Job::new("b", 10)).unwrap();
        assert_eq!(ls.lane_queue_depth(0).unwrap(), 1);
        assert_eq!(ls.lane_queue_depth(1).unwrap(), 1);
    }

    #[test]
    fn submit_with_affinity() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        ls.add_lane(1, 4);
        let lane = ls.submit(Job::new("a", 10).with_affinity(1)).unwrap();
        assert_eq!(lane, 1);
        assert_eq!(ls.lane_queue_depth(1).unwrap(), 1);
        assert_eq!(ls.lane_queue_depth(0).unwrap(), 0);
    }

    #[test]
    fn dispatch_and_complete() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        ls.submit(Job::new("a", 10)).unwrap();
        ls.tick();
        let dispatched = ls.dispatch();
        assert_eq!(dispatched.len(), 1);
        assert_eq!(ls.busy_count(), 1);
        ls.complete(0);
        assert_eq!(ls.idle_count(), 1);
        assert_eq!(ls.lane_stats(0).unwrap().jobs_completed, 1);
    }

    #[test]
    fn complete_with_cost() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        ls.submit(Job::new("a", 10)).unwrap();
        ls.dispatch();
        ls.complete_with_cost(0, 10);
        assert_eq!(ls.lane_stats(0).unwrap().total_cost, 10);
    }

    #[test]
    fn full_lane_rejects() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 1);
        ls.submit(Job::new("a", 10).with_affinity(0)).unwrap();
        let err = ls.submit(Job::new("b", 10).with_affinity(0));
        assert!(err.is_err());
    }

    #[test]
    fn total_dispatched() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        ls.add_lane(1, 4);
        ls.submit(Job::new("a", 10)).unwrap();
        ls.submit(Job::new("b", 10)).unwrap();
        ls.dispatch();
        assert_eq!(ls.total_dispatched(), 2);
    }

    #[test]
    fn all_lane_stats() {
        let mut ls = LaneScheduler::new();
        ls.add_lane(0, 4);
        ls.add_lane(1, 4);
        assert_eq!(ls.all_lane_stats().len(), 2);
    }
}
