use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    pub name: String,
    pub version: (u8, u8, u8),
    pub capabilities: Vec<String>,
    pub enabled: bool,
}

impl ModuleInfo {
    pub fn new(name: &str, major: u8, minor: u8, patch: u8) -> Self {
        Self {
            name: name.to_string(),
            version: (major, minor, patch),
            capabilities: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_capability(mut self, cap: &str) -> Self {
        if !self.capabilities.contains(&cap.to_string()) {
            self.capabilities.push(cap.to_string());
        }
        self
    }

    pub fn with_capabilities(mut self, caps: &[&str]) -> Self {
        for c in caps {
            self = self.with_capability(c);
        }
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn version_string(&self) -> String {
        format!("{}.{}.{}", self.version.0, self.version.1, self.version.2)
    }

    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.iter().any(|c| c == cap)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    Duplicate { name: String },
    NotFound { name: String },
    CircularDep { chain: Vec<String> },
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Duplicate { name } => write!(f, "duplicate module: {name}"),
            ManifestError::NotFound { name } => write!(f, "module not found: {name}"),
            ManifestError::CircularDep { chain } => {
                write!(f, "circular dependency: {}", chain.join(" -> "))
            }
        }
    }
}

impl std::error::Error for ManifestError {}

#[derive(Debug, Clone)]
pub struct ModuleManifest {
    modules: BTreeMap<String, ModuleInfo>,
    deps: BTreeMap<String, Vec<String>>,
}

impl ModuleManifest {
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            deps: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, info: ModuleInfo) -> Result<(), ManifestError> {
        if self.modules.contains_key(&info.name) {
            return Err(ManifestError::Duplicate { name: info.name.clone() });
        }
        self.modules.insert(info.name.clone(), info);
        Ok(())
    }

    pub fn add_dep(&mut self, module: &str, dep: &str) -> Result<(), ManifestError> {
        if !self.modules.contains_key(module) {
            return Err(ManifestError::NotFound { name: module.to_string() });
        }
        if !self.modules.contains_key(dep) {
            return Err(ManifestError::NotFound { name: dep.to_string() });
        }
        let entry = self.deps.entry(module.to_string()).or_default();
        if !entry.contains(&dep.to_string()) {
            entry.push(dep.to_string());
        }
        Ok(())
    }

    pub fn check_cycles(&self) -> Result<(), ManifestError> {
        for name in self.modules.keys() {
            let mut visited = Vec::new();
            self.dfs(name, &mut visited)?;
        }
        Ok(())
    }

    fn dfs(&self, node: &str, path: &mut Vec<String>) -> Result<(), ManifestError> {
        if path.iter().any(|p| p == node) {
            path.push(node.to_string());
            let cycle_start = path.iter().position(|p| p == node).unwrap();
            let chain: Vec<String> = path[cycle_start..].to_vec();
            return Err(ManifestError::CircularDep { chain });
        }
        path.push(node.to_string());
        if let Some(deps) = self.deps.get(node) {
            for dep in deps {
                self.dfs(dep, path)?;
            }
        }
        path.pop();
        Ok(())
    }

