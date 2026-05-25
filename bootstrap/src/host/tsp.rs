pub struct Tsp;

impl Tsp {
    pub fn nearest_neighbor(points: &[(f64, f64)]) -> (Vec<usize>, f64) {
        let n = points.len();
        if n <= 1 { return ((0..n).collect(), 0.0); }
        let mut best_tour = None;
        let mut best_cost = f64::MAX;
        let mut tour = vec![0usize; n];
        let mut visited = vec![false; n];
        for start in 0..n.min(5) {
            tour[0] = start;
            visited[start] = true;
            let mut cost = 0.0f64;
            for i in 1..n {
                let (cx, cy) = points[tour[i - 1]];
                let mut best_j = 0;
                let mut best_d = f64::MAX;
                for j in 0..n {
                    if visited[j] { continue; }
                    let d = (points[j].0 - cx).hypot(points[j].1 - cy);
                    if d < best_d { best_d = d; best_j = j; }
                }
                tour[i] = best_j;
                visited[best_j] = true;
                cost += best_d;
            }
            let (lx, ly) = points[tour[n - 1]];
            let (sx, sy) = points[tour[0]];
            cost += (lx - sx).hypot(ly - sy);
            if cost < best_cost { best_cost = cost; best_tour = Some(tour.clone()); }
            for v in &mut visited { *v = false; }
        }
        (best_tour.unwrap_or(tour), best_cost)
    }

    pub fn tour_cost(points: &[(f64, f64)], tour: &[usize]) -> f64 {
        if tour.len() <= 1 { return 0.0; }
        let mut cost = 0.0f64;
        for i in 0..tour.len() {
            let j = (i + 1) % tour.len();
            let (ax, ay) = points[tour[i]];
            let (bx, by) = points[tour[j]];
            cost += (ax - bx).hypot(ay - by);
        }
        cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_points() {
        let pts = vec![(0.0,0.0),(1.0,0.0),(0.5,1.0)];
        let (tour, cost) = Tsp::nearest_neighbor(&pts);
        assert_eq!(tour.len(), 3);
        assert!(cost > 0.0);
        assert!(cost < 5.0);
    }

    #[test]
    fn colinear() {
        let pts = vec![(0.0,0.0),(1.0,0.0),(2.0,0.0)];
        let (_, cost) = Tsp::nearest_neighbor(&pts);
        assert!((cost - 4.0).abs() < 1e-9);
    }

    #[test]
    fn single() { let (t, c) = Tsp::nearest_neighbor(&[(0.0,0.0)]); assert_eq!(t, vec![0]); assert_eq!(c, 0.0); }

    #[test]
    fn empty() { let (t, c) = Tsp::nearest_neighbor(&[]); assert!(t.is_empty()); assert_eq!(c, 0.0); }

    #[test]
    fn tour_cost_match() {
        let pts = vec![(0.0,0.0),(3.0,0.0),(3.0,4.0)];
        let (tour, cost) = Tsp::nearest_neighbor(&pts);
        assert!((Tsp::tour_cost(&pts, &tour) - cost).abs() < 1e-9);
    }
}
