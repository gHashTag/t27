use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointError {
    NotFound { name: String },
    Duplicate { name: String },
    Overlap { a: String, b: String },
    OutOfRange { offset: u32 },
}

impl std::fmt::Display for EndpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndpointError::NotFound { name } => write!(f, "endpoint not found: {name}"),
            EndpointError::Duplicate { name } => write!(f, "duplicate endpoint: {name}"),
            EndpointError::Overlap { a, b } => write!(f, "endpoints overlap: {a} and {b}"),
            EndpointError::OutOfRange { offset } => write!(f, "offset out of range: 0x{offset:X}"),
        }
    }
}

impl std::error::Error for EndpointError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl std::fmt::Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::ReadOnly => write!(f, "ro"),
            Access::WriteOnly => write!(f, "wo"),
            Access::ReadWrite => write!(f, "rw"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub name: String,
    pub base: u32,
    pub size: u32,
    pub access: Access,
    pub description: String,
}

impl Endpoint {
    pub fn new(name: &str, base: u32, size: u32, access: Access) -> Self {
        Self {
            name: name.to_string(),
            base,
            size,
            access,
            description: String::new(),
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn end(&self) -> u32 {
        self.base + self.size
    }

    pub fn contains(&self, offset: u32) -> bool {
        offset >= self.base && offset < self.end()
    }

    pub fn offset_of(&self, offset: u32) -> Option<u32> {
        if self.contains(offset) {
            Some(offset - self.base)
        } else {
            None
        }
    }

    pub fn can_read(&self) -> bool {
        self.access == Access::ReadOnly || self.access == Access::ReadWrite
    }

    pub fn can_write(&self) -> bool {
        self.access == Access::WriteOnly || self.access == Access::ReadWrite
    }

    fn overlaps(&self, other: &Endpoint) -> bool {
        self.base < other.end() && other.base < self.end()
    }
}

#[derive(Debug, Clone)]
pub struct EndpointRegistry {
    endpoints: BTreeMap<String, Endpoint>,
}

impl EndpointRegistry {
    pub fn new() -> Self {
        Self {
            endpoints: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, ep: Endpoint) -> Result<(), EndpointError> {
        if self.endpoints.contains_key(&ep.name) {
            return Err(EndpointError::Duplicate { name: ep.name.clone() });
        }
        for existing in self.endpoints.values() {
            if ep.overlaps(existing) {
                return Err(EndpointError::Overlap {
                    a: ep.name.clone(),
                    b: existing.name.clone(),
                });
            }
        }
        self.endpoints.insert(ep.name.clone(), ep);
        Ok(())
    }

    pub fn unregister(&mut self, name: &str) -> bool {
        self.endpoints.remove(name).is_some()
    }

    pub fn get(&self, name: &str) -> Option<&Endpoint> {
        self.endpoints.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Endpoint> {
        self.endpoints.get_mut(name)
    }

    pub fn find_by_offset(&self, offset: u32) -> Option<&Endpoint> {
        self.endpoints.values().find(|e| e.contains(offset))
    }

    pub fn validate_read(&self, offset: u32) -> Result<&Endpoint, EndpointError> {
        let ep = self.find_by_offset(offset).ok_or(EndpointError::OutOfRange { offset })?;
        if !ep.can_read() {
            return Err(EndpointError::OutOfRange { offset });
        }
        Ok(ep)
    }

    pub fn validate_write(&self, offset: u32) -> Result<&Endpoint, EndpointError> {
        let ep = self.find_by_offset(offset).ok_or(EndpointError::OutOfRange { offset })?;
        if !ep.can_write() {
            return Err(EndpointError::OutOfRange { offset });
        }
        Ok(ep)
    }

    pub fn len(&self) -> usize {
        self.endpoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.endpoints.is_empty()
    }

    pub fn names(&self) -> Vec<&str> {
        self.endpoints.keys().map(|s| s.as_str()).collect()
    }

    pub fn endpoints(&self) -> Vec<&Endpoint> {
        self.endpoints.values().collect()
    }

    pub fn clear(&mut self) {
        self.endpoints.clear();
    }
}

impl Default for EndpointRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_contains() {
        let ep = Endpoint::new("ctrl", 0x100, 0x10, Access::ReadWrite);
        assert!(ep.contains(0x100));
        assert!(ep.contains(0x10F));
        assert!(!ep.contains(0x110));
    }

    #[test]
    fn endpoint_offset() {
        let ep = Endpoint::new("ctrl", 0x100, 0x10, Access::ReadWrite);
        assert_eq!(ep.offset_of(0x105), Some(5));
        assert_eq!(ep.offset_of(0x200), None);
    }

    #[test]
    fn endpoint_access() {
        let ro = Endpoint::new("stat", 0, 1, Access::ReadOnly);
        assert!(ro.can_read());
        assert!(!ro.can_write());
        let wo = Endpoint::new("pulse", 0, 1, Access::WriteOnly);
        assert!(!wo.can_read());
        assert!(wo.can_write());
    }

    #[test]
    fn endpoint_with_description() {
        let ep = Endpoint::new("x", 0, 1, Access::ReadWrite).with_description("test");
        assert_eq!(ep.description, "test");
    }

    #[test]
    fn register_and_get() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("ctrl", 0x100, 0x10, Access::ReadWrite)).unwrap();
        assert!(r.get("ctrl").is_some());
        assert!(r.get("missing").is_none());
    }

    #[test]
    fn register_duplicate() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("ctrl", 0x100, 0x10, Access::ReadWrite)).unwrap();
        let err = r.register(Endpoint::new("ctrl", 0x200, 0x10, Access::ReadWrite)).unwrap_err();
        assert!(matches!(err, EndpointError::Duplicate { .. }));
    }

    #[test]
    fn register_overlap() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("a", 0x100, 0x20, Access::ReadWrite)).unwrap();
        let err = r.register(Endpoint::new("b", 0x110, 0x20, Access::ReadWrite)).unwrap_err();
        assert!(matches!(err, EndpointError::Overlap { .. }));
    }

    #[test]
    fn find_by_offset() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("ctrl", 0x100, 0x10, Access::ReadWrite)).unwrap();
        assert_eq!(r.find_by_offset(0x105).unwrap().name, "ctrl");
        assert!(r.find_by_offset(0x200).is_none());
    }

    #[test]
    fn validate_read_ok() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("stat", 0x100, 0x10, Access::ReadOnly)).unwrap();
        r.validate_read(0x105).unwrap();
    }

    #[test]
    fn validate_read_write_only_fails() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("pulse", 0x100, 0x10, Access::WriteOnly)).unwrap();
        assert!(r.validate_read(0x105).is_err());
    }

    #[test]
    fn validate_write_ok() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("ctrl", 0x100, 0x10, Access::ReadWrite)).unwrap();
        r.validate_write(0x105).unwrap();
    }

    #[test]
    fn unregister() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("x", 0, 1, Access::ReadWrite)).unwrap();
        assert!(r.unregister("x"));
        assert!(!r.unregister("x"));
    }

    #[test]
    fn names_sorted() {
        let mut r = EndpointRegistry::new();
        r.register(Endpoint::new("bravo", 0x100, 1, Access::ReadWrite)).unwrap();
        r.register(Endpoint::new("alpha", 0x200, 1, Access::ReadWrite)).unwrap();
        assert_eq!(r.names(), vec!["alpha", "bravo"]);
    }

    #[test]
    fn error_display() {
        assert!(EndpointError::NotFound { name: "x".into() }.to_string().contains("x"));
        assert!(EndpointError::Overlap { a: "a".into(), b: "b".into() }.to_string().contains("overlap"));
    }

    #[test]
    fn access_display() {
        assert_eq!(Access::ReadOnly.to_string(), "ro");
        assert_eq!(Access::ReadWrite.to_string(), "rw");
    }
}
