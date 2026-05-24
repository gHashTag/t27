use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    String(String),
}

impl std::fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigValue::Bool(v) => write!(f, "{v}"),
            ConfigValue::U8(v) => write!(f, "{v}"),
            ConfigValue::U16(v) => write!(f, "{v}"),
            ConfigValue::U32(v) => write!(f, "{v}"),
            ConfigValue::U64(v) => write!(f, "{v}"),
            ConfigValue::String(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    NotFound { key: String },
    TypeMismatch { key: String },
    ReadOnly { key: String },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::NotFound { key } => write!(f, "not found: {key}"),
            RegistryError::TypeMismatch { key } => write!(f, "type mismatch: {key}"),
            RegistryError::ReadOnly { key } => write!(f, "read-only: {key}"),
        }
    }
}

impl std::error::Error for RegistryError {}

#[derive(Debug, Clone)]
struct Entry {
    value: ConfigValue,
    read_only: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigRegistry {
    entries: BTreeMap<String, Entry>,
    change_count: u64,
}

impl ConfigRegistry {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            change_count: 0,
        }
    }

    pub fn set(&mut self, key: &str, value: ConfigValue) -> Result<(), RegistryError> {
        if let Some(entry) = self.entries.get(key) {
            if entry.read_only {
                return Err(RegistryError::ReadOnly { key: key.to_string() });
            }
        }
        self.entries.insert(key.to_string(), Entry { value, read_only: false });
        self.change_count += 1;
        Ok(())
    }

    pub fn set_readonly(&mut self, key: &str, value: ConfigValue) {
        self.entries.insert(key.to_string(), Entry { value, read_only: true });
        self.change_count += 1;
    }

    pub fn get(&self, key: &str) -> Result<&ConfigValue, RegistryError> {
        self.entries
            .get(key)
            .map(|e| &e.value)
            .ok_or(RegistryError::NotFound { key: key.to_string() })
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, RegistryError> {
        match self.get(key)? {
            ConfigValue::Bool(v) => Ok(*v),
            _ => Err(RegistryError::TypeMismatch { key: key.to_string() }),
        }
    }

    pub fn get_u32(&self, key: &str) -> Result<u32, RegistryError> {
        match self.get(key)? {
            ConfigValue::U32(v) => Ok(*v),
            _ => Err(RegistryError::TypeMismatch { key: key.to_string() }),
        }
    }

    pub fn get_u64(&self, key: &str) -> Result<u64, RegistryError> {
        match self.get(key)? {
            ConfigValue::U64(v) => Ok(*v),
            _ => Err(RegistryError::TypeMismatch { key: key.to_string() }),
        }
    }

    pub fn get_string(&self, key: &str) -> Result<&str, RegistryError> {
        match self.get(key)? {
            ConfigValue::String(v) => Ok(v),
            _ => Err(RegistryError::TypeMismatch { key: key.to_string() }),
        }
    }

    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> bool {
        if let Some(entry) = self.entries.get(key) {
            if entry.read_only {
                return false;
            }
        }
        self.entries.remove(key).is_some()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn change_count(&self) -> u64 {
        self.change_count
    }

    pub fn is_readonly(&self, key: &str) -> bool {
        self.entries.get(key).map_or(false, |e| e.read_only)
    }

    pub fn clear(&mut self) {
        self.entries.retain(|_, e| e.read_only);
    }

    pub fn snapshot(&self) -> Vec<(&str, &ConfigValue, bool)> {
        self.entries
            .iter()
            .map(|(k, e)| (k.as_str(), &e.value, e.read_only))
            .collect()
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let mut r = ConfigRegistry::new();
        r.set("timeout", ConfigValue::U32(5000)).unwrap();
        assert_eq!(r.get_u32("timeout").unwrap(), 5000);
    }

    #[test]
    fn type_mismatch() {
        let mut r = ConfigRegistry::new();
        r.set("flag", ConfigValue::Bool(true)).unwrap();
        let err = r.get_u32("flag").unwrap_err();
        assert!(matches!(err, RegistryError::TypeMismatch { .. }));
    }

    #[test]
    fn not_found() {
        let r = ConfigRegistry::new();
        let err = r.get("missing").unwrap_err();
        assert!(matches!(err, RegistryError::NotFound { .. }));
    }

    #[test]
    fn readonly_cannot_be_overwritten() {
        let mut r = ConfigRegistry::new();
        r.set_readonly("version", ConfigValue::U8(1));
        let err = r.set("version", ConfigValue::U8(2)).unwrap_err();
        assert!(matches!(err, RegistryError::ReadOnly { .. }));
    }

    #[test]
    fn readonly_cannot_be_removed() {
        let mut r = ConfigRegistry::new();
        r.set_readonly("ver", ConfigValue::U8(1));
        assert!(!r.remove("ver"));
        assert!(r.contains("ver"));
    }

    #[test]
    fn all_types() {
        let mut r = ConfigRegistry::new();
        r.set("b", ConfigValue::Bool(true)).unwrap();
        r.set("u8", ConfigValue::U8(42)).unwrap();
        r.set("u16", ConfigValue::U16(1000)).unwrap();
        r.set("u32", ConfigValue::U32(99999)).unwrap();
        r.set("u64", ConfigValue::U64(1_000_000)).unwrap();
        r.set("s", ConfigValue::String("hello".into())).unwrap();
        assert!(r.get_bool("b").unwrap());
        assert_eq!(r.get_u64("u64").unwrap(), 1_000_000);
        assert_eq!(r.get_string("s").unwrap(), "hello");
    }

    #[test]
    fn remove_normal() {
        let mut r = ConfigRegistry::new();
        r.set("x", ConfigValue::U32(1)).unwrap();
        assert!(r.remove("x"));
        assert!(!r.contains("x"));
        assert!(!r.remove("x"));
    }

    #[test]
    fn keys_sorted() {
        let mut r = ConfigRegistry::new();
        r.set("charlie", ConfigValue::U32(3)).unwrap();
        r.set("alpha", ConfigValue::U32(1)).unwrap();
        r.set("bravo", ConfigValue::U32(2)).unwrap();
        assert_eq!(r.keys(), vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn clear_preserves_readonly() {
        let mut r = ConfigRegistry::new();
        r.set("x", ConfigValue::U32(1)).unwrap();
        r.set_readonly("ver", ConfigValue::U8(1));
        r.clear();
        assert!(!r.contains("x"));
        assert!(r.contains("ver"));
    }

    #[test]
    fn change_count() {
        let mut r = ConfigRegistry::new();
        r.set("a", ConfigValue::U32(1)).unwrap();
        r.set("b", ConfigValue::U32(2)).unwrap();
        assert_eq!(r.change_count(), 2);
    }

    #[test]
    fn is_readonly() {
        let mut r = ConfigRegistry::new();
        r.set("a", ConfigValue::U32(1)).unwrap();
        r.set_readonly("b", ConfigValue::U32(2));
        assert!(!r.is_readonly("a"));
        assert!(r.is_readonly("b"));
    }

    #[test]
    fn snapshot() {
        let mut r = ConfigRegistry::new();
        r.set("x", ConfigValue::U32(1)).unwrap();
        r.set_readonly("y", ConfigValue::U32(2));
        let snap = r.snapshot();
        assert_eq!(snap.len(), 2);
    }

    #[test]
    fn error_display() {
        assert!(RegistryError::NotFound { key: "k".into() }.to_string().contains("k"));
        assert!(RegistryError::ReadOnly { key: "k".into() }.to_string().contains("read-only"));
    }

    #[test]
    fn default_is_empty() {
        assert!(ConfigRegistry::default().is_empty());
    }
}
