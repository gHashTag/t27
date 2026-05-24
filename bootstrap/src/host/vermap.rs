use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn is_compatible(&self, other: &SemVer) -> bool {
        self.major == other.major && self.minor == other.minor
    }

    pub fn is_breaking(&self, other: &SemVer) -> bool {
        self.major != other.major
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerError {
    Duplicate { component: String },
    NotFound { component: String },
    Incompatible { component: String, have: SemVer, need: SemVer },
    Breaking { component: String, have: SemVer, need: SemVer },
    CircularUpgrade { component: String },
}

impl std::fmt::Display for VerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerError::Duplicate { component } => write!(f, "{component} already registered"),
            VerError::NotFound { component } => write!(f, "{component} not found"),
            VerError::Incompatible { component, have, need } =>
                write!(f, "{component}: have {have}, need {need} (incompatible)"),
            VerError::Breaking { component, have, need } =>
                write!(f, "{component}: have {have}, need {need} (breaking)"),
            VerError::CircularUpgrade { component } =>
                write!(f, "{component}: circular upgrade path"),
        }
    }
}

impl std::error::Error for VerError {}

#[derive(Debug, Clone)]
struct VerEntry {
    version: SemVer,
    upgrade_from: Option<SemVer>,
}

#[derive(Debug, Clone)]
pub struct ComponentVer {
    pub component: String,
    pub version: SemVer,
    pub upgrade_from: Option<SemVer>,
}

#[derive(Debug, Clone)]
pub struct VerMap {
    entries: BTreeMap<String, VerEntry>,
}

impl VerMap {
    pub fn new() -> Self { Self { entries: BTreeMap::new() } }

    pub fn register(&mut self, component: &str, version: SemVer) -> Result<(), VerError> {
        if self.entries.contains_key(component) {
            return Err(VerError::Duplicate { component: component.to_string() });
        }
        self.entries.insert(component.to_string(), VerEntry { version, upgrade_from: None });
        Ok(())
    }

    pub fn register_with_upgrade(&mut self, component: &str, version: SemVer, from: SemVer) -> Result<(), VerError> {
        if self.entries.contains_key(component) {
            return Err(VerError::Duplicate { component: component.to_string() });
        }
        self.entries.insert(component.to_string(), VerEntry { version, upgrade_from: Some(from) });
        Ok(())
    }

    pub fn upgrade(&mut self, component: &str, new_ver: SemVer) -> Result<SemVer, VerError> {
        let entry = self.entries.get(component)
            .ok_or_else(|| VerError::NotFound { component: component.to_string() })?;
        let old_ver = entry.version;
        if new_ver <= old_ver {
            return Err(VerError::CircularUpgrade { component: component.to_string() });
        }
        let entry = self.entries.get_mut(component).unwrap();
        entry.upgrade_from = Some(old_ver);
        entry.version = new_ver;
        Ok(old_ver)
    }

    pub fn get(&self, component: &str) -> Option<SemVer> {
        self.entries.get(component).map(|e| e.version)
    }

    pub fn check(&self, component: &str, need: SemVer) -> Result<(), VerError> {
        let have = self.entries.get(component)
            .ok_or_else(|| VerError::NotFound { component: component.to_string() })?.version;
        if have.is_breaking(&need) {
            return Err(VerError::Breaking { component: component.to_string(), have, need });
        }
        if !have.is_compatible(&need) && have < need {
            return Err(VerError::Incompatible { component: component.to_string(), have, need });
        }
        Ok(())
    }

    pub fn component_count(&self) -> usize { self.entries.len() }

    pub fn all(&self) -> Vec<ComponentVer> {
        self.entries.iter().map(|(name, e)| ComponentVer {
            component: name.clone(),
            version: e.version,
            upgrade_from: e.upgrade_from,
        }).collect()
    }

    pub fn needs_upgrade(&self, component: &str, target: SemVer) -> bool {
        self.entries.get(component).map(|e| e.version < target).unwrap_or(false)
    }
}

impl Default for VerMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() {
        let vm = VerMap::new();
        assert_eq!(vm.component_count(), 0);
    }

    #[test]
    fn register() {
        let mut vm = VerMap::new();
        vm.register("core", SemVer::new(1, 0, 0)).unwrap();
        assert_eq!(vm.get("core"), Some(SemVer::new(1, 0, 0)));
    }

    #[test]
    fn duplicate() {
        let mut vm = VerMap::new();
        vm.register("core", SemVer::new(1, 0, 0)).unwrap();
        let err = vm.register("core", SemVer::new(2, 0, 0)).unwrap_err();
        assert!(matches!(err, VerError::Duplicate { .. }));
    }

    #[test]
    fn upgrade() {
        let mut vm = VerMap::new();
        vm.register("core", SemVer::new(1, 0, 0)).unwrap();
        let old = vm.upgrade("core", SemVer::new(1, 1, 0)).unwrap();
        assert_eq!(old, SemVer::new(1, 0, 0));
        assert_eq!(vm.get("core"), Some(SemVer::new(1, 1, 0)));
    }

    #[test]
    fn downgrade_rejected() {
        let mut vm = VerMap::new();
        vm.register("core", SemVer::new(2, 0, 0)).unwrap();
        let err = vm.upgrade("core", SemVer::new(1, 0, 0)).unwrap_err();
        assert!(matches!(err, VerError::CircularUpgrade { .. }));
    }

    #[test]
    fn check_compatible() {
        let mut vm = VerMap::new();
        vm.register("core", SemVer::new(1, 2, 0)).unwrap();
        vm.check("core", SemVer::new(1, 2, 0)).unwrap();
        vm.check("core", SemVer::new(1, 2, 5)).unwrap();
    }

    #[test]
    fn check_breaking() {
        let mut vm = VerMap::new();
        vm.register("core", SemVer::new(1, 0, 0)).unwrap();
        let err = vm.check("core", SemVer::new(2, 0, 0)).unwrap_err();
        assert!(matches!(err, VerError::Breaking { .. }));
    }

    #[test]
    fn check_incompatible() {
        let mut vm = VerMap::new();
        vm.register("core", SemVer::new(1, 0, 0)).unwrap();
        let err = vm.check("core", SemVer::new(1, 3, 0)).unwrap_err();
        assert!(matches!(err, VerError::Incompatible { .. }));
    }

    #[test]
    fn semver_display() {
        assert_eq!(SemVer::new(1, 2, 3).to_string(), "1.2.3");
    }

    #[test]
    fn all_components() {
        let mut vm = VerMap::new();
        vm.register("a", SemVer::new(1, 0, 0)).unwrap();
        vm.register("b", SemVer::new(2, 0, 0)).unwrap();
        assert_eq!(vm.all().len(), 2);
    }

    #[test]
    fn needs_upgrade_check() {
        let mut vm = VerMap::new();
        vm.register("x", SemVer::new(1, 0, 0)).unwrap();
        assert!(vm.needs_upgrade("x", SemVer::new(2, 0, 0)));
        assert!(!vm.needs_upgrade("x", SemVer::new(0, 9, 0)));
    }

    #[test]
    fn error_display() {
        assert!(VerError::NotFound { component: "x".into() }.to_string().contains("x"));
    }
}
