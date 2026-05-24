use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlError {
    SlotNotFound { slot: usize },
    ContextNotFound { ctx: u64 },
    SlotExists { slot: usize },
}

impl std::fmt::Display for TlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlError::SlotNotFound { slot } => write!(f, "slot {slot} not found"),
            TlError::ContextNotFound { ctx } => write!(f, "context {ctx} not found"),
            TlError::SlotExists { slot } => write!(f, "slot {slot} exists"),
        }
    }
}

impl std::error::Error for TlError {}

#[derive(Debug, Clone)]
struct SlotDef {
    id: usize,
    name: String,
    default: u64,
}

#[derive(Debug, Clone)]
pub struct ThreadLocal {
    slots: Vec<SlotDef>,
    contexts: BTreeMap<u64, Vec<u64>>,
    active_ctx: u64,
    total_gets: u64,
    total_sets: u64,
}

impl ThreadLocal {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            contexts: BTreeMap::new(),
            active_ctx: 0,
            total_gets: 0,
            total_sets: 0,
        }
    }

    pub fn register_slot(&mut self, name: &str, default: u64) -> Result<usize, TlError> {
        if self.slots.iter().any(|s| s.name == name) {
            return Err(TlError::SlotExists { slot: self.slots.len() });
        }
        let id = self.slots.len();
        self.slots.push(SlotDef { id, name: name.to_string(), default });
        for values in self.contexts.values_mut() {
            values.push(default);
        }
        Ok(id)
    }

    pub fn register_context(&mut self, ctx: u64) {
        let defaults: Vec<u64> = self.slots.iter().map(|s| s.default).collect();
        self.contexts.insert(ctx, defaults);
    }

    pub fn set_active(&mut self, ctx: u64) -> Result<(), TlError> {
        if !self.contexts.contains_key(&ctx) {
            return Err(TlError::ContextNotFound { ctx });
        }
        self.active_ctx = ctx;
        Ok(())
    }

    pub fn active_context(&self) -> u64 {
        self.active_ctx
    }

    pub fn get(&mut self, slot: usize) -> Result<u64, TlError> {
        if slot >= self.slots.len() {
            return Err(TlError::SlotNotFound { slot });
        }
        self.total_gets += 1;
        let ctx = self.contexts.get(&self.active_ctx)
            .ok_or(TlError::ContextNotFound { ctx: self.active_ctx })?;
        Ok(ctx[slot])
    }

    pub fn set(&mut self, slot: usize, value: u64) -> Result<(), TlError> {
        if slot >= self.slots.len() {
            return Err(TlError::SlotNotFound { slot });
        }
        let ctx = self.contexts.get_mut(&self.active_ctx)
            .ok_or(TlError::ContextNotFound { ctx: self.active_ctx })?;
        ctx[slot] = value;
        self.total_sets += 1;
        Ok(())
    }

    pub fn get_ctx(&self, ctx: u64, slot: usize) -> Option<u64> {
        self.contexts.get(&ctx).and_then(|v| v.get(slot).copied())
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn context_count(&self) -> usize {
        self.contexts.len()
    }

    pub fn slot_name(&self, slot: usize) -> Option<&str> {
        self.slots.get(slot).map(|s| s.name.as_str())
    }

    pub fn total_gets(&self) -> u64 { self.total_gets }
    pub fn total_sets(&self) -> u64 { self.total_sets }

    pub fn reset_context(&mut self, ctx: u64) -> Result<(), TlError> {
        let values = self.contexts.get_mut(&ctx)
            .ok_or(TlError::ContextNotFound { ctx })?;
        for (i, slot) in self.slots.iter().enumerate() {
            values[i] = slot.default;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.contexts.clear();
        self.active_ctx = 0;
    }
}

impl Default for ThreadLocal {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tls() {
        let tls = ThreadLocal::new();
        assert_eq!(tls.slot_count(), 0);
        assert_eq!(tls.context_count(), 0);
    }

    #[test]
    fn register_slot_and_context() {
        let mut tls = ThreadLocal::new();
        tls.register_slot("freq", 66).unwrap();
        tls.register_context(1);
        tls.register_context(2);
        assert_eq!(tls.slot_count(), 1);
        assert_eq!(tls.context_count(), 2);
    }

    #[test]
    fn set_active_and_get_set() {
        let mut tls = ThreadLocal::new();
        let slot = tls.register_slot("val", 0).unwrap();
        tls.register_context(1);
        tls.set_active(1).unwrap();
        tls.set(slot, 42).unwrap();
        assert_eq!(tls.get(slot).unwrap(), 42);
    }

    #[test]
    fn per_context_isolation() {
        let mut tls = ThreadLocal::new();
        let slot = tls.register_slot("x", 0).unwrap();
        tls.register_context(1);
        tls.register_context(2);
        tls.set_active(1).unwrap();
        tls.set(slot, 100).unwrap();
        tls.set_active(2).unwrap();
        tls.set(slot, 200).unwrap();
        assert_eq!(tls.get_ctx(1, slot), Some(100));
        assert_eq!(tls.get_ctx(2, slot), Some(200));
    }

    #[test]
    fn slot_not_found() {
        let mut tls = ThreadLocal::new();
        tls.register_context(1);
        tls.set_active(1).unwrap();
        let err = tls.get(99).unwrap_err();
        assert!(matches!(err, TlError::SlotNotFound { .. }));
    }

    #[test]
    fn context_not_found() {
        let mut tls = ThreadLocal::new();
        let err = tls.set_active(99).unwrap_err();
        assert!(matches!(err, TlError::ContextNotFound { .. }));
    }

    #[test]
    fn slot_name() {
        let mut tls = ThreadLocal::new();
        tls.register_slot("freq", 66).unwrap();
        assert_eq!(tls.slot_name(0), Some("freq"));
    }

    #[test]
    fn default_values() {
        let mut tls = ThreadLocal::new();
        let slot = tls.register_slot("x", 42).unwrap();
        tls.register_context(1);
        tls.set_active(1).unwrap();
        assert_eq!(tls.get(slot).unwrap(), 42);
    }

    #[test]
    fn reset_context() {
        let mut tls = ThreadLocal::new();
        let slot = tls.register_slot("x", 10).unwrap();
        tls.register_context(1);
        tls.set_active(1).unwrap();
        tls.set(slot, 99).unwrap();
        tls.reset_context(1).unwrap();
        assert_eq!(tls.get(slot).unwrap(), 10);
    }

    #[test]
    fn stats() {
        let mut tls = ThreadLocal::new();
        let slot = tls.register_slot("x", 0).unwrap();
        tls.register_context(1);
        tls.set_active(1).unwrap();
        tls.set(slot, 1).unwrap();
        tls.get(slot).unwrap();
        assert_eq!(tls.total_sets(), 1);
        assert_eq!(tls.total_gets(), 1);
    }

    #[test]
    fn clear() {
        let mut tls = ThreadLocal::new();
        tls.register_slot("x", 0).unwrap();
        tls.register_context(1);
        tls.clear();
        assert_eq!(tls.slot_count(), 0);
        assert_eq!(tls.context_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(TlError::SlotNotFound { slot: 5 }.to_string().contains("5"));
    }
}
