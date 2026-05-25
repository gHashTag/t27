use std::collections::{BinaryHeap, BTreeMap};
use std::cmp::Reverse;

pub struct Dijkstra2 {
    adj: BTreeMap<u64, Vec<(u64, u64)>>,
    total_edges: u64,
}

impl Dijkstra2 {
    pub fn new() -> Self { Self { adj: BTreeMap::new(), total_edges: 0 } }

    pub fn add_edge(&mut self, from: u64, to: u64, w: u64) {
        self.total_edges += 1;
        self.adj.entry(from).or_default().push((to, w));
        self.adj.entry(to).or_insert_with(Vec::new);
    }

    pub fn shortest_path(&mut self, src: u64, dst: u64) -> Option<(u64, Vec<u64>)> {
        let mut dist: BTreeMap<u64, u64> = BTreeMap::new();
        let mut prev: BTreeMap<u64, u64> = BTreeMap::new();
        let mut heap = BinaryHeap::new();
        dist.insert(src, 0);
        heap.push(Reverse((0, src)));
        while let Some(Reverse((d, u))) = heap.pop() {
            if u == dst { break; }
            if d > *dist.get(&u).unwrap_or(&u64::MAX) { continue; }
            for &(v, w) in self.adj.get(&u).unwrap_or(&vec![]) {
                let nd = d + w;
                if nd < *dist.get(&v).unwrap_or(&u64::MAX) {
                    dist.insert(v, nd);
                    prev.insert(v, u);
                    heap.push(Reverse((nd, v)));
                }
            }
        }
        let cost = dist.get(&dst).copied()?;
        let mut path = vec![dst];
        let mut cur = dst;
        while let Some(&p) = prev.get(&cur) { path.push(p); cur = p; }
        path.reverse();
        Some((cost, path))
    }

    pub fn total_edges(&self) -> u64 { self.total_edges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut d = Dijkstra2::new();
        d.add_edge(0, 1, 10); d.add_edge(1, 2, 20);
        let (cost, path) = d.shortest_path(0, 2).unwrap();
        assert_eq!(cost, 30);
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn shorter() {
        let mut d = Dijkstra2::new();
        d.add_edge(0, 2, 100); d.add_edge(0, 1, 10); d.add_edge(1, 2, 20);
        let (cost, path) = d.shortest_path(0, 2).unwrap();
        assert_eq!(cost, 30);
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn disconnected() { let mut d = Dijkstra2::new(); d.add_edge(0, 1, 5); assert!(d.shortest_path(0, 3).is_none()); }

    #[test]
    fn same_node() {
        let mut d = Dijkstra2::new();
        d.add_edge(0, 1, 5);
        let (cost, path) = d.shortest_path(0, 0).unwrap();
        assert_eq!(cost, 0); assert_eq!(path, vec![0]);
    }

    #[test]
    fn diamond() {
        let mut d = Dijkstra2::new();
        d.add_edge(0, 1, 1); d.add_edge(0, 2, 3); d.add_edge(1, 3, 4); d.add_edge(2, 3, 1);
        let (cost, path) = d.shortest_path(0, 3).unwrap();
        assert_eq!(cost, 4);
        assert_eq!(path, vec![0, 2, 3]);
    }

    #[test]
    fn stats() {
        let mut d = Dijkstra2::new();
        d.add_edge(0, 1, 5); d.add_edge(1, 2, 3);
        assert_eq!(d.total_edges(), 2);
    }
}
