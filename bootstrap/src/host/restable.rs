use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub enum ResError {
    ResourceExists { name: String },
    ResourceNotFound { name: String },
    AllocExists { name: String, id: u64 },
    AllocNotFound { id: u64 },
    CapacityExceeded { name: String, cap: u64, used: u64 },
    NotAllocated { name: String, id: u64 },
}

impl std::fmt::Display for ResError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResError::ResourceExists { name } => write!(f, "resource {name} exists"),
            ResError::ResourceNotFound { name } => write!(f, "resource {name} not found"),
            ResError::AllocExists { name, id } => write!(f, "alloc {id} already exists in {name}"),
            ResError::AllocNotFound { id } => write!(f, "alloc {id} not found"),
            ResError::CapacityExceeded { name, cap, used } =>
                write!(f, "{name}: capacity {cap} exceeded (used {used})"),
            ResError::NotAllocated { name, id } => write!(f, "{id} not allocated in {name}"),
        }
    }
}

impl std::error::Error for ResError {}

struct Resource {
    name: String,
    capacity: u64,
    allocated: u64,
}

struct Allocation {
    id: u64,
    resource: String,
    amount: u64,
}

#[derive(Debug, Clone)]
pub struct AllocInfo {
    pub id: u64,
    pub resource: String,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct ResourceInfo {
    pub name: String,
    pub capacity: u64,
    pub allocated: u64,
    pub available: u64,
    pub allocation_count: usize,
}

pub struct ResourceTable {
    resources: BTreeMap<String, Resource>,
    allocations: BTreeMap<u64, Allocation>,
    resource_allocs: BTreeMap<String, BTreeSet<u64>>,
    next_alloc_id: u64,
    total_allocs: u64,
    total_deallocs: u64,
}

impl ResourceTable {
    pub fn new() -> Self {
        Self { resources: BTreeMap::new(), allocations: BTreeMap::new(), resource_allocs: BTreeMap::new(), next_alloc_id: 1, total_allocs: 0, total_deallocs: 0 }
    }

    pub fn register(&mut self, name: &str, capacity: u64) -> Result<(), ResError> {
        if self.resources.contains_key(name) {
            return Err(ResError::ResourceExists { name: name.to_string() });
        }
        self.resources.insert(name.to_string(), Resource { name: name.to_string(), capacity, allocated: 0 });
        self.resource_allocs.insert(name.to_string(), BTreeSet::new());
        Ok(())
    }

    pub fn allocate(&mut self, resource: &str, amount: u64) -> Result<u64, ResError> {
        let res = self.resources.get(resource)
            .ok_or_else(|| ResError::ResourceNotFound { name: resource.to_string() })?;
        if res.allocated + amount > res.capacity {
            return Err(ResError::CapacityExceeded { name: resource.to_string(), cap: res.capacity, used: res.allocated });
        }
        let id = self.next_alloc_id;
        self.next_alloc_id += 1;
        self.resources.get_mut(resource).unwrap().allocated += amount;
        self.allocations.insert(id, Allocation { id, resource: resource.to_string(), amount });
        self.resource_allocs.get_mut(resource).unwrap().insert(id);
        self.total_allocs += 1;
        Ok(id)
    }

    pub fn deallocate(&mut self, id: u64) -> Result<u64, ResError> {
        let alloc = self.allocations.remove(&id)
            .ok_or(ResError::AllocNotFound { id })?;
        self.resources.get_mut(&alloc.resource).unwrap().allocated -= alloc.amount;
        self.resource_allocs.get_mut(&alloc.resource).unwrap().remove(&id);
        self.total_deallocs += 1;
        Ok(alloc.amount)
    }

    pub fn resource_info(&self, name: &str) -> Option<ResourceInfo> {
        self.resources.get(name).map(|r| ResourceInfo {
            name: r.name.clone(), capacity: r.capacity, allocated: r.allocated,
            available: r.capacity - r.allocated,
            allocation_count: self.resource_allocs.get(name).map(|s| s.len()).unwrap_or(0),
        })
    }

    pub fn alloc_info(&self, id: u64) -> Option<AllocInfo> {
        self.allocations.get(&id).map(|a| AllocInfo { id: a.id, resource: a.resource.clone(), amount: a.amount })
    }

    pub fn leaks(&self) -> Vec<AllocInfo> {
        self.allocations.values().map(|a| AllocInfo { id: a.id, resource: a.resource.clone(), amount: a.amount }).collect()
    }

    pub fn resource_count(&self) -> usize { self.resources.len() }
    pub fn active_allocs(&self) -> usize { self.allocations.len() }
    pub fn total_allocs(&self) -> u64 { self.total_allocs }
    pub fn total_deallocs(&self) -> u64 { self.total_deallocs }
}

impl Default for ResourceTable {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_table() {
        let rt = ResourceTable::new();
        assert_eq!(rt.resource_count(), 0);
    }

    #[test]
    fn register_resource() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        let info = rt.resource_info("mem").unwrap();
        assert_eq!(info.capacity, 1024);
        assert_eq!(info.available, 1024);
    }

    #[test]
    fn duplicate_resource() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        let err = rt.register("mem", 2048).unwrap_err();
        assert!(matches!(err, ResError::ResourceExists { .. }));
    }

    #[test]
    fn allocate_deallocate() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        let id = rt.allocate("mem", 256).unwrap();
        let info = rt.resource_info("mem").unwrap();
        assert_eq!(info.allocated, 256);
        assert_eq!(info.available, 768);
        let freed = rt.deallocate(id).unwrap();
        assert_eq!(freed, 256);
        assert_eq!(rt.resource_info("mem").unwrap().available, 1024);
    }

    #[test]
    fn capacity_exceeded() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 100).unwrap();
        rt.allocate("mem", 60).unwrap();
        let err = rt.allocate("mem", 50).unwrap_err();
        assert!(matches!(err, ResError::CapacityExceeded { .. }));
    }

    #[test]
    fn dealloc_not_found() {
        let mut rt = ResourceTable::new();
        let err = rt.deallocate(999).unwrap_err();
        assert!(matches!(err, ResError::AllocNotFound { .. }));
    }

    #[test]
    fn multiple_resources() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        rt.register("gpu", 4096).unwrap();
        rt.allocate("mem", 512).unwrap();
        rt.allocate("gpu", 2048).unwrap();
        assert_eq!(rt.active_allocs(), 2);
    }

    #[test]
    fn leaks() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        rt.allocate("mem", 100).unwrap();
        rt.allocate("mem", 200).unwrap();
        assert_eq!(rt.leaks().len(), 2);
    }

    #[test]
    fn no_leaks() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        let id = rt.allocate("mem", 100).unwrap();
        rt.deallocate(id).unwrap();
        assert!(rt.leaks().is_empty());
    }

    #[test]
    fn stats() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        let a = rt.allocate("mem", 100).unwrap();
        let b = rt.allocate("mem", 200).unwrap();
        rt.deallocate(a).unwrap();
        assert_eq!(rt.total_allocs(), 2);
        assert_eq!(rt.total_deallocs(), 1);
    }

    #[test]
    fn alloc_info() {
        let mut rt = ResourceTable::new();
        rt.register("mem", 1024).unwrap();
        let id = rt.allocate("mem", 333).unwrap();
        let info = rt.alloc_info(id).unwrap();
        assert_eq!(info.amount, 333);
        assert_eq!(info.resource, "mem");
    }

    #[test]
    fn error_display() {
        assert!(ResError::ResourceNotFound { name: "x".into() }.to_string().contains("x"));
    }
}
