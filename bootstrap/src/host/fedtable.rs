use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeHealth {
    Healthy,
    Suspect,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FedError {
    NodeExists { id: u64 },
    NodeNotFound { id: u64 },
    NoHealthyNodes,
}

impl std::fmt::Display for FedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FedError::NodeExists { id } => write!(f, "node {id} exists"),
            FedError::NodeNotFound { id } => write!(f, "node {id} not found"),
            FedError::NoHealthyNodes => write!(f, "no healthy nodes"),
        }
    }
}

impl std::error::Error for FedError {}

#[derive(Debug, Clone)]
struct FedNode {
    id: u64,
    health: NodeHealth,
    weight: u32,
    epoch: u64,
    total_heartbeats: u64,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: u64,
    pub health: NodeHealth,
    pub weight: u32,
    pub epoch: u64,
    pub total_heartbeats: u64,
}

#[derive(Debug, Clone)]
pub struct FederationTable {
    nodes: BTreeMap<u64, FedNode>,
    current_epoch: u64,
    total_heartbeats: u64,
    total_failures: u64,
}

impl FederationTable {
    pub fn new() -> Self {
        Self { nodes: BTreeMap::new(), current_epoch: 0, total_heartbeats: 0, total_failures: 0 }
    }

    pub fn join(&mut self, id: u64, weight: u32) -> Result<(), FedError> {
        if self.nodes.contains_key(&id) {
            return Err(FedError::NodeExists { id });
        }
        self.nodes.insert(id, FedNode {
            id, health: NodeHealth::Healthy, weight, epoch: self.current_epoch, total_heartbeats: 0,
        });
        Ok(())
    }

    pub fn leave(&mut self, id: u64) -> Result<NodeInfo, FedError> {
        let node = self.nodes.remove(&id).ok_or(FedError::NodeNotFound { id })?;
        Ok(NodeInfo { id: node.id, health: node.health, weight: node.weight, epoch: node.epoch, total_heartbeats: node.total_heartbeats })
    }

    pub fn heartbeat(&mut self, id: u64) -> Result<(), FedError> {
        let node = self.nodes.get_mut(&id).ok_or(FedError::NodeNotFound { id })?;
        node.total_heartbeats += 1;
        self.total_heartbeats += 1;
        if node.health == NodeHealth::Suspect {
            node.health = NodeHealth::Healthy;
        }
        Ok(())
    }

    pub fn mark_suspect(&mut self, id: u64) -> Result<(), FedError> {
        let node = self.nodes.get_mut(&id).ok_or(FedError::NodeNotFound { id })?;
        if node.health == NodeHealth::Healthy {
            node.health = NodeHealth::Suspect;
        }
        Ok(())
    }

    pub fn mark_failed(&mut self, id: u64) -> Result<(), FedError> {
        let node = self.nodes.get_mut(&id).ok_or(FedError::NodeNotFound { id })?;
        node.health = NodeHealth::Failed;
        self.total_failures += 1;
        Ok(())
    }

    pub fn health(&self, id: u64) -> Option<NodeHealth> {
        self.nodes.get(&id).map(|n| n.health)
    }

    pub fn weight(&self, id: u64) -> Option<u32> {
        self.nodes.get(&id).map(|n| n.weight)
    }

    pub fn node_info(&self, id: u64) -> Option<NodeInfo> {
        self.nodes.get(&id).map(|n| NodeInfo {
            id: n.id, health: n.health, weight: n.weight, epoch: n.epoch, total_heartbeats: n.total_heartbeats,
        })
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }

    pub fn healthy_count(&self) -> usize {
        self.nodes.values().filter(|n| n.health == NodeHealth::Healthy).count()
    }

    pub fn suspect_count(&self) -> usize {
        self.nodes.values().filter(|n| n.health == NodeHealth::Suspect).count()
    }

    pub fn failed_count(&self) -> usize {
        self.nodes.values().filter(|n| n.health == NodeHealth::Failed).count()
    }

    pub fn total_weight(&self) -> u64 {
        self.nodes.values().filter(|n| n.health != NodeHealth::Failed).map(|n| n.weight as u64).sum()
    }

    pub fn healthy_weight(&self) -> u64 {
        self.nodes.values().filter(|n| n.health == NodeHealth::Healthy).map(|n| n.weight as u64).sum()
    }

    pub fn node_ids(&self) -> Vec<u64> {
        self.nodes.keys().copied().collect()
    }

    pub fn pick_healthy(&self) -> Result<u64, FedError> {
        self.nodes.values()
            .filter(|n| n.health == NodeHealth::Healthy)
            .max_by_key(|n| n.weight)
            .map(|n| n.id)
            .ok_or(FedError::NoHealthyNodes)
    }

    pub fn advance_epoch(&mut self) -> u64 {
        self.current_epoch += 1;
        for node in self.nodes.values_mut() {
            node.epoch = self.current_epoch;
        }
        self.current_epoch
    }

    pub fn current_epoch(&self) -> u64 { self.current_epoch }
    pub fn total_heartbeats(&self) -> u64 { self.total_heartbeats }
    pub fn total_failures(&self) -> u64 { self.total_failures }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.current_epoch = 0;
    }
}

impl Default for FederationTable {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_table() {
        let ft = FederationTable::new();
        assert_eq!(ft.node_count(), 0);
    }

    #[test]
    fn join_and_count() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        ft.join(2, 200).unwrap();
        assert_eq!(ft.node_count(), 2);
        assert_eq!(ft.healthy_count(), 2);
    }

    #[test]
    fn duplicate_join() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        let err = ft.join(1, 200).unwrap_err();
        assert!(matches!(err, FedError::NodeExists { .. }));
    }

    #[test]
    fn leave() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        let info = ft.leave(1).unwrap();
        assert_eq!(info.weight, 100);
        assert_eq!(ft.node_count(), 0);
    }

    #[test]
    fn leave_not_found() {
        let mut ft = FederationTable::new();
        let err = ft.leave(99).unwrap_err();
        assert!(matches!(err, FedError::NodeNotFound { .. }));
    }

    #[test]
    fn heartbeat_recovers_suspect() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        ft.mark_suspect(1).unwrap();
        assert_eq!(ft.health(1), Some(NodeHealth::Suspect));
        ft.heartbeat(1).unwrap();
        assert_eq!(ft.health(1), Some(NodeHealth::Healthy));
    }

    #[test]
    fn mark_failed() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        ft.mark_failed(1).unwrap();
        assert_eq!(ft.failed_count(), 1);
        assert_eq!(ft.healthy_weight(), 0);
    }

    #[test]
    fn pick_healthy_highest_weight() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        ft.join(2, 300).unwrap();
        ft.join(3, 200).unwrap();
        assert_eq!(ft.pick_healthy().unwrap(), 2);
    }

    #[test]
    fn no_healthy_nodes() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        ft.mark_failed(1).unwrap();
        let err = ft.pick_healthy().unwrap_err();
        assert!(matches!(err, FedError::NoHealthyNodes));
    }

    #[test]
    fn advance_epoch() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        ft.advance_epoch();
        assert_eq!(ft.current_epoch(), 1);
        assert_eq!(ft.node_info(1).unwrap().epoch, 1);
    }

    #[test]
    fn weights() {
        let mut ft = FederationTable::new();
        ft.join(1, 100).unwrap();
        ft.join(2, 200).unwrap();
        ft.mark_failed(2);
        assert_eq!(ft.total_weight(), 100);
        assert_eq!(ft.healthy_weight(), 100);
    }

    #[test]
    fn error_display() {
        assert!(FedError::NodeNotFound { id: 5 }.to_string().contains("5"));
    }
}
