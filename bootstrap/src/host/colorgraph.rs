use std::collections::BTreeMap;

pub struct ColorGraph {
    adj: Vec<Vec<usize>>,
    n: usize,
    total_colors_used: usize,
}

impl ColorGraph {
    pub fn new(n: usize) -> Self { Self { adj: vec![Vec::new(); n], n, total_colors_used: 0 } }

    pub fn add_edge(&mut self, a: usize, b: usize) {
        if a < self.n && b < self.n { self.adj[a].push(b); self.adj[b].push(a); }
    }

    pub fn greedy_color(&mut self, order: &[usize]) -> Vec<usize> {
        let mut colors = vec![0usize; self.n];
        for (i, &node) in order.iter().enumerate() {
            let mut used = vec![false; self.n];
            for &nbr in &self.adj[node] {
                if colors[nbr] > 0 && colors[nbr] - 1 < self.n { used[colors[nbr] - 1] = true; }
            }
            let mut c = 0;
            while c < self.n && used[c] { c += 1; }
            colors[node] = c + 1;
        }
        self.total_colors_used = *colors.iter().max().unwrap_or(&0);
        colors
    }

    pub fn greedy_natural(&mut self) -> Vec<usize> {
        let order: Vec<usize> = (0..self.n).collect();
        self.greedy_color(&order)
    }

    pub fn greedy_largest_degree(&mut self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.n).collect();
        order.sort_by(|&a, &b| self.adj[b].len().cmp(&self.adj[a].len()));
        self.greedy_color(&order)
    }

    pub fn dsatur(&mut self) -> Vec<usize> {
        let mut colors = vec![0usize; self.n];
        let mut sat: Vec<usize> = vec![0; self.n];
        let mut colored = vec![false; self.n];
        for _ in 0..self.n {
            let best = (0..self.n)
                .filter(|&i| !colored[i])
                .max_by(|&a, &b| {
                    sat[a].cmp(&sat[b])
                        .then_with(|| self.adj[b].len().cmp(&self.adj[a].len()))
                })
                .unwrap();
            let mut used = vec![false; self.n];
            for &nbr in &self.adj[best] {
                if colors[nbr] > 0 { used[colors[nbr] - 1] = true; }
            }
            let mut c = 0;
            while c < self.n && used[c] { c += 1; }
            colors[best] = c + 1;
            colored[best] = true;
            for &nbr in &self.adj[best] {
                if !colored[nbr] {
                    let mut neighbor_colors = std::collections::HashSet::new();
                    for &nn in &self.adj[nbr] {
                        if colors[nn] > 0 { neighbor_colors.insert(colors[nn]); }
                    }
                    sat[nbr] = neighbor_colors.len();
                }
            }
        }
        self.total_colors_used = *colors.iter().max().unwrap_or(&0);
        colors
    }

    pub fn is_valid(&self, colors: &[usize]) -> bool {
        for node in 0..self.n {
            for &nbr in &self.adj[node] {
                if colors[node] == colors[nbr] { return false; }
            }
        }
        true
    }

    pub fn chromatic_lower_bound(&self) -> usize {
        let mut max_deg = 0;
        for node in 0..self.n {
            let mut neighbors = std::collections::HashSet::new();
            for &nbr in &self.adj[node] { neighbors.insert(nbr); }
            if neighbors.len() > max_deg { max_deg = neighbors.len(); }
        }
        if max_deg == 0 { if self.n > 0 { 1 } else { 0 } } else { max_deg + 1 }
    }

    pub fn node_count(&self) -> usize { self.n }
    pub fn edge_count(&self) -> usize { self.adj.iter().map(|v| v.len()).sum::<usize>() / 2 }
    pub fn colors_used(&self) -> usize { self.total_colors_used }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph() { let g = ColorGraph::new(0); assert_eq!(g.node_count(), 0); }

    #[test]
    fn no_edges() {
        let mut g = ColorGraph::new(5);
        let colors = g.greedy_natural();
        assert!(g.is_valid(&colors));
        assert_eq!(g.colors_used(), 1);
    }

    #[test]
    fn triangle() {
        let mut g = ColorGraph::new(3);
        g.add_edge(0, 1); g.add_edge(1, 2); g.add_edge(0, 2);
        let colors = g.greedy_natural();
        assert!(g.is_valid(&colors));
        assert_eq!(g.colors_used(), 3);
    }

    #[test]
    fn bipartite() {
        let mut g = ColorGraph::new(4);
        g.add_edge(0, 2); g.add_edge(0, 3); g.add_edge(1, 2); g.add_edge(1, 3);
        let colors = g.greedy_natural();
        assert!(g.is_valid(&colors));
        assert!(g.colors_used() <= 2);
    }

    #[test]
    fn dsatur_triangle() {
        let mut g = ColorGraph::new(3);
        g.add_edge(0, 1); g.add_edge(1, 2); g.add_edge(0, 2);
        let colors = g.dsatur();
        assert!(g.is_valid(&colors));
        assert_eq!(g.colors_used(), 3);
    }

    #[test]
    fn largest_degree() {
        let mut g = ColorGraph::new(5);
        g.add_edge(0, 1); g.add_edge(0, 2); g.add_edge(0, 3); g.add_edge(0, 4);
        g.add_edge(1, 2);
        let colors = g.greedy_largest_degree();
        assert!(g.is_valid(&colors));
    }

    #[test]
    fn valid_check() {
        let mut g = ColorGraph::new(3);
        g.add_edge(0, 1);
        assert!(g.is_valid(&[1, 2, 1]));
        assert!(!g.is_valid(&[1, 1, 1]));
    }

    #[test]
    fn lower_bound() {
        let mut g = ColorGraph::new(3);
        g.add_edge(0, 1); g.add_edge(1, 2); g.add_edge(0, 2);
        assert!(g.chromatic_lower_bound() >= 3);
    }

    #[test]
    fn edge_count() {
        let mut g = ColorGraph::new(4);
        g.add_edge(0, 1); g.add_edge(2, 3);
        assert_eq!(g.edge_count(), 2);
    }
}
