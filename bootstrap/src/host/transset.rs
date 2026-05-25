use std::collections::BTreeMap;

pub struct TransSet {
    adj: BTreeMap<u64, Vec<u64>>,
    closure: BTreeMap<u64, Vec<u64>>,
    dirty: bool,
    total_add: u64,
    total_query: u64,
}

impl TransSet {
    pub fn new() -> Self { Self { adj: BTreeMap::new(), closure: BTreeMap::new(), dirty: false, total_add: 0, total_query: 0 } }

    pub fn add_edge(&mut self, from: u64, to: u64) {
        self.total_add += 1;
        self.adj.entry(from).or_default().push(to);
        self.adj.entry(to).or_insert_with(Vec::new);
        self.dirty = true;
    }

    fn recompute(&mut self) {
        if !self.dirty { return; }
        let nodes: Vec<u64> = self.adj.keys().copied().collect();
        let mut reach: BTreeMap<u64, BTreeMap<u64, bool>> = BTreeMap::new();
        for &n in &nodes {
            let mut map = BTreeMap::new();
            map.insert(n, true);
            if let Some(nbrs) = self.adj.get(&n) { for &nb in nbrs { map.insert(nb, true); } }
            reach.insert(n, map);
        }
        for &k in &nodes {
            for &i in &nodes {
                for &j in &nodes {
                    let ik = *reach.get(&i).and_then(|m| m.get(&k)).unwrap_or(&false);
                    let kj = *reach.get(&k).and_then(|m| m.get(&j)).unwrap_or(&false);
                    if ik && kj { reach.get_mut(&i).unwrap().insert(j, true); }
                }
            }
        }
        self.closure.clear();
        for (n, map) in reach {
            let mut v: Vec<u64> = map.keys().copied().collect();
            v.sort();
            self.closure.insert(n, v);
        }
        self.dirty = false;
    }

    pub fn reachable(&mut self, from: u64, to: u64) -> bool {
        self.total_query += 1;
        self.recompute();
        self.closure.get(&from).map(|v| v.binary_search(&to).is_ok()).unwrap_or(false)
    }

    pub fn closure_of(&mut self, from: u64) -> Vec<u64> {
        self.recompute();
        self.closure.get(&from).cloned().unwrap_or_default()
    }

    pub fn edge_count(&self) -> usize { self.adj.values().map(|v| v.len()).sum() }
    pub fn node_count(&self) -> usize { self.adj.len() }
    pub fn total_add(&self) -> u64 { self.total_add }
    pub fn total_query(&self) -> u64 { self.total_query }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct() {
        let mut ts = TransSet::new();
        ts.add_edge(1, 2);
        assert!(ts.reachable(1, 2));
        assert!(!ts.reachable(2, 1));
    }

    #[test]
    fn transitive() {
        let mut ts = TransSet::new();
        ts.add_edge(1, 2); ts.add_edge(2, 3);
        assert!(ts.reachable(1, 3));
    }

    #[test]
    fn cycle() {
        let mut ts = TransSet::new();
        ts.add_edge(1, 2); ts.add_edge(2, 3); ts.add_edge(3, 1);
        assert!(ts.reachable(1, 1));
    }

    #[test]
    fn closure_of() {
        let mut ts = TransSet::new();
        ts.add_edge(1, 2); ts.add_edge(2, 3);
        let c = ts.closure_of(1);
        assert!(c.contains(&2));
        assert!(c.contains(&3));
    }

    #[test]
    fn no_path() {
        let mut ts = TransSet::new();
        ts.add_edge(1, 2); ts.add_edge(3, 4);
        assert!(!ts.reachable(1, 4));
    }

    #[test]
    fn stats() {
        let mut ts = TransSet::new();
        ts.add_edge(1, 2); ts.reachable(1, 2);
        assert_eq!(ts.total_add(), 1);
        assert_eq!(ts.total_query(), 1);
    }
}
