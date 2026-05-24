use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CdError {
    KeyNotFound { key: u64 },
    NotOnCooldown { key: u64 },
}

impl std::fmt::Display for CdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CdError::KeyNotFound { key } => write!(f, "key {key} not found"),
            CdError::NotOnCooldown { key } => write!(f, "key {key} not on cooldown"),
        }
    }
}

impl std::error::Error for CdError {}

struct Entry {
    key: u64,
    duration: u64,
    started_at: Option<u64>,
    total_activations: u64,
    total_expiries: u64,
}

pub struct Cooldown {
    entries: BTreeMap<u64, Entry>,
    total_activations: u64,
    total_expiries: u64,
    total_rejections: u64,
}

impl Cooldown {
    pub fn new() -> Self { Self { entries: BTreeMap::new(), total_activations: 0, total_expiries: 0, total_rejections: 0 } }

    pub fn register(&mut self, key: u64, duration: u64) {
        self.entries.insert(key, Entry { key, duration, started_at: None, total_activations: 0, total_expiries: 0 });
    }

    pub fn trigger(&mut self, key: u64, now: u64) -> bool {
        if let Some(e) = self.entries.get_mut(&key) {
            if let Some(started) = e.started_at {
                if now < started + e.duration {
                    self.total_rejections += 1;
                    return false;
                }
            }
            e.started_at = Some(now);
            e.total_activations += 1;
            self.total_activations += 1;
            true
        } else { false }
    }

    pub fn is_on_cooldown(&self, key: u64, now: u64) -> Option<bool> {
        let e = self.entries.get(&key)?;
        match e.started_at {
            Some(started) => Some(now < started + e.duration),
            None => Some(false),
        }
    }

    pub fn remaining(&self, key: u64, now: u64) -> Option<u64> {
        let e = self.entries.get(&key)?;
        match e.started_at {
            Some(started) => {
                let end = started + e.duration;
                if now < end { Some(end - now) } else { Some(0) }
            }
            None => Some(0),
        }
    }

    pub fn expire(&mut self, key: u64) -> Result<(), CdError> {
        let e = self.entries.get_mut(&key).ok_or(CdError::KeyNotFound { key })?;
        if e.started_at.is_none() { return Err(CdError::NotOnCooldown { key }); }
        e.started_at = None;
        e.total_expiries += 1;
        self.total_expiries += 1;
        Ok(())
    }

    pub fn tick(&mut self, now: u64) -> Vec<u64> {
        let expired: Vec<u64> = self.entries.iter()
            .filter(|(_, e)| {
                if let Some(started) = e.started_at { now >= started + e.duration } else { false }
            })
            .map(|(&k, _)| k)
            .collect();
        for &k in &expired {
            let e = self.entries.get_mut(&k).unwrap();
            e.started_at = None;
            e.total_expiries += 1;
            self.total_expiries += 1;
        }
        expired
    }

    pub fn reset(&mut self, key: u64) -> Result<(), CdError> {
        let e = self.entries.get_mut(&key).ok_or(CdError::KeyNotFound { key })?;
        e.started_at = None;
        Ok(())
    }

    pub fn entry_count(&self) -> usize { self.entries.len() }
    pub fn active_count(&self, now: u64) -> usize {
        self.entries.values().filter(|e| e.started_at.map_or(false, |s| now < s + e.duration)).count()
    }
    pub fn total_activations(&self) -> u64 { self.total_activations }
    pub fn total_expiries(&self) -> u64 { self.total_expiries }
    pub fn total_rejections(&self) -> u64 { self.total_rejections }
}

impl Default for Cooldown {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cd() { assert_eq!(Cooldown::new().entry_count(), 0); }

    #[test]
    fn trigger_allows() {
        let mut cd = Cooldown::new();
        cd.register(1, 100);
        assert!(cd.trigger(1, 0));
        assert_eq!(cd.remaining(1, 0), Some(100));
    }

    #[test]
    fn cooldown_blocks() {
        let mut cd = Cooldown::new();
        cd.register(1, 100);
        cd.trigger(1, 0);
        assert!(!cd.trigger(1, 50));
        assert_eq!(cd.total_rejections(), 1);
    }

    #[test]
    fn cooldown_expires() {
        let mut cd = Cooldown::new();
        cd.register(1, 100);
        cd.trigger(1, 0);
        assert!(cd.trigger(1, 100));
    }

    #[test]
    fn tick_expiry() {
        let mut cd = Cooldown::new();
        cd.register(1, 10); cd.register(2, 20);
        cd.trigger(1, 0); cd.trigger(2, 0);
        let expired = cd.tick(15);
        assert_eq!(expired.len(), 1);
        assert!(expired.contains(&1));
    }

    #[test]
    fn is_on_cooldown() {
        let mut cd = Cooldown::new();
        cd.register(1, 100);
        cd.trigger(1, 0);
        assert_eq!(cd.is_on_cooldown(1, 50), Some(true));
        assert_eq!(cd.is_on_cooldown(1, 200), Some(false));
    }

    #[test]
    fn manual_expire() {
        let mut cd = Cooldown::new();
        cd.register(1, 100);
        cd.trigger(1, 0);
        cd.expire(1).unwrap();
        assert!(cd.trigger(1, 0));
    }

    #[test]
    fn reset() {
        let mut cd = Cooldown::new();
        cd.register(1, 100);
        cd.trigger(1, 0);
        cd.reset(1).unwrap();
        assert!(cd.trigger(1, 0));
    }

    #[test]
    fn active_count() {
        let mut cd = Cooldown::new();
        cd.register(1, 10); cd.register(2, 100);
        cd.trigger(1, 0); cd.trigger(2, 0);
        assert_eq!(cd.active_count(5), 2);
        assert_eq!(cd.active_count(50), 1);
    }

    #[test]
    fn stats() {
        let mut cd = Cooldown::new();
        cd.register(1, 100);
        cd.trigger(1, 0);
        cd.trigger(1, 50);
        assert_eq!(cd.total_activations(), 1);
        assert_eq!(cd.total_rejections(), 1);
    }

    #[test]
    fn error_display() { assert!(CdError::KeyNotFound { key: 1 }.to_string().contains("1")); }
}
