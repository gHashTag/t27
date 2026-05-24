use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl std::fmt::Display for WorkPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkPriority::Low => write!(f, "low"),
            WorkPriority::Normal => write!(f, "normal"),
            WorkPriority::High => write!(f, "high"),
            WorkPriority::Critical => write!(f, "critical"),
        }
    }
}

pub type WorkId = u64;
pub type WorkFn = fn(WorkId);

#[derive(Debug, Clone)]
pub struct WorkItem {
    pub id: WorkId,
    pub name: String,
    pub priority: WorkPriority,
    pub cost: u32,
    pub work_fn: WorkFn,
    pub submitted_us: u64,
}

impl WorkItem {
    pub fn new(name: &str, priority: WorkPriority, cost: u32, work_fn: WorkFn, submitted_us: u64) -> Self {
        static mut NEXT: WorkId = 1;
        let id = unsafe { let i = NEXT; NEXT += 1; i };
        Self { id, name: name.to_string(), priority, cost, work_fn, submitted_us }
    }
}

#[derive(Debug, Clone)]
pub struct WorkResult {
    pub id: WorkId,
    pub name: String,
    pub priority: WorkPriority,
    pub cost: u32,
    pub wait_us: u64,
}

#[derive(Debug, Clone)]
pub struct WorkloadScheduler {
    queues: BTreeMap<WorkPriority, Vec<WorkItem>>,
    budget_per_cycle: u32,
    spent_this_cycle: u32,
    total_dispatched: u64,
    total_cost: u64,
    total_wait_us: u64,
}

impl WorkloadScheduler {
    pub fn new(budget_per_cycle: u32) -> Self {
        Self {
            queues: BTreeMap::new(),
            budget_per_cycle,
            spent_this_cycle: 0,
            total_dispatched: 0,
            total_cost: 0,
            total_wait_us: 0,
        }
    }

    pub fn submit(&mut self, item: WorkItem) -> WorkId {
        let id = item.id;
        self.queues.entry(item.priority).or_default().push(item);
        id
    }

    pub fn dispatch(&mut self, now_us: u64) -> Option<WorkResult> {
        for pri in [WorkPriority::Critical, WorkPriority::High, WorkPriority::Normal, WorkPriority::Low] {
            if let Some(queue) = self.queues.get_mut(&pri) {
                if let Some(item) = queue.first() {
                    if self.spent_this_cycle + item.cost > self.budget_per_cycle {
                        continue;
                    }
                    let item = queue.remove(0);
                    self.spent_this_cycle += item.cost;
                    let wait_us = now_us.saturating_sub(item.submitted_us);
                    (item.work_fn)(item.id);
                    let result = WorkResult {
                        id: item.id,
                        name: item.name.clone(),
                        priority: item.priority,
                        cost: item.cost,
                        wait_us,
                    };
                    self.total_dispatched += 1;
                    self.total_cost += item.cost as u64;
                    self.total_wait_us += wait_us;
                    return Some(result);
                }
            }
        }
        None
    }

    pub fn dispatch_all(&mut self, now_us: u64) -> Vec<WorkResult> {
        let mut results = Vec::new();
        while let Some(r) = self.dispatch(now_us) {
            results.push(r);
        }
        results
    }

    pub fn new_cycle(&mut self) {
        self.spent_this_cycle = 0;
    }

    pub fn pending_count(&self) -> usize {
        self.queues.values().map(|q| q.len()).sum()
    }

    pub fn pending_by_priority(&self, pri: WorkPriority) -> usize {
        self.queues.get(&pri).map_or(0, |q| q.len())
    }

    pub fn budget_per_cycle(&self) -> u32 {
        self.budget_per_cycle
    }

    pub fn spent_this_cycle(&self) -> u32 {
        self.spent_this_cycle
    }

    pub fn remaining_budget(&self) -> u32 {
        self.budget_per_cycle.saturating_sub(self.spent_this_cycle)
    }

    pub fn total_dispatched(&self) -> u64 {
        self.total_dispatched
    }

    pub fn total_cost(&self) -> u64 {
        self.total_cost
    }

