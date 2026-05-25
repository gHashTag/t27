pub struct DSet {
    parent: Vec<usize>,
    rank: Vec<u8>,
    size: Vec<usize>,
    sets: usize,
    total_union: u64,
    total_find: u64,
}

impl DSet {
    pub fn new(n: usize) -> Self {
        Self { parent: (0..n).collect(), rank: vec![0; n], size: vec![1; n], sets: n,
               total_union: 0, total_find: 0 }
    }

    pub fn find(&mut self, x: usize) -> usize {
        self.total_find += 1;
        if self.parent[x] != x { self.parent[x] = self.find(self.parent[x]); }
        self.parent[x]
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb { return false; }
        self.total_union += 1;
        let (big, small) = if self.rank[ra] >= self.rank[rb] { (ra, rb) } else { (rb, ra) };
        self.parent[small] = big;
        self.size[big] += self.size[small];
        if self.rank[ra] == self.rank[rb] { self.rank[big] += 1; }
        self.sets -= 1;
        true
    }

    pub fn connected(&mut self, a: usize, b: usize) -> bool { self.find(a) == self.find(b) }
    pub fn set_size(&mut self, x: usize) -> usize { let r = self.find(x); self.size[r] }
    pub fn num_sets(&self) -> usize { self.sets }
    pub fn len(&self) -> usize { self.parent.len() }
    pub fn total_union(&self) -> u64 { self.total_union }
    pub fn total_find(&self) -> u64 { self.total_find }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton() {
        let mut ds = DSet::new(5);
        assert_eq!(ds.num_sets(), 5);
        assert!(ds.connected(0, 0));
    }

    #[test]
    fn union_two() {
        let mut ds = DSet::new(5);
        assert!(ds.union(0, 1));
        assert!(ds.connected(0, 1));
        assert_eq!(ds.num_sets(), 4);
    }

    #[test]
    fn union_same() {
        let mut ds = DSet::new(3);
        ds.union(0, 1);
        assert!(!ds.union(0, 1));
    }

    #[test]
    fn chain() {
        let mut ds = DSet::new(10);
        for i in 0..9 { ds.union(i, i + 1); }
        assert!(ds.connected(0, 9));
        assert_eq!(ds.num_sets(), 1);
        assert_eq!(ds.set_size(0), 10);
    }

    #[test]
    fn set_size() {
        let mut ds = DSet::new(4);
        ds.union(0, 1); ds.union(2, 3);
        assert_eq!(ds.set_size(0), 2);
        assert_eq!(ds.set_size(2), 2);
    }

    #[test]
    fn stats() {
        let mut ds = DSet::new(3);
        ds.union(0, 1); ds.find(0);
        assert_eq!(ds.total_union(), 1);
        assert!(ds.total_find() >= 2);
    }
}
