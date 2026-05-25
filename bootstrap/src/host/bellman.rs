use std::collections::BTreeMap;

pub struct Bellman {
    edges: Vec<(u64, u64, i64)>,
    nodes: BTreeMap<u64, ()>,
}

impl Bellman {
    pub fn new() -> Self { Self { edges: Vec::new(), nodes: BTreeMap::new() } }

    pub fn add_edge(&mut self, from: u64, to: u64, weight: i64) {
        self.edges.push((from, to, weight));
        self.nodes.insert(from, ());
        self.nodes.insert(to, ());
    }

    pub fn shortest_path(&self, src: u64) -> (BTreeMap<u64, i64>, bool) {
        let n = self.nodes.len();
        let mut dist: BTreeMap<u64, i64> = BTreeMap::new();
        for &node in self.nodes.keys() { dist.insert(node, i64::MAX); }
        dist.insert(src, 0);
        for _ in 0..n {
            let mut updated = false;
            for &(u, v, w) in &self.edges {
                if dist[&u] != i64::MAX && dist[&u].saturating_add(w) < dist[&v] {
                    dist.insert(v, dist[&u].saturating_add(w));
                    updated = true;
                }
            }
            if !updated { break; }
        }
        let has_neg_cycle = {
            let mut found = false;
            for &(u, v, w) in &self.edges {
                if dist[&u] != i64::MAX && dist[&u].saturating_add(w) < dist[&v] { found = true; break; }
            }
            found
        };
        (dist, has_neg_cycle)
    }

    pub fn distance(&self, src: u64, dst: u64) -> Option<i64> {
        let (dist, neg) = self.shortest_path(src);
        if neg { return None; }
        dist.get(&dst).copied().filter(|&d| d != i64::MAX)
    }

    pub fn edge_count(&self) -> usize { self.edges.len() }
    pub fn node_count(&self) -> usize { self.nodes.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive() {
        let mut bf = Bellman::new();
        bf.add_edge(0, 1, 10); bf.add_edge(1, 2, 20);
        assert_eq!(bf.distance(0, 2), Some(30));
    }

    #[test]
    fn negative_edge() {
        let mut bf = Bellman::new();
        bf.add_edge(0, 1, 5); bf.add_edge(1, 2, -3);
        assert_eq!(bf.distance(0, 2), Some(2));
    }

    #[test]
    fn negative_cycle() {
        let mut bf = Bellman::new();
        bf.add_edge(0, 1, 1); bf.add_edge(1, 2, -1); bf.add_edge(2, 0, -1);
        assert_eq!(bf.distance(0, 2), None);
    }

    #[test]
    fn disconnected() {
        let mut bf = Bellman::new();
        bf.add_edge(0, 1, 5); bf.add_edge(2, 3, 5);
        assert_eq!(bf.distance(0, 3), None);
    }

    #[test]
    fn self_dist() {
        let mut bf = Bellman::new();
        bf.add_edge(0, 1, 5);
        assert_eq!(bf.distance(0, 0), Some(0));
    }

    #[test]
    fn stats() {
        let mut bf = Bellman::new();
        bf.add_edge(0, 1, 5); bf.add_edge(1, 2, 3);
        assert_eq!(bf.edge_count(), 2);
        assert_eq!(bf.node_count(), 3);
    }
}
