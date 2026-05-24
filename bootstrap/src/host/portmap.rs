use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LogicalPort(pub u16);

impl std::fmt::Display for LogicalPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lp{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PhysicalPort(pub u16);

impl std::fmt::Display for PhysicalPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pp{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortMapError {
    LogicalInUse { port: LogicalPort },
    PhysicalInUse { port: PhysicalPort },
    LogicalNotFound { port: LogicalPort },
    PhysicalNotFound { port: PhysicalPort },
}

impl std::fmt::Display for PortMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortMapError::LogicalInUse { port } => write!(f, "logical port in use: {port}"),
            PortMapError::PhysicalInUse { port } => write!(f, "physical port in use: {port}"),
            PortMapError::LogicalNotFound { port } => write!(f, "logical port not found: {port}"),
            PortMapError::PhysicalNotFound { port } => write!(f, "physical port not found: {port}"),
        }
    }
}

impl std::error::Error for PortMapError {}

#[derive(Debug, Clone)]
pub struct PortMapping {
    pub logical: LogicalPort,
    pub physical: PhysicalPort,
    pub bandwidth_mbps: u32,
    pub active: bool,
}

impl PortMapping {
    pub fn new(logical: LogicalPort, physical: PhysicalPort, bandwidth_mbps: u32) -> Self {
        Self {
            logical,
            physical,
            bandwidth_mbps,
            active: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortMapper {
    by_logical: BTreeMap<u16, PortMapping>,
    by_physical: BTreeMap<u16, PortMapping>,
    total_mapped: u64,
    total_unmapped: u64,
}

impl PortMapper {
    pub fn new() -> Self {
        Self {
            by_logical: BTreeMap::new(),
            by_physical: BTreeMap::new(),
            total_mapped: 0,
            total_unmapped: 0,
        }
    }

    pub fn map(&mut self, mapping: PortMapping) -> Result<(), PortMapError> {
        if self.by_logical.contains_key(&mapping.logical.0) {
            return Err(PortMapError::LogicalInUse { port: mapping.logical });
        }
        if self.by_physical.contains_key(&mapping.physical.0) {
            return Err(PortMapError::PhysicalInUse { port: mapping.physical });
        }
        self.total_mapped += 1;
        self.by_logical.insert(mapping.logical.0, mapping.clone());
        self.by_physical.insert(mapping.physical.0, mapping);
        Ok(())
    }

    pub fn unmap_logical(&mut self, port: LogicalPort) -> Result<PortMapping, PortMapError> {
        let mapping = self.by_logical.remove(&port.0)
            .ok_or(PortMapError::LogicalNotFound { port })?;
        self.by_physical.remove(&mapping.physical.0);
        self.total_unmapped += 1;
        Ok(mapping)
    }

    pub fn unmap_physical(&mut self, port: PhysicalPort) -> Result<PortMapping, PortMapError> {
        let mapping = self.by_physical.remove(&port.0)
            .ok_or(PortMapError::PhysicalNotFound { port })?;
        self.by_logical.remove(&mapping.logical.0);
        self.total_unmapped += 1;
        Ok(mapping)
    }

    pub fn lookup_logical(&self, port: LogicalPort) -> Option<&PortMapping> {
        self.by_logical.get(&port.0)
    }

    pub fn lookup_physical(&self, port: PhysicalPort) -> Option<&PortMapping> {
        self.by_physical.get(&port.0)
    }

    pub fn resolve_to_physical(&self, logical: LogicalPort) -> Option<PhysicalPort> {
        self.by_logical.get(&logical.0).map(|m| m.physical)
    }

    pub fn resolve_to_logical(&self, physical: PhysicalPort) -> Option<LogicalPort> {
        self.by_physical.get(&physical.0).map(|m| m.logical)
    }

    pub fn set_active(&mut self, logical: LogicalPort, active: bool) -> bool {
        if let Some(mapping) = self.by_logical.get_mut(&logical.0) {
            mapping.active = active;
            if let Some(pm) = self.by_physical.get_mut(&mapping.physical.0) {
                pm.active = active;
            }
            true
        } else {
            false
        }
    }

    pub fn active_mappings(&self) -> Vec<&PortMapping> {
        self.by_logical.values().filter(|m| m.active).collect()
    }

    pub fn mapping_count(&self) -> usize {
        self.by_logical.len()
    }

    pub fn total_bandwidth(&self) -> u64 {
        self.by_logical.values()
            .filter(|m| m.active)
            .map(|m| m.bandwidth_mbps as u64)
            .sum()
    }

    pub fn total_mapped(&self) -> u64 {
        self.total_mapped
    }

    pub fn total_unmapped(&self) -> u64 {
        self.total_unmapped
    }

    pub fn clear(&mut self) {
        self.by_logical.clear();
        self.by_physical.clear();
    }
}

impl Default for PortMapper {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_display() {
        assert_eq!(LogicalPort(1).to_string(), "lp1");
        assert_eq!(PhysicalPort(2).to_string(), "pp2");
    }

    #[test]
    fn map_and_lookup() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 1000)).unwrap();
        assert_eq!(pm.resolve_to_physical(LogicalPort(1)), Some(PhysicalPort(10)));
        assert_eq!(pm.resolve_to_logical(PhysicalPort(10)), Some(LogicalPort(1)));
    }

