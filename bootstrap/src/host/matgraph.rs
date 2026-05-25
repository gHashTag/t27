#[derive(Debug, Clone, PartialEq)]
pub enum MgError {
    NodeOutOfRange { node: usize, n: usize },
}

impl std::fmt::Display for MgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MgError::NodeOutOfRange { node, n } => write!(f, "node {node} out of range (n={n})"),
        }
    }
}

impl std::error::Error for MgError {}

pub struct MatGraph {
    adj: Vec<Vec<i64>>,
    n: usize,
    total_adds: u64,
    total_removes: u64,
    total_queries: u64,
}

impl MatGraph {
    const INF: i64 = i64::MAX / 2;

    pub fn new(n: usize) -> Self {
        let mut adj = vec![vec![Self::INF; n]; n];
        for i in 0..n { adj[i][i] = 0; }
        Self { adj, n, total_adds: 0, total_removes: 0, total_queries: 0 }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: i64) -> Result<(), MgError> {
        if from >= self.n || to >= self.n { return Err(MgError::NodeOutOfRange { node: from.max(to), n: self.n }); }
        self.total_adds += 1;
        self.adj[from][to] = weight;
        Ok(())
    }

    pub fn remove_edge(&mut self, from: usize, to: usize) -> Result<(), MgError> {
        if from >= self.n || to >= self.n { return Err(MgError::NodeOutOfRange { node: from.max(to), n: self.n }); }
        self.total_removes += 1;
        self.adj[from][to] = Self::INF;
        Ok(())
    }

    pub fn has_edge(&mut self, from: usize, to: usize) -> bool {
        self.total_queries += 1;
        from < self.n && to < self.n && self.adj[from][to] < Self::INF
    }

    pub fn weight(&mut self, from: usize, to: usize) -> Option<i64> {
        self.total_queries += 1;
        if from >= self.n || to >= self.n { return None; }
        let w = self.adj[from][to];
        if w < Self::INF { Some(w) } else { None }
    }

    pub fn floyd_warshall(&mut self) -> Vec<Vec<i64>> {
        self.total_queries += 1;
        let mut dist = self.adj.clone();
        for k in 0..self.n {
            for i in 0..self.n {
                for j in 0..self.n {
                    let via = dist[i][k].saturating_add(dist[k][j]);
                    if via < dist[i][j] { dist[i][j] = via; }
                }
            }
        }
        dist
    }

    pub fn connected_components(&mut self) -> Vec<Vec<usize>> {
        self.total_queries += 1;
        let mut parent: Vec<usize> = (0..self.n).collect();
        fn find(parent: &mut Vec<usize>, x: usize) -> usize {
            if parent[x] != x { parent[x] = find(parent, parent[x]); }
            parent[x]
        }
        for i in 0..self.n {
            for j in 0..self.n {
                if self.adj[i][j] < Self::INF {
                    let ri = find(&mut parent, i);
                    let rj = find(&mut parent, j);
                    if ri != rj { parent[ri] = rj; }
                }
            }
        }
        let mut groups: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        for i in 0..self.n { groups.entry(find(&mut parent, i)).or_default().push(i); }
        groups.into_values().collect()
    }

    pub fn degree(&mut self, node: usize) -> (usize, usize) {
        self.total_queries += 1;
        let out_deg = self.adj[node].iter().filter(|&&w| w < Self::INF && w != 0).count();
        let in_deg = (0..self.n).filter(|&i| self.adj[i][node] < Self::INF && self.adj[i][node] != 0).count();
        (in_deg, out_deg)
    }

    pub fn neighbors(&mut self, node: usize) -> Vec<(usize, i64)> {
        self.total_queries += 1;
        let mut result = Vec::new();
        for j in 0..self.n {
            if self.adj[node][j] < Self::INF && (node != j || self.adj[node][j] != 0) {
                result.push((j, self.adj[node][j]));
            }
        }
        result
    }

    pub fn node_count(&self) -> usize { self.n }
    pub fn total_adds(&self) -> u64 { self.total_adds }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mg() { let g = MatGraph::new(3); assert_eq!(g.node_count(), 3); }

    #[test]
    fn add_has_edge() {
        let mut g = MatGraph::new(3);
        g.add_edge(0, 1, 5).unwrap();
        assert!(g.has_edge(0, 1));
        assert!(!g.has_edge(1, 0));
    }

    #[test]
    fn weight() {
        let mut g = MatGraph::new(3);
        g.add_edge(0, 1, 42).unwrap();
        assert_eq!(g.weight(0, 1), Some(42));
    }

    #[test]
    fn remove_edge() {
        let mut g = MatGraph::new(3);
        g.add_edge(0, 1, 5).unwrap(); g.remove_edge(0, 1).unwrap();
        assert!(!g.has_edge(0, 1));
    }

    #[test]
    fn floyd_warshall() {
        let mut g = MatGraph::new(3);
        g.add_edge(0, 1, 1).unwrap(); g.add_edge(1, 2, 2).unwrap(); g.add_edge(0, 2, 10).unwrap();
        let dist = g.floyd_warshall();
        assert_eq!(dist[0][2], 3);
    }

    #[test]
    fn components() {
        let mut g = MatGraph::new(4);
        g.add_edge(0, 1, 1).unwrap(); g.add_edge(2, 3, 1).unwrap();
        let cc = g.connected_components();
        assert_eq!(cc.len(), 2);
    }

    #[test]
    fn degree() {
        let mut g = MatGraph::new(3);
        g.add_edge(0, 1, 1).unwrap(); g.add_edge(0, 2, 1).unwrap(); g.add_edge(1, 0, 1).unwrap();
        let (in_deg, out_deg) = g.degree(0);
        assert_eq!(in_deg, 1);
        assert_eq!(out_deg, 2);
    }

    #[test]
    fn neighbors() {
        let mut g = MatGraph::new(3);
        g.add_edge(0, 1, 3).unwrap(); g.add_edge(0, 2, 7).unwrap();
        let nb = g.neighbors(0);
        assert_eq!(nb.len(), 2);
    }

    #[test]
    fn out_of_range() {
        let mut g = MatGraph::new(2);
        assert!(g.add_edge(5, 0, 1).is_err());
    }

    #[test]
    fn error_display() { assert!(MgError::NodeOutOfRange { node: 5, n: 3 }.to_string().contains("5")); }
}
