use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TierId(u32);

impl TierId {
    pub fn new(id: u32) -> Self { Self(id) }
    pub fn raw(&self) -> u32 { self.0 }
}

#[derive(Debug, Clone)]
pub struct CounterId {
    pub tier: TierId,
    pub name: String,
}

impl CounterId {
    pub fn new(tier: TierId, name: &str) -> Self {
        Self { tier, name: name.to_string() }
    }
}

#[derive(Debug, Clone)]
struct TierCounter {
    value: u64,
    high_water: u64,
    total_increments: u64,
}

impl TierCounter {
    fn new() -> Self {
        Self { value: 0, high_water: 0, total_increments: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct TieredCounter {
    tiers: BTreeMap<u32, BTreeMap<String, TierCounter>>,
    total_snapshots: u64,
}

impl TieredCounter {
    pub fn new() -> Self {
        Self { tiers: BTreeMap::new(), total_snapshots: 0 }
    }

    fn ensure(&mut self, tier: TierId, name: &str) {
        self.tiers.entry(tier.0).or_default()
            .entry(name.to_string()).or_insert_with(TierCounter::new);
    }

    pub fn inc(&mut self, tier: TierId, name: &str, delta: u64) -> u64 {
        self.ensure(tier, name);
        let counter = self.tiers.get_mut(&tier.0).unwrap().get_mut(name).unwrap();
        counter.value += delta;
        counter.total_increments += 1;
        if counter.value > counter.high_water {
            counter.high_water = counter.value;
        }
        counter.value
    }

    pub fn dec(&mut self, tier: TierId, name: &str, delta: u64) -> u64 {
        self.ensure(tier, name);
        let counter = self.tiers.get_mut(&tier.0).unwrap().get_mut(name).unwrap();
        counter.value = counter.value.saturating_sub(delta);
        counter.value
    }

    pub fn set(&mut self, tier: TierId, name: &str, value: u64) {
        self.ensure(tier, name);
        let counter = self.tiers.get_mut(&tier.0).unwrap().get_mut(name).unwrap();
        counter.value = value;
        if value > counter.high_water {
            counter.high_water = value;
        }
    }

    pub fn get(&self, tier: TierId, name: &str) -> u64 {
        self.tiers.get(&tier.0)
            .and_then(|m| m.get(name))
            .map(|c| c.value)
            .unwrap_or(0)
    }

    pub fn high_water(&self, tier: TierId, name: &str) -> u64 {
        self.tiers.get(&tier.0)
            .and_then(|m| m.get(name))
            .map(|c| c.high_water)
            .unwrap_or(0)
    }

    pub fn total_increments(&self, tier: TierId, name: &str) -> u64 {
        self.tiers.get(&tier.0)
            .and_then(|m| m.get(name))
            .map(|c| c.total_increments)
            .unwrap_or(0)
    }

    pub fn tier_names(&self, tier: TierId) -> Vec<&str> {
        self.tiers.get(&tier.0)
            .map(|m| m.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn tier_total(&self, tier: TierId) -> u64 {
        self.tiers.get(&tier.0)
            .map(|m| m.values().map(|c| c.value).sum())
            .unwrap_or(0)
    }

    pub fn rollup(&self, dest: TierId, src: TierId) -> u64 {
        let src_total = self.tier_total(src);
        src_total
    }

    pub fn snapshot(&mut self) -> BTreeMap<u32, BTreeMap<String, u64>> {
        self.total_snapshots += 1;
        self.tiers.iter().map(|(&tier, counters)| {
            let map = counters.iter().map(|(name, c)| (name.clone(), c.value)).collect();
            (tier, map)
        }).collect()
    }

    pub fn reset_tier(&mut self, tier: TierId) {
        if let Some(counters) = self.tiers.get_mut(&tier.0) {
            for counter in counters.values_mut() {
                counter.value = 0;
                counter.high_water = 0;
                counter.total_increments = 0;
            }
        }
    }

    pub fn reset_all(&mut self) {
        for counters in self.tiers.values_mut() {
            for counter in counters.values_mut() {
                counter.value = 0;
                counter.high_water = 0;
                counter.total_increments = 0;
            }
        }
    }

    pub fn tier_count(&self) -> usize {
        self.tiers.len()
    }

    pub fn total_snapshots(&self) -> u64 {
        self.total_snapshots
    }
}

impl Default for TieredCounter {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> TierId { TierId::new(0) }
    fn t1() -> TierId { TierId::new(1) }

    #[test]
    fn new_counter() {
        let tc = TieredCounter::new();
        assert_eq!(tc.tier_count(), 0);
    }

    #[test]
    fn inc_and_get() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "ops", 10);
        tc.inc(t0(), "ops", 5);
        assert_eq!(tc.get(t0(), "ops"), 15);
    }

    #[test]
    fn dec_saturating() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "val", 5);
        tc.dec(t0(), "val", 10);
        assert_eq!(tc.get(t0(), "val"), 0);
    }

    #[test]
    fn set_value() {
        let mut tc = TieredCounter::new();
        tc.set(t0(), "x", 42);
        assert_eq!(tc.get(t0(), "x"), 42);
    }

    #[test]
    fn high_water() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "mem", 100);
        tc.inc(t0(), "mem", 50);
        tc.dec(t0(), "mem", 75);
        assert_eq!(tc.get(t0(), "mem"), 75);
        assert_eq!(tc.high_water(t0(), "mem"), 150);
    }

    #[test]
    fn separate_tiers() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "ops", 10);
        tc.inc(t1(), "ops", 20);
        assert_eq!(tc.get(t0(), "ops"), 10);
        assert_eq!(tc.get(t1(), "ops"), 20);
    }

    #[test]
    fn tier_total() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "a", 10);
        tc.inc(t0(), "b", 20);
        assert_eq!(tc.tier_total(t0()), 30);
    }

    #[test]
    fn tier_names() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "beta", 1);
        tc.inc(t0(), "alpha", 1);
        assert_eq!(tc.tier_names(t0()), vec!["alpha", "beta"]);
    }

    #[test]
    fn snapshot() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "x", 5);
        tc.inc(t1(), "y", 10);
        let snap = tc.snapshot();
        assert_eq!(snap[&0]["x"], 5);
        assert_eq!(snap[&1]["y"], 10);
        assert_eq!(tc.total_snapshots(), 1);
    }

    #[test]
    fn reset_tier() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "x", 100);
        tc.inc(t1(), "y", 200);
        tc.reset_tier(t0());
        assert_eq!(tc.get(t0(), "x"), 0);
        assert_eq!(tc.get(t1(), "y"), 200);
    }

    #[test]
    fn reset_all() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "x", 100);
        tc.inc(t1(), "y", 200);
        tc.reset_all();
        assert_eq!(tc.get(t0(), "x"), 0);
        assert_eq!(tc.get(t1(), "y"), 0);
    }

    #[test]
    fn total_increments() {
        let mut tc = TieredCounter::new();
        tc.inc(t0(), "ops", 1);
        tc.inc(t0(), "ops", 1);
        tc.inc(t0(), "ops", 1);
        assert_eq!(tc.total_increments(t0(), "ops"), 3);
    }

    #[test]
    fn get_missing() {
        let tc = TieredCounter::new();
        assert_eq!(tc.get(TierId::new(99), "nope"), 0);
    }
}
