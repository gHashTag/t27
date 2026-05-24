use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum UfError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
}

impl std::fmt::Display for UfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UfError::NodeExists { id } => write!(f, "node {id} exists"),
            UfError::NodeNotFound { id } => write!(f, "node {id} not found"),
        }
    }
}

impl std::error::Error for UfError {}

struct Node {
    parent: u64,
    rank: u32,
    size: u64,
}

pub struct UnionFind {
    nodes: BTreeMap<u64, Node>,
    total_unions: u64,
}

impl UnionFind {
    pub fn new() -> Self { Self { nodes: BTreeMap::new(), total_unions: 0 } }

    pub fn add(&mut self, id: u64) -> Result<(), UfError> {
        if self.nodes.contains_key(&id) { return Err(UfError::NodeExists { id }); }
        self.nodes.insert(id, Node { parent: id, rank: 0, size: 1 });
        Ok(())
    }

    pub fn find(&mut self, id: u64) -> Result<u64, UfError> {
        if !self.nodes.contains_key(&id) { return Err(UfError::NodeNotFound { id }); }
        let root = self.find_root(id);
        self.compress(id, root);
        Ok(root)
    }

    fn find_root(&self, mut id: u64) -> u64 {
        while self.nodes[&id].parent != id { id = self.nodes[&id].parent; }
        id
    }

    fn compress(&mut self, mut id: u64, root: u64) {
        while self.nodes[&id].parent != id {
            let next = self.nodes[&id].parent;
            self.nodes.get_mut(&id).unwrap().parent = root;
            id = next;
        }
    }

    pub fn union(&mut self, a: u64, b: u64) -> Result<bool, UfError> {
        let root_a = self.find(a)?;
        let root_b = self.find(b)?;
        if root_a == root_b { return Ok(false); }
        let (rank_a, rank_b) = (self.nodes[&root_a].rank, self.nodes[&root_b].rank);
        let (new_root, child) = if rank_a > rank_b { (root_a, root_b) } else if rank_b > rank_a { (root_b, root_a) } else { (root_a, root_b) };
        self.nodes.get_mut(&child).unwrap().parent = new_root;
        self.nodes.get_mut(&new_root).unwrap().size += self.nodes[&child].size;
        if rank_a == rank_b { self.nodes.get_mut(&new_root).unwrap().rank += 1; }
        self.total_unions += 1;
        Ok(true)
    }

    pub fn connected(&mut self, a: u64, b: u64) -> Result<bool, UfError> {
        Ok(self.find(a)? == self.find(b)?)
    }

    pub fn size(&mut self, id: u64) -> Result<u64, UfError> {
        let root = self.find(id)?;
        Ok(self.nodes[&root].size)
    }

    pub fn component_count(&self) -> usize {
        self.nodes.iter().filter(|(&id, n)| n.parent == id).count()
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn total_unions(&self) -> u64 { self.total_unions }

    pub fn components(&mut self) -> Vec<Vec<u64>> {
        let mut groups: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        let ids: Vec<u64> = self.nodes.keys().copied().collect();
        for id in ids {
            let root = self.find(id).unwrap();
            groups.entry(root).or_default().push(id);
        }
        groups.into_values().collect()
    }
}

impl Default for UnionFind {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uf() {
        let uf = UnionFind::new();
        assert_eq!(uf.node_count(), 0);
    }

    #[test]
    fn add_nodes() {
        let mut uf = UnionFind::new();
        uf.add(1).unwrap(); uf.add(2).unwrap(); uf.add(3).unwrap();
        assert_eq!(uf.node_count(), 3);
        assert_eq!(uf.component_count(), 3);
    }

    #[test]
    fn duplicate_node() {
        let mut uf = UnionFind::new();
        uf.add(1).unwrap();
        let err = uf.add(1).unwrap_err();
        assert!(matches!(err, UfError::NodeExists { .. }));
    }

    #[test]
    fn union_find() {
        let mut uf = UnionFind::new();
        uf.add(1).unwrap(); uf.add(2).unwrap();
        assert!(uf.union(1, 2).unwrap());
        assert_eq!(uf.find(1).unwrap(), uf.find(2).unwrap());
    }

    #[test]
    fn already_connected() {
        let mut uf = UnionFind::new();
        uf.add(1).unwrap(); uf.add(2).unwrap();
        uf.union(1, 2).unwrap();
        assert!(!uf.union(1, 2).unwrap());
    }

    #[test]
    fn connected() {
        let mut uf = UnionFind::new();
        uf.add(1).unwrap(); uf.add(2).unwrap(); uf.add(3).unwrap();
        uf.union(1, 2).unwrap();
        assert!(uf.connected(1, 2).unwrap());
        assert!(!uf.connected(1, 3).unwrap());
    }

    #[test]
    fn transitive() {
        let mut uf = UnionFind::new();
        uf.add(1).unwrap(); uf.add(2).unwrap(); uf.add(3).unwrap();
        uf.union(1, 2).unwrap();
        uf.union(2, 3).unwrap();
        assert!(uf.connected(1, 3).unwrap());
        assert_eq!(uf.component_count(), 1);
    }

    #[test]
    fn size() {
        let mut uf = UnionFind::new();
        uf.add(1).unwrap(); uf.add(2).unwrap(); uf.add(3).unwrap();
        uf.union(1, 2).unwrap();
        assert_eq!(uf.size(1).unwrap(), 2);
        assert_eq!(uf.size(3).unwrap(), 1);
    }

    #[test]
    fn components() {
        let mut uf = UnionFind::new();
        for i in 1..=6 { uf.add(i).unwrap(); }
        uf.union(1, 2).unwrap();
        uf.union(3, 4).unwrap();
        uf.union(5, 6).unwrap();
        let comps = uf.components();
        assert_eq!(comps.len(), 3);
    }

    #[test]
    fn not_found() {
        let mut uf = UnionFind::new();
        let err = uf.find(99).unwrap_err();
        assert!(matches!(err, UfError::NodeNotFound { .. }));
    }

    #[test]
    fn stats() {
        let mut uf = UnionFind::new();
        for i in 1..=4 { uf.add(i).unwrap(); }
        uf.union(1, 2).unwrap();
        uf.union(3, 4).unwrap();
        assert_eq!(uf.total_unions(), 2);
    }

    #[test]
    fn error_display() {
        assert!(UfError::NodeNotFound { id: 5 }.to_string().contains("5"));
    }
}
