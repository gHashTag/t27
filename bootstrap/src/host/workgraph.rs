use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum WgErr {
    Cycle { from: u64, to: u64 },
    NodeExists { id: u64 },
    NotFound { id: u64 },
    PendingDeps { id: u64, deps: Vec<u64> },
}

impl std::fmt::Display for WgErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgErr::Cycle { from, to } => write!(f, "cycle {from}->{to}"),
            WgErr::NodeExists { id } => write!(f, "node {id} exists"),
            WgErr::NotFound { id } => write!(f, "node {id} not found"),
            WgErr::PendingDeps { id, deps } => write!(f, "node {id} has pending deps {deps:?}"),
        }
    }
}

impl std::error::Error for WgErr {}

#[derive(Clone)]
struct Task {
    id: u64,
    payload: Vec<u8>,
    deps: BTreeSet<u64>,
    done: bool,
}

pub struct WorkGraph {
    tasks: BTreeMap<u64, Task>,
    total_executed: u64,
    total_added: u64,
}

impl WorkGraph {
    pub fn new() -> Self { Self { tasks: BTreeMap::new(), total_executed: 0, total_added: 0 } }

    pub fn add_task(&mut self, id: u64, payload: Vec<u8>, deps: &[u64]) -> Result<(), WgErr> {
        self.total_added += 1;
        if self.tasks.contains_key(&id) { return Err(WgErr::NodeExists { id }); }
        let mut dep_set = BTreeSet::new();
        for &d in deps { dep_set.insert(d); }
        self.tasks.insert(id, Task { id, payload, deps: dep_set, done: false });
        if self.has_cycle() {
            self.tasks.remove(&id);
            return Err(WgErr::Cycle { from: id, to: *deps.first().unwrap_or(&id) });
        }
        Ok(())
    }

    fn has_cycle(&self) -> bool {
        let mut visited = BTreeSet::new();
        let mut rec = BTreeSet::new();
        for &id in self.tasks.keys() {
            if self.dfs_cycle(id, &mut visited, &mut rec) { return true; }
        }
        false
    }

    fn dfs_cycle(&self, id: u64, visited: &mut BTreeSet<u64>, rec: &mut BTreeSet<u64>) -> bool {
        if visited.contains(&id) { return false; }
        visited.insert(id);
        rec.insert(id);
        if let Some(t) = self.tasks.get(&id) {
            for &dep in &t.deps {
                if !visited.contains(&dep) && self.dfs_cycle(dep, visited, rec) { return true; }
                if rec.contains(&dep) { return true; }
            }
        }
        rec.remove(&id);
        false
    }

    pub fn ready_tasks(&self) -> Vec<u64> {
        self.tasks.iter().filter(|(_, t)| !t.done && t.deps.iter().all(|d| self.tasks.get(d).map_or(false, |t| t.done))).map(|(id, _)| *id).collect()
    }

    pub fn complete(&mut self, id: u64) -> Result<Vec<u8>, WgErr> {
        let pending: Vec<u64> = {
            let t = self.tasks.get(&id).ok_or(WgErr::NotFound { id })?;
            t.deps.iter().filter(|&&d| !self.tasks.get(&d).map_or(false, |t| t.done)).copied().collect()
        };
        if !pending.is_empty() { return Err(WgErr::PendingDeps { id, deps: pending }); }
        let t = self.tasks.get_mut(&id).unwrap();
        let payload = t.payload.clone();
        t.done = true;
        self.total_executed += 1;
        Ok(payload)
    }

    pub fn topo_order(&self) -> Vec<u64> {
        let mut indeg: BTreeMap<u64, usize> = self.tasks.keys().map(|&id| (id, 0)).collect();
        for t in self.tasks.values() {
            for &_dep in &t.deps {
                if let Some(e) = indeg.get_mut(&t.id) { *e += 1; }
            }
        }
        let mut q: VecDeque<u64> = indeg.iter().filter(|(_, &d)| d == 0).map(|(&id, _)| id).collect();
        let mut result = Vec::new();
        while let Some(id) = q.pop_front() {
            result.push(id);
            for (&other_id, other) in &self.tasks {
                if other.deps.contains(&id) {
                    if let Some(e) = indeg.get_mut(&other_id) {
                        *e -= 1;
                        if *e == 0 { q.push_back(other_id); }
                    }
                }
            }
        }
        result
    }

    pub fn task_count(&self) -> usize { self.tasks.len() }
    pub fn done_count(&self) -> usize { self.tasks.values().filter(|t| t.done).count() }
    pub fn total_executed(&self) -> u64 { self.total_executed }
    pub fn total_added(&self) -> u64 { self.total_added }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_task() {
        let mut wg = WorkGraph::new();
        wg.add_task(1, b"a".to_vec(), &[]).unwrap();
        wg.add_task(2, b"b".to_vec(), &[1]).unwrap();
        assert_eq!(wg.task_count(), 2);
    }

    #[test]
    fn duplicate() {
        let mut wg = WorkGraph::new();
        wg.add_task(1, vec![], &[]).unwrap();
        assert!(wg.add_task(1, vec![], &[]).is_err());
    }

    #[test]
    fn ready_and_complete() {
        let mut wg = WorkGraph::new();
        wg.add_task(1, b"x".to_vec(), &[]).unwrap();
        wg.add_task(2, b"y".to_vec(), &[1]).unwrap();
        assert_eq!(wg.ready_tasks(), vec![1]);
        wg.complete(1).unwrap();
        assert_eq!(wg.ready_tasks(), vec![2]);
        wg.complete(2).unwrap();
        assert_eq!(wg.done_count(), 2);
    }

    #[test]
    fn pending_deps() {
        let mut wg = WorkGraph::new();
        wg.add_task(1, vec![], &[]).unwrap();
        wg.add_task(2, vec![], &[1]).unwrap();
        assert!(wg.complete(2).is_err());
    }

    #[test]
    fn topo_order() {
        let mut wg = WorkGraph::new();
        wg.add_task(3, vec![], &[1, 2]).unwrap();
        wg.add_task(1, vec![], &[]).unwrap();
        wg.add_task(2, vec![], &[]).unwrap();
        let order = wg.topo_order();
        assert!(order.iter().position(|&x| x == 1).unwrap() < order.iter().position(|&x| x == 3).unwrap());
    }

    #[test]
    fn cycle_detect() {
        let mut wg = WorkGraph::new();
        wg.add_task(1, vec![], &[]).unwrap();
        wg.add_task(2, vec![], &[1]).unwrap();
        assert!(wg.add_task(1, vec![], &[2]).is_err());
    }

    #[test]
    fn not_found() { assert!(WorkGraph::new().complete(1).is_err()); }

    #[test]
    fn stats() {
        let mut wg = WorkGraph::new();
        wg.add_task(1, vec![], &[]).unwrap();
        wg.complete(1).unwrap();
        assert_eq!(wg.total_added(), 1);
        assert_eq!(wg.total_executed(), 1);
    }

    #[test]
    fn error_display() { assert!(WgErr::Cycle { from: 1, to: 2 }.to_string().contains("cycle")); }
}
