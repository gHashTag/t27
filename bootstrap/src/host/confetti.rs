use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Cf2Error {
    EmptyTable,
    InvalidWeight { id: u64 },
    ItemNotFound { id: u64 },
}

impl std::fmt::Display for Cf2Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cf2Error::EmptyTable => write!(f, "empty table"),
            Cf2Error::InvalidWeight { id } => write!(f, "invalid weight for {id}"),
            Cf2Error::ItemNotFound { id } => write!(f, "item {id} not found"),
        }
    }
}

impl std::error::Error for Cf2Error {}

#[derive(Clone)]
struct AliasEntry {
    id: u64,
    prob: f64,
    alias: u64,
}

pub struct Confetti {
    ids: Vec<u64>,
    weights: BTreeMap<u64, f64>,
    alias_table: Vec<AliasEntry>,
    built: bool,
    total_samples: u64,
    sample_counts: BTreeMap<u64, u64>,
}

impl Confetti {
    pub fn new() -> Self { Self { ids: Vec::new(), weights: BTreeMap::new(), alias_table: Vec::new(), built: false, total_samples: 0, sample_counts: BTreeMap::new() } }

    pub fn add(&mut self, id: u64, weight: f64) -> Result<(), Cf2Error> {
        if weight <= 0.0 { return Err(Cf2Error::InvalidWeight { id }); }
        if !self.weights.contains_key(&id) { self.ids.push(id); }
        self.weights.insert(id, weight);
        self.built = false;
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<(), Cf2Error> {
        if !self.weights.contains_key(&id) { return Err(Cf2Error::ItemNotFound { id }); }
        self.weights.remove(&id);
        self.ids.retain(|&x| x != id);
        self.built = false;
        Ok(())
    }

    pub fn build(&mut self) -> Result<(), Cf2Error> {
        if self.ids.is_empty() { return Err(Cf2Error::EmptyTable); }
        let n = self.ids.len();
        let total: f64 = self.weights.values().sum();
        let avg = total / n as f64;
        let mut scaled: Vec<(usize, f64)> = self.ids.iter().enumerate()
            .map(|(i, &id)| (i, self.weights[&id] / avg))
            .collect();
        let mut small: Vec<usize> = Vec::new();
        let mut large: Vec<usize> = Vec::new();
        for &(i, s) in &scaled {
            if s < 1.0 { small.push(i); } else { large.push(i); }
        }
        self.alias_table = vec![AliasEntry { id: 0, prob: 0.0, alias: 0 }; n];
        for i in 0..n { self.alias_table[i].id = self.ids[i]; }
        while !small.is_empty() && !large.is_empty() {
            let s = small.pop().unwrap();
            let l = large.pop().unwrap();
            self.alias_table[s].prob = scaled[s].1;
            self.alias_table[s].alias = self.ids[l];
            scaled[l].1 -= 1.0 - scaled[s].1;
            if scaled[l].1 < 1.0 { small.push(l); } else { large.push(l); }
        }
        for &i in &large { self.alias_table[i].prob = 1.0; }
        for &i in &small { self.alias_table[i].prob = 1.0; }
        self.built = true;
        Ok(())
    }

    pub fn sample(&mut self, rng: u64) -> Result<u64, Cf2Error> {
        if !self.built { self.build()?; }
        if self.ids.is_empty() { return Err(Cf2Error::EmptyTable); }
        let n = self.alias_table.len();
        let col = (rng % n as u64) as usize;
        let coin = (rng.wrapping_mul(0x2545F4914F6CDD1D) as f64) / u64::MAX as f64;
        let entry = &self.alias_table[col];
        let result = if coin < entry.prob { entry.id } else { entry.alias };
        self.total_samples += 1;
        *self.sample_counts.entry(result).or_insert(0) += 1;
        Ok(result)
    }

    pub fn sample_counts(&self) -> &BTreeMap<u64, u64> { &self.sample_counts }
    pub fn item_count(&self) -> usize { self.ids.len() }
    pub fn total_samples(&self) -> u64 { self.total_samples }
    pub fn is_built(&self) -> bool { self.built }
}

impl Default for Confetti {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sampler() { assert!(Confetti::new().item_count() == 0); }

    #[test]
    fn add_build_sample() {
        let mut c = Confetti::new();
        c.add(1, 1.0).unwrap(); c.add(2, 1.0).unwrap();
        c.build().unwrap();
        let s = c.sample(42).unwrap();
        assert!(s == 1 || s == 2);
    }

    #[test]
    fn auto_build() {
        let mut c = Confetti::new();
        c.add(1, 1.0).unwrap();
        let s = c.sample(0).unwrap();
        assert_eq!(s, 1);
    }

    #[test]
    fn weighted_distribution() {
        let mut c = Confetti::new();
        c.add(1, 1.0).unwrap(); c.add(2, 9.0).unwrap();
        c.build().unwrap();
        for i in 0..1000 { c.sample(i).unwrap(); }
        let c2_count = *c.sample_counts().get(&2).unwrap_or(&0);
        assert!(c2_count > 600);
    }

    #[test]
    fn remove() {
        let mut c = Confetti::new();
        c.add(1, 1.0).unwrap(); c.add(2, 1.0).unwrap();
        c.remove(1).unwrap();
        assert_eq!(c.item_count(), 1);
        assert!(!c.is_built());
    }

    #[test]
    fn remove_missing() {
        let mut c = Confetti::new();
        let err = c.remove(99).unwrap_err();
        assert!(matches!(err, Cf2Error::ItemNotFound { .. }));
    }

    #[test]
    fn invalid_weight() {
        let mut c = Confetti::new();
        let err = c.add(1, 0.0).unwrap_err();
        assert!(matches!(err, Cf2Error::InvalidWeight { .. }));
    }

    #[test]
    fn empty_sample() {
        let mut c = Confetti::new();
        let err = c.sample(0).unwrap_err();
        assert!(matches!(err, Cf2Error::EmptyTable));
    }

    #[test]
    fn stats() {
        let mut c = Confetti::new();
        c.add(1, 1.0).unwrap();
        c.sample(0).unwrap();
        assert_eq!(c.total_samples(), 1);
    }

    #[test]
    fn error_display() { assert!(Cf2Error::EmptyTable.to_string().contains("empty")); }
}
