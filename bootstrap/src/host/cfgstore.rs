use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CfgError {
    LayerExists { name: String },
    LayerNotFound { name: String },
    ValidationFailed { key: String, reason: String },
}

impl std::fmt::Display for CfgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfgError::LayerExists { name } => write!(f, "layer {name} exists"),
            CfgError::LayerNotFound { name } => write!(f, "layer {name} not found"),
            CfgError::ValidationFailed { key, reason } => write!(f, "{key}: {reason}"),
        }
    }
}

impl std::error::Error for CfgError {}

pub type Validator = Box<dyn Fn(&str, &str) -> Result<(), String>>;

struct Layer {
    name: String,
    priority: u32,
    values: BTreeMap<String, String>,
}

pub struct ConfigStore {
    layers: BTreeMap<String, Layer>,
    layer_order: Vec<String>,
    validators: BTreeMap<String, Validator>,
    total_gets: u64,
    total_sets: u64,
}

impl ConfigStore {
    pub fn new() -> Self { Self { layers: BTreeMap::new(), layer_order: Vec::new(), validators: BTreeMap::new(), total_gets: 0, total_sets: 0 } }

    pub fn add_layer(&mut self, name: &str, priority: u32) -> Result<(), CfgError> {
        if self.layers.contains_key(name) { return Err(CfgError::LayerExists { name: name.to_string() }); }
        self.layers.insert(name.to_string(), Layer { name: name.to_string(), priority, values: BTreeMap::new() });
        self.layer_order.push(name.to_string());
        self.layer_order.sort_by_key(|n| std::cmp::Reverse(self.layers[n].priority));
        Ok(())
    }

    pub fn set(&mut self, layer: &str, key: &str, value: &str) -> Result<(), CfgError> {
        let l = self.layers.get_mut(layer).ok_or_else(|| CfgError::LayerNotFound { name: layer.to_string() })?;
        if let Some(v) = self.validators.get(key) {
            v(key, value).map_err(|reason| CfgError::ValidationFailed { key: key.to_string(), reason })?;
        }
        l.values.insert(key.to_string(), value.to_string());
        self.total_sets += 1;
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        self.total_gets += 1;
        for lname in &self.layer_order {
            if let Some(l) = self.layers.get(lname) {
                if let Some(v) = l.values.get(key) { return Some(v.clone()); }
            }
        }
        None
    }

    pub fn get_from(&self, layer: &str, key: &str) -> Option<String> {
        self.layers.get(layer).and_then(|l| l.values.get(key).cloned())
    }

    pub fn add_validator(&mut self, key: &str, validator: Validator) {
        self.validators.insert(key.to_string(), validator);
    }

    pub fn remove_layer(&mut self, name: &str) -> Result<usize, CfgError> {
        let l = self.layers.remove(name).ok_or_else(|| CfgError::LayerNotFound { name: name.to_string() })?;
        self.layer_order.retain(|n| n != name);
        Ok(l.values.len())
    }

    pub fn layer_count(&self) -> usize { self.layers.len() }
    pub fn layer_keys(&self, layer: &str) -> Option<Vec<String>> {
        self.layers.get(layer).map(|l| l.values.keys().cloned().collect())
    }
    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_sets(&self) -> u64 { self.total_sets }
}

impl Default for ConfigStore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_store() { assert_eq!(ConfigStore::new().layer_count(), 0); }

    #[test]
    fn add_layer() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.add_layer("override", 10).unwrap();
        assert_eq!(cs.layer_count(), 2);
    }

    #[test]
    fn duplicate_layer() {
        let mut cs = ConfigStore::new();
        cs.add_layer("x", 0).unwrap();
        let err = cs.add_layer("x", 0).unwrap_err();
        assert!(matches!(err, CfgError::LayerExists { .. }));
    }

    #[test]
    fn set_get() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.set("base", "port", "8080").unwrap();
        assert_eq!(cs.get("port"), Some("8080".to_string()));
    }

    #[test]
    fn priority_override() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.add_layer("env", 10).unwrap();
        cs.set("base", "port", "8080").unwrap();
        cs.set("env", "port", "9090").unwrap();
        assert_eq!(cs.get("port"), Some("9090".to_string()));
    }

    #[test]
    fn layer_not_found() {
        let mut cs = ConfigStore::new();
        let err = cs.set("nope", "k", "v").unwrap_err();
        assert!(matches!(err, CfgError::LayerNotFound { .. }));
    }

    #[test]
    fn validator() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.add_validator("port", Box::new(|_k, v| {
            if v.parse::<u16>().is_ok() { Ok(()) } else { Err("not a port".to_string()) }
        }));
        cs.set("base", "port", "8080").unwrap();
        let err = cs.set("base", "port", "abc").unwrap_err();
        assert!(matches!(err, CfgError::ValidationFailed { .. }));
    }

    #[test]
    fn remove_layer() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.set("base", "k", "v").unwrap();
        let removed = cs.remove_layer("base").unwrap();
        assert_eq!(removed, 1);
        assert_eq!(cs.layer_count(), 0);
    }

    #[test]
    fn get_from() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.set("base", "k", "v").unwrap();
        assert_eq!(cs.get_from("base", "k"), Some("v".to_string()));
        assert_eq!(cs.get_from("base", "x"), None);
    }

    #[test]
    fn layer_keys() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.set("base", "a", "1").unwrap();
        cs.set("base", "b", "2").unwrap();
        let keys = cs.layer_keys("base").unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn stats() {
        let mut cs = ConfigStore::new();
        cs.add_layer("base", 0).unwrap();
        cs.set("base", "k", "v").unwrap();
        cs.get("k"); cs.get("missing");
        assert_eq!(cs.total_sets(), 1);
        assert_eq!(cs.total_gets(), 2);
    }

    #[test]
    fn error_display() { assert!(CfgError::LayerNotFound { name: "x".into() }.to_string().contains("x")); }
}
