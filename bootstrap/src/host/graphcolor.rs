pub struct GraphColor;

impl GraphColor {
    pub fn color_greedy(adj: &[Vec<usize>]) -> Vec<usize> {
        let n = adj.len();
        if n == 0 { return Vec::new(); }
        let mut colors = vec![0usize; n];
        let mut assigned = vec![false; n];
        for v in 0..n {
            let mut used = vec![false; n + 1];
            for &u in &adj[v] {
                if assigned[u] { used[colors[u]] = true; }
            }
            let mut c = 0;
            while used[c] { c += 1; }
            colors[v] = c;
            assigned[v] = true;
        }
        colors
    }

    pub fn num_colors(colors: &[usize]) -> usize {
        if colors.is_empty() { return 0; }
        colors.iter().max().map(|&m| m + 1).unwrap_or(0)
    }

    pub fn is_valid(adj: &[Vec<usize>], colors: &[usize]) -> bool {
        for (v, neighbors) in adj.iter().enumerate() {
            for &u in neighbors {
                if v < u && colors[v] == colors[u] { return false; }
            }
        }
        true
    }

    pub fn bipartite_check(adj: &[Vec<usize>]) -> Option<Vec<usize>> {
        let n = adj.len();
        let mut colors = vec![2usize; n];
        for start in 0..n {
            if colors[start] != 2 { continue; }
            colors[start] = 0;
            let mut stack = vec![start];
            while let Some(v) = stack.pop() {
                for &u in &adj[v] {
                    if colors[u] == 2 {
                        colors[u] = 1 - colors[v];
                        stack.push(u);
                    } else if colors[u] == colors[v] {
                        return None;
                    }
                }
            }
        }
        Some(colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle() {
        let adj = vec![vec![1,2], vec![0,2], vec![0,1]];
        let c = GraphColor::color_greedy(&adj);
        assert!(GraphColor::is_valid(&adj, &c));
        assert!(GraphColor::num_colors(&c) <= 3);
    }

    #[test]
    fn bipartite() {
        let adj = vec![vec![1], vec![0,2], vec![1]];
        let c = GraphColor::color_greedy(&adj);
        assert!(GraphColor::is_valid(&adj, &c));
        assert!(GraphColor::num_colors(&c) <= 2);
    }

    #[test]
    fn bipartite_check_yes() {
        let adj = vec![vec![1,3], vec![0,2], vec![1,3], vec![0,2]];
        assert!(GraphColor::bipartite_check(&adj).is_some());
    }

    #[test]
    fn bipartite_check_no() {
        let adj = vec![vec![1,2], vec![0,2], vec![0,1]];
        assert!(GraphColor::bipartite_check(&adj).is_none());
    }

    #[test]
    fn empty() {
        assert!(GraphColor::color_greedy(&[]).is_empty());
        assert_eq!(GraphColor::num_colors(&[]), 0);
    }

    #[test]
    fn isolated() {
        let adj = vec![vec![], vec![], vec![]];
        let c = GraphColor::color_greedy(&adj);
        assert!(GraphColor::is_valid(&adj, &c));
        assert_eq!(GraphColor::num_colors(&c), 1);
    }
}
