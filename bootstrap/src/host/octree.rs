use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } }
    pub fn distance_sq(&self, other: &Point3) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OctError {
    PointExists { id: u64 },
    PointNotFound { id: u64 },
    CapacityExceeded { id: u64 },
}

impl std::fmt::Display for OctError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OctError::PointExists { id } => write!(f, "point {id} exists"),
            OctError::PointNotFound { id } => write!(f, "point {id} not found"),
            OctError::CapacityExceeded { id } => write!(f, "capacity exceeded inserting {id}"),
        }
    }
}

impl std::error::Error for OctError {}

struct Entry {
    id: u64,
    pos: Point3,
    data: Vec<u8>,
}

pub struct Octree {
    entries: BTreeMap<u64, Entry>,
    total_inserts: u64,
    total_removes: u64,
    total_queries: u64,
}

impl Octree {
    pub fn new() -> Self { Self { entries: BTreeMap::new(), total_inserts: 0, total_removes: 0, total_queries: 0 } }

    pub fn insert(&mut self, id: u64, pos: Point3, data: Vec<u8>) -> Result<(), OctError> {
        if self.entries.contains_key(&id) { return Err(OctError::PointExists { id }); }
        self.entries.insert(id, Entry { id, pos, data });
        self.total_inserts += 1;
        Ok(())
    }

    pub fn remove(&mut self, id: u64) -> Result<(Point3, Vec<u8>), OctError> {
        let e = self.entries.remove(&id).ok_or(OctError::PointNotFound { id })?;
        self.total_removes += 1;
        Ok((e.pos, e.data))
    }

    pub fn get(&self, id: u64) -> Option<(Point3, &[u8])> {
        self.entries.get(&id).map(|e| (e.pos, e.data.as_slice()))
    }

    pub fn radius_query(&mut self, center: &Point3, radius: f64) -> Vec<(u64, Point3)> {
        self.total_queries += 1;
        let r_sq = radius * radius;
        self.entries.values()
            .filter(|e| e.pos.distance_sq(center) <= r_sq)
            .map(|e| (e.id, e.pos))
            .collect()
    }

    pub fn nearest(&mut self, center: &Point3, k: usize) -> Vec<(u64, Point3, f64)> {
        self.total_queries += 1;
        let mut dists: Vec<_> = self.entries.values()
            .map(|e| (e.id, e.pos, e.pos.distance_sq(center).sqrt()))
            .collect();
        dists.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        dists.truncate(k);
        dists
    }

    pub fn contains(&self, id: u64) -> bool { self.entries.contains_key(&id) }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn total_inserts(&self) -> u64 { self.total_inserts }
    pub fn total_removes(&self) -> u64 { self.total_removes }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

impl Default for Octree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree() { assert!(Octree::new().is_empty()); }

    #[test]
    fn insert_get() {
        let mut o = Octree::new();
        o.insert(1, Point3::new(1.0, 2.0, 3.0), b"data".to_vec()).unwrap();
        let (pos, data) = o.get(1).unwrap();
        assert_eq!(pos, Point3::new(1.0, 2.0, 3.0));
        assert_eq!(data, b"data");
    }

    #[test]
    fn duplicate() {
        let mut o = Octree::new();
        o.insert(1, Point3::new(0.0, 0.0, 0.0), vec![]).unwrap();
        let err = o.insert(1, Point3::new(1.0, 1.0, 1.0), vec![]).unwrap_err();
        assert!(matches!(err, OctError::PointExists { .. }));
    }

    #[test]
    fn remove() {
        let mut o = Octree::new();
        o.insert(1, Point3::new(0.0, 0.0, 0.0), b"x".to_vec()).unwrap();
        let (pos, data) = o.remove(1).unwrap();
        assert_eq!(data, b"x");
        assert!(o.is_empty());
    }

    #[test]
    fn remove_missing() {
        let mut o = Octree::new();
        let err = o.remove(99).unwrap_err();
        assert!(matches!(err, OctError::PointNotFound { .. }));
    }

    #[test]
    fn radius_query() {
        let mut o = Octree::new();
        o.insert(1, Point3::new(0.0, 0.0, 0.0), vec![]).unwrap();
        o.insert(2, Point3::new(5.0, 5.0, 5.0), vec![]).unwrap();
        o.insert(3, Point3::new(1.0, 0.0, 0.0), vec![]).unwrap();
        let result = o.radius_query(&Point3::new(0.0, 0.0, 0.0), 2.0);
        let ids: Vec<u64> = result.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3));
        assert!(!ids.contains(&2));
    }

    #[test]
    fn nearest() {
        let mut o = Octree::new();
        o.insert(1, Point3::new(0.0, 0.0, 0.0), vec![]).unwrap();
        o.insert(2, Point3::new(10.0, 0.0, 0.0), vec![]).unwrap();
        o.insert(3, Point3::new(2.0, 0.0, 0.0), vec![]).unwrap();
        let result = o.nearest(&Point3::new(1.0, 0.0, 0.0), 2);
        assert_eq!(result.len(), 2);
        let ids: Vec<u64> = result.iter().map(|(id, _, _)| *id).collect();
        assert!(!ids.contains(&2));
    }

    #[test]
    fn contains() {
        let mut o = Octree::new();
        o.insert(1, Point3::new(0.0, 0.0, 0.0), vec![]).unwrap();
        assert!(o.contains(1));
        assert!(!o.contains(2));
    }

    #[test]
    fn distance_sq() {
        let a = Point3::new(1.0, 2.0, 3.0);
        let b = Point3::new(4.0, 6.0, 3.0);
        assert!((a.distance_sq(&b) - 25.0).abs() < 0.001);
    }

    #[test]
    fn stats() {
        let mut o = Octree::new();
        o.insert(1, Point3::new(0.0, 0.0, 0.0), vec![]).unwrap();
        o.remove(1).unwrap();
        assert_eq!(o.total_inserts(), 1);
        assert_eq!(o.total_removes(), 1);
    }

    #[test]
    fn error_display() { assert!(OctError::PointExists { id: 1 }.to_string().contains("1")); }
}
