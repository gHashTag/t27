use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CfgError {
    MissingRequired { key: String },
    InvalidType { key: String, expected: String },
    ValidationFailed { key: String, reason: String },
    ConstraintViolation { msg: String },
}

impl std::fmt::Display for CfgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CfgError::MissingRequired { key } => write!(f, "missing required: {key}"),
            CfgError::InvalidType { key, expected } => write!(f, "{key}: expected {expected}"),
            CfgError::ValidationFailed { key, reason } => write!(f, "{key}: {reason}"),
            CfgError::ConstraintViolation { msg } => write!(f, "constraint: {msg}"),
        }
    }
}

impl std::error::Error for CfgError {}

#[derive(Debug, Clone, PartialEq)]
pub enum CfgValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl CfgValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self { CfgValue::Bool(b) => Some(*b), _ => None }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self { CfgValue::Int(i) => Some(*i), _ => None }
    }
    pub fn as_float(&self) -> Option<f64> {
        match self { CfgValue::Float(f) => Some(*f), CfgValue::Int(i) => Some(*i as f64), _ => None }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self { CfgValue::Str(s) => Some(s.as_str()), _ => None }
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            CfgValue::Bool(_) => "bool",
            CfgValue::Int(_) => "int",
            CfgValue::Float(_) => "float",
            CfgValue::Str(_) => "str",
        }
    }
}

pub type Validator = fn(&CfgValue) -> Result<(), String>;

#[derive(Clone)]
pub struct FieldSpec {
    pub key: &'static str,
    pub default: Option<CfgValue>,
    pub required: bool,
    pub validator: Option<Validator>,
}

#[derive(Clone)]
pub struct TypedConfig {
    values: BTreeMap<String, CfgValue>,
    specs: Vec<FieldSpec>,
}

impl TypedConfig {
    pub fn new(specs: Vec<FieldSpec>) -> Self {
        let mut values = BTreeMap::new();
        for spec in &specs {
            if let Some(ref def) = spec.default {
                values.insert(spec.key.to_string(), def.clone());
            }
        }
        Self { values, specs }
    }

    pub fn set(&mut self, key: &str, value: CfgValue) -> Result<(), CfgError> {
        if let Some(spec) = self.specs.iter().find(|s| s.key == key) {
            if let Some(v) = &spec.validator {
                v(&value).map_err(|reason| CfgError::ValidationFailed { key: key.to_string(), reason })?;
            }
            self.values.insert(key.to_string(), value);
            Ok(())
        } else {
            self.values.insert(key.to_string(), value);
            Ok(())
        }
    }

    pub fn get(&self, key: &str) -> Option<&CfgValue> {
        self.values.get(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.values.get(key).and_then(|v| v.as_int())
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| v.as_str())
    }

