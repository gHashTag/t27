use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IdmError {
    LeftExists,
    RightExists,
    PairNotFound,
}

impl std::fmt::Display for IdmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdmError::LeftExists => write!(f, "left exists"),
            IdmError::RightExists => write!(f, "right exists"),
            IdmError::PairNotFound => write!(f, "pair not found"),
        }
    }
}

impl std::error::Error for IdmError {}

pub struct IdMap<L, R> {
    left_to_right: BTreeMap<L, R>,
    right_to_left: BTreeMap<R, L>,
    total_inserts: u64,
    total_removes: u64,
    total_lookups: u64,
}

impl<L: Ord + Clone + std::fmt::Debug, R: Ord + Clone + std::fmt::Debug> IdMap<L, R> {
    pub fn new() -> Self {
        Self { left_to_right: BTreeMap::new(), right_to_left: BTreeMap::new(), total_inserts: 0, total_removes: 0, total_lookups: 0 }
    }

    pub fn insert(&mut self, left: L, right: R) -> Result<(), IdmError> {
        if self.left_to_right.contains_key(&left) { return Err(IdmError::LeftExists); }
        if self.right_to_left.contains_key(&right) { return Err(IdmError::RightExists); }
        self.left_to_right.insert(left.clone(), right.clone());
        self.right_to_left.insert(right, left);
        self.total_inserts += 1;
        Ok(())
    }

    pub fn remove_left(&mut self, left: &L) -> Result<R, IdmError> {
        let right = self.left_to_right.remove(left).ok_or(IdmError::PairNotFound)?;
        self.right_to_left.remove(&right);
        self.total_removes += 1;
        Ok(right)
    }

    pub fn remove_right(&mut self, right: &R) -> Result<L, IdmError> {
        let left = self.right_to_left.remove(right).ok_or(IdmError::PairNotFound)?;
        self.left_to_right.remove(&left);
        self.total_removes += 1;
        Ok(left)
    }

    pub fn get_right(&mut self, left: &L) -> Option<&R> {
        self.total_lookups += 1;
        self.left_to_right.get(left)
    }

    pub fn get_left(&mut self, right: &R) -> Option<&L> {
        self.total_lookups += 1;
        self.right_to_left.get(right)
    }

    pub fn contains_left(&self, left: &L) -> bool { self.left_to_right.contains_key(left) }
    pub fn contains_right(&self, right: &R) -> bool { self.right_to_left.contains_key(right) }
    pub fn len(&self) -> usize { self.left_to_right.len() }
    pub fn is_empty(&self) -> bool { self.left_to_right.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_lookups(&self) -> u64 { self.total_lookups }
}

impl<L: Ord + Clone + std::fmt::Debug, R: Ord + Clone + std::fmt::Debug> Default for IdMap<L, R> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { let m: IdMap<u64, u64> = IdMap::new(); assert!(m.is_empty()); }

    #[test]
    fn insert_lookup() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        m.insert(1u64, 100u64).unwrap();
        assert_eq!(m.get_right(&1), Some(&100));
        assert_eq!(m.get_left(&100), Some(&1));
    }

    #[test]
    fn duplicate_left() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        m.insert(1, 100).unwrap();
        let err = m.insert(1, 200).unwrap_err();
        assert!(matches!(err, IdmError::LeftExists));
    }

    #[test]
    fn duplicate_right() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        m.insert(1, 100).unwrap();
        let err = m.insert(2, 100).unwrap_err();
        assert!(matches!(err, IdmError::RightExists));
    }

    #[test]
    fn remove_left() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        m.insert(1, 100).unwrap();
        let r = m.remove_left(&1).unwrap();
        assert_eq!(r, 100);
        assert!(m.is_empty());
    }

    #[test]
    fn remove_right() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        m.insert(1, 100).unwrap();
        let l = m.remove_right(&100).unwrap();
        assert_eq!(l, 1);
        assert!(m.is_empty());
    }

    #[test]
    fn remove_missing() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        let err = m.remove_left(&99).unwrap_err();
        assert!(matches!(err, IdmError::PairNotFound));
    }

    #[test]
    fn contains() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        m.insert(1, 100).unwrap();
        assert!(m.contains_left(&1));
        assert!(m.contains_right(&100));
        assert!(!m.contains_left(&2));
    }

    #[test]
    fn string_keys() {
        let mut m: IdMap<String, u64> = IdMap::new();
        m.insert("hello".to_string(), 42).unwrap();
        assert_eq!(m.get_right(&"hello".to_string()), Some(&42));
        assert_eq!(m.get_left(&42), Some(&"hello".to_string()));
    }

    #[test]
    fn stats() {
        let mut m: IdMap<u64, u64> = IdMap::new();
        m.insert(1, 100).unwrap();
        m.get_right(&1);
        m.remove_left(&1).unwrap();
        assert_eq!(m.total_inserts(), 1);
        assert_eq!(m.total_removes(), 1);
        assert_eq!(m.total_lookups(), 1);
    }

    #[test]
    fn error_display() { assert!(IdmError::LeftExists.to_string().contains("left")); }
}