    #[test]
    fn map_duplicate_logical() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 100)).unwrap();
        let err = pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(20), 100)).unwrap_err();
        assert!(matches!(err, PortMapError::LogicalInUse { .. }));
    }

    #[test]
    fn map_duplicate_physical() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 100)).unwrap();
        let err = pm.map(PortMapping::new(LogicalPort(2), PhysicalPort(10), 100)).unwrap_err();
        assert!(matches!(err, PortMapError::PhysicalInUse { .. }));
    }

    #[test]
    fn unmap_logical() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 100)).unwrap();
        let m = pm.unmap_logical(LogicalPort(1)).unwrap();
        assert_eq!(m.physical, PhysicalPort(10));
        assert_eq!(pm.mapping_count(), 0);
    }

    #[test]
    fn unmap_not_found() {
        let mut pm = PortMapper::new();
        let err = pm.unmap_logical(LogicalPort(99)).unwrap_err();
        assert!(matches!(err, PortMapError::LogicalNotFound { .. }));
    }

    #[test]
    fn set_active() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 100)).unwrap();
        pm.set_active(LogicalPort(1), false);
        assert_eq!(pm.active_mappings().len(), 0);
        pm.set_active(LogicalPort(1), true);
        assert_eq!(pm.active_mappings().len(), 1);
    }

    #[test]
    fn total_bandwidth() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 500)).unwrap();
        pm.map(PortMapping::new(LogicalPort(2), PhysicalPort(20), 300)).unwrap();
        assert_eq!(pm.total_bandwidth(), 800);
    }

    #[test]
    fn bandwidth_excludes_inactive() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 500)).unwrap();
        pm.map(PortMapping::new(LogicalPort(2), PhysicalPort(20), 300)).unwrap();
        pm.set_active(LogicalPort(2), false);
        assert_eq!(pm.total_bandwidth(), 500);
    }

    #[test]
    fn stats() {
        let mut pm = PortMapper::new();
        pm.map(PortMapping::new(LogicalPort(1), PhysicalPort(10), 100)).unwrap();
        pm.unmap_logical(LogicalPort(1)).unwrap();
        assert_eq!(pm.total_mapped(), 1);
        assert_eq!(pm.total_unmapped(), 1);
    }

    #[test]
    fn error_display() {
        assert!(PortMapError::LogicalInUse { port: LogicalPort(1) }.to_string().contains("lp1"));
        assert!(PortMapError::PhysicalNotFound { port: PhysicalPort(5) }.to_string().contains("pp5"));
    }
}
