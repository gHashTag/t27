use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum TqError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    CycleDetected,
}

impl std::fmt::Display for TqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TqError::NodeExists { id } => write!(f, "node {id} exists"),
            TqError::NodeNotFound { id } => write!(f, "node {id} not found"),
            TqError::CycleDetected => write!(f, "cycle detected"),
        }
    }
}

impl std::error::Error for TqError {}

struct Node {
    id: u64,
    deps: BTreeSet<u64>,
    dependents: BTreeSet<u64>,
}

pub struct TopoQ {
    nodes: BTreeMap<u64, Node>,
    total_added: u64,
    total_edges: u64,
    total_sorts: u64,
}

impl TopoQ {
    pub fn new() -> Self { Self { nodes: BTreeMap::new(), total_added: 0, total_edges: 0, total_sorts: 0 } }

    pub fn add_node(&mut self, id: u64) -> Result<(), TqError> {
        if self.nodes.contains_key(&id) { return Err(TqError::NodeExists { id }); }
        self.nodes.insert(id, Node { id, deps: BTreeSet::new(), dependents: BTreeSet::new() });
        self.total_added += 1;
        Ok(())
    }

    pub fn add_edge(&mut self, from: u64, to: u64) -> Result<(), TqError> {
        if !self.nodes.contains_key(&from) { return Err(TqError::NodeNotFound { id: from }); }
        if !self.nodes.contains_key(&to) { return Err(TqError::NodeNotFound { id: to }); }
        self.nodes.get_mut(&from).unwrap().deps.insert(to);
        self.nodes.get_mut(&to).unwrap().dependents.insert(from);
        self.total_edges += 1;
        Ok(())
    }

    pub fn sort(&mut self) -> Result<Vec<u64>, TqError> {
        self.total_sorts += 1;
        let mut in_degree: BTreeMap<u64, usize> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            in_degree.insert(id, node.deps.len());
        }
        let mut queue: VecDeque<u64> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();
        let mut result = Vec::new();
        while let Some(id) = queue.pop_front() {
            result.push(id);
            if let Some(node) = self.nodes.get(&id) {
                for &dep_id in &node.dependents {
                    if let Some(d) = in_degree.get_mut(&dep_id) {
                        *d -= 1;
                        if *d == 0 { queue.push_back(dep_id); }
                    }
                }
            }
        }
        if result.len() != self.nodes.len() { return Err(TqError::CycleDetected); }
        Ok(result)
    }

    pub fn sort_levels(&mut self) -> Result<Vec<Vec<u64>>, TqError> {
        self.total_sorts += 1;
        let mut in_degree: BTreeMap<u64, usize> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            in_degree.insert(id, node.deps.len());
        }
        let mut current: Vec<u64> = in_degree.iter()
            .filter(|(_, &d)| d == 0)
            .map(|(&id, _)| id)
            .collect();
        current.sort();
        let mut levels = Vec::new();
        while !current.is_empty() {
            levels.push(current.clone());
            let mut next = Vec::new();
            for &id in &current {
                if let Some(node) = self.nodes.get(&id) {
                    for &dep_id in &node.dependents {
                        if let Some(d) = in_degree.get_mut(&dep_id) {
                            *d -= 1;
                            if *d == 0 { next.push(dep_id); }
                        }
                    }
                }
            }
            next.sort();
            current = next;
        }
        let total: usize = levels.iter().map(|l| l.len()).sum();
        if total != self.nodes.len() { return Err(TqError::CycleDetected); }
        Ok(levels)
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn total_added(&self) -> u64 { self.total_added }
    pub fn total_edges(&self) -> u64 { self.total_edges }
}

impl Default for TopoQ {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() { assert!(TopoQ::new().node_count() == 0); }

    #[test]
    fn linear_order() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap(); tq.add_node(2).unwrap(); tq.add_node(3).unwrap();
        tq.add_edge(2, 1).unwrap(); tq.add_edge(3, 2).unwrap();
        let order = tq.sort().unwrap();
        assert_eq!(order, vec![1, 2, 3]);
    }

    #[test]
    fn diamond() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap(); tq.add_node(2).unwrap(); tq.add_node(3).unwrap(); tq.add_node(4).unwrap();
        tq.add_edge(2, 1).unwrap(); tq.add_edge(3, 1).unwrap();
        tq.add_edge(4, 2).unwrap(); tq.add_edge(4, 3).unwrap();
        let order = tq.sort().unwrap();
        assert_eq!(order[0], 1);
        assert_eq!(order[3], 4);
    }

    #[test]
    fn cycle() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap(); tq.add_node(2).unwrap();
        tq.add_edge(2, 1).unwrap(); tq.add_edge(1, 2).unwrap();
        let err = tq.sort().unwrap_err();
        assert!(matches!(err, TqError::CycleDetected));
    }

    #[test]
    fn levels() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap(); tq.add_node(2).unwrap(); tq.add_node(3).unwrap();
        tq.add_edge(2, 1).unwrap(); tq.add_edge(3, 1).unwrap();
        let levels = tq.sort_levels().unwrap();
        assert_eq!(levels[0], vec![1]);
        assert_eq!(levels[1].len(), 2);
    }

    #[test]
    fn duplicate_node() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap();
        let err = tq.add_node(1).unwrap_err();
        assert!(matches!(err, TqError::NodeExists { .. }));
    }

    #[test]
    fn edge_missing_node() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap();
        let err = tq.add_edge(1, 99).unwrap_err();
        assert!(matches!(err, TqError::NodeNotFound { .. }));
    }

    #[test]
    fn no_deps() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap(); tq.add_node(2).unwrap();
        let order = tq.sort().unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn stats() {
        let mut tq = TopoQ::new();
        tq.add_node(1).unwrap(); tq.add_node(2).unwrap();
        tq.add_edge(2, 1).unwrap();
        tq.sort().unwrap();
        assert_eq!(tq.total_added(), 2);
        assert_eq!(tq.total_edges(), 1);
    }

    #[test]
    fn error_display() { assert!(TqError::CycleDetected.to_string().contains("cycle")); }
}
