#[derive(Debug, Clone, PartialEq)]
pub enum PsError {
    IndexOutOfRange { idx: usize, len: usize },
}

impl std::fmt::Display for PsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PsError::IndexOutOfRange { idx, len } => write!(f, "index {idx} out of range (len={len})"),
        }
    }
}

impl std::error::Error for PsError {}

pub struct PartSet {
    parent: Vec<usize>,
    size: Vec<usize>,
    rank: Vec<usize>,
    count: usize,
    total_finds: u64,
    total_unions: u64,
}

impl PartSet {
    pub fn new(n: usize) -> Self {
        Self { parent: (0..n).collect(), size: vec![1; n], rank: vec![0; n], count: n, total_finds: 0, total_unions: 0 }
    }

    pub fn find(&mut self, mut x: usize) -> Result<usize, PsError> {
        self.total_finds += 1;
        if x >= self.parent.len() { return Err(PsError::IndexOutOfRange { idx: x, len: self.parent.len() }); }
        let root = loop {
            if self.parent[x] == x { break x; }
            x = self.parent[x];
        };
        let mut cur = x;
        while self.parent[cur] != root {
            let next = self.parent[cur];
            self.parent[cur] = root;
            cur = next;
        }
        Ok(root)
    }

    pub fn union(&mut self, a: usize, b: usize) -> Result<bool, PsError> {
        self.total_unions += 1;
        let ra = self.find(a)?;
        let rb = self.find(b)?;
        if ra == rb { return Ok(false); }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
            self.size[rb] += self.size[ra];
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
            self.size[ra] += self.size[rb];
        } else {
            self.parent[rb] = ra;
            self.size[ra] += self.size[rb];
            self.rank[ra] += 1;
        }
        self.count -= 1;
        Ok(true)
    }

    pub fn connected(&mut self, a: usize, b: usize) -> Result<bool, PsError> {
        Ok(self.find(a)? == self.find(b)?)
    }

    pub fn component_size(&mut self, x: usize) -> Result<usize, PsError> {
        let root = self.find(x)?;
        Ok(self.size[root])
    }

    pub fn components(&mut self) -> Vec<Vec<usize>> {
        let mut groups: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        for i in 0..self.parent.len() {
            let root = self.find(i).unwrap();
            groups.entry(root).or_default().push(i);
        }
        groups.into_values().collect()
    }

    pub fn len(&self) -> usize { self.parent.len() }
    pub fn count(&self) -> usize { self.count }
    pub fn total_finds(&self) -> u64 { self.total_finds }
    pub fn total_unions(&self) -> u64 { self.total_unions }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ps() { let ps = PartSet::new(5); assert_eq!(ps.count(), 5); }

    #[test]
    fn find_self() {
        let mut ps = PartSet::new(5);
        assert_eq!(ps.find(0).unwrap(), 0);
    }

    #[test]
    fn union_find() {
        let mut ps = PartSet::new(5);
        ps.union(0, 1).unwrap();
        assert_eq!(ps.find(0).unwrap(), ps.find(1).unwrap());
        assert_eq!(ps.count(), 4);
    }

    #[test]
    fn union_twice() {
        let mut ps = PartSet::new(3);
        assert!(ps.union(0, 1).unwrap());
        assert!(!ps.union(0, 1).unwrap());
    }

    #[test]
    fn connected() {
        let mut ps = PartSet::new(5);
        ps.union(0, 1).unwrap(); ps.union(2, 3).unwrap();
        assert!(ps.connected(0, 1).unwrap());
        assert!(!ps.connected(0, 2).unwrap());
    }

    #[test]
    fn component_size() {
        let mut ps = PartSet::new(5);
        ps.union(0, 1).unwrap(); ps.union(0, 2).unwrap();
        assert_eq!(ps.component_size(0).unwrap(), 3);
    }

    #[test]
    fn components() {
        let mut ps = PartSet::new(4);
        ps.union(0, 1).unwrap(); ps.union(2, 3).unwrap();
        let cc = ps.components();
        assert_eq!(cc.len(), 2);
    }

    #[test]
    fn out_of_range() {
        let mut ps = PartSet::new(3);
        assert!(ps.find(5).is_err());
    }

    #[test]
    fn path_compression() {
        let mut ps = PartSet::new(10);
        for i in 1..10 { ps.union(0, i).unwrap(); }
        let root = ps.find(9).unwrap();
        assert_eq!(ps.parent[9], root);
    }

    #[test]
    fn stats() {
        let mut ps = PartSet::new(3);
        ps.union(0, 1).unwrap(); ps.find(0).unwrap();
        assert_eq!(ps.total_unions(), 1);
        assert_eq!(ps.total_finds(), 3);
    }

    #[test]
    fn error_display() { assert!(PsError::IndexOutOfRange { idx: 5, len: 3 }.to_string().contains("5")); }
}
