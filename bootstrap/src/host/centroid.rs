pub struct Centroid;

impl Centroid {
    pub fn mean(points: &[(f64, f64)]) -> (f64, f64) {
        if points.is_empty() { return (0.0, 0.0); }
        let n = points.len() as f64;
        let sx: f64 = points.iter().map(|p| p.0).sum();
        let sy: f64 = points.iter().map(|p| p.1).sum();
        (sx / n, sy / n)
    }

    pub fn weighted_mean(points: &[(f64, f64, f64)]) -> (f64, f64) {
        let tw: f64 = points.iter().map(|p| p.2).sum();
        if tw == 0.0 { return (0.0, 0.0); }
        let sx: f64 = points.iter().map(|p| p.0 * p.2).sum();
        let sy: f64 = points.iter().map(|p| p.1 * p.2).sum();
        (sx / tw, sy / tw)
    }

    pub fn polygon_area(vertices: &[(f64, f64)]) -> f64 {
        let n = vertices.len();
        if n < 3 { return 0.0; }
        let mut area = 0.0f64;
        for i in 0..n {
            let j = (i + 1) % n;
            area += vertices[i].0 * vertices[j].1;
            area -= vertices[j].0 * vertices[i].1;
        }
        (area / 2.0).abs()
    }

    pub fn polygon_centroid(vertices: &[(f64, f64)]) -> (f64, f64) {
        let n = vertices.len();
        if n < 3 { return Self::mean(vertices); }
        let mut area = 0.0f64;
        let mut cx = 0.0f64;
        let mut cy = 0.0f64;
        for i in 0..n {
            let j = (i + 1) % n;
            let cross = vertices[i].0 * vertices[j].1 - vertices[j].0 * vertices[i].1;
            area += cross;
            cx += (vertices[i].0 + vertices[j].0) * cross;
            cy += (vertices[i].1 + vertices[j].1) * cross;
        }
        area /= 2.0;
        if area.abs() < 1e-12 { return Self::mean(vertices); }
        (cx / (6.0 * area), cy / (6.0 * area))
    }

    pub fn bounding_box(points: &[(f64, f64)]) -> ((f64, f64), (f64, f64)) {
        if points.is_empty() { return ((0.0, 0.0), (0.0, 0.0)); }
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for &(x, y) in points {
            min_x = min_x.min(x); min_y = min_y.min(y);
            max_x = max_x.max(x); max_y = max_y.max(y);
        }
        ((min_x, min_y), (max_x, max_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_basic() {
        let pts = vec![(0.0, 0.0), (2.0, 4.0)];
        let (mx, my) = Centroid::mean(&pts);
        assert!((mx - 1.0).abs() < 1e-9);
        assert!((my - 2.0).abs() < 1e-9);
    }

    #[test]
    fn square_area() {
        let sq = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        assert!((Centroid::polygon_area(&sq) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn triangle_centroid() {
        let tri = vec![(0.0, 0.0), (6.0, 0.0), (3.0, 6.0)];
        let (cx, cy) = Centroid::polygon_centroid(&tri);
        assert!((cx - 3.0).abs() < 1e-9);
        assert!((cy - 2.0).abs() < 1e-9);
    }

    #[test]
    fn bounding_box() {
        let pts = vec![(1.0, 2.0), (5.0, 8.0), (3.0, 4.0)];
        let (mn, mx) = Centroid::bounding_box(&pts);
        assert!((mn.0 - 1.0).abs() < 1e-9);
        assert!((mx.1 - 8.0).abs() < 1e-9);
    }

    #[test]
    fn weighted() {
        let pts = vec![(0.0, 0.0, 1.0), (10.0, 10.0, 3.0)];
        let (cx, cy) = Centroid::weighted_mean(&pts);
        assert!((cx - 7.5).abs() < 1e-9);
    }

    #[test]
    fn empty() { assert_eq!(Centroid::mean(&[]), (0.0, 0.0)); }
}
