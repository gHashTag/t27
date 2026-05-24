use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerError {
    VersionMismatch { expected: u64, found: u64 },
    NoHistory,
    MaxHistory,
}

impl std::fmt::Display for VerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerError::VersionMismatch { expected, found } => {
                write!(f, "expected v{expected}, found v{found}")
            }
            VerError::NoHistory => write!(f, "no history"),
            VerError::MaxHistory => write!(f, "history full"),
        }
    }
}

impl std::error::Error for VerError {}

#[derive(Debug, Clone)]
struct VersionedEntry<T> {
    value: T,
    version: u64,
}

#[derive(Debug, Clone)]
pub struct VersionedCell<T> {
    current: T,
    version: u64,
    history: VecDeque<VersionedEntry<T>>,
    max_history: usize,
    total_writes: u64,
    total_rollbacks: u64,
}

impl<T: Clone> VersionedCell<T> {
    pub fn new(value: T, max_history: usize) -> Self {
        Self {
            current: value,
            version: 0,
            history: VecDeque::with_capacity(max_history),
            max_history: max_history.max(1),
            total_writes: 0,
            total_rollbacks: 0,
        }
    }

    pub fn get(&self) -> &T {
        &self.current
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn set(&mut self, value: T) -> u64 {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(VersionedEntry { value: self.current.clone(), version: self.version });
        self.version += 1;
        self.current = value;
        self.total_writes += 1;
        self.version
    }

    pub fn cas(&mut self, expected_version: u64, value: T) -> Result<u64, VerError> {
        if self.version != expected_version {
            return Err(VerError::VersionMismatch { expected: expected_version, found: self.version });
        }
        Ok(self.set(value))
    }

    pub fn rollback(&mut self) -> Result<u64, VerError> {
        let entry = self.history.pop_back().ok_or(VerError::NoHistory)?;
        self.current = entry.value;
        self.version = entry.version;
        self.total_rollbacks += 1;
        Ok(self.version)
    }

    pub fn rollback_to(&mut self, target_version: u64) -> Result<u64, VerError> {
        while self.version > target_version {
            if self.history.is_empty() { return Err(VerError::NoHistory); }
            let entry = self.history.pop_back().unwrap();
            self.current = entry.value;
            self.version = entry.version;
            self.total_rollbacks += 1;
            if self.version == target_version { return Ok(self.version); }
        }
        Err(VerError::VersionMismatch { expected: target_version, found: self.version })
    }

    pub fn history_depth(&self) -> usize {
        self.history.len()
    }

    pub fn max_history(&self) -> usize {
        self.max_history
    }

    pub fn total_writes(&self) -> u64 {
        self.total_writes
    }

    pub fn total_rollbacks(&self) -> u64 {
        self.total_rollbacks
    }

    pub fn is_clean(&self) -> bool {
        self.history.is_empty()
    }

    pub fn reset(&mut self, value: T) {
        self.current = value;
        self.version = 0;
        self.history.clear();
        self.total_writes = 0;
        self.total_rollbacks = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cell() {
        let vc: VersionedCell<i32> = VersionedCell::new(42, 10);
        assert_eq!(*vc.get(), 42);
        assert_eq!(vc.version(), 0);
    }

    #[test]
    fn set_increments_version() {
        let mut vc = VersionedCell::new(0, 10);
        let v1 = vc.set(10);
        let v2 = vc.set(20);
        assert_eq!(v1, 1);
        assert_eq!(v2, 2);
        assert_eq!(*vc.get(), 20);
    }

    #[test]
    fn cas_success() {
        let mut vc = VersionedCell::new(0, 10);
        let v0 = vc.version();
        vc.set(10);
        let v = vc.cas(1, 20).unwrap();
        assert_eq!(v, 2);
        assert_eq!(*vc.get(), 20);
    }

    #[test]
    fn cas_version_mismatch() {
        let mut vc = VersionedCell::new(0, 10);
        vc.set(10);
        let err = vc.cas(0, 20).unwrap_err();
        assert!(matches!(err, VerError::VersionMismatch { expected: 0, found: 1 }));
    }

    #[test]
    fn rollback_one() {
        let mut vc = VersionedCell::new(0, 10);
        vc.set(10);
        vc.set(20);
        vc.rollback().unwrap();
        assert_eq!(*vc.get(), 10);
        assert_eq!(vc.version(), 1);
    }

    #[test]
    fn rollback_to_version() {
        let mut vc = VersionedCell::new(0, 10);
        vc.set(10);
        vc.set(20);
        vc.set(30);
        vc.rollback_to(1).unwrap();
        assert_eq!(*vc.get(), 10);
        assert_eq!(vc.version(), 1);
    }

    #[test]
    fn rollback_empty() {
        let mut vc: VersionedCell<i32> = VersionedCell::new(0, 10);
        let err = vc.rollback().unwrap_err();
        assert!(matches!(err, VerError::NoHistory));
    }

    #[test]
    fn history_depth() {
        let mut vc = VersionedCell::new(0, 10);
        assert_eq!(vc.history_depth(), 0);
        vc.set(1);
        vc.set(2);
        assert_eq!(vc.history_depth(), 2);
    }

    #[test]
    fn history_eviction() {
        let mut vc = VersionedCell::new(0, 3);
        vc.set(1);
        vc.set(2);
        vc.set(3);
        vc.set(4);
        assert_eq!(vc.history_depth(), 3);
        vc.rollback().unwrap();
        assert_eq!(*vc.get(), 3);
    }

    #[test]
    fn stats() {
        let mut vc = VersionedCell::new(0, 10);
        vc.set(1);
        vc.set(2);
        vc.rollback().unwrap();
        assert_eq!(vc.total_writes(), 2);
        assert_eq!(vc.total_rollbacks(), 1);
    }

    #[test]
    fn reset() {
        let mut vc = VersionedCell::new(0, 10);
        vc.set(1);
        vc.reset(99);
        assert_eq!(*vc.get(), 99);
        assert_eq!(vc.version(), 0);
        assert!(vc.is_clean());
    }

    #[test]
    fn error_display() {
        assert!(VerError::NoHistory.to_string().contains("history"));
    }
}
