use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    SelfLoop { id: u64 },
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::NodeExists { id } => write!(f, "node {id} exists"),
            GraphError::NodeNotFound { id } => write!(f, "node {id} not found"),
            GraphError::SelfLoop { id } => write!(f, "self-loop on {id}"),
        }
    }
}

impl std::error::Error for GraphError {}

#[derive(Debug, Clone)]
struct Edge {
    to: u64,
    weight: u64,
}

#[derive(Debug, Clone)]
pub struct AdjGraph {
    adj: BTreeMap<u64, Vec<Edge>>,
    nodes: BTreeSet<u64>,
    total_edges: u64,
}

impl AdjGraph {
    pub fn new() -> Self {
        Self { adj: BTreeMap::new(), nodes: BTreeSet::new(), total_edges: 0 }
    }

    pub fn add_node(&mut self, id: u64) -> Result<(), GraphError> {
        if self.nodes.contains(&id) {
            return Err(GraphError::NodeExists { id });
        }
        self.nodes.insert(id);
        self.adj.insert(id, Vec::new());
        Ok(())
    }

    pub fn remove_node(&mut self, id: u64) -> Result<u64, GraphError> {
        if !self.nodes.contains(&id) {
            return Err(GraphError::NodeNotFound { id });
        }
        let edges_out = self.adj.remove(&id).map(|v| v.len() as u64).unwrap_or(0);
        for edges in self.adj.values_mut() {
            let before = edges.len();
            edges.retain(|e| e.to != id);
            self.total_edges -= (before - edges.len()) as u64;
        }
        self.nodes.remove(&id);
        self.total_edges -= edges_out;
        Ok(edges_out)
    }

    pub fn add_edge(&mut self, from: u64, to: u64, weight: u64) -> Result<(), GraphError> {
        if from == to { return Err(GraphError::SelfLoop { id: from }); }
        if !self.nodes.contains(&from) { return Err(GraphError::NodeNotFound { id: from }); }
        if !self.nodes.contains(&to) { return Err(GraphError::NodeNotFound { id: to }); }
        self.adj.get_mut(&from).unwrap().push(Edge { to, weight });
        self.total_edges += 1;
        Ok(())
    }

    pub fn neighbors(&self, id: u64) -> Vec<(u64, u64)> {
        self.adj.get(&id).map(|edges| edges.iter().map(|e| (e.to, e.weight)).collect()).unwrap_or_default()
    }

    pub fn out_degree(&self, id: u64) -> usize {
        self.adj.get(&id).map(|v| v.len()).unwrap_or(0)
    }

    pub fn in_degree(&self, id: u64) -> usize {
        self.adj.values().map(|edges| edges.iter().filter(|e| e.to == id).count()).sum()
    }

    pub fn has_node(&self, id: u64) -> bool {
        self.nodes.contains(&id)
    }

    pub fn has_edge(&self, from: u64, to: u64) -> bool {
        self.adj.get(&from).map(|edges| edges.iter().any(|e| e.to == to)).unwrap_or(false)
    }

    pub fn edge_weight(&self, from: u64, to: u64) -> Option<u64> {
        self.adj.get(&from).and_then(|edges| edges.iter().find(|e| e.to == to).map(|e| e.weight))
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> u64 { self.total_edges }

    pub fn bfs(&self, start: u64) -> Vec<u64> {
        if !self.nodes.contains(&start) { return Vec::new(); }
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut order = Vec::new();
        visited.insert(start);
        queue.push_back(start);
        while let Some(node) = queue.pop_front() {
            order.push(node);
            if let Some(edges) = self.adj.get(&node) {
                for edge in edges {
                    if visited.insert(edge.to) {
                        queue.push_back(edge.to);
                    }
                }
            }
        }
        order
    }

    pub fn node_ids(&self) -> Vec<u64> {
        self.nodes.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.adj.clear();
        self.nodes.clear();
        self.total_edges = 0;
    }
}

impl Default for AdjGraph {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_graph() {
        let g = AdjGraph::new();
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn add_node() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap();
        g.add_node(2).unwrap();
        assert_eq!(g.node_count(), 2);
        assert!(g.has_node(1));
    }

    #[test]
    fn duplicate_node() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap();
        let err = g.add_node(1).unwrap_err();
        assert!(matches!(err, GraphError::NodeExists { .. }));
    }

    #[test]
    fn add_edge_and_neighbors() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap(); g.add_node(2).unwrap(); g.add_node(3).unwrap();
        g.add_edge(1, 2, 10).unwrap();
        g.add_edge(1, 3, 20).unwrap();
        let nb = g.neighbors(1);
        assert_eq!(nb.len(), 2);
        assert!(g.has_edge(1, 2));
        assert_eq!(g.edge_weight(1, 2), Some(10));
    }

    #[test]
    fn self_loop_rejected() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap();
        let err = g.add_edge(1, 1, 5).unwrap_err();
        assert!(matches!(err, GraphError::SelfLoop { .. }));
    }

    #[test]
    fn degrees() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap(); g.add_node(2).unwrap(); g.add_node(3).unwrap();
        g.add_edge(1, 2, 1).unwrap();
        g.add_edge(1, 3, 1).unwrap();
        g.add_edge(2, 3, 1).unwrap();
        assert_eq!(g.out_degree(1), 2);
        assert_eq!(g.in_degree(3), 2);
    }

    #[test]
    fn remove_node() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap(); g.add_node(2).unwrap();
        g.add_edge(1, 2, 5).unwrap();
        g.remove_node(2).unwrap();
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn bfs_traversal() {
        let mut g = AdjGraph::new();
        for i in 1..=5 { g.add_node(i).unwrap(); }
        g.add_edge(1, 2, 1).unwrap();
        g.add_edge(1, 3, 1).unwrap();
        g.add_edge(2, 4, 1).unwrap();
        g.add_edge(3, 5, 1).unwrap();
        let order = g.bfs(1);
        assert_eq!(order[0], 1);
        assert!(order.contains(&2) && order.contains(&3));
        assert_eq!(order.len(), 5);
    }

    #[test]
    fn bfs_disconnected() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap(); g.add_node(2).unwrap();
        let order = g.bfs(1);
        assert_eq!(order, vec![1]);
    }

    #[test]
    fn remove_not_found() {
        let mut g = AdjGraph::new();
        let err = g.remove_node(99).unwrap_err();
        assert!(matches!(err, GraphError::NodeNotFound { .. }));
    }

    #[test]
    fn clear() {
        let mut g = AdjGraph::new();
        g.add_node(1).unwrap();
        g.clear();
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn error_display() {
        assert!(GraphError::SelfLoop { id: 1 }.to_string().contains("1"));
    }
}
