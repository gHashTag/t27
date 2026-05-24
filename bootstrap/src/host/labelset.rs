use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum LabelError {
    InvalidKey { key: String },
    InvalidValue { key: String, value: String },
}

impl std::fmt::Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabelError::InvalidKey { key } => write!(f, "invalid key: {key}"),
            LabelError::InvalidValue { key, value } => write!(f, "invalid value for {key}: {value}"),
        }
    }
}

impl std::error::Error for LabelError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Label { pub key: String, pub value: String }

pub struct LabelSet {
    labels: BTreeMap<String, String>,
    validators: BTreeMap<String, Box<dyn Fn(&str) -> bool>>,
    total_sets: u64,
    total_deletes: u64,
    total_matches: u64,
}

impl LabelSet {
    pub fn new() -> Self { Self { labels: BTreeMap::new(), validators: BTreeMap::new(), total_sets: 0, total_deletes: 0, total_matches: 0 } }

    pub fn add_validator(&mut self, key: &str, validator: Box<dyn Fn(&str) -> bool>) {
        self.validators.insert(key.to_string(), validator);
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), LabelError> {
        if key.is_empty() { return Err(LabelError::InvalidKey { key: key.to_string() }); }
        if let Some(validator) = self.validators.get(key) {
            if !validator(value) { return Err(LabelError::InvalidValue { key: key.to_string(), value: value.to_string() }); }
        }
        self.labels.insert(key.to_string(), value.to_string());
        self.total_sets += 1;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> { self.labels.get(key).map(|s| s.as_str()) }

    pub fn delete(&mut self, key: &str) -> bool {
        self.total_deletes += 1;
        self.labels.remove(key).is_some()
    }

    pub fn has(&self, key: &str, value: &str) -> bool {
        self.labels.get(key).map(|v| v == value).unwrap_or(false)
    }

    pub fn matches(&mut self, selectors: &[(String, String)]) -> bool {
        self.total_matches += 1;
        selectors.iter().all(|(k, v)| self.labels.get(k).map(|lv| lv == v).unwrap_or(false))
    }

    pub fn matches_any(&mut self, selectors: &[(String, String)]) -> bool {
        self.total_matches += 1;
        selectors.iter().any(|(k, v)| self.labels.get(k).map(|lv| lv == v).unwrap_or(false))
    }

    pub fn intersect(&self, other: &LabelSet) -> LabelSet {
        let mut result = LabelSet::new();
        for (k, v) in &self.labels {
            if other.labels.get(k) == Some(v) {
                result.labels.insert(k.clone(), v.clone());
            }
        }
        result
    }

    pub fn union(&self, other: &LabelSet) -> LabelSet {
        let mut result = self.clone();
        for (k, v) in &other.labels {
            result.labels.insert(k.clone(), v.clone());
        }
        result
    }

    pub fn diff(&self, other: &LabelSet) -> LabelSet {
        let mut result = LabelSet::new();
        for (k, v) in &self.labels {
            if other.labels.get(k) != Some(v) {
                result.labels.insert(k.clone(), v.clone());
            }
        }
        result
    }

    pub fn len(&self) -> usize { self.labels.len() }
    pub fn is_empty(&self) -> bool { self.labels.is_empty() }
    pub fn keys(&self) -> Vec<String> { self.labels.keys().cloned().collect() }
    pub fn to_vec(&self) -> Vec<Label> { self.labels.iter().map(|(k, v)| Label { key: k.clone(), value: v.clone() }).collect() }
    pub fn total_sets(&self) -> u64 { self.total_sets }
    pub fn total_deletes(&self) -> u64 { self.total_deletes }
    pub fn total_matches(&self) -> u64 { self.total_matches }
}

impl Clone for LabelSet {
    fn clone(&self) -> Self { Self { labels: self.labels.clone(), validators: BTreeMap::new(), total_sets: self.total_sets, total_deletes: self.total_deletes, total_matches: self.total_matches } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set() { assert!(LabelSet::new().is_empty()); }

    #[test]
    fn set_get() {
        let mut ls = LabelSet::new();
        ls.set("env", "prod").unwrap();
        assert_eq!(ls.get("env"), Some("prod"));
    }

    #[test]
    fn has() {
        let mut ls = LabelSet::new();
        ls.set("env", "prod").unwrap();
        assert!(ls.has("env", "prod"));
        assert!(!ls.has("env", "dev"));
    }

    #[test]
    fn matches() {
        let mut ls = LabelSet::new();
        ls.set("env", "prod").unwrap(); ls.set("team", "api").unwrap();
        assert!(ls.matches(&[("env".into(), "prod".into()), ("team".into(), "api".into())]));
        assert!(!ls.matches(&[("env".into(), "dev".into())]));
    }

    #[test]
    fn matches_any() {
        let mut ls = LabelSet::new();
        ls.set("env", "prod").unwrap();
        assert!(ls.matches_any(&[("env".into(), "dev".into()), ("env".into(), "prod".into())]));
    }

    #[test]
    fn intersect() {
        let mut a = LabelSet::new();
        a.set("env", "prod").unwrap(); a.set("team", "api").unwrap();
        let mut b = LabelSet::new();
        b.set("env", "prod").unwrap(); b.set("team", "infra").unwrap();
        let i = a.intersect(&b);
        assert_eq!(i.len(), 1);
        assert_eq!(i.get("env"), Some("prod"));
    }

    #[test]
    fn union_sets() {
        let mut a = LabelSet::new();
        a.set("env", "prod").unwrap();
        let mut b = LabelSet::new();
        b.set("team", "api").unwrap();
        let u = a.union(&b);
        assert_eq!(u.len(), 2);
    }

    #[test]
    fn diff_sets() {
        let mut a = LabelSet::new();
        a.set("env", "prod").unwrap(); a.set("team", "api").unwrap();
        let mut b = LabelSet::new();
        b.set("env", "prod").unwrap();
        let d = a.diff(&b);
        assert_eq!(d.len(), 1);
        assert_eq!(d.get("team"), Some("api"));
    }

    #[test]
    fn delete() {
        let mut ls = LabelSet::new();
        ls.set("x", "y").unwrap();
        assert!(ls.delete("x"));
        assert!(ls.is_empty());
    }

    #[test]
    fn invalid_key() {
        let mut ls = LabelSet::new();
        let err = ls.set("", "val").unwrap_err();
        assert!(matches!(err, LabelError::InvalidKey { .. }));
    }

    #[test]
    fn stats() {
        let mut ls = LabelSet::new();
        ls.set("a", "1").unwrap();
        ls.delete("a");
        assert_eq!(ls.total_sets(), 1);
        assert_eq!(ls.total_deletes(), 1);
    }
}
