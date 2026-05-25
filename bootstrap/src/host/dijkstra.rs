use std::collections::{BinaryHeap, BTreeMap};
use std::cmp::Reverse;

pub struct Dijkstra {
    adj: BTreeMap<u64, Vec<(u64, u64)>>,
    total_relax: u64,
}

impl Dijkstra {
    pub fn new() -> Self { Self { adj: BTreeMap::new(), total_relax: 0 } }

    pub fn add_edge(&mut self, from: u64, to: u64, weight: u64) {
        self.adj.entry(from).or_default().push((to, weight));
        self.adj.entry(to).or_insert_with(Vec::new);
    }

    pub fn shortest_path(&mut self, src: u64) -> BTreeMap<u64, u64> {
        let mut dist: BTreeMap<u64, u64> = BTreeMap::new();
        let mut heap = BinaryHeap::new();
        dist.insert(src, 0);
        heap.push(Reverse((0, src)));
        while let Some(Reverse((d, u))) = heap.pop() {
            if d > *dist.get(&u).unwrap_or(&u64::MAX) { continue; }
            for &(v, w) in self.adj.get(&u).unwrap_or(&vec![]) {
                self.total_relax += 1;
                let nd = d + w;
                if nd < *dist.get(&v).unwrap_or(&u64::MAX) {
                    dist.insert(v, nd);
                    heap.push(Reverse((nd, v)));
                }
            }
        }
        dist
    }

    pub fn distance(&mut self, src: u64, dst: u64) -> Option<u64> {
        let d = self.shortest_path(src);
        d.get(&dst).copied().filter(|&v| v < u64::MAX)
    }

    pub fn total_relax(&self) -> u64 { self.total_relax }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut d = Dijkstra::new();
        d.add_edge(0, 1, 10); d.add_edge(1, 2, 20);
        assert_eq!(d.distance(0, 2), Some(30));
    }

    #[test]
    fn shorter_path() {
        let mut d = Dijkstra::new();
        d.add_edge(0, 2, 100); d.add_edge(0, 1, 10); d.add_edge(1, 2, 20);
        assert_eq!(d.distance(0, 2), Some(30));
    }

    #[test]
    fn disconnected() {
        let mut d = Dijkstra::new();
        d.add_edge(0, 1, 10); d.add_edge(2, 3, 5);
        assert_eq!(d.distance(0, 3), None);
    }

    #[test]
    fn self_loop() {
        let mut d = Dijkstra::new();
        d.add_edge(0, 0, 5);
        assert_eq!(d.distance(0, 0), Some(0));
    }

    #[test]
    fn all_nodes() {
        let mut d = Dijkstra::new();
        d.add_edge(0, 1, 1); d.add_edge(1, 2, 1); d.add_edge(0, 2, 5);
        let sp = d.shortest_path(0);
        assert_eq!(sp[&0], 0);
        assert_eq!(sp[&1], 1);
        assert_eq!(sp[&2], 2);
    }

    #[test]
    fn stats() {
        let mut d = Dijkstra::new();
        d.add_edge(0, 1, 10); d.shortest_path(0);
        assert!(d.total_relax() > 0);
    }
}
