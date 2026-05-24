use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IdxError {
    IndexExists { name: String },
    IndexNotFound { name: String },
    DuplicateKey { index: String, key: Vec<u8>, doc: u64 },
}

impl std::fmt::Display for IdxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdxError::IndexExists { name } => write!(f, "index {name} exists"),
            IdxError::IndexNotFound { name } => write!(f, "index {name} not found"),
            IdxError::DuplicateKey { index, key, doc } => write!(f, "{index}: dup key {:?} doc {doc}", key),
        }
    }
}

impl std::error::Error for IdxError {}

struct Index {
    name: String,
    unique: bool,
    entries: BTreeMap<Vec<u8>, Vec<u64>>,
    total_inserts: u64,
    total_deletes: u64,
}

pub struct IndexEngine {
    indexes: BTreeMap<String, Index>,
    total_lookups: u64,
}

impl IndexEngine {
    pub fn new() -> Self { Self { indexes: BTreeMap::new(), total_lookups: 0 } }

    pub fn create_index(&mut self, name: &str, unique: bool) -> Result<(), IdxError> {
        if self.indexes.contains_key(name) { return Err(IdxError::IndexExists { name: name.to_string() }); }
        self.indexes.insert(name.to_string(), Index { name: name.to_string(), unique, entries: BTreeMap::new(), total_inserts: 0, total_deletes: 0 });
        Ok(())
    }

    pub fn drop_index(&mut self, name: &str) -> Result<(), IdxError> {
        if self.indexes.remove(name).is_none() { return Err(IdxError::IndexNotFound { name: name.to_string() }); }
        Ok(())
    }

    pub fn insert(&mut self, index: &str, key: Vec<u8>, doc_id: u64) -> Result<(), IdxError> {
        let idx = self.indexes.get_mut(index).ok_or_else(|| IdxError::IndexNotFound { name: index.to_string() })?;
        if idx.unique && idx.entries.contains_key(&key) {
            return Err(IdxError::DuplicateKey { index: index.to_string(), key: key.clone(), doc: doc_id });
        }
        idx.entries.entry(key).or_default().push(doc_id);
        idx.total_inserts += 1;
        Ok(())
    }

    pub fn delete(&mut self, index: &str, key: &[u8], doc_id: u64) -> Result<bool, IdxError> {
        let idx = self.indexes.get_mut(index).ok_or_else(|| IdxError::IndexNotFound { name: index.to_string() })?;
        let Some(docs) = idx.entries.get_mut(key) else { return Ok(false); };
        let before = docs.len();
        docs.retain(|&d| d != doc_id);
        let removed = before > docs.len();
        if removed { idx.total_deletes += 1; }
        Ok(removed)
    }

    pub fn lookup(&mut self, index: &str, key: &[u8]) -> Option<Vec<u64>> {
        self.total_lookups += 1;
        self.indexes.get(index).and_then(|idx| idx.entries.get(key).cloned())
    }

    pub fn range(&mut self, index: &str, start: &[u8], end: &[u8]) -> Vec<(Vec<u8>, Vec<u64>)> {
        self.total_lookups += 1;
        self.indexes.get(index).map(|idx| {
            idx.entries.range(start.to_vec()..end.to_vec())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }).unwrap_or_default()
    }

    pub fn prefix(&mut self, index: &str, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u64>)> {
        self.total_lookups += 1;
        self.indexes.get(index).map(|idx| {
            let mut end = prefix.to_vec();
            for b in end.iter_mut().rev() {
                if *b < 255 { *b += 1; break; }
                *b = 0;
            }
            idx.entries.range(prefix.to_vec()..end)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }).unwrap_or_default()
    }

    pub fn index_count(&self) -> usize { self.indexes.len() }
    pub fn index_size(&self, index: &str) -> Option<usize> { self.indexes.get(index).map(|idx| idx.entries.len()) }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
    pub fn index_inserts(&self, index: &str) -> Option<u64> { self.indexes.get(index).map(|idx| idx.total_inserts) }
    pub fn index_deletes(&self, index: &str) -> Option<u64> { self.indexes.get(index).map(|idx| idx.total_deletes) }
}

impl Default for IndexEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_engine() { assert_eq!(IndexEngine::new().index_count(), 0); }

    #[test]
    fn create_insert_lookup() {
        let mut e = IndexEngine::new();
        e.create_index("pk", true).unwrap();
        e.insert("pk", b"k1".to_vec(), 1).unwrap();
        let docs = e.lookup("pk", b"k1").unwrap();
        assert_eq!(docs, vec![1]);
    }

    #[test]
    fn unique_violation() {
        let mut e = IndexEngine::new();
        e.create_index("pk", true).unwrap();
        e.insert("pk", b"k1".to_vec(), 1).unwrap();
        let err = e.insert("pk", b"k1".to_vec(), 2).unwrap_err();
        assert!(matches!(err, IdxError::DuplicateKey { .. }));
    }

    #[test]
    fn non_unique_multi() {
        let mut e = IndexEngine::new();
        e.create_index("tag", false).unwrap();
        e.insert("tag", b"rust".to_vec(), 1).unwrap();
        e.insert("tag", b"rust".to_vec(), 2).unwrap();
        let docs = e.lookup("tag", b"rust").unwrap();
        assert_eq!(docs, vec![1, 2]);
    }

    #[test]
    fn delete_doc() {
        let mut e = IndexEngine::new();
        e.create_index("pk", true).unwrap();
        e.insert("pk", b"k1".to_vec(), 1).unwrap();
        assert!(e.delete("pk", b"k1", 1).unwrap());
        let docs = e.lookup("pk", b"k1").unwrap_or_default();
        assert!(!docs.iter().any(|&d| d == 1));
    }

    #[test]
    fn range_query() {
        let mut e = IndexEngine::new();
        e.create_index("ts", false).unwrap();
        e.insert("ts", b"2024-01".to_vec(), 1).unwrap();
        e.insert("ts", b"2024-02".to_vec(), 2).unwrap();
        e.insert("ts", b"2024-03".to_vec(), 3).unwrap();
        let results = e.range("ts", b"2024-01", b"2024-03");
        assert!(results.len() >= 1);
    }

    #[test]
    fn prefix_query() {
        let mut e = IndexEngine::new();
        e.create_index("name", false).unwrap();
        e.insert("name", b"alice".to_vec(), 1).unwrap();
        e.insert("name", b"alicia".to_vec(), 2).unwrap();
        e.insert("name", b"bob".to_vec(), 3).unwrap();
        let results = e.prefix("name", b"al");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn drop_index() {
        let mut e = IndexEngine::new();
        e.create_index("pk", true).unwrap();
        e.drop_index("pk").unwrap();
        assert_eq!(e.index_count(), 0);
    }

    #[test]
    fn not_found() {
        let mut e = IndexEngine::new();
        let err = e.drop_index("x").unwrap_err();
        assert!(matches!(err, IdxError::IndexNotFound { .. }));
    }

    #[test]
    fn duplicate_index() {
        let mut e = IndexEngine::new();
        e.create_index("pk", true).unwrap();
        let err = e.create_index("pk", true).unwrap_err();
        assert!(matches!(err, IdxError::IndexExists { .. }));
    }

    #[test]
    fn stats() {
        let mut e = IndexEngine::new();
        e.create_index("pk", true).unwrap();
        e.insert("pk", b"k1".to_vec(), 1).unwrap();
        e.lookup("pk", b"k1");
        assert_eq!(e.index_inserts("pk"), Some(1));
        assert_eq!(e.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(IdxError::IndexNotFound { name: "x".into() }.to_string().contains("x")); }
}
