use std::collections::{BTreeMap, VecDeque};

pub struct Topo {
    adj: BTreeMap<u64, Vec<u64>>,
    in_degree: BTreeMap<u64, u64>,
    total_add_edge: u64,
    total_sorts: u64,
}

impl Topo {
    pub fn new() -> Self { Self { adj: BTreeMap::new(), in_degree: BTreeMap::new(), total_add_edge: 0, total_sorts: 0 } }

    pub fn add_edge(&mut self, from: u64, to: u64) {
        self.total_add_edge += 1;
        self.adj.entry(from).or_default().push(to);
        self.adj.entry(to).or_insert_with(Vec::new);
        *self.in_degree.entry(to).or_insert(0) += 1;
        self.in_degree.entry(from).or_insert(0);
    }

    pub fn sort(&mut self) -> (Vec<u64>, bool) {
        self.total_sorts += 1;
        let mut deg = self.in_degree.clone();
        let mut queue = VecDeque::new();
        for (&node, &d) in &deg { if d == 0 { queue.push_back(node); } }
        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node);
            for &nbr in self.adj.get(&node).unwrap_or(&vec![]) {
                let d = deg.get_mut(&nbr).unwrap();
                *d -= 1;
                if *d == 0 { queue.push_back(nbr); }
            }
        }
        let dag = result.len() == self.adj.len();
        (result, dag)
    }

    pub fn is_dag(&mut self) -> bool { self.sort().1 }
    pub fn node_count(&self) -> usize { self.adj.len() }
    pub fn total_add_edge(&self) -> u64 { self.total_add_edge }
    pub fn total_sorts(&self) -> u64 { self.total_sorts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear() {
        let mut t = Topo::new();
        t.add_edge(0, 1); t.add_edge(1, 2);
        let (order, dag) = t.sort();
        assert!(dag);
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn diamond() {
        let mut t = Topo::new();
        t.add_edge(0, 1); t.add_edge(0, 2); t.add_edge(1, 3); t.add_edge(2, 3);
        let (order, dag) = t.sort();
        assert!(dag);
        assert_eq!(order[0], 0);
        assert_eq!(order[3], 3);
    }

    #[test]
    fn cycle() {
        let mut t = Topo::new();
        t.add_edge(0, 1); t.add_edge(1, 2); t.add_edge(2, 0);
        assert!(!t.is_dag());
    }

    #[test]
    fn self_loop() {
        let mut t = Topo::new();
        t.add_edge(0, 0);
        assert!(!t.is_dag());
    }

    #[test]
    fn empty() { let (order, dag) = Topo::new().sort(); assert!(dag); assert!(order.is_empty()); }

    #[test]
    fn stats() {
        let mut t = Topo::new();
        t.add_edge(0, 1); t.sort();
        assert_eq!(t.total_add_edge(), 1);
        assert_eq!(t.total_sorts(), 1);
    }
}
