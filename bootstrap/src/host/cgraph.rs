use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum CgError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
}

impl std::fmt::Display for CgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CgError::NodeExists { id } => write!(f, "node {id} exists"),
            CgError::NodeNotFound { id } => write!(f, "node {id} not found"),
        }
    }
}

impl std::error::Error for CgError {}

struct Node {
    id: u64,
    neighbors: BTreeSet<u64>,
    data: Vec<u8>,
}

pub struct CompactGraph {
    nodes: BTreeMap<u64, Node>,
    directed: bool,
    total_edges: u64,
    total_traversals: u64,
}

impl CompactGraph {
    pub fn new(directed: bool) -> Self { Self { nodes: BTreeMap::new(), directed, total_edges: 0, total_traversals: 0 } }

    pub fn add_node(&mut self, id: u64, data: Vec<u8>) -> Result<(), CgError> {
        if self.nodes.contains_key(&id) { return Err(CgError::NodeExists { id }); }
        self.nodes.insert(id, Node { id, neighbors: BTreeSet::new(), data });
        Ok(())
    }

    pub fn add_edge(&mut self, from: u64, to: u64) -> Result<(), CgError> {
        if !self.nodes.contains_key(&from) { return Err(CgError::NodeNotFound { id: from }); }
        if !self.nodes.contains_key(&to) { return Err(CgError::NodeNotFound { id: to }); }
        let added = self.nodes.get_mut(&from).unwrap().neighbors.insert(to);
        if added { self.total_edges += 1; }
        if !self.directed {
            self.nodes.get_mut(&to).unwrap().neighbors.insert(from);
        }
        Ok(())
    }

    pub fn neighbors(&self, id: u64) -> Option<Vec<u64>> { self.nodes.get(&id).map(|n| n.neighbors.iter().copied().collect()) }

    pub fn bfs(&mut self, start: u64) -> Vec<u64> {
        self.total_traversals += 1;
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        if !self.nodes.contains_key(&start) { return result; }
        queue.push_back(start);
        visited.insert(start);
        while let Some(id) = queue.pop_front() {
            result.push(id);
            if let Some(node) = self.nodes.get(&id) {
                for &nb in &node.neighbors {
                    if visited.insert(nb) { queue.push_back(nb); }
                }
            }
        }
        result
    }

    pub fn dfs(&mut self, start: u64) -> Vec<u64> {
        self.total_traversals += 1;
        let mut visited = BTreeSet::new();
        let mut stack = vec![start];
        let mut result = Vec::new();
        while let Some(id) = stack.pop() {
            if !visited.insert(id) { continue; }
            result.push(id);
            if let Some(node) = self.nodes.get(&id) {
                for &nb in node.neighbors.iter().rev() { stack.push(nb); }
            }
        }
        result
    }

    pub fn connected_components(&mut self) -> Vec<Vec<u64>> {
        self.total_traversals += 1;
        let mut visited = BTreeSet::new();
        let mut components = Vec::new();
        for &id in self.nodes.keys() {
            if visited.contains(&id) { continue; }
            let mut comp = Vec::new();
            let mut stack = vec![id];
            while let Some(nid) = stack.pop() {
                if !visited.insert(nid) { continue; }
                comp.push(nid);
                if let Some(node) = self.nodes.get(&nid) {
                    for &nb in &node.neighbors { stack.push(nb); }
                }
            }
            components.push(comp);
        }
        components
    }

    pub fn shortest_path(&mut self, from: u64, to: u64) -> Option<Vec<u64>> {
        self.total_traversals += 1;
        if from == to { return Some(vec![from]); }
        let mut visited = BTreeSet::new();
        let mut prev: BTreeMap<u64, u64> = BTreeMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(from);
        visited.insert(from);
        while let Some(id) = queue.pop_front() {
            if id == to { break; }
            if let Some(node) = self.nodes.get(&id) {
                for &nb in &node.neighbors {
                    if visited.insert(nb) {
                        prev.insert(nb, id);
                        queue.push_back(nb);
                    }
                }
            }
        }
        if !prev.contains_key(&to) && from != to { return None; }
        let mut path = Vec::new();
        let mut cur = to;
        path.push(cur);
        while let Some(&p) = prev.get(&cur) {
            path.push(p);
            cur = p;
        }
        path.reverse();
        Some(path)
    }

