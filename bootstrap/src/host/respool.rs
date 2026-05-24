#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    Exhausted { resource: String },
    NotFound { id: u64 },
    AlreadyReleased { id: u64 },
    InvalidCapacity { resource: String, capacity: usize },
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::Exhausted { resource } => write!(f, "pool exhausted: {resource}"),
            PoolError::NotFound { id } => write!(f, "resource not found: {id}"),
            PoolError::AlreadyReleased { id } => write!(f, "already released: {id}"),
            PoolError::InvalidCapacity { resource, capacity } => {
                write!(f, "invalid capacity {capacity} for {resource}")
            }
        }
    }
}

impl std::error::Error for PoolError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Free,
    Allocated,
}

#[derive(Debug, Clone)]
pub struct ResourceEntry {
    pub id: u64,
    pub state: ResourceState,
    pub tag: String,
}

#[derive(Debug, Clone)]
pub struct ResourcePool {
    name: String,
    capacity: usize,
    entries: Vec<ResourceEntry>,
    next_id: u64,
    total_acquired: u64,
    total_released: u64,
}

impl ResourcePool {
    pub fn new(name: &str, capacity: usize) -> Self {
        let entries = (0..capacity)
            .map(|i| ResourceEntry {
                id: i as u64,
                state: ResourceState::Free,
                tag: String::new(),
            })
            .collect();
        Self {
            name: name.to_string(),
            capacity,
            entries,
            next_id: capacity as u64,
            total_acquired: 0,
            total_released: 0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn available(&self) -> usize {
        self.entries.iter().filter(|e| e.state == ResourceState::Free).count()
    }

    pub fn allocated(&self) -> usize {
        self.entries.iter().filter(|e| e.state == ResourceState::Allocated).count()
    }

    pub fn acquire(&mut self, tag: &str) -> Result<u64, PoolError> {
        let idx = self
            .entries
            .iter()
            .position(|e| e.state == ResourceState::Free)
            .ok_or(PoolError::Exhausted { resource: self.name.clone() })?;
        self.entries[idx].state = ResourceState::Allocated;
        self.entries[idx].tag = tag.to_string();
        self.total_acquired += 1;
        Ok(self.entries[idx].id)
    }

    pub fn release(&mut self, id: u64) -> Result<(), PoolError> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or(PoolError::NotFound { id })?;
        if entry.state == ResourceState::Free {
            return Err(PoolError::AlreadyReleased { id });
        }
        entry.state = ResourceState::Free;
        entry.tag.clear();
        self.total_released += 1;
        Ok(())
    }

    pub fn get(&self, id: u64) -> Option<&ResourceEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn allocated_entries(&self) -> Vec<&ResourceEntry> {
        self.entries.iter().filter(|e| e.state == ResourceState::Allocated).collect()
    }

    pub fn total_acquired(&self) -> u64 {
        self.total_acquired
    }

    pub fn total_released(&self) -> u64 {
        self.total_released
    }

    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            return 0.0;
        }
        self.allocated() as f64 / self.capacity as f64
    }

    pub fn release_all(&mut self) -> usize {
        let count = self.allocated();
        for entry in &mut self.entries {
            if entry.state == ResourceState::Allocated {
                entry.state = ResourceState::Free;
                entry.tag.clear();
                self.total_released += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool() {
        let p = ResourcePool::new("buf", 4);
        assert_eq!(p.name(), "buf");
        assert_eq!(p.capacity(), 4);
        assert_eq!(p.available(), 4);
        assert_eq!(p.allocated(), 0);
    }

    #[test]
    fn acquire_one() {
        let mut p = ResourcePool::new("buf", 4);
        let id = p.acquire("tx").unwrap();
        assert_eq!(p.available(), 3);
        assert_eq!(p.allocated(), 1);
        let entry = p.get(id).unwrap();
        assert_eq!(entry.tag, "tx");
        assert_eq!(entry.state, ResourceState::Allocated);
    }

    #[test]
    fn acquire_all_exhausts() {
        let mut p = ResourcePool::new("buf", 2);
        p.acquire("a").unwrap();
        p.acquire("b").unwrap();
        let err = p.acquire("c").unwrap_err();
        assert!(matches!(err, PoolError::Exhausted { .. }));
    }

    #[test]
    fn release_ok() {
        let mut p = ResourcePool::new("buf", 4);
        let id = p.acquire("tx").unwrap();
        p.release(id).unwrap();
        assert_eq!(p.available(), 4);
        assert_eq!(p.allocated(), 0);
        assert_eq!(p.total_released(), 1);
    }

    #[test]
    fn release_not_found() {
        let mut p = ResourcePool::new("buf", 4);
        let err = p.release(999).unwrap_err();
        assert!(matches!(err, PoolError::NotFound { .. }));
    }

    #[test]
    fn release_already_released() {
        let mut p = ResourcePool::new("buf", 4);
        let id = p.acquire("tx").unwrap();
        p.release(id).unwrap();
        let err = p.release(id).unwrap_err();
        assert!(matches!(err, PoolError::AlreadyReleased { .. }));
    }

    #[test]
    fn reuse_released() {
        let mut p = ResourcePool::new("buf", 2);
        let id1 = p.acquire("a").unwrap();
        p.release(id1).unwrap();
        let id2 = p.acquire("b").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(p.get(id2).unwrap().tag, "b");
    }

    #[test]
    fn allocated_entries() {
        let mut p = ResourcePool::new("buf", 4);
        p.acquire("x").unwrap();
        p.acquire("y").unwrap();
        assert_eq!(p.allocated_entries().len(), 2);
    }

    #[test]
    fn utilization() {
        let mut p = ResourcePool::new("buf", 4);
        assert!((p.utilization() - 0.0).abs() < 0.01);
        p.acquire("x").unwrap();
        assert!((p.utilization() - 0.25).abs() < 0.01);
    }

    #[test]
    fn release_all() {
        let mut p = ResourcePool::new("buf", 4);
        p.acquire("a").unwrap();
        p.acquire("b").unwrap();
        let count = p.release_all();
        assert_eq!(count, 2);
        assert_eq!(p.available(), 4);
    }

    #[test]
    fn total_acquired_released() {
        let mut p = ResourcePool::new("buf", 4);
        let id = p.acquire("a").unwrap();
        p.release(id).unwrap();
        assert_eq!(p.total_acquired(), 1);
        assert_eq!(p.total_released(), 1);
    }

    #[test]
    fn error_display() {
        assert!(PoolError::Exhausted { resource: "buf".into() }.to_string().contains("buf"));
        assert!(PoolError::NotFound { id: 42 }.to_string().contains("42"));
        assert!(PoolError::AlreadyReleased { id: 1 }.to_string().contains("1"));
    }
}
