//! Minimal runtime test
use std::collections::HashMap;

#[derive(Debug)]
pub struct TestRuntime {
    cache: HashMap<String, f64>,
}

impl TestRuntime {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn test(&self) -> f64 {
        1.0
    }
}
