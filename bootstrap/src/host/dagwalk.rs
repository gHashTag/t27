use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum DagError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    CycleDetected { from: u64, to: u64 },
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DagError::NodeExists { id } => write!(f, "node {id} exists"),
            DagError::NodeNotFound { id } => write!(f, "node {id} not found"),
            DagError::CycleDetected { from, to } => write!(f, "cycle: {from}->{to}"),
        }
    }
}

impl std::error::Error for DagError {}

struct Node {
    id: u64,
    out_edges: BTreeSet<u64>,
    in_edges: BTreeSet<u64>,
}

pub struct DagWalker {
    nodes: BTreeMap<u64, Node>,
    total_adds: u64,
    total_edges: u64,
    total_walks: u64,
}

pub type Visitor = Box<dyn FnMut(u64, u32)>;

impl DagWalker {
    pub fn new() -> Self { Self { nodes: BTreeMap::new(), total_adds: 0, total_edges: 0, total_walks: 0 } }

    pub fn add_node(&mut self, id: u64) -> Result<(), DagError> {
        if self.nodes.contains_key(&id) { return Err(DagError::NodeExists { id }); }
        self.nodes.insert(id, Node { id, out_edges: BTreeSet::new(), in_edges: BTreeSet::new() });
        self.total_adds += 1;
        Ok(())
    }

    pub fn add_edge(&mut self, from: u64, to: u64) -> Result<(), DagError> {
        if !self.nodes.contains_key(&from) { return Err(DagError::NodeNotFound { id: from }); }
        if !self.nodes.contains_key(&to) { return Err(DagError::NodeNotFound { id: to }); }
        if self.would_cycle(from, to) { return Err(DagError::CycleDetected { from, to }); }
        self.nodes.get_mut(&from).unwrap().out_edges.insert(to);
        self.nodes.get_mut(&to).unwrap().in_edges.insert(from);
        self.total_edges += 1;
        Ok(())
    }

    fn would_cycle(&self, from: u64, to: u64) -> bool {
        if from == to { return true; }
        let mut visited = BTreeSet::new();
        let mut stack = vec![to];
        while let Some(n) = stack.pop() {
            if n == from { return true; }
            if visited.insert(n) {
                if let Some(node) = self.nodes.get(&n) {
                    for &child in &node.out_edges { stack.push(child); }
                }
            }
        }
        false
    }

    pub fn topo_sort(&mut self) -> Vec<u64> {
        self.total_walks += 1;
        let mut in_deg: BTreeMap<u64, usize> = self.nodes.keys().map(|&id| (id, self.nodes[&id].in_edges.len())).collect();
        let mut queue: VecDeque<u64> = in_deg.iter().filter(|(_, &d)| d == 0).map(|(&id, _)| id).collect();
        let mut result = Vec::new();
        while let Some(id) = queue.pop_front() {
            result.push(id);
            if let Some(node) = self.nodes.get(&id) {
                for &child in &node.out_edges {
                    let deg = in_deg.get_mut(&child).unwrap();
                    *deg -= 1;
                    if *deg == 0 { queue.push_back(child); }
                }
            }
        }
        result
    }

    pub fn bfs(&mut self, start: u64) -> Vec<u64> {
        self.total_walks += 1;
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        if !self.nodes.contains_key(&start) { return result; }
        queue.push_back(start);
        visited.insert(start);
        while let Some(id) = queue.pop_front() {
            result.push(id);
            if let Some(node) = self.nodes.get(&id) {
                for &child in &node.out_edges {
                    if visited.insert(child) { queue.push_back(child); }
                }
            }
        }
        result
    }

    pub fn dfs(&mut self, start: u64) -> Vec<u64> {
        self.total_walks += 1;
        let mut visited = BTreeSet::new();
        let mut stack = vec![start];
        let mut result = Vec::new();
        while let Some(id) = stack.pop() {
            if !visited.insert(id) { continue; }
            result.push(id);
            if let Some(node) = self.nodes.get(&id) {
                for &child in node.out_edges.iter().rev() { stack.push(child); }
            }
        }
        result
    }

    pub fn roots(&self) -> Vec<u64> {
        self.nodes.iter().filter(|(_, n)| n.in_edges.is_empty()).map(|(&id, _)| id).collect()
    }

    pub fn leaves(&self) -> Vec<u64> {
        self.nodes.iter().filter(|(_, n)| n.out_edges.is_empty()).map(|(&id, _)| id).collect()
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.total_edges as usize }
    pub fn total_walks(&self) -> u64 { self.total_walks }
}

impl Default for DagWalker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dag() { assert_eq!(DagWalker::new().node_count(), 0); }

    #[test]
    fn add_nodes() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap(); d.add_node(3).unwrap();
        assert_eq!(d.node_count(), 3);
    }

    #[test]
    fn topo_linear() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap(); d.add_node(3).unwrap();
        d.add_edge(1, 2).unwrap(); d.add_edge(2, 3).unwrap();
        let order = d.topo_sort();
        assert_eq!(order, vec![1, 2, 3]);
    }

    #[test]
    fn topo_diamond() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap(); d.add_node(3).unwrap(); d.add_node(4).unwrap();
        d.add_edge(1, 2).unwrap(); d.add_edge(1, 3).unwrap();
        d.add_edge(2, 4).unwrap(); d.add_edge(3, 4).unwrap();
        let order = d.topo_sort();
        assert_eq!(order[0], 1);
        assert_eq!(order[3], 4);
    }

    #[test]
    fn cycle_detection() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap();
        d.add_edge(1, 2).unwrap();
        let err = d.add_edge(2, 1).unwrap_err();
        assert!(matches!(err, DagError::CycleDetected { .. }));
    }

    #[test]
    fn bfs_traversal() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap(); d.add_node(3).unwrap();
        d.add_edge(1, 2).unwrap(); d.add_edge(1, 3).unwrap();
        let order = d.bfs(1);
        assert_eq!(order[0], 1);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn dfs_traversal() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap(); d.add_node(3).unwrap();
        d.add_edge(1, 2).unwrap(); d.add_edge(2, 3).unwrap();
        let order = d.dfs(1);
        assert_eq!(order, vec![1, 2, 3]);
    }

    #[test]
    fn roots_leaves() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap(); d.add_node(3).unwrap();
        d.add_edge(1, 2).unwrap(); d.add_edge(2, 3).unwrap();
        assert_eq!(d.roots(), vec![1]);
        assert_eq!(d.leaves(), vec![3]);
    }

    #[test]
    fn duplicate_node() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap();
        let err = d.add_node(1).unwrap_err();
        assert!(matches!(err, DagError::NodeExists { .. }));
    }

    #[test]
    fn stats() {
        let mut d = DagWalker::new();
        d.add_node(1).unwrap(); d.add_node(2).unwrap();
        d.add_edge(1, 2).unwrap();
        d.topo_sort();
        assert_eq!(d.total_walks(), 1);
    }

    #[test]
    fn error_display() { assert!(DagError::NodeNotFound { id: 3 }.to_string().contains("3")); }
}
