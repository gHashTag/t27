use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub struct PartGraph {
    partitions: BTreeMap<u64, BTreeSet<u64>>,
    edges: BTreeMap<u64, BTreeSet<u64>>,
    total_added: u64,
    total_traversals: u64,
}

impl PartGraph {
    pub fn new() -> Self { Self { partitions: BTreeMap::new(), edges: BTreeMap::new(), total_added: 0, total_traversals: 0 } }

    pub fn add_node(&mut self, node: u64, partition: u64) {
        self.total_added += 1;
        self.partitions.entry(partition).or_default().insert(node);
        self.edges.entry(node).or_default();
    }

    pub fn add_edge(&mut self, from: u64, to: u64) {
        self.total_added += 1;
        self.edges.entry(from).or_default().insert(to);
        self.edges.entry(to).or_default().insert(from);
    }

    pub fn partition_of(&self, node: u64) -> Option<u64> {
        for (&pid, nodes) in &self.partitions {
            if nodes.contains(&node) { return Some(pid); }
        }
        None
    }

    pub fn intra_edges(&self, partition: u64) -> Vec<(u64, u64)> {
        let nodes = match self.partitions.get(&partition) { Some(n) => n, None => return Vec::new() };
        let mut result = Vec::new();
        for &n in nodes {
            if let Some(neighbors) = self.edges.get(&n) {
                for &nb in neighbors {
                    if nodes.contains(&nb) && n < nb { result.push((n, nb)); }
                }
            }
        }
        result
    }

    pub fn inter_edges(&self, partition: u64) -> Vec<(u64, u64)> {
        let nodes = match self.partitions.get(&partition) { Some(n) => n, None => return Vec::new() };
        let mut result = Vec::new();
        for &n in nodes {
            if let Some(neighbors) = self.edges.get(&n) {
                for &nb in neighbors {
                    if !nodes.contains(&nb) { result.push((n, nb)); }
                }
            }
        }
        result
    }

    pub fn bfs(&mut self, start: u64) -> Vec<u64> {
        self.total_traversals += 1;
        let mut visited = BTreeSet::new();
        let mut q = VecDeque::new();
        let mut result = Vec::new();
        q.push_back(start);
        visited.insert(start);
        while let Some(node) = q.pop_front() {
            result.push(node);
            if let Some(neighbors) = self.edges.get(&node) {
                for &nb in neighbors {
                    if visited.insert(nb) { q.push_back(nb); }
                }
            }
        }
        result
    }

    pub fn partition_count(&self) -> usize { self.partitions.len() }
    pub fn node_count(&self) -> usize { self.edges.len() }
    pub fn edge_count(&self) -> usize { self.edges.values().map(|s| s.len()).sum::<usize>() / 2 }
    pub fn partition_size(&self, pid: u64) -> usize { self.partitions.get(&pid).map(|s| s.len()).unwrap_or(0) }
    pub fn total_added(&self) -> u64 { self.total_added }
    pub fn total_traversals(&self) -> u64 { self.total_traversals }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_node() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 0); pg.add_node(2, 0); pg.add_node(3, 1);
        assert_eq!(pg.node_count(), 3);
        assert_eq!(pg.partition_count(), 2);
    }

    #[test]
    fn partition_of() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 10);
        assert_eq!(pg.partition_of(1), Some(10));
        assert_eq!(pg.partition_of(99), None);
    }

    #[test]
    fn intra_edges() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 0); pg.add_node(2, 0); pg.add_node(3, 1);
        pg.add_edge(1, 2); pg.add_edge(1, 3);
        let intra = pg.intra_edges(0);
        assert_eq!(intra.len(), 1);
    }

    #[test]
    fn inter_edges() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 0); pg.add_node(2, 1);
        pg.add_edge(1, 2);
        assert_eq!(pg.inter_edges(0).len(), 1);
        assert_eq!(pg.inter_edges(1).len(), 1);
    }

    #[test]
    fn bfs() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 0); pg.add_node(2, 0); pg.add_node(3, 0);
        pg.add_edge(1, 2); pg.add_edge(2, 3);
        let order = pg.bfs(1);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn edge_count() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 0); pg.add_node(2, 0);
        pg.add_edge(1, 2);
        assert_eq!(pg.edge_count(), 1);
    }

    #[test]
    fn partition_size() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 0); pg.add_node(2, 0); pg.add_node(3, 1);
        assert_eq!(pg.partition_size(0), 2);
        assert_eq!(pg.partition_size(1), 1);
    }

    #[test]
    fn stats() {
        let mut pg = PartGraph::new();
        pg.add_node(1, 0); pg.add_edge(1, 2);
        pg.bfs(1);
        assert_eq!(pg.total_added(), 2);
        assert_eq!(pg.total_traversals(), 1);
    }
}