    pub fn topological_order(&self) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = std::collections::BTreeSet::new();
        let mut temp = std::collections::BTreeSet::new();
        for name in self.modules.keys() {
            self.topo_visit(name, &mut visited, &mut temp, &mut result);
        }
        result
    }

    fn topo_visit(
        &self,
        node: &str,
        visited: &mut std::collections::BTreeSet<String>,
        temp: &mut std::collections::BTreeSet<String>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(node) {
            return;
        }
        if temp.contains(node) {
            return;
        }
        temp.insert(node.to_string());
        if let Some(deps) = self.deps.get(node) {
            for dep in deps {
                self.topo_visit(dep, visited, temp, result);
            }
        }
        temp.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());
    }

    pub fn get(&self, name: &str) -> Option<&ModuleInfo> {
        self.modules.get(name)
    }

    pub fn enabled(&self) -> Vec<&ModuleInfo> {
        self.modules.values().filter(|m| m.enabled).collect()
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    pub fn names(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ModuleManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_info_new() {
        let m = ModuleInfo::new("dma", 1, 2, 3);
        assert_eq!(m.name, "dma");
        assert_eq!(m.version, (1, 2, 3));
        assert!(m.enabled);
    }

    #[test]
    fn module_info_capabilities() {
        let m = ModuleInfo::new("x", 0, 0, 1)
            .with_capability("read")
            .with_capability("write")
            .with_capability("read");
        assert_eq!(m.capabilities.len(), 2);
        assert!(m.has_capability("read"));
        assert!(!m.has_capability("exec"));
    }

    #[test]
    fn module_info_disabled() {
        let m = ModuleInfo::new("x", 0, 0, 1).disabled();
        assert!(!m.enabled);
    }

    #[test]
    fn module_version_string() {
        let m = ModuleInfo::new("x", 2, 1, 0);
        assert_eq!(m.version_string(), "2.1.0");
    }

    #[test]
    fn register_and_get() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("dma", 1, 0, 0)).unwrap();
        assert!(mf.get("dma").is_some());
        assert!(mf.get("missing").is_none());
    }

    #[test]
    fn register_duplicate() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("a", 1, 0, 0)).unwrap();
        let err = mf.register(ModuleInfo::new("a", 2, 0, 0)).unwrap_err();
        assert!(matches!(err, ManifestError::Duplicate { .. }));
    }

    #[test]
    fn add_dep_ok() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("a", 1, 0, 0)).unwrap();
        mf.register(ModuleInfo::new("b", 1, 0, 0)).unwrap();
        mf.add_dep("a", "b").unwrap();
    }

    #[test]
    fn add_dep_missing() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("a", 1, 0, 0)).unwrap();
        let err = mf.add_dep("a", "missing").unwrap_err();
        assert!(matches!(err, ManifestError::NotFound { .. }));
    }

    #[test]
    fn check_cycles_ok() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("a", 1, 0, 0)).unwrap();
        mf.register(ModuleInfo::new("b", 1, 0, 0)).unwrap();
        mf.add_dep("a", "b").unwrap();
        assert!(mf.check_cycles().is_ok());
    }

    #[test]
    fn check_cycles_detected() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("a", 1, 0, 0)).unwrap();
        mf.register(ModuleInfo::new("b", 1, 0, 0)).unwrap();
        mf.deps.insert("a".into(), vec!["b".into()]);
        mf.deps.insert("b".into(), vec!["a".into()]);
        let err = mf.check_cycles().unwrap_err();
        assert!(matches!(err, ManifestError::CircularDep { .. }));
    }

    #[test]
    fn topological_order() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("driver", 1, 0, 0)).unwrap();
        mf.register(ModuleInfo::new("dma", 1, 0, 0)).unwrap();
        mf.add_dep("driver", "dma").unwrap();
        let order = mf.topological_order();
        let dma_pos = order.iter().position(|n| n == "dma").unwrap();
        let driver_pos = order.iter().position(|n| n == "driver").unwrap();
        assert!(dma_pos < driver_pos);
    }

    #[test]
    fn enabled_filters() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("a", 1, 0, 0)).unwrap();
        mf.register(ModuleInfo::new("b", 1, 0, 0).disabled()).unwrap();
        assert_eq!(mf.enabled().len(), 1);
        assert_eq!(mf.enabled()[0].name, "a");
    }

    #[test]
    fn names_sorted() {
        let mut mf = ModuleManifest::new();
        mf.register(ModuleInfo::new("bravo", 1, 0, 0)).unwrap();
        mf.register(ModuleInfo::new("alpha", 1, 0, 0)).unwrap();
        assert_eq!(mf.names(), vec!["alpha", "bravo"]);
    }

    #[test]
    fn with_capabilities_batch() {
        let m = ModuleInfo::new("x", 1, 0, 0).with_capabilities(&["a", "b", "c"]);
        assert_eq!(m.capabilities.len(), 3);
    }

    #[test]
    fn error_display() {
        assert!(ManifestError::Duplicate { name: "x".into() }.to_string().contains("x"));
        assert!(ManifestError::CircularDep { chain: vec!["a".into(), "b".into()] }.to_string().contains("circular"));
    }
}
