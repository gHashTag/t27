use std::collections::BTreeMap;

pub struct QuadTree {
    points: BTreeMap<(i64, i64), Vec<u8>>,
    total_inserts: u64,
    total_queries: u64,
}

impl QuadTree {
    pub fn new() -> Self { Self { points: BTreeMap::new(), total_inserts: 0, total_queries: 0 } }

    pub fn insert(&mut self, x: i64, y: i64, data: Vec<u8>) {
        self.total_inserts += 1;
        self.points.insert((x, y), data);
    }

    pub fn query_rect(&mut self, x1: i64, y1: i64, x2: i64, y2: i64) -> Vec<(i64, i64, &[u8])> {
        self.total_queries += 1;
        let (min_x, max_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        self.points.range((min_x, min_y)..=(max_x, max_y))
            .filter(|(&(x, y), _)| x >= min_x && x <= max_x && y >= min_y && y <= max_y)
            .map(|(&(x, y), v)| (x, y, v.as_slice()))
            .collect()
    }

    pub fn query_radius(&mut self, cx: i64, cy: i64, radius: i64) -> Vec<(i64, i64, &[u8])> {
        self.total_queries += 1;
        let r2 = (radius * radius) as i128;
        self.points.iter()
            .filter(|(&(x, y), _)| {
                let dx = (x - cx) as i128;
                let dy = (y - cy) as i128;
                dx * dx + dy * dy <= r2
            })
            .map(|(&(x, y), v)| (x, y, v.as_slice()))
            .collect()
    }

    pub fn nearest(&mut self, x: i64, y: i64) -> Option<(i64, i64, u64, &[u8])> {
        self.total_queries += 1;
        let mut best: Option<(i64, i64, u64, &Vec<u8>)> = None;
        for (&(px, py), v) in &self.points {
            let dx = (px - x).unsigned_abs() as u64;
            let dy = (py - y).unsigned_abs() as u64;
            let d2 = dx * dx + dy * dy;
            if best.is_none() || d2 < best.unwrap().2 { best = Some((px, py, d2, v)); }
        }
        best.map(|(px, py, d, v)| (px, py, d, v.as_slice()))
    }

    pub fn count(&self) -> usize { self.points.len() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_query_rect() {
        let mut qt = QuadTree::new();
        qt.insert(5, 5, b"a".to_vec());
        qt.insert(50, 50, b"b".to_vec());
        let r = qt.query_rect(0, 0, 10, 10);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].2, b"a");
    }

    #[test]
    fn query_radius() {
        let mut qt = QuadTree::new();
        qt.insert(0, 0, b"near".to_vec());
        qt.insert(100, 100, b"far".to_vec());
        let r = qt.query_radius(0, 0, 5);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn nearest() {
        let mut qt = QuadTree::new();
        qt.insert(10, 10, b"a".to_vec());
        qt.insert(100, 100, b"b".to_vec());
        let (x, y, _, v) = qt.nearest(9, 9).unwrap();
        assert_eq!(x, 10);
        assert_eq!(v, b"a");
    }

    #[test]
    fn empty_query() { let mut qt = QuadTree::new(); assert!(qt.query_rect(0, 0, 10, 10).is_empty()); }

    #[test]
    fn count() {
        let mut qt = QuadTree::new();
        qt.insert(0, 0, vec![]); qt.insert(1, 1, vec![]);
        assert_eq!(qt.count(), 2);
    }

    #[test]
    fn stats() {
        let mut qt = QuadTree::new();
        qt.insert(0, 0, vec![]); qt.query_rect(0, 0, 10, 10);
        assert_eq!(qt.total_inserts(), 1);
        assert_eq!(qt.total_queries(), 1);
    }
}
