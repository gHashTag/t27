use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug)]
struct OrderedF64(f64);

impl PartialEq for OrderedF64 { fn eq(&self, other: &Self) -> bool { self.0.to_bits() == other.0.to_bits() } }
impl Eq for OrderedF64 {}
impl PartialOrd for OrderedF64 { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }
impl Ord for OrderedF64 { fn cmp(&self, other: &Self) -> Ordering { self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal) } }

pub struct Dijkstra3;

impl Dijkstra3 {
    pub fn shortest_paths(adj: &[Vec<(usize, f64)>], src: usize) -> (Vec<f64>, Vec<Option<usize>>) {
        let n = adj.len();
        let mut dist = vec![f64::INFINITY; n];
        let mut prev = vec![None; n];
        dist[src] = 0.0;
        let mut heap = BinaryHeap::new();
        heap.push(std::cmp::Reverse((OrderedF64(0.0), src)));
        while let Some(std::cmp::Reverse((OrderedF64(d), u))) = heap.pop() {
            if d > dist[u] { continue; }
            for &(v, w) in &adj[u] {
                let nd = d + w;
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = Some(u);
                    heap.push(std::cmp::Reverse((OrderedF64(nd), v)));
                }
            }
        }
        (dist, prev)
    }

    pub fn reconstruct_path(prev: &[Option<usize>], target: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut cur = target;
        loop {
            path.push(cur);
            match prev[cur] {
                Some(p) => cur = p,
                None => break,
            }
        }
        path.reverse();
        path
    }

    pub fn distance_matrix(adj: &[Vec<(usize, f64)>]) -> Vec<Vec<f64>> {
        let n = adj.len();
        (0..n).map(|s| Self::shortest_paths(adj, s).0).collect()
    }

    pub fn graph_diameter(adj: &[Vec<(usize, f64)>]) -> f64 {
        let n = adj.len();
        (0..n).map(|s| {
            let (dist, _) = Self::shortest_paths(adj, s);
            dist.iter().filter(|&&d| d.is_finite()).fold(0.0f64, |a, &b| a.max(b))
        }).fold(0.0f64, |a, b| a.max(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> Vec<Vec<(usize, f64)>> {
        vec![
            vec![(1, 1.0), (2, 4.0)],
            vec![(0, 1.0), (2, 2.0), (3, 6.0)],
            vec![(0, 4.0), (1, 2.0), (3, 3.0)],
            vec![(1, 6.0), (2, 3.0)],
        ]
    }

    #[test]
    fn shortest() {
        let (dist, _) = Dijkstra3::shortest_paths(&graph(), 0);
        assert!((dist[0] - 0.0).abs() < 1e-9);
        assert!((dist[1] - 1.0).abs() < 1e-9);
        assert!((dist[2] - 3.0).abs() < 1e-9);
        assert!((dist[3] - 6.0).abs() < 1e-9);
    }

    #[test]
    fn path() {
        let (_, prev) = Dijkstra3::shortest_paths(&graph(), 0);
        let p = Dijkstra3::reconstruct_path(&prev, 3);
        assert_eq!(p[0], 0);
        assert_eq!(*p.last().unwrap(), 3);
    }

    #[test]
    fn distance_matrix() {
        let dm = Dijkstra3::distance_matrix(&graph());
        assert!((dm[0][3] - 6.0).abs() < 1e-9);
        assert!((dm[3][0] - 6.0).abs() < 1e-9);
    }

    #[test]
    fn diameter() { assert!((Dijkstra3::graph_diameter(&graph()) - 6.0).abs() < 1e-9); }

    #[test]
    fn single_node() {
        let g: Vec<Vec<(usize, f64)>> = vec![vec![]];
        let (d, _) = Dijkstra3::shortest_paths(&g, 0);
        assert_eq!(d[0], 0.0);
    }
}
