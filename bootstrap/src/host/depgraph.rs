use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepError {
    Cycle { node: u64 },
    NodeExists { node: u64 },
    NodeNotFound { node: u64 },
    SelfDep { node: u64 },
}

impl std::fmt::Display for DepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepError::Cycle { node } => write!(f, "cycle at {node}"),
            DepError::NodeExists { node } => write!(f, "node {node} exists"),
            DepError::NodeNotFound { node } => write!(f, "node {node} not found"),
            DepError::SelfDep { node } => write!(f, "self-dep on {node}"),
        }
    }
}

impl std::error::Error for DepError {}

#[derive(Debug, Clone)]
pub struct DepGraph {
    deps: BTreeMap<u64, BTreeSet<u64>>,
    reverse: BTreeMap<u64, BTreeSet<u64>>,
    total_nodes: u64,
    total_edges: u64,
}

impl DepGraph {
    pub fn new() -> Self {
        Self { deps: BTreeMap::new(), reverse: BTreeMap::new(), total_nodes: 0, total_edges: 0 }
    }

    pub fn add_node(&mut self, node: u64) -> Result<(), DepError> {
        if self.deps.contains_key(&node) {
            return Err(DepError::NodeExists { node });
        }
        self.deps.insert(node, BTreeSet::new());
        self.reverse.insert(node, BTreeSet::new());
        self.total_nodes += 1;
        Ok(())
    }

    pub fn add_dep(&mut self, node: u64, dep: u64) -> Result<(), DepError> {
        if node == dep { return Err(DepError::SelfDep { node }); }
        if !self.deps.contains_key(&node) { return Err(DepError::NodeNotFound { node }); }
        if !self.deps.contains_key(&dep) { return Err(DepError::NodeNotFound { node: dep }); }
        if self.would_cycle(node, dep) {
            return Err(DepError::Cycle { node });
        }
        let deps_set = self.deps.get_mut(&node).unwrap();
        if deps_set.insert(dep) {
            self.reverse.get_mut(&dep).unwrap().insert(node);
            self.total_edges += 1;
        }
        Ok(())
    }

    fn would_cycle(&self, from: u64, to: u64) -> bool {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(to);
        while let Some(n) = queue.pop_front() {
            if n == from { return true; }
            if visited.insert(n) {
                if let Some(deps) = self.deps.get(&n) {
                    for &d in deps { queue.push_back(d); }
                }
            }
        }
        false
    }

    pub fn remove_node(&mut self, node: u64) -> Result<u64, DepError> {
        if !self.deps.contains_key(&node) {
            return Err(DepError::NodeNotFound { node });
        }
        let edge_count = self.deps[&node].len() as u64;
        for &dep in &self.deps[&node] {
            if let Some(rset) = self.reverse.get_mut(&dep) { rset.remove(&node); }
        }
        if let Some(rset) = self.reverse.get(&node) {
            for &dependent in rset {
                if let Some(dset) = self.deps.get_mut(&dependent) { dset.remove(&node); }
            }
        }
        self.deps.remove(&node);
        self.reverse.remove(&node);
        self.total_edges -= edge_count;
        self.total_nodes -= 1;
        Ok(edge_count)
    }

    pub fn deps_of(&self, node: u64) -> Vec<u64> {
        self.deps.get(&node).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }

    pub fn dependents(&self, node: u64) -> Vec<u64> {
        self.reverse.get(&node).map(|s| s.iter().copied().collect()).unwrap_or_default()
    }

    pub fn has_node(&self, node: u64) -> bool { self.deps.contains_key(&node) }

