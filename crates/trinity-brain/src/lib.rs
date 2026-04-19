//! trios-brain — Memory module for TRIOS
//!
//! R0: Pure Rust stub implementation (HashMap-based)
//! R1: FFI envelope (planned)
//! R2: Persistence / HSLM hook (planned)

use std::collections::HashMap;
use std::sync::{Mutex, LazyLock};

/// Brain error types
#[derive(Debug, Clone, PartialEq)]
pub enum BrainError {
    KeyNotFound,
    InvalidKey,
    StorageError,
    SerializationError,
}

impl std::fmt::Display for BrainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrainError::KeyNotFound => write!(f, "Key not found in brain"),
            BrainError::InvalidKey => write!(f, "Invalid key format"),
            BrainError::StorageError => write!(f, "Storage operation failed"),
            BrainError::SerializationError => write!(f, "Serialization failed"),
        }
    }
}

impl std::error::Error for BrainError {}

/// Thread-local brain storage (R0 stub)
struct BrainStorage {
    data: HashMap<String, Vec<u8>>,
}

impl BrainStorage {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn remember(&mut self, key: &str, value: &[u8]) {
        self.data.insert(key.to_string(), value.to_vec());
    }

    fn recall(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn forget(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    fn count(&self) -> usize {
        self.data.len()
    }
}

/// Global brain instance (thread-safe for R0)
static BRAIN: LazyLock<Mutex<BrainStorage>> = LazyLock::new(|| {
    Mutex::new(BrainStorage::new())
});

/// Store a value in brain memory
///
/// # Arguments
/// * `key` - Memory key (UTF-8 string, non-empty)
/// * `value` - Binary value to store
///
/// # Returns
/// * `Ok(())` - Value stored successfully
/// * `Err(BrainError::InvalidKey)` - Key is empty
///
/// # Example
/// ```no_run
/// use trios_brain::brain_remember;
///
/// brain_remember("test_key", b"hello world").unwrap();
/// ```
pub fn brain_remember(key: &str, value: &[u8]) -> Result<(), BrainError> {
    if key.is_empty() {
        return Err(BrainError::InvalidKey);
    }

    let mut brain = BRAIN.lock()
        .map_err(|_| BrainError::StorageError)?;

    brain.remember(key, value);
    Ok(())
}

/// Recall a value from brain memory
///
/// # Arguments
/// * `key` - Memory key to retrieve
///
/// # Returns
/// * `Ok(Vec<u8>)` - Found value
/// * `Err(BrainError::KeyNotFound)` - Key doesn't exist
/// * `Err(BrainError::InvalidKey)` - Key is empty
///
/// # Example
/// ```no_run
/// use trios_brain::{brain_remember, brain_recall};
///
/// brain_remember("test_key", b"hello world").unwrap();
/// let value = brain_recall("test_key").unwrap();
/// assert_eq!(value, b"hello world");
/// ```
pub fn brain_recall(key: &str) -> Result<Vec<u8>, BrainError> {
    if key.is_empty() {
        return Err(BrainError::InvalidKey);
    }

    let brain = BRAIN.lock()
        .map_err(|_| BrainError::StorageError)?;

    brain.recall(key)
        .ok_or(BrainError::KeyNotFound)
}

/// Forget a value from brain memory
///
/// # Arguments
/// * `key` - Memory key to remove
///
/// # Returns
/// * `Ok(true)` - Key was found and removed
/// * `Ok(false)` - Key didn't exist
/// * `Err(BrainError::InvalidKey)` - Key is empty
pub fn brain_forget(key: &str) -> Result<bool, BrainError> {
    if key.is_empty() {
        return Err(BrainError::InvalidKey);
    }

    let mut brain = BRAIN.lock()
        .map_err(|_| BrainError::StorageError)?;

    Ok(brain.forget(key))
}

/// Get total number of stored memories
pub fn brain_count() -> usize {
    BRAIN.lock()
        .map(|brain| brain.count())
        .unwrap_or(0)
}

/// Clear all memories (for testing only)
#[cfg(test)]
pub fn brain_clear() {
    let mut brain = BRAIN.lock().unwrap();
    *brain = BrainStorage::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Setup: clear brain before each test
    fn setup() {
        brain_clear();
    }

    #[test]
    fn test_remember_and_recall() {
        setup();

        let key = "test_key";
        let value = b"hello world";

        brain_remember(key, value).unwrap();
        let recalled = brain_recall(key).unwrap();

        assert_eq!(recalled, value);
    }

    #[test]
    fn test_overwrite() {
        setup();

        let key = "overwrite_key";

        brain_remember(key, b"original").unwrap();
        brain_remember(key, b"updated").unwrap();

        let recalled = brain_recall(key).unwrap();
        assert_eq!(recalled, b"updated");
    }

    #[test]
    fn test_missing_key() {
        setup();

        let result = brain_recall("nonexistent_key");
        assert_eq!(result, Err(BrainError::KeyNotFound));
    }

    #[test]
    fn test_invalid_key() {
        setup();

        let empty_result = brain_remember("", b"value");
        assert_eq!(empty_result, Err(BrainError::InvalidKey));

        let recall_empty = brain_recall("");
        assert_eq!(recall_empty, Err(BrainError::InvalidKey));
    }

    #[test]
    fn test_forget() {
        setup();

        brain_remember("temp_key", b"value").unwrap();

        let removed = brain_forget("temp_key").unwrap();
        assert!(removed);

        let recall_result = brain_recall("temp_key");
        assert_eq!(recall_result, Err(BrainError::KeyNotFound));
    }

    #[test]
    fn test_forget_nonexistent() {
        setup();

        let removed = brain_forget("nonexistent").unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_count() {
        setup();

        assert_eq!(brain_count(), 0);

        brain_remember("key1", b"value1").unwrap();
        brain_remember("key2", b"value2").unwrap();

        assert_eq!(brain_count(), 2);

        brain_forget("key1").unwrap();

        assert_eq!(brain_count(), 1);
    }

    #[test]
    fn test_binary_data() {
        setup();

        let binary_data: Vec<u8> = (0..256).map(|i| i as u8).collect();

        brain_remember("binary", &binary_data).unwrap();
        let recalled = brain_recall("binary").unwrap();

        assert_eq!(recalled, binary_data);
        assert_eq!(recalled.len(), 256);
    }
}
