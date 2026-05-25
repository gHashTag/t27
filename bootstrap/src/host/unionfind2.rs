pub struct UnionFind2 {
    parent: Vec<usize>,
    rank: Vec<u32>,
    size: Vec<usize>,
    count: usize,
}

impl UnionFind2 {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            size: vec![1; n],
            count: n,
        }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb { return false; }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => {
                self.parent[ra] = rb;
                self.size[rb] += self.size[ra];
            }
            std::cmp::Ordering::Greater => {
                self.parent[rb] = ra;
                self.size[ra] += self.size[rb];
            }
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.size[ra] += self.size[rb];
                self.rank[ra] += 1;
            }
        }
        self.count -= 1;
        true
    }

    pub fn connected(&mut self, a: usize, b: usize) -> bool { self.find(a) == self.find(b) }

    pub fn component_size(&mut self, x: usize) -> usize { let r = self.find(x); self.size[r] }

    pub fn count(&self) -> usize { self.count }

    pub fn len(&self) -> usize { self.parent.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_union() {
        let mut uf = UnionFind2::new(5);
        assert!(uf.union(0, 1));
        assert!(uf.union(2, 3));
        assert!(uf.connected(0, 1));
        assert!(uf.connected(2, 3));
        assert!(!uf.connected(0, 2));
    }

    #[test]
    fn no_double_union() {
        let mut uf = UnionFind2::new(3);
        assert!(uf.union(0, 1));
        assert!(!uf.union(0, 1));
    }

    #[test]
    fn transitive() {
        let mut uf = UnionFind2::new(4);
        uf.union(0, 1); uf.union(1, 2); uf.union(2, 3);
        assert!(uf.connected(0, 3));
    }

    #[test]
    fn component_size() {
        let mut uf = UnionFind2::new(4);
        uf.union(0, 1); uf.union(1, 2);
        assert_eq!(uf.component_size(0), 3);
        assert_eq!(uf.component_size(3), 1);
    }

    #[test]
    fn count() {
        let mut uf = UnionFind2::new(5);
        assert_eq!(uf.count(), 5);
        uf.union(0, 1); uf.union(2, 3);
        assert_eq!(uf.count(), 3);
    }

    #[test]
    fn len() { assert_eq!(UnionFind2::new(10).len(), 10); }
}