    pub fn topological_sort(&self) -> Result<Vec<u64>, DepError> {
        let mut in_degree: BTreeMap<u64, usize> = self.deps.keys().map(|&n| (n, self.deps[&n].len())).collect();
        let mut queue: VecDeque<u64> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0).map(|(&n, _)| n).collect();
        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node);
            for &dependent in &self.reverse[&node] {
                let entry = in_degree.get_mut(&dependent).unwrap();
                *entry -= 1;
                if *entry == 0 { queue.push_back(dependent); }
            }
        }
        if result.len() != self.deps.len() {
            let cycle_node = *self.deps.keys().find(|n| !result.contains(n)).unwrap();
            return Err(DepError::Cycle { node: cycle_node });
        }
        Ok(result)
    }

    pub fn node_count(&self) -> usize { self.deps.len() }
    pub fn edge_count(&self) -> u64 { self.total_edges }
    pub fn total_nodes(&self) -> u64 { self.total_nodes }
}

impl Default for DepGraph {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_graph() {
        let dg = DepGraph::new();
        assert_eq!(dg.node_count(), 0);
    }

    #[test]
    fn add_node() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap();
        dg.add_node(2).unwrap();
        assert_eq!(dg.node_count(), 2);
    }

    #[test]
    fn duplicate_node() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap();
        let err = dg.add_node(1).unwrap_err();
        assert!(matches!(err, DepError::NodeExists { .. }));
    }

    #[test]
    fn add_dep() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap(); dg.add_node(2).unwrap();
        dg.add_dep(1, 2).unwrap();
        assert_eq!(dg.deps_of(1), vec![2]);
        assert_eq!(dg.dependents(2), vec![1]);
        assert_eq!(dg.edge_count(), 1);
    }

    #[test]
    fn self_dep_rejected() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap();
        let err = dg.add_dep(1, 1).unwrap_err();
        assert!(matches!(err, DepError::SelfDep { .. }));
    }

    #[test]
    fn cycle_rejected() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap(); dg.add_node(2).unwrap(); dg.add_node(3).unwrap();
        dg.add_dep(1, 2).unwrap();
        dg.add_dep(2, 3).unwrap();
        let err = dg.add_dep(3, 1).unwrap_err();
        assert!(matches!(err, DepError::Cycle { .. }));
    }

    #[test]
    fn topological_sort() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap(); dg.add_node(2).unwrap(); dg.add_node(3).unwrap();
        dg.add_dep(3, 1).unwrap();
        dg.add_dep(3, 2).unwrap();
        dg.add_dep(2, 1).unwrap();
        let order = dg.topological_sort().unwrap();
        assert_eq!(order.len(), 3);
        assert!(order.iter().position(|&n| n == 1).unwrap() < order.iter().position(|&n| n == 2).unwrap());
        assert!(order.iter().position(|&n| n == 2).unwrap() < order.iter().position(|&n| n == 3).unwrap());
    }

    #[test]
    fn remove_node() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap(); dg.add_node(2).unwrap();
        dg.add_dep(1, 2).unwrap();
        dg.remove_node(2).unwrap();
        assert_eq!(dg.deps_of(1).len(), 0);
        assert_eq!(dg.node_count(), 1);
    }

    #[test]
    fn node_not_found() {
        let mut dg = DepGraph::new();
        let err = dg.remove_node(99).unwrap_err();
        assert!(matches!(err, DepError::NodeNotFound { .. }));
    }

    #[test]
    fn no_deps_order() {
        let mut dg = DepGraph::new();
        dg.add_node(1).unwrap(); dg.add_node(2).unwrap(); dg.add_node(3).unwrap();
        let order = dg.topological_sort().unwrap();
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn diamond_deps() {
        let mut dg = DepGraph::new();
        for i in 1..=4 { dg.add_node(i).unwrap(); }
        dg.add_dep(4, 2).unwrap();
        dg.add_dep(4, 3).unwrap();
        dg.add_dep(2, 1).unwrap();
        dg.add_dep(3, 1).unwrap();
        let order = dg.topological_sort().unwrap();
        assert_eq!(order[0], 1);
        assert_eq!(order[3], 4);
    }

    #[test]
    fn error_display() {
        assert!(DepError::Cycle { node: 5 }.to_string().contains("5"));
    }
}
