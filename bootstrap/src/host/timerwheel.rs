use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TwError {
    NotFound { id: u64 },
}

impl std::fmt::Display for TwError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwError::NotFound { id } => write!(f, "timer {id} not found"),
        }
    }
}

impl std::error::Error for TwError {}

struct Timer {
    id: u64,
    deadline: u64,
    callback: String,
    cancelled: bool,
}

pub struct TimerWheel {
    slots: Vec<Vec<usize>>,
    current: u64,
    wheel_size: usize,
    levels: usize,
    timers: Vec<Timer>,
    free: Vec<usize>,
    registry: BTreeMap<u64, usize>,
    total_schedules: u64,
    total_cancels: u64,
    total_fires: u64,
    total_ticks: u64,
}

impl TimerWheel {
    pub fn new(wheel_size: usize, levels: usize) -> Self {
        let total_slots = wheel_size * levels;
        Self { slots: vec![Vec::new(); total_slots], current: 0, wheel_size, levels, timers: Vec::new(), free: Vec::new(), registry: BTreeMap::new(), total_schedules: 0, total_cancels: 0, total_fires: 0, total_ticks: 0 }
    }

    fn slot_for(&self, deadline: u64) -> usize {
        let diff = deadline.saturating_sub(self.current);
        let level = if diff < self.wheel_size as u64 { 0 }
            else if diff < (self.wheel_size as u64) * (self.wheel_size as u64) { 1 }
            else if diff < (self.wheel_size as u64).pow(3) { 2 }
            else { 3 };
        let level = level.min(self.levels - 1);
        let offset = if level == 0 { (deadline % self.wheel_size as u64) as usize }
            else { ((deadline >> (level * 8)) % self.wheel_size as u64) as usize + level * self.wheel_size };
        offset.min(self.slots.len() - 1)
    }

    pub fn schedule(&mut self, id: u64, deadline: u64, callback: String) -> Result<(), TwError> {
        if self.registry.contains_key(&id) { return Err(TwError::NotFound { id }); }
        self.total_schedules += 1;
        let idx = if let Some(f) = self.free.pop() {
            self.timers[f] = Timer { id, deadline, callback, cancelled: false };
            f
        } else {
            let i = self.timers.len();
            self.timers.push(Timer { id, deadline, callback, cancelled: false });
            i
        };
        self.registry.insert(id, idx);
        let slot = self.slot_for(deadline);
        self.slots[slot].push(idx);
        Ok(())
    }

    pub fn cancel(&mut self, id: u64) -> Result<String, TwError> {
        let &idx = self.registry.get(&id).ok_or(TwError::NotFound { id })?;
        self.timers[idx].cancelled = true;
        let cb = self.timers[idx].callback.clone();
        self.registry.remove(&id);
        self.free.push(idx);
        self.total_cancels += 1;
        Ok(cb)
    }

    pub fn tick(&mut self) -> Vec<(u64, String)> {
        self.total_ticks += 1;
        let mut fired = Vec::new();
        let slot = (self.current % self.wheel_size as u64) as usize;
        let indices: Vec<usize> = self.slots[slot].drain(..).collect();
        for idx in indices {
            if self.timers[idx].cancelled { continue; }
            if self.timers[idx].deadline <= self.current {
                let t = &self.timers[idx];
                fired.push((t.id, t.callback.clone()));
                self.registry.remove(&t.id);
                self.free.push(idx);
                self.total_fires += 1;
            } else {
                let new_slot = self.slot_for(self.timers[idx].deadline);
                self.slots[new_slot].push(idx);
            }
        }
        self.current += 1;
        fired
    }

    pub fn advance(&mut self, ticks: u64) -> Vec<(u64, String)> {
        let mut all = Vec::new();
        for _ in 0..ticks { all.extend(self.tick()); }
        all
    }

    pub fn now(&self) -> u64 { self.current }
    pub fn pending(&self) -> usize { self.registry.len() }
    pub fn total_schedules(&self) -> u64 { self.total_schedules }
    pub fn total_cancels(&self) -> u64 { self.total_cancels }
    pub fn total_fires(&self) -> u64 { self.total_fires }
    pub fn total_ticks(&self) -> u64 { self.total_ticks }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tw() { let tw = TimerWheel::new(64, 4); assert_eq!(tw.pending(), 0); }

    #[test]
    fn schedule_fire() {
        let mut tw = TimerWheel::new(64, 4);
        tw.schedule(1, 0, "cb1".to_string()).unwrap();
        let fired = tw.tick();
        assert_eq!(fired.len(), 1);
        assert_eq!(fired[0].0, 1);
    }

    #[test]
    fn cancel() {
        let mut tw = TimerWheel::new(64, 4);
        tw.schedule(1, 10, "cb".to_string()).unwrap();
        tw.cancel(1).unwrap();
        let fired = tw.advance(15);
        assert!(fired.is_empty());
    }

    #[test]
    fn cancel_not_found() { assert!(TimerWheel::new(64, 4).cancel(1).is_err()); }

    #[test]
    fn multiple() {
        let mut tw = TimerWheel::new(64, 4);
        tw.schedule(1, 3, "a".to_string()).unwrap();
        tw.schedule(2, 5, "b".to_string()).unwrap();
        tw.schedule(3, 3, "c".to_string()).unwrap();
        let mut all = Vec::new();
        all.extend(tw.advance(3));
        all.extend(tw.advance(3));
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn advance() {
        let mut tw = TimerWheel::new(64, 4);
        tw.schedule(1, 3, "far".to_string()).unwrap();
        let fired = tw.advance(5);
        assert_eq!(fired.len(), 1);
    }

    #[test]
    fn stats() {
        let mut tw = TimerWheel::new(64, 4);
        tw.schedule(1, 1, "x".to_string()).unwrap();
        tw.advance(2);
        assert_eq!(tw.total_schedules(), 1);
        assert_eq!(tw.total_fires(), 1);
        assert_eq!(tw.total_ticks(), 2);
    }

    #[test]
    fn error_display() { assert!(TwError::NotFound { id: 1 }.to_string().contains("not found")); }
}