    pub fn avg_wait_us(&self) -> f64 {
        if self.total_dispatched == 0 { 0.0 } else { self.total_wait_us as f64 / self.total_dispatched as f64 }
    }

    pub fn clear(&mut self) {
        self.queues.clear();
        self.spent_this_cycle = 0;
    }
}

static mut DISPATCHED: u64 = 0;
fn test_work(_id: WorkId) { unsafe { DISPATCHED += 1; } }

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() { unsafe { DISPATCHED = 0; } }

    #[test]
    fn priority_display() {
        assert_eq!(WorkPriority::Critical.to_string(), "critical");
    }

    #[test]
    fn submit_and_pending() {
        let mut ws = WorkloadScheduler::new(100);
        ws.submit(WorkItem::new("a", WorkPriority::Normal, 10, test_work, 0));
        assert_eq!(ws.pending_count(), 1);
        assert_eq!(ws.pending_by_priority(WorkPriority::Normal), 1);
    }

    #[test]
    fn dispatch_highest_priority_first() {
        setup();
        let mut ws = WorkloadScheduler::new(100);
        ws.submit(WorkItem::new("low", WorkPriority::Low, 10, test_work, 0));
        ws.submit(WorkItem::new("crit", WorkPriority::Critical, 10, test_work, 0));
        let r = ws.dispatch(0).unwrap();
        assert_eq!(r.name, "crit");
        assert_eq!(r.priority, WorkPriority::Critical);
    }

    #[test]
    fn dispatch_respects_budget() {
        setup();
        let mut ws = WorkloadScheduler::new(15);
        ws.submit(WorkItem::new("a", WorkPriority::Normal, 10, test_work, 0));
        ws.submit(WorkItem::new("b", WorkPriority::Normal, 10, test_work, 0));
        let r1 = ws.dispatch(0).unwrap();
        assert_eq!(r1.name, "a");
        let r2 = ws.dispatch(0);
        assert!(r2.is_none());
        assert_eq!(ws.remaining_budget(), 5);
    }

    #[test]
    fn new_cycle_resets_budget() {
        setup();
        let mut ws = WorkloadScheduler::new(15);
        ws.submit(WorkItem::new("a", WorkPriority::Normal, 10, test_work, 0));
        ws.dispatch(0).unwrap();
        ws.new_cycle();
        assert_eq!(ws.remaining_budget(), 15);
    }

    #[test]
    fn dispatch_all_within_budget() {
        setup();
        let mut ws = WorkloadScheduler::new(30);
        ws.submit(WorkItem::new("a", WorkPriority::High, 10, test_work, 0));
        ws.submit(WorkItem::new("b", WorkPriority::Normal, 10, test_work, 0));
        ws.submit(WorkItem::new("c", WorkPriority::Low, 5, test_work, 0));
        let results = ws.dispatch_all(0);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].priority, WorkPriority::High);
        assert_eq!(results[2].priority, WorkPriority::Low);
    }

    #[test]
    fn wait_time_tracking() {
        setup();
        let mut ws = WorkloadScheduler::new(100);
        ws.submit(WorkItem::new("a", WorkPriority::Normal, 10, test_work, 100));
        let r = ws.dispatch(200).unwrap();
        assert_eq!(r.wait_us, 100);
        assert!((ws.avg_wait_us() - 100.0).abs() < 0.01);
    }

    #[test]
    fn stats() {
        setup();
        let mut ws = WorkloadScheduler::new(100);
        ws.submit(WorkItem::new("a", WorkPriority::Normal, 25, test_work, 0));
        ws.dispatch(0).unwrap();
        assert_eq!(ws.total_dispatched(), 1);
        assert_eq!(ws.total_cost(), 25);
    }

    #[test]
    fn empty_dispatch() {
        let mut ws = WorkloadScheduler::new(100);
        assert!(ws.dispatch(0).is_none());
    }

    #[test]
    fn clear() {
        let mut ws = WorkloadScheduler::new(100);
        ws.submit(WorkItem::new("a", WorkPriority::Normal, 10, test_work, 0));
        ws.clear();
        assert_eq!(ws.pending_count(), 0);
    }
}
