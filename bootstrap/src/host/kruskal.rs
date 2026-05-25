#[derive(Debug, Clone)]
pub struct Edge { pub from: usize, pub to: usize, pub weight: u64 }

pub struct Kruskal {
    parent: Vec<usize>,
    rank: Vec<u8>,
    total_edges: u64,
}

impl Kruskal {
    pub fn new(n: usize) -> Self { Self { parent: (0..n).collect(), rank: vec![0; n], total_edges: 0 } }

    fn find(&mut self, x: usize) -> usize { if self.parent[x] != x { self.parent[x] = self.find(self.parent[x]); } self.parent[x] }

    fn union(&mut self, a: usize, b: usize) -> bool {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb { return false; }
        if self.rank[ra] < self.rank[rb] { self.parent[ra] = rb; }
        else if self.rank[ra] > self.rank[rb] { self.parent[rb] = ra; }
        else { self.parent[rb] = ra; self.rank[ra] += 1; }
        true
    }

    pub fn mst(&mut self, mut edges: Vec<Edge>) -> (Vec<Edge>, u64) {
        edges.sort_by_key(|e| e.weight);
        let mut result = Vec::new();
        for e in edges {
            if self.union(e.from, e.to) {
                self.total_edges += 1;
                result.push(e);
            }
        }
        let w: u64 = result.iter().map(|e| e.weight).sum();
        (result, w)
    }

    pub fn total_edges(&self) -> u64 { self.total_edges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle() {
        let mut k = Kruskal::new(3);
        let (mst, w) = k.mst(vec![Edge{from:0,to:1,weight:1}, Edge{from:1,to:2,weight:2}, Edge{from:0,to:2,weight:5}]);
        assert_eq!(mst.len(), 2); assert_eq!(w, 3);
    }

    #[test]
    fn line() {
        let mut k = Kruskal::new(3);
        let (_, w) = k.mst(vec![Edge{from:0,to:1,weight:10}, Edge{from:1,to:2,weight:20}]);
        assert_eq!(w, 30);
    }

    #[test]
    fn complete4() {
        let edges = vec![
            Edge{from:0,to:1,weight:4}, Edge{from:0,to:2,weight:1}, Edge{from:0,to:3,weight:3},
            Edge{from:1,to:2,weight:2}, Edge{from:1,to:3,weight:5}, Edge{from:2,to:3,weight:6},
        ];
        let mut k = Kruskal::new(4);
        let (mst, w) = k.mst(edges);
        assert_eq!(mst.len(), 3); assert_eq!(w, 6);
    }

    #[test]
    fn disconnected() {
        let mut k = Kruskal::new(4);
        let (mst, _) = k.mst(vec![Edge{from:0,to:1,weight:1}]);
        assert_eq!(mst.len(), 1);
    }

    #[test]
    fn single() {
        let mut k = Kruskal::new(1);
        let (mst, w) = k.mst(vec![]);
        assert!(mst.is_empty()); assert_eq!(w, 0);
    }
}
