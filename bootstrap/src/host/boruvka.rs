pub struct Boruvka {
    parent: Vec<usize>,
    rank: Vec<u8>,
    total_edges: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: u64,
}

impl Boruvka {
    pub fn new(n: usize) -> Self { Self { parent: (0..n).collect(), rank: vec![0; n], total_edges: 0 } }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x { self.parent[x] = self.find(self.parent[x]); }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb { return false; }
        if self.rank[ra] < self.rank[rb] { self.parent[ra] = rb; }
        else if self.rank[ra] > self.rank[rb] { self.parent[rb] = ra; }
        else { self.parent[rb] = ra; self.rank[ra] += 1; }
        true
    }

    pub fn mst(&mut self, n: usize, edges: &[Edge]) -> (Vec<Edge>, u64) {
        let mut result = Vec::new();
        let mut components = n;
        while components > 1 {
            let mut cheapest: Vec<Option<usize>> = vec![None; n];
            for (i, e) in edges.iter().enumerate() {
                let set_u = self.find(e.from);
                let set_v = self.find(e.to);
                if set_u == set_v { continue; }
                for &s in &[set_u, set_v] {
                    match cheapest[s] {
                        None => cheapest[s] = Some(i),
                        Some(ci) => if edges[ci].weight > e.weight { cheapest[s] = Some(i); }
                    }
                }
            }
            let mut merged = false;
            for i in 0..n {
                if let Some(ei) = cheapest[i] {
                    let e = &edges[ei];
                    if self.union(e.from, e.to) {
                        result.push(e.clone());
                        components -= 1;
                        merged = true;
                    }
                }
            }
            if !merged { break; }
        }
        self.total_edges = result.len() as u64;
        let total_weight: u64 = result.iter().map(|e| e.weight).sum();
        (result, total_weight)
    }

    pub fn total_edges(&self) -> u64 { self.total_edges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle() {
        let edges = vec![Edge { from: 0, to: 1, weight: 1 }, Edge { from: 1, to: 2, weight: 2 }, Edge { from: 0, to: 2, weight: 5 }];
        let mut b = Boruvka::new(3);
        let (mst, w) = b.mst(3, &edges);
        assert_eq!(mst.len(), 2);
        assert_eq!(w, 3);
    }

    #[test]
    fn line() {
        let edges = vec![Edge { from: 0, to: 1, weight: 10 }, Edge { from: 1, to: 2, weight: 20 }];
        let mut b = Boruvka::new(3);
        let (mst, w) = b.mst(3, &edges);
        assert_eq!(mst.len(), 2);
        assert_eq!(w, 30);
    }

    #[test]
    fn disconnected() {
        let edges = vec![Edge { from: 0, to: 1, weight: 1 }];
        let mut b = Boruvka::new(4);
        let (mst, _) = b.mst(4, &edges);
        assert_eq!(mst.len(), 1);
    }

    #[test]
    fn single_node() {
        let mut b = Boruvka::new(1);
        let (mst, w) = b.mst(1, &[]);
        assert!(mst.is_empty());
        assert_eq!(w, 0);
    }

    #[test]
    fn complete4() {
        let edges = vec![
            Edge { from: 0, to: 1, weight: 4 }, Edge { from: 0, to: 2, weight: 1 }, Edge { from: 0, to: 3, weight: 3 },
            Edge { from: 1, to: 2, weight: 2 }, Edge { from: 1, to: 3, weight: 5 }, Edge { from: 2, to: 3, weight: 6 },
        ];
        let mut b = Boruvka::new(4);
        let (mst, w) = b.mst(4, &edges);
        assert_eq!(mst.len(), 3);
        assert_eq!(w, 6);
    }
}
