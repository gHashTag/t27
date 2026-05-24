use std::collections::{BTreeMap, BinaryHeap};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum TopoError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    EdgeExists { from: u64, to: u64 },
    NoPath { from: u64, to: u64 },
}

impl std::fmt::Display for TopoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopoError::NodeExists { id } => write!(f, "node {id} exists"),
            TopoError::NodeNotFound { id } => write!(f, "node {id} not found"),
            TopoError::EdgeExists { from, to } => write!(f, "edge {from}->{to} exists"),
            TopoError::NoPath { from, to } => write!(f, "no path {from}->{to}"),
        }
    }
}

impl std::error::Error for TopoError {}

struct Edge { to: u64, latency_us: u64 }

struct Node {
    id: u64,
    edges: Vec<Edge>,
}

#[derive(Debug, Clone)]
pub struct PathResult { pub path: Vec<u64>, pub total_latency: u64 }

#[derive(Eq, PartialEq)]
struct HeapEntry { cost: u64, node: u64 }

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering { other.cost.cmp(&self.cost) }
}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

pub struct TopoMap {
    nodes: BTreeMap<u64, Node>,
    total_edges: u64,
    total_queries: u64,
}

impl TopoMap {
    pub fn new() -> Self { Self { nodes: BTreeMap::new(), total_edges: 0, total_queries: 0 } }

    pub fn add_node(&mut self, id: u64) -> Result<(), TopoError> {
        if self.nodes.contains_key(&id) { return Err(TopoError::NodeExists { id }); }
        self.nodes.insert(id, Node { id, edges: Vec::new() });
        Ok(())
    }

    pub fn add_edge(&mut self, from: u64, to: u64, latency_us: u64) -> Result<(), TopoError> {
        if !self.nodes.contains_key(&from) { return Err(TopoError::NodeNotFound { id: from }); }
        if !self.nodes.contains_key(&to) { return Err(TopoError::NodeNotFound { id: to }); }
        let node = self.nodes.get_mut(&from).unwrap();
        if node.edges.iter().any(|e| e.to == to) { return Err(TopoError::EdgeExists { from, to }); }
        node.edges.push(Edge { to, latency_us });
        self.total_edges += 1;
        Ok(())
    }

    pub fn shortest_path(&mut self, from: u64, to: u64) -> Result<PathResult, TopoError> {
        if !self.nodes.contains_key(&from) { return Err(TopoError::NodeNotFound { id: from }); }
        if !self.nodes.contains_key(&to) { return Err(TopoError::NodeNotFound { id: to }); }
        self.total_queries += 1;
        let mut dist: BTreeMap<u64, u64> = BTreeMap::new();
        let mut prev: BTreeMap<u64, u64> = BTreeMap::new();
        let mut heap = BinaryHeap::new();
        for &id in self.nodes.keys() { dist.insert(id, u64::MAX); }
        dist.insert(from, 0);
        heap.push(HeapEntry { cost: 0, node: from });
        while let Some(HeapEntry { cost, node }) = heap.pop() {
            if cost > dist[&node] { continue; }
            if node == to { break; }
            if let Some(n) = self.nodes.get(&node) {
                for edge in &n.edges {
                    let new_cost = cost.saturating_add(edge.latency_us);
                    if new_cost < dist[&edge.to] {
                        dist.insert(edge.to, new_cost);
                        prev.insert(edge.to, node);
                        heap.push(HeapEntry { cost: new_cost, node: edge.to });
                    }
                }
            }
        }
        let total_latency = dist[&to];
        if total_latency == u64::MAX { return Err(TopoError::NoPath { from, to }); }
        let mut path = Vec::new();
        let mut current = to;
        path.push(current);
        while let Some(&p) = prev.get(&current) {
            path.push(p);
            current = p;
        }
        path.reverse();
        Ok(PathResult { path, total_latency })
    }

    pub fn neighbors(&self, id: u64) -> Option<Vec<(u64, u64)>> {
        self.nodes.get(&id).map(|n| n.edges.iter().map(|e| (e.to, e.latency_us)).collect())
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.total_edges as usize }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

impl Default for TopoMap {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map() { assert_eq!(TopoMap::new().node_count(), 0); }

    #[test]
    fn add_nodes_edges() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap(); t.add_node(2).unwrap();
        t.add_edge(1, 2, 100).unwrap();
        assert_eq!(t.edge_count(), 1);
    }

    #[test]
    fn shortest_path_linear() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap(); t.add_node(2).unwrap(); t.add_node(3).unwrap();
        t.add_edge(1, 2, 10).unwrap(); t.add_edge(2, 3, 20).unwrap();
        let r = t.shortest_path(1, 3).unwrap();
        assert_eq!(r.path, vec![1, 2, 3]);
        assert_eq!(r.total_latency, 30);
    }

    #[test]
    fn shortest_path_bypass() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap(); t.add_node(2).unwrap(); t.add_node(3).unwrap();
        t.add_edge(1, 2, 10).unwrap(); t.add_edge(2, 3, 100).unwrap();
        t.add_edge(1, 3, 50).unwrap();
        let r = t.shortest_path(1, 3).unwrap();
        assert_eq!(r.path, vec![1, 3]);
        assert_eq!(r.total_latency, 50);
    }

    #[test]
    fn no_path() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap(); t.add_node(2).unwrap();
        let err = t.shortest_path(1, 2).unwrap_err();
        assert!(matches!(err, TopoError::NoPath { .. }));
    }

    #[test]
    fn duplicate_node() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap();
        let err = t.add_node(1).unwrap_err();
        assert!(matches!(err, TopoError::NodeExists { .. }));
    }

    #[test]
    fn duplicate_edge() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap(); t.add_node(2).unwrap();
        t.add_edge(1, 2, 10).unwrap();
        let err = t.add_edge(1, 2, 20).unwrap_err();
        assert!(matches!(err, TopoError::EdgeExists { .. }));
    }

    #[test]
    fn neighbors() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap(); t.add_node(2).unwrap(); t.add_node(3).unwrap();
        t.add_edge(1, 2, 10).unwrap(); t.add_edge(1, 3, 20).unwrap();
        let nb = t.neighbors(1).unwrap();
        assert_eq!(nb.len(), 2);
    }

    #[test]
    fn stats() {
        let mut t = TopoMap::new();
        t.add_node(1).unwrap(); t.add_node(2).unwrap();
        t.add_edge(1, 2, 10).unwrap();
        t.shortest_path(1, 2).unwrap();
        assert_eq!(t.total_queries(), 1);
    }

    #[test]
    fn error_display() { assert!(TopoError::NoPath { from: 1, to: 2 }.to_string().contains("1")); }
}
