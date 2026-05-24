use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffOp {
    Add { key: String, value: String },
    Remove { key: String },
    Change { key: String, old: String, new: String },
}

impl std::fmt::Display for DiffOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffOp::Add { key, value } => write!(f, "+{key}={value}"),
            DiffOp::Remove { key } => write!(f, "-{key}"),
            DiffOp::Change { key, old, new } => write!(f, "~{key}:{old}->{new}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigDiff {
    ops: Vec<DiffOp>,
}

impl ConfigDiff {
    pub fn new(ops: Vec<DiffOp>) -> Self {
        Self { ops }
    }

    pub fn ops(&self) -> &[DiffOp] {
        &self.ops
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn additions(&self) -> Vec<&DiffOp> {
        self.ops.iter().filter(|o| matches!(o, DiffOp::Add { .. })).collect()
    }

    pub fn removals(&self) -> Vec<&DiffOp> {
        self.ops.iter().filter(|o| matches!(o, DiffOp::Remove { .. })).collect()
    }

    pub fn changes(&self) -> Vec<&DiffOp> {
        self.ops.iter().filter(|o| matches!(o, DiffOp::Change { .. })).collect()
    }
}

#[derive(Debug, Clone)]
pub struct ConfigMap {
    entries: BTreeMap<String, String>,
}

impl ConfigMap {
    pub fn new() -> Self {
        Self { entries: BTreeMap::new() }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.entries.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    pub fn diff(&self, other: &ConfigMap) -> ConfigDiff {
        let mut ops = Vec::new();
        for (key, value) in &self.entries {
            match other.entries.get(key) {
                None => ops.push(DiffOp::Remove { key: key.clone() }),
                Some(new_val) if new_val != value => ops.push(DiffOp::Change {
                    key: key.clone(),
                    old: value.clone(),
                    new: new_val.clone(),
                }),
                _ => {}
            }
        }
        for (key, value) in &other.entries {
            if !self.entries.contains_key(key) {
                ops.push(DiffOp::Add { key: key.clone(), value: value.clone() });
            }
        }
        ConfigDiff::new(ops)
    }

    pub fn apply(&mut self, diff: &ConfigDiff) -> usize {
        let mut applied = 0;
        for op in &diff.ops {
            match op {
                DiffOp::Add { key, value } => {
                    self.entries.insert(key.clone(), value.clone());
                    applied += 1;
                }
                DiffOp::Remove { key } => {
                    if self.entries.remove(key).is_some() {
                        applied += 1;
                    }
                }
                DiffOp::Change { key, new, .. } => {
                    if self.entries.contains_key(key) {
                        self.entries.insert(key.clone(), new.clone());
                        applied += 1;
                    }
                }
            }
        }
        applied
    }

    pub fn clone_map(&self) -> BTreeMap<String, String> {
        self.entries.clone()
    }
}

impl Default for ConfigMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_op_display() {
        let add = DiffOp::Add { key: "x".into(), value: "1".into() };
        assert_eq!(add.to_string(), "+x=1");
        let rm = DiffOp::Remove { key: "y".into() };
        assert_eq!(rm.to_string(), "-y");
        let ch = DiffOp::Change { key: "z".into(), old: "a".into(), new: "b".into() };
        assert_eq!(ch.to_string(), "~z:a->b");
    }

    #[test]
    fn set_get_remove() {
        let mut cm = ConfigMap::new();
        cm.set("freq", "66");
        assert_eq!(cm.get("freq"), Some("66"));
        assert!(cm.remove("freq"));
        assert_eq!(cm.get("freq"), None);
    }

    #[test]
    fn diff_no_change() {
        let mut a = ConfigMap::new();
        a.set("x", "1");
        let mut b = ConfigMap::new();
        b.set("x", "1");
        let diff = a.diff(&b);
        assert!(diff.is_empty());
    }

    #[test]
    fn diff_addition() {
        let a = ConfigMap::new();
        let mut b = ConfigMap::new();
        b.set("x", "1");
        let diff = a.diff(&b);
        assert_eq!(diff.additions().len(), 1);
        assert_eq!(diff.len(), 1);
    }

    #[test]
    fn diff_removal() {
        let mut a = ConfigMap::new();
        a.set("x", "1");
        let b = ConfigMap::new();
        let diff = a.diff(&b);
        assert_eq!(diff.removals().len(), 1);
    }

    #[test]
    fn diff_change() {
        let mut a = ConfigMap::new();
        a.set("x", "1");
        let mut b = ConfigMap::new();
        b.set("x", "2");
        let diff = a.diff(&b);
        assert_eq!(diff.changes().len(), 1);
    }

    #[test]
    fn diff_mixed() {
        let mut a = ConfigMap::new();
        a.set("a", "1");
        a.set("b", "2");
        let mut b = ConfigMap::new();
        b.set("a", "1");
        b.set("b", "3");
        b.set("c", "4");
        let diff = a.diff(&b);
        assert_eq!(diff.additions().len(), 1);
        assert_eq!(diff.changes().len(), 1);
        assert_eq!(diff.len(), 2);
    }

    #[test]
    fn apply_add() {
        let mut a = ConfigMap::new();
        let mut b = ConfigMap::new();
        b.set("x", "1");
        let diff = a.diff(&b);
        let applied = a.apply(&diff);
        assert_eq!(applied, 1);
        assert_eq!(a.get("x"), Some("1"));
    }

    #[test]
    fn apply_roundtrip() {
        let mut a = ConfigMap::new();
        a.set("x", "1");
        a.set("y", "2");
        let original = a.clone_map();
        let mut b = ConfigMap::new();
        b.set("x", "1");
        b.set("y", "2");
        let diff = a.diff(&b);
        let applied = a.apply(&diff);
        assert_eq!(applied, 0);
        assert_eq!(a.clone_map(), original);
    }

    #[test]
    fn apply_change_and_verify() {
        let mut a = ConfigMap::new();
        a.set("freq", "66");
        let mut b = ConfigMap::new();
        b.set("freq", "100");
        let diff = a.diff(&b);
        a.apply(&diff);
        assert_eq!(a.get("freq"), Some("100"));
    }

    #[test]
    fn keys_sorted() {
        let mut cm = ConfigMap::new();
        cm.set("bravo", "2");
        cm.set("alpha", "1");
        assert_eq!(cm.keys(), vec!["alpha", "bravo"]);
    }
}