    pub fn validate(&self) -> Result<(), Vec<CfgError>> {
        let mut errors = Vec::new();
        for spec in &self.specs {
            if spec.required && !self.values.contains_key(spec.key) {
                errors.push(CfgError::MissingRequired { key: spec.key.to_string() });
            }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    pub fn keys(&self) -> Vec<&str> {
        self.values.keys().map(|s| s.as_str()).collect()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn merge(&mut self, other: &TypedConfig) -> usize {
        let mut count = 0;
        for (key, value) in &other.values {
            if !self.values.contains_key(key) {
                self.values.insert(key.clone(), value.clone());
                count += 1;
            }
        }
        count
    }

    pub fn layer(&mut self, overrides: &BTreeMap<String, CfgValue>) -> usize {
        let mut count = 0;
        for (key, value) in overrides {
            if self.set(key, value.clone()).is_ok() {
                count += 1;
            }
        }
        count
    }
}

fn positive_int(v: &CfgValue) -> Result<(), String> {
    match v.as_int() {
        Some(n) if n > 0 => Ok(()),
        Some(n) => Err(format!("must be positive, got {n}")),
        None => Err("expected int".into()),
    }
}

fn non_empty_str(v: &CfgValue) -> Result<(), String> {
    match v.as_str() {
        Some(s) if !s.is_empty() => Ok(()),
        Some(_) => Err("must be non-empty".into()),
        None => Err("expected str".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_value_types() {
        assert_eq!(CfgValue::Bool(true).type_name(), "bool");
        assert_eq!(CfgValue::Int(42).type_name(), "int");
        assert_eq!(CfgValue::Float(3.14).type_name(), "float");
        assert_eq!(CfgValue::Str("hi".into()).type_name(), "str");
    }

    #[test]
    fn cfg_value_conversions() {
        assert_eq!(CfgValue::Bool(true).as_bool(), Some(true));
        assert_eq!(CfgValue::Int(7).as_int(), Some(7));
        assert_eq!(CfgValue::Int(7).as_float(), Some(7.0));
        assert_eq!(CfgValue::Str("x".into()).as_str(), Some("x"));
        assert_eq!(CfgValue::Bool(true).as_int(), None);
    }

    #[test]
    fn new_with_defaults() {
        let cfg = TypedConfig::new(vec![
            FieldSpec { key: "port", default: Some(CfgValue::Int(8080)), required: false, validator: None },
        ]);
        assert_eq!(cfg.get_int("port"), Some(8080));
    }

    #[test]
    fn set_and_get() {
        let mut cfg = TypedConfig::new(vec![]);
        cfg.set("freq", CfgValue::Int(66)).unwrap();
        assert_eq!(cfg.get_int("freq"), Some(66));
    }

    #[test]
    fn validation_passes() {
        let cfg = TypedConfig::new(vec![
            FieldSpec { key: "name", default: Some(CfgValue::Str("test".into())), required: true, validator: None },
        ]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validation_missing_required() {
        let cfg = TypedConfig::new(vec![
            FieldSpec { key: "name", default: None, required: true, validator: None },
        ]);
        let errs = cfg.validate().unwrap_err();
        assert_eq!(errs.len(), 1);
        assert!(matches!(&errs[0], CfgError::MissingRequired { key } if key == "name"));
    }

    #[test]
    fn validator_rejects() {
        let mut cfg = TypedConfig::new(vec![
            FieldSpec { key: "count", default: None, required: false, validator: Some(positive_int) },
        ]);
        let err = cfg.set("count", CfgValue::Int(-1)).unwrap_err();
        assert!(matches!(err, CfgError::ValidationFailed { .. }));
    }

    #[test]
    fn validator_accepts() {
        let mut cfg = TypedConfig::new(vec![
            FieldSpec { key: "count", default: None, required: false, validator: Some(positive_int) },
        ]);
        cfg.set("count", CfgValue::Int(5)).unwrap();
        assert_eq!(cfg.get_int("count"), Some(5));
    }

    #[test]
    fn merge_adds_missing() {
        let mut a = TypedConfig::new(vec![
            FieldSpec { key: "x", default: Some(CfgValue::Int(1)), required: false, validator: None },
        ]);
        let mut b = TypedConfig::new(vec![]);
        b.set("y", CfgValue::Int(2)).unwrap();
        let count = a.merge(&b);
        assert_eq!(count, 1);
        assert_eq!(a.get_int("y"), Some(2));
        assert_eq!(a.get_int("x"), Some(1));
    }

    #[test]
    fn layer_overrides() {
        let mut cfg = TypedConfig::new(vec![
            FieldSpec { key: "x", default: Some(CfgValue::Int(1)), required: false, validator: None },
        ]);
        let mut overrides = BTreeMap::new();
        overrides.insert("x".into(), CfgValue::Int(99));
        cfg.layer(&overrides);
        assert_eq!(cfg.get_int("x"), Some(99));
    }

    #[test]
    fn keys_and_len() {
        let mut cfg = TypedConfig::new(vec![]);
        cfg.set("b", CfgValue::Int(2)).unwrap();
        cfg.set("a", CfgValue::Int(1)).unwrap();
        assert_eq!(cfg.keys(), vec!["a", "b"]);
        assert_eq!(cfg.len(), 2);
        assert!(!cfg.is_empty());
    }

    #[test]
    fn error_display() {
        let e = CfgError::MissingRequired { key: "port".into() };
        assert!(e.to_string().contains("port"));
        let e2 = CfgError::ConstraintViolation { msg: "bad".into() };
        assert!(e2.to_string().contains("bad"));
    }
}