    pub fn has_edge(&self, from: u64, to: u64) -> bool {
        self.nodes.get(&from).map(|n| n.neighbors.contains(&to)).unwrap_or(false)
    }

    pub fn node_data(&self, id: u64) -> Option<&[u8]> { self.nodes.get(&id).map(|n| n.data.as_slice()) }
    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.total_edges as usize }
    pub fn total_traversals(&self) -> u64 { self.total_traversals }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_graph() { assert_eq!(CompactGraph::new(true).node_count(), 0); }

    #[test]
    fn add_nodes_edges() {
        let mut g = CompactGraph::new(false);
        g.add_node(1, vec![]).unwrap(); g.add_node(2, vec![]).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(g.has_edge(1, 2));
        assert!(g.has_edge(2, 1));
    }

    #[test]
    fn directed() {
        let mut g = CompactGraph::new(true);
        g.add_node(1, vec![]).unwrap(); g.add_node(2, vec![]).unwrap();
        g.add_edge(1, 2).unwrap();
        assert!(g.has_edge(1, 2));
        assert!(!g.has_edge(2, 1));
    }

    #[test]
    fn bfs_traversal() {
        let mut g = CompactGraph::new(false);
        g.add_node(1, vec![]).unwrap(); g.add_node(2, vec![]).unwrap(); g.add_node(3, vec![]).unwrap();
        g.add_edge(1, 2).unwrap(); g.add_edge(1, 3).unwrap();
        let order = g.bfs(1);
        assert_eq!(order[0], 1);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn dfs_traversal() {
        let mut g = CompactGraph::new(false);
        g.add_node(1, vec![]).unwrap(); g.add_node(2, vec![]).unwrap(); g.add_node(3, vec![]).unwrap();
        g.add_edge(1, 2).unwrap(); g.add_edge(2, 3).unwrap();
        let order = g.dfs(1);
        assert_eq!(order[0], 1);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn shortest_path() {
        let mut g = CompactGraph::new(false);
        g.add_node(1, vec![]).unwrap(); g.add_node(2, vec![]).unwrap(); g.add_node(3, vec![]).unwrap();
        g.add_edge(1, 2).unwrap(); g.add_edge(2, 3).unwrap();
        let path = g.shortest_path(1, 3).unwrap();
        assert_eq!(path, vec![1, 2, 3]);
    }

    #[test]
    fn no_path() {
        let mut g = CompactGraph::new(true);
        g.add_node(1, vec![]).unwrap(); g.add_node(2, vec![]).unwrap();
        assert!(g.shortest_path(1, 2).is_none());
    }

    #[test]
    fn components() {
        let mut g = CompactGraph::new(false);
        g.add_node(1, vec![]).unwrap(); g.add_node(2, vec![]).unwrap(); g.add_node(3, vec![]);
        g.add_edge(1, 2).unwrap();
        let comps = g.connected_components();
        assert_eq!(comps.len(), 2);
    }

    #[test]
    fn duplicate_node() {
        let mut g = CompactGraph::new(true);
        g.add_node(1, vec![]).unwrap();
        let err = g.add_node(1, vec![]).unwrap_err();
        assert!(matches!(err, CgError::NodeExists { .. }));
    }

    #[test]
    fn node_data() {
        let mut g = CompactGraph::new(true);
        g.add_node(1, b"data".to_vec()).unwrap();
        assert_eq!(g.node_data(1), Some(b"data".as_slice()));
    }

    #[test]
    fn error_display() { assert!(CgError::NodeNotFound { id: 3 }.to_string().contains("3")); }
}
