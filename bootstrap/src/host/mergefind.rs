pub struct MergeFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
    history: Vec<(usize, usize, u8)>,
    saved_sets: Vec<usize>,
    sets: usize,
    total_union: u64,
    total_find: u64,
}

impl MergeFind {
    pub fn new(n: usize) -> Self {
        Self { parent: (0..n).collect(), rank: vec![0; n], history: Vec::new(), saved_sets: Vec::new(), sets: n, total_union: 0, total_find: 0 }
    }

    pub fn find(&mut self, mut x: usize) -> usize {
        self.total_find += 1;
        while self.parent[x] != x { x = self.parent[x]; }
        x
    }

    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb { return false; }
        self.total_union += 1;
        self.saved_sets.push(self.sets);
        if self.rank[ra] < self.rank[rb] {
            self.history.push((ra, self.parent[ra], self.rank[ra]));
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.history.push((rb, self.parent[rb], self.rank[rb]));
            self.parent[rb] = ra;
        } else {
            self.history.push((rb, self.parent[rb], self.rank[rb]));
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
        self.sets -= 1;
        true
    }

    pub fn rollback(&mut self) -> bool {
        let (node, old_parent, old_rank) = match self.history.pop() {
            Some(h) => h,
            None => return false,
        };
        self.parent[node] = old_parent;
        self.rank[node] = old_rank;
        if let Some(s) = self.saved_sets.pop() { self.sets = s; }
        true
    }

    pub fn snapshot(&self) -> usize { self.saved_sets.len() }

    pub fn rollback_to(&mut self, snap: usize) {
        while self.history.len() > snap { self.rollback(); }
    }

    pub fn connected(&mut self, a: usize, b: usize) -> bool { self.find(a) == self.find(b) }
    pub fn num_sets(&self) -> usize { self.sets }
    pub fn len(&self) -> usize { self.parent.len() }
    pub fn total_union(&self) -> u64 { self.total_union }
    pub fn total_find(&self) -> u64 { self.total_find }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_find() {
        let mut mf = MergeFind::new(5);
        assert!(mf.union(0, 1));
        assert!(mf.connected(0, 1));
        assert_eq!(mf.num_sets(), 4);
    }

    #[test]
    fn rollback() {
        let mut mf = MergeFind::new(5);
        mf.union(0, 1);
        let snap = mf.snapshot();
        mf.union(1, 2);
        assert!(mf.connected(0, 2));
        mf.rollback_to(snap);
        assert!(!mf.connected(0, 2));
        assert!(mf.connected(0, 1));
    }

    #[test]
    fn full_rollback() {
        let mut mf = MergeFind::new(3);
        mf.union(0, 1); mf.union(1, 2);
        mf.rollback_to(0);
        assert_eq!(mf.num_sets(), 3);
    }

    #[test]
    fn empty_rollback() { assert!(!MergeFind::new(3).rollback()); }

    #[test]
    fn chain() {
        let mut mf = MergeFind::new(10);
        for i in 0..9 { mf.union(i, i + 1); }
        assert!(mf.connected(0, 9));
        assert_eq!(mf.num_sets(), 1);
    }

    #[test]
    fn stats() {
        let mut mf = MergeFind::new(3);
        mf.union(0, 1); mf.find(0);
        assert_eq!(mf.total_union(), 1);
        assert!(mf.total_find() >= 1);
    }
}
