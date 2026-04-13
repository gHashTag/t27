//! Minimal test of runtime
use std::collections::HashMap;

pub struct MinimalRuntime {
    cache: HashMap<String, f64>,
}

impl MinimalRuntime {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn eval(&mut self, x: f64) -> f64 {
        x
    }
}
