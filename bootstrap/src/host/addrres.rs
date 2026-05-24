use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddrError {
    NotFound { name: String },
    Duplicate { name: String },
    AliasCycle { chain: Vec<String> },
    OutOfBounds { name: String, offset: u32 },
}

impl std::fmt::Display for AddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddrError::NotFound { name } => write!(f, "address not found: {name}"),
            AddrError::Duplicate { name } => write!(f, "duplicate: {name}"),
            AddrError::AliasCycle { chain } => {
                write!(f, "alias cycle: {}", chain.join(" -> "))
            }
            AddrError::OutOfBounds { name, offset } => {
                write!(f, "{name}: offset 0x{offset:X} out of bounds")
            }
        }
    }
}

impl std::error::Error for AddrError {}

#[derive(Debug, Clone)]
pub struct AddrMapping {
    pub name: String,
    pub base: u32,
    pub size: u32,
    pub alias_of: Option<String>,
}

impl AddrMapping {
    pub fn region(name: &str, base: u32, size: u32) -> Self {
        Self {
            name: name.to_string(),
            base,
            size,
            alias_of: None,
        }
    }

    pub fn alias(name: &str, target: &str) -> Self {
        Self {
            name: name.to_string(),
            base: 0,
            size: 0,
            alias_of: Some(target.to_string()),
        }
    }

    pub fn end(&self) -> u32 {
        self.base + self.size
    }

    pub fn contains(&self, offset: u32) -> bool {
        self.size > 0 && offset >= self.base && offset < self.end()
    }
}

#[derive(Debug, Clone)]
pub struct AddressResolver {
    mappings: BTreeMap<String, AddrMapping>,
}

impl AddressResolver {
    pub fn new() -> Self {
        Self {
            mappings: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, mapping: AddrMapping) -> Result<(), AddrError> {
        if self.mappings.contains_key(&mapping.name) {
            return Err(AddrError::Duplicate { name: mapping.name.clone() });
        }
        self.mappings.insert(mapping.name.clone(), mapping);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.mappings.remove(name).is_some()
    }

    fn resolve_chain(&self, name: &str) -> Result<String, AddrError> {
        let mut visited = Vec::new();
        let mut current = name.to_string();
        loop {
            if visited.contains(&current) {
                visited.push(current.clone());
                return Err(AddrError::AliasCycle { chain: visited });
            }
            let mapping = self
                .mappings
                .get(&current)
                .ok_or_else(|| AddrError::NotFound { name: current.clone() })?;
            match &mapping.alias_of {
                Some(target) => {
                    visited.push(current);
                    current = target.clone();
                }
                None => return Ok(current),
            }
        }
    }

    pub fn resolve(&self, name: &str) -> Result<(u32, u32), AddrError> {
        let resolved_name = self.resolve_chain(name)?;
        let mapping = self.mappings.get(&resolved_name).unwrap();
        Ok((mapping.base, mapping.size))
    }

    pub fn resolve_base(&self, name: &str) -> Result<u32, AddrError> {
        let (base, _) = self.resolve(name)?;
        Ok(base)
    }

    pub fn resolve_offset(&self, name: &str, offset: u32) -> Result<u32, AddrError> {
        let (base, size) = self.resolve(name)?;
        if offset >= size {
            return Err(AddrError::OutOfBounds {
                name: name.to_string(),
                offset,
            });
        }
        Ok(base + offset)
    }

    pub fn lookup_by_address(&self, addr: u32) -> Option<(&str, u32)> {
        for mapping in self.mappings.values() {
            if mapping.alias_of.is_none() && mapping.contains(addr) {
                return Some((&mapping.name, addr - mapping.base));
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    pub fn names(&self) -> Vec<&str> {
        self.mappings.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for AddressResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_region() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("ctrl", 0x100, 0x20)).unwrap();
        let (base, size) = ar.resolve("ctrl").unwrap();
        assert_eq!(base, 0x100);
        assert_eq!(size, 0x20);
    }

    #[test]
    fn add_duplicate() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("x", 0, 1)).unwrap();
        let err = ar.add(AddrMapping::region("x", 0x100, 1)).unwrap_err();
        assert!(matches!(err, AddrError::Duplicate { .. }));
    }

    #[test]
    fn alias_resolves() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("ctrl", 0x100, 0x20)).unwrap();
        ar.add(AddrMapping::alias("control", "ctrl")).unwrap();
        let (base, _) = ar.resolve("control").unwrap();
        assert_eq!(base, 0x100);
    }

    #[test]
    fn alias_chain() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("csr", 0x200, 0x10)).unwrap();
        ar.add(AddrMapping::alias("a", "csr")).unwrap();
        ar.add(AddrMapping::alias("b", "a")).unwrap();
        assert_eq!(ar.resolve_base("b").unwrap(), 0x200);
    }

    #[test]
    fn alias_cycle_detected() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::alias("a", "b")).unwrap();
        ar.add(AddrMapping::alias("b", "a")).unwrap();
        let err = ar.resolve("a").unwrap_err();
        assert!(matches!(err, AddrError::AliasCycle { .. }));
    }

    #[test]
    fn not_found() {
        let ar = AddressResolver::new();
        let err = ar.resolve("missing").unwrap_err();
        assert!(matches!(err, AddrError::NotFound { .. }));
    }

    #[test]
    fn resolve_offset_ok() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("ctrl", 0x100, 0x20)).unwrap();
        assert_eq!(ar.resolve_offset("ctrl", 0x05).unwrap(), 0x105);
    }

    #[test]
    fn resolve_offset_out_of_bounds() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("ctrl", 0x100, 0x20)).unwrap();
        let err = ar.resolve_offset("ctrl", 0x20).unwrap_err();
        assert!(matches!(err, AddrError::OutOfBounds { .. }));
    }

    #[test]
    fn lookup_by_address() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("ctrl", 0x100, 0x20)).unwrap();
        let (name, off) = ar.lookup_by_address(0x105).unwrap();
        assert_eq!(name, "ctrl");
        assert_eq!(off, 5);
    }

    #[test]
    fn lookup_by_address_miss() {
        let ar = AddressResolver::new();
        assert!(ar.lookup_by_address(0).is_none());
    }

    #[test]
    fn remove() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("x", 0, 1)).unwrap();
        assert!(ar.remove("x"));
        assert!(!ar.remove("x"));
    }

    #[test]
    fn names_sorted() {
        let mut ar = AddressResolver::new();
        ar.add(AddrMapping::region("bravo", 0x200, 1)).unwrap();
        ar.add(AddrMapping::region("alpha", 0x100, 1)).unwrap();
        assert_eq!(ar.names(), vec!["alpha", "bravo"]);
    }

    #[test]
    fn error_display() {
        assert!(AddrError::NotFound { name: "x".into() }.to_string().contains("x"));
        assert!(AddrError::AliasCycle { chain: vec!["a".into(), "b".into()] }.to_string().contains("cycle"));
        assert!(AddrError::OutOfBounds { name: "x".into(), offset: 99 }.to_string().contains("0x63"));
    }
}
