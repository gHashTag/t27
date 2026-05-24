#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SparseError {
    OutOfRange { value: usize, universe: usize },
    AlreadyMember { value: usize },
}

impl std::fmt::Display for SparseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SparseError::OutOfRange { value, universe } => write!(f, "{value} >= universe {universe}"),
            SparseError::AlreadyMember { value } => write!(f, "{value} already in set"),
        }
    }
}

impl std::error::Error for SparseError {}

#[derive(Debug, Clone)]
pub struct SparseSet {
    sparse: Vec<usize>,
    dense: Vec<usize>,
    universe: usize,
    total_inserts: u64,
    total_removes: u64,
    peak_size: usize,
}

impl SparseSet {
    pub fn new(universe: usize) -> Self {
        Self {
            sparse: vec![0; universe],
            dense: Vec::new(),
            universe,
            total_inserts: 0,
            total_removes: 0,
            peak_size: 0,
        }
    }

    pub fn universe(&self) -> usize {
        self.universe
    }

    pub fn insert(&mut self, value: usize) -> Result<bool, SparseError> {
        if value >= self.universe {
            return Err(SparseError::OutOfRange { value, universe: self.universe });
        }
        if self.contains(value) {
            return Err(SparseError::AlreadyMember { value });
        }
        self.sparse[value] = self.dense.len();
        self.dense.push(value);
        self.total_inserts += 1;
        if self.dense.len() > self.peak_size {
            self.peak_size = self.dense.len();
        }
        Ok(true)
    }

    pub fn remove(&mut self, value: usize) -> bool {
        if !self.contains(value) { return false; }
        let idx = self.sparse[value];
        let last = self.dense.len() - 1;
        if idx != last {
            let last_val = self.dense[last];
            self.dense[idx] = last_val;
            self.sparse[last_val] = idx;
        }
        self.dense.pop();
        self.total_removes += 1;
        true
    }

    pub fn contains(&self, value: usize) -> bool {
        if value >= self.universe { return false; }
        let idx = self.sparse[value];
        idx < self.dense.len() && self.dense[idx] == value
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn clear(&mut self) {
        self.dense.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.dense.iter().copied()
    }

    pub fn members(&self) -> Vec<usize> {
        self.dense.clone()
    }

    pub fn random_member(&self) -> Option<usize> {
        if self.dense.is_empty() { return None; }
        Some(self.dense[0])
    }

    pub fn capacity(&self) -> usize {
        self.universe
    }

    pub fn total_inserts(&self) -> u64 {
        self.total_inserts
    }

    pub fn total_removes(&self) -> u64 {
        self.total_removes
    }

    pub fn peak_size(&self) -> usize {
        self.peak_size
    }

    pub fn density(&self) -> f64 {
        if self.universe == 0 { 0.0 } else { self.dense.len() as f64 / self.universe as f64 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set() {
        let ss = SparseSet::new(100);
        assert_eq!(ss.universe(), 100);
        assert!(ss.is_empty());
    }

    #[test]
    fn insert_and_contains() {
        let mut ss = SparseSet::new(100);
        ss.insert(5).unwrap();
        assert!(ss.contains(5));
        assert!(!ss.contains(6));
        assert_eq!(ss.len(), 1);
    }

    #[test]
    fn duplicate_insert() {
        let mut ss = SparseSet::new(100);
        ss.insert(5).unwrap();
        let err = ss.insert(5).unwrap_err();
        assert!(matches!(err, SparseError::AlreadyMember { value: 5 }));
    }

    #[test]
    fn out_of_range() {
        let mut ss = SparseSet::new(10);
        let err = ss.insert(10).unwrap_err();
        assert!(matches!(err, SparseError::OutOfRange { .. }));
    }

    #[test]
    fn remove() {
        let mut ss = SparseSet::new(100);
        ss.insert(5).unwrap();
        assert!(ss.remove(5));
        assert!(!ss.contains(5));
        assert!(ss.is_empty());
    }

    #[test]
    fn remove_not_present() {
        let mut ss = SparseSet::new(100);
        assert!(!ss.remove(5));
    }

    #[test]
    fn swap_remove_integrity() {
        let mut ss = SparseSet::new(100);
        ss.insert(1).unwrap();
        ss.insert(2).unwrap();
        ss.insert(3).unwrap();
        ss.remove(2);
        assert!(ss.contains(1));
        assert!(ss.contains(3));
        assert_eq!(ss.len(), 2);
    }

    #[test]
    fn clear() {
        let mut ss = SparseSet::new(100);
        ss.insert(1).unwrap();
        ss.insert(2).unwrap();
        ss.clear();
        assert!(ss.is_empty());
        assert!(!ss.contains(1));
    }

    #[test]
    fn members() {
        let mut ss = SparseSet::new(100);
        ss.insert(10).unwrap();
        ss.insert(20).unwrap();
        ss.insert(30).unwrap();
        let m = ss.members();
        assert_eq!(m.len(), 3);
        assert!(m.contains(&10) && m.contains(&20) && m.contains(&30));
    }

    #[test]
    fn peak_size() {
        let mut ss = SparseSet::new(100);
        ss.insert(1).unwrap();
        ss.insert(2).unwrap();
        ss.insert(3).unwrap();
        ss.remove(2);
        assert_eq!(ss.peak_size(), 3);
    }

    #[test]
    fn stats() {
        let mut ss = SparseSet::new(100);
        ss.insert(1).unwrap();
        ss.insert(2).unwrap();
        ss.remove(1);
        assert_eq!(ss.total_inserts(), 2);
        assert_eq!(ss.total_removes(), 1);
        assert!((ss.density() - 0.01).abs() < 0.001);
    }

    #[test]
    fn error_display() {
        assert!(SparseError::OutOfRange { value: 10, universe: 5 }.to_string().contains("10"));
    }
}
