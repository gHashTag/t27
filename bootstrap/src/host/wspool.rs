use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum WsError {
    PoolFull { worker: u64 },
    PoolEmpty,
    WorkerNotFound { worker: u64 },
    InvalidWorker { worker: u64 },
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::PoolFull { worker } => write!(f, "worker {worker} pool full"),
            WsError::PoolEmpty => write!(f, "all pools empty"),
            WsError::WorkerNotFound { worker } => write!(f, "worker {worker} not found"),
            WsError::InvalidWorker { worker } => write!(f, "invalid worker {worker}"),
        }
    }
}

impl std::error::Error for WsError {}

struct Worker<Q> {
    id: u64,
    local: VecDeque<Q>,
    capacity: usize,
    pushes: u64,
    pops: u64,
    stolen_from: u64,
    stolen_by: u64,
}

#[derive(Debug, Clone)]
pub struct WsStats {
    pub total_pushes: u64,
    pub total_pops: u64,
    pub total_steals: u64,
    pub total_stolen_items: u64,
}

pub struct WsPool<Q> {
    workers: Vec<Worker<Q>>,
    steal_idx: usize,
}

impl<Q> WsPool<Q> {
    pub fn new(worker_count: usize, per_worker_cap: usize) -> Self {
        let workers = (0..worker_count).map(|i| Worker {
            id: i as u64, local: VecDeque::new(), capacity: per_worker_cap, pushes: 0, pops: 0, stolen_from: 0, stolen_by: 0,
        }).collect();
        Self { workers, steal_idx: 0 }
    }

    pub fn push(&mut self, worker: u64, item: Q) -> Result<(), WsError> {
        let w = self.workers.get_mut(worker as usize).ok_or(WsError::WorkerNotFound { worker })?;
        if w.local.len() >= w.capacity { return Err(WsError::PoolFull { worker }); }
        w.local.push_back(item);
        w.pushes += 1;
        Ok(())
    }

    pub fn pop(&mut self, worker: u64) -> Option<Q> {
        let w = self.workers.get_mut(worker as usize)?;
        if let Some(item) = w.local.pop_front() {
            w.pops += 1;
            return Some(item);
        }
        None
    }

    pub fn steal(&mut self, thief: u64) -> Result<Vec<Q>, WsError> {
        if self.workers.is_empty() { return Err(WsError::PoolEmpty); }
        let thief_idx = thief as usize;
        if thief_idx >= self.workers.len() { return Err(WsError::InvalidWorker { worker: thief }); }
        let mut stolen = Vec::new();
        for _ in 0..self.workers.len() {
            let victim_idx = self.steal_idx % self.workers.len();
            self.steal_idx += 1;
            if victim_idx == thief_idx { continue; }
            let victim_len = self.workers[victim_idx].local.len();
            if victim_len <= 1 { continue; }
            let take = victim_len / 2;
            let items: Vec<Q> = (0..take).filter_map(|_| self.workers[victim_idx].local.pop_front()).collect();
            stolen.extend(items);
            self.workers[thief_idx].stolen_by += 1;
            self.workers[victim_idx].stolen_from += 1;
            return Ok(stolen);
        }
        Err(WsError::PoolEmpty)
    }

    pub fn len(&self, worker: u64) -> Option<usize> {
        self.workers.get(worker as usize).map(|w| w.local.len())
    }

    pub fn total_len(&self) -> usize {
        self.workers.iter().map(|w| w.local.len()).sum()
    }

    pub fn worker_count(&self) -> usize { self.workers.len() }

    pub fn stats(&self) -> WsStats {
        WsStats {
            total_pushes: self.workers.iter().map(|w| w.pushes).sum(),
            total_pops: self.workers.iter().map(|w| w.pops).sum(),
            total_steals: self.workers.iter().map(|w| w.stolen_by).sum(),
            total_stolen_items: self.workers.iter().map(|w| w.stolen_from).sum(),
        }
    }

    pub fn is_empty(&self) -> bool { self.total_len() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() {
        let p: WsPool<i32> = WsPool::new(4, 8);
        assert_eq!(p.worker_count(), 4);
        assert!(p.is_empty());
    }

    #[test]
    fn push_pop() {
        let mut p = WsPool::new(2, 8);
        p.push(0, 42).unwrap();
        p.push(0, 43).unwrap();
        assert_eq!(p.len(0), Some(2));
        assert_eq!(p.pop(0), Some(42));
        assert_eq!(p.pop(0), Some(43));
        assert_eq!(p.pop(0), None);
    }

    #[test]
    fn pool_full() {
        let mut p = WsPool::new(1, 2);
        p.push(0, 1).unwrap();
        p.push(0, 2).unwrap();
        let err = p.push(0, 3).unwrap_err();
        assert!(matches!(err, WsError::PoolFull { .. }));
    }

    #[test]
    fn steal_half() {
        let mut p = WsPool::new(2, 16);
        for i in 0..8 { p.push(0, i).unwrap(); }
        let stolen = p.steal(1).unwrap();
        assert_eq!(stolen.len(), 4);
        assert_eq!(p.len(0), Some(4));
    }

    #[test]
    fn steal_empty() {
        let mut p: WsPool<i32> = WsPool::new(2, 8);
        let err = p.steal(0).unwrap_err();
        assert!(matches!(err, WsError::PoolEmpty));
    }

    #[test]
    fn steal_from_self_skip() {
        let mut p = WsPool::new(2, 16);
        for i in 0..10 { p.push(1, i).unwrap(); }
        let stolen = p.steal(0).unwrap();
        assert_eq!(stolen.len(), 5);
    }

    #[test]
    fn worker_not_found() {
        let mut p: WsPool<i32> = WsPool::new(2, 8);
        let err = p.push(99, 1).unwrap_err();
        assert!(matches!(err, WsError::WorkerNotFound { .. }));
    }

    #[test]
    fn total_len() {
        let mut p = WsPool::new(3, 16);
        p.push(0, 1).unwrap(); p.push(1, 2).unwrap(); p.push(2, 3).unwrap();
        assert_eq!(p.total_len(), 3);
    }

    #[test]
    fn stats() {
        let mut p = WsPool::new(2, 16);
        p.push(0, 1).unwrap(); p.push(0, 2).unwrap();
        p.pop(0).unwrap();
        let s = p.stats();
        assert_eq!(s.total_pushes, 2);
        assert_eq!(s.total_pops, 1);
    }

    #[test]
    fn multi_steal() {
        let mut p = WsPool::new(3, 32);
        for i in 0..20 { p.push(0, i).unwrap(); }
        let s1 = p.steal(1).unwrap();
        let s2 = p.steal(2).unwrap();
        assert_eq!(s1.len() + s2.len() + p.len(0).unwrap(), 20);
    }

    #[test]
    fn single_item_no_steal() {
        let mut p = WsPool::new(2, 16);
        p.push(0, 42).unwrap();
        let err = p.steal(1).unwrap_err();
        assert!(matches!(err, WsError::PoolEmpty));
    }

    #[test]
    fn error_display() {
        assert!(WsError::PoolEmpty.to_string().contains("empty"));
    }
}
