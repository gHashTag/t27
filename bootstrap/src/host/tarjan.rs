use std::collections::BTreeMap;

pub struct Tarjan {
    adj: BTreeMap<u64, Vec<u64>>,
    index: BTreeMap<u64, usize>,
    lowlink: BTreeMap<u64, usize>,
    on_stack: BTreeMap<u64, bool>,
    stack: Vec<u64>,
    counter: usize,
    sccs: Vec<Vec<u64>>,
    total_add_edge: u64,
    total_scc_calls: u64,
}

impl Tarjan {
    pub fn new() -> Self {
        Self { adj: BTreeMap::new(), index: BTreeMap::new(), lowlink: BTreeMap::new(), on_stack: BTreeMap::new(),
               stack: Vec::new(), counter: 0, sccs: Vec::new(), total_add_edge: 0, total_scc_calls: 0 }
    }

    pub fn add_edge(&mut self, from: u64, to: u64) {
        self.total_add_edge += 1;
        self.adj.entry(from).or_default().push(to);
        self.adj.entry(to).or_insert_with(Vec::new);
    }

    pub fn find_sccs(&mut self) -> &[Vec<u64>] {
        self.total_scc_calls += 1;
        self.index.clear(); self.lowlink.clear(); self.on_stack.clear();
        self.stack.clear(); self.sccs.clear(); self.counter = 0;
        let nodes: Vec<u64> = self.adj.keys().copied().collect();
        for n in nodes { if !self.index.contains_key(&n) { self.strongconnect(n); } }
        &self.sccs
    }

    fn strongconnect(&mut self, v: u64) {
        self.index.insert(v, self.counter);
        self.lowlink.insert(v, self.counter);
        self.counter += 1;
        self.stack.push(v);
        self.on_stack.insert(v, true);
        let neighbors: Vec<u64> = self.adj.get(&v).cloned().unwrap_or_default();
        for w in neighbors {
            if !self.index.contains_key(&w) {
                self.strongconnect(w);
                let lw = self.lowlink[&w];
                let lv = self.lowlink[&v];
                self.lowlink.insert(v, lv.min(lw));
            } else if *self.on_stack.get(&w).unwrap_or(&false) {
                let iw = self.index[&w];
                let lv = self.lowlink[&v];
                self.lowlink.insert(v, lv.min(iw));
            }
        }
        if self.lowlink[&v] == self.index[&v] {
            let mut scc = Vec::new();
            loop {
                let w = self.stack.pop().unwrap();
                self.on_stack.insert(w, false);
                scc.push(w);
                if w == v { break; }
            }
            scc.sort();
            self.sccs.push(scc);
        }
    }

    pub fn scc_count(&mut self) -> usize { self.find_sccs().len() }
    pub fn total_add_edge(&self) -> u64 { self.total_add_edge }
    pub fn total_scc_calls(&self) -> u64 { self.total_scc_calls }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single() {
        let mut t = Tarjan::new();
        t.add_edge(1, 1);
        assert_eq!(t.scc_count(), 1);
    }

    #[test]
    fn dag() {
        let mut t = Tarjan::new();
        t.add_edge(1, 2); t.add_edge(2, 3);
        assert_eq!(t.scc_count(), 3);
    }

    #[test]
    fn cycle() {
        let mut t = Tarjan::new();
        t.add_edge(1, 2); t.add_edge(2, 3); t.add_edge(3, 1);
        assert_eq!(t.scc_count(), 1);
    }

    #[test]
    fn mixed() {
        let mut t = Tarjan::new();
        t.add_edge(1, 2); t.add_edge(2, 1);
        t.add_edge(2, 3); t.add_edge(3, 4); t.add_edge(4, 3);
        let sccs = t.find_sccs();
        assert_eq!(sccs.len(), 2);
    }

    #[test]
    fn disconnected() {
        let mut t = Tarjan::new();
        t.add_edge(1, 2); t.add_edge(3, 4);
        assert_eq!(t.scc_count(), 4);
    }

    #[test]
    fn stats() {
        let mut t = Tarjan::new();
        t.add_edge(1, 2); t.scc_count();
        assert_eq!(t.total_add_edge(), 1);
        assert_eq!(t.total_scc_calls(), 1);
    }
}
