//! Per-phase wall-clock timing for `--time`.

use std::time::Instant;

#[derive(Default)]
pub struct Phases {
    pub entries: Vec<(&'static str, f64)>,
}

impl Phases {
    /// Run `f`, record its wall time in milliseconds under `name`.
    pub fn time<T>(&mut self, name: &'static str, f: impl FnOnce() -> T) -> T {
        let t0 = Instant::now();
        let r = f();
        self.entries.push((name, t0.elapsed().as_secs_f64() * 1e3));
        r
    }

    pub fn total(&self) -> f64 {
        self.entries.iter().map(|e| e.1).sum()
    }

    pub fn report(&self) -> String {
        let mut s = String::from("t27b phases (ms):");
        for (n, ms) in &self.entries {
            s.push_str(&format!(" {}={:.3}", n, ms));
        }
        s.push_str(&format!(" total={:.3}", self.total()));
        s
    }
}
