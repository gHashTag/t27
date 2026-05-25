use std::collections::{BTreeMap, VecDeque};

pub struct SparseGraph {
    adj: BTreeMap<u64, Vec<u64>>,
    directed: bool,
    total_add_edge: u64,
    total_traversals: u64,
}

impl SparseGraph {
    pub fn new(directed: bool) -> Self { Self { adj: BTreeMap::new(), directed, total_add_edge: 0, total_traversals: 0 } }

    pub fn add_edge(&mut self, from: u64, to: u64) {
        self.total_add_edge += 1;
        self.adj.entry(from).or_default().push(to);
        self.adj.entry(to).or_insert_with(Vec::new);
        if !self.directed { self.adj.entry(to).or_default().push(from); }
    }

    pub fn neighbors(&self, node: u64) -> &[u64] { self.adj.get(&node).map(|v| v.as_slice()).unwrap_or(&[]) }

    pub fn bfs(&mut self, start: u64) -> Vec<u64> {
        self.total_traversals += 1;
        let mut visited = BTreeMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        visited.insert(start, true);
        queue.push_back(start);
        while let Some(node) = queue.pop_front() {
            result.push(node);
            for &nbr in self.neighbors(node) {
                if !visited.contains_key(&nbr) { visited.insert(nbr, true); queue.push_back(nbr); }
            }
        }
        result
    }

    pub fn dfs(&mut self, start: u64) -> Vec<u64> {
        self.total_traversals += 1;
        let mut visited = BTreeMap::new();
        let mut result = Vec::new();
        self.dfs_rec(start, &mut visited, &mut result);
        result
    }

    fn dfs_rec(&self, node: u64, visited: &mut BTreeMap<u64, bool>, result: &mut Vec<u64>) {
        visited.insert(node, true);
        result.push(node);
        for &nbr in self.neighbors(node) {
            if !visited.contains_key(&nbr) { self.dfs_rec(nbr, visited, result); }
        }
    }

    pub fn has_path(&mut self, from: u64, to: u64) -> bool { self.bfs(from).contains(&to) }
    pub fn node_count(&self) -> usize { self.adj.len() }
    pub fn edge_count(&self) -> usize { self.adj.values().map(|v| v.len()).sum::<usize>() / if self.directed { 1 } else { 2 } }
    pub fn total_add_edge(&self) -> u64 { self.total_add_edge }
    pub fn total_traversals(&self) -> u64 { self.total_traversals }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs() {
        let mut g = SparseGraph::new(false);
        g.add_edge(0, 1); g.add_edge(0, 2); g.add_edge(1, 3);
        let order = g.bfs(0);
        assert_eq!(order[0], 0);
        assert!(order.contains(&3));
    }

    #[test]
    fn dfs() {
        let mut g = SparseGraph::new(false);
        g.add_edge(0, 1); g.add_edge(1, 2); g.add_edge(2, 3);
        let order = g.dfs(0);
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn has_path() {
        let mut g = SparseGraph::new(false);
        g.add_edge(0, 1); g.add_edge(1, 2);
        assert!(g.has_path(0, 2));
        assert!(!g.has_path(0, 5));
    }

    #[test]
    fn counts() {
        let mut g = SparseGraph::new(false);
        g.add_edge(0, 1); g.add_edge(1, 2);
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn isolated() {
        let mut g = SparseGraph::new(false);
        g.add_edge(0, 1);
        assert_eq!(g.bfs(5), vec![5]);
    }

    #[test]
    fn stats() {
        let mut g = SparseGraph::new(false);
        g.add_edge(0, 1); g.bfs(0);
        assert_eq!(g.total_add_edge(), 1);
        assert_eq!(g.total_traversals(), 1);
    }
}
