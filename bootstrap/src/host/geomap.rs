use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum GeoErr {
    OutOfBounds { x: f64, y: f64 },
}

impl std::fmt::Display for GeoErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeoErr::OutOfBounds { x, y } => write!(f, "({x},{y}) out of bounds"),
        }
    }
}

impl std::error::Error for GeoErr {}

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
    data: Vec<u8>,
}

pub struct GeoMap {
    cell_size: f64,
    grid: BTreeMap<(i64, i64), Vec<Point>>,
    total_inserts: u64,
    total_queries: u64,
}

impl GeoMap {
    pub fn new(cell_size: f64) -> Self { Self { cell_size: cell_size.max(0.001), grid: BTreeMap::new(), total_inserts: 0, total_queries: 0 } }

    fn cell(&self, x: f64, y: f64) -> (i64, i64) {
        ((x / self.cell_size).floor() as i64, (y / self.cell_size).floor() as i64)
    }

    pub fn insert(&mut self, x: f64, y: f64, data: Vec<u8>) {
        self.total_inserts += 1;
        let c = self.cell(x, y);
        self.grid.entry(c).or_default().push(Point { x, y, data });
    }

    pub fn query_radius(&mut self, cx: f64, cy: f64, radius: f64) -> Vec<(f64, f64, &[u8])> {
        self.total_queries += 1;
        let r2 = radius * radius;
        let (cx_cell, cy_cell) = self.cell(cx, cy);
        let span = (radius / self.cell_size).ceil() as i64 + 1;
        let mut result = Vec::new();
        for dx in -span..=span {
            for dy in -span..=span {
                let c = (cx_cell + dx, cy_cell + dy);
                if let Some(pts) = self.grid.get(&c) {
                    for p in pts {
                        let d2 = (p.x - cx) * (p.x - cx) + (p.y - cy) * (p.y - cy);
                        if d2 <= r2 { result.push((p.x, p.y, p.data.as_slice())); }
                    }
                }
            }
        }
        result
    }

    pub fn query_bbox(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> Vec<(f64, f64, &[u8])> {
        self.total_queries += 1;
        let (min_x, max_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        let (c1x, c1y) = self.cell(min_x, min_y);
        let (c2x, c2y) = self.cell(max_x, max_y);
        let mut result = Vec::new();
        for cx in c1x..=c2x {
            for cy in c1y..=c2y {
                if let Some(pts) = self.grid.get(&(cx, cy)) {
                    for p in pts {
                        if p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y {
                            result.push((p.x, p.y, p.data.as_slice()));
                        }
                    }
                }
            }
        }
        result
    }

    pub fn count(&self) -> usize { self.grid.values().map(|v| v.len()).sum() }
    pub fn cell_count(&self) -> usize { self.grid.len() }
    pub fn cell_size(&self) -> f64 { self.cell_size }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_count() {
        let mut gm = GeoMap::new(10.0);
        gm.insert(5.0, 5.0, b"a".to_vec());
        gm.insert(15.0, 15.0, b"b".to_vec());
        assert_eq!(gm.count(), 2);
    }

    #[test]
    fn radius_query() {
        let mut gm = GeoMap::new(10.0);
        gm.insert(0.0, 0.0, b"near".to_vec());
        gm.insert(100.0, 100.0, b"far".to_vec());
        let r = gm.query_radius(0.0, 0.0, 5.0);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].2, b"near");
    }

    #[test]
    fn radius_multiple() {
        let mut gm = GeoMap::new(10.0);
        gm.insert(0.0, 0.0, vec![]); gm.insert(1.0, 1.0, vec![]); gm.insert(50.0, 50.0, vec![]);
        let r = gm.query_radius(0.0, 0.0, 5.0);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn bbox_query() {
        let mut gm = GeoMap::new(10.0);
        gm.insert(5.0, 5.0, b"in".to_vec());
        gm.insert(50.0, 50.0, b"out".to_vec());
        let r = gm.query_bbox(0.0, 0.0, 10.0, 10.0);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn empty_query() {
        let mut gm = GeoMap::new(10.0);
        assert!(gm.query_radius(0.0, 0.0, 100.0).is_empty());
    }

    #[test]
    fn cell_grouping() {
        let mut gm = GeoMap::new(10.0);
        gm.insert(1.0, 1.0, vec![]); gm.insert(2.0, 2.0, vec![]);
        assert_eq!(gm.cell_count(), 1);
    }

    #[test]
    fn stats() {
        let mut gm = GeoMap::new(10.0);
        gm.insert(0.0, 0.0, vec![]);
        gm.query_radius(0.0, 0.0, 1.0);
        assert_eq!(gm.total_inserts(), 1);
        assert_eq!(gm.total_queries(), 1);
    }

    #[test]
    fn error_display() { assert!(GeoErr::OutOfBounds { x: 1.0, y: 2.0 }.to_string().contains("bounds")); }
}
