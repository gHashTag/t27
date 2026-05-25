pub struct KdTree2 {
    points: Vec<(f64, f64, u64)>,
}

impl KdTree2 {
    pub fn new() -> Self { Self { points: Vec::new() } }

    pub fn insert(&mut self, x: f64, y: f64, id: u64) { self.points.push((x, y, id)); }

    pub fn nearest(&self, qx: f64, qy: f64) -> Option<(f64, f64, u64)> {
        if self.points.is_empty() { return None; }
        let mut best = None;
        let mut best_d = f64::MAX;
        for &(px, py, id) in &self.points {
            let d = (px - qx) * (px - qx) + (py - qy) * (py - qy);
            if d < best_d { best_d = d; best = Some((px, py, id)); }
        }
        best
    }

    pub fn within(&self, qx: f64, qy: f64, radius: f64) -> Vec<(f64, f64, u64)> {
        let r2 = radius * radius;
        self.points.iter().filter(|(px, py, _)| {
            let d = (px - qx) * (px - qx) + (py - qy) * (py - qy);
            d <= r2
        }).copied().collect()
    }

    pub fn knn(&self, qx: f64, qy: f64, k: usize) -> Vec<(f64, f64, u64, f64)> {
        let mut scored: Vec<_> = self.points.iter().map(|&(px, py, id)| {
            let d = (px - qx) * (px - qx) + (py - qy) * (py - qy);
            (px, py, id, d)
        }).collect();
        scored.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
        scored.into_iter().take(k).collect()
    }

    pub fn len(&self) -> usize { self.points.len() }
    pub fn is_empty(&self) -> bool { self.points.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest() {
        let mut kd = KdTree2::new();
        kd.insert(0.0, 0.0, 1); kd.insert(10.0, 10.0, 2);
        assert_eq!(kd.nearest(1.0, 1.0).unwrap().2, 1);
    }

    #[test]
    fn empty() { assert!(KdTree2::new().nearest(0.0, 0.0).is_none()); }

    #[test]
    fn within_radius() {
        let mut kd = KdTree2::new();
        kd.insert(0.0, 0.0, 1); kd.insert(5.0, 5.0, 2); kd.insert(100.0, 100.0, 3);
        let r = kd.within(0.0, 0.0, 10.0);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn knn() {
        let mut kd = KdTree2::new();
        kd.insert(0.0, 0.0, 1); kd.insert(1.0, 1.0, 2); kd.insert(10.0, 10.0, 3);
        let k2 = kd.knn(0.0, 0.0, 2);
        assert_eq!(k2.len(), 2);
        assert_eq!(k2[0].2, 1);
        assert_eq!(k2[1].2, 2);
    }

    #[test]
    fn single() {
        let mut kd = KdTree2::new();
        kd.insert(5.0, 5.0, 42);
        assert_eq!(kd.nearest(0.0, 0.0).unwrap().2, 42);
    }

    #[test]
    fn many_points() {
        let mut kd = KdTree2::new();
        for i in 0..100u64 { kd.insert(i as f64, i as f64, i); }
        assert_eq!(kd.len(), 100);
        assert_eq!(kd.nearest(50.5, 50.5).unwrap().2, 50);
    }
}
