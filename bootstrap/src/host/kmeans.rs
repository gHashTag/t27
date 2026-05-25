pub struct KMeans;

impl KMeans {
    pub fn cluster(points: &[(f64, f64)], k: usize, max_iter: usize) -> (Vec<(f64, f64)>, Vec<usize>) {
        let n = points.len();
        if n == 0 || k == 0 { return (Vec::new(), Vec::new()); }
        let k = k.min(n);
        let mut centroids: Vec<(f64, f64)> = points[..k].to_vec();
        let mut labels = vec![0usize; n];
        for _ in 0..max_iter {
            for (i, &(px, py)) in points.iter().enumerate() {
                let mut best = 0;
                let mut best_d = f64::MAX;
                for (j, &(cx, cy)) in centroids.iter().enumerate() {
                    let d = (px - cx).powi(2) + (py - cy).powi(2);
                    if d < best_d { best_d = d; best = j; }
                }
                labels[i] = best;
            }
            let mut new_c = vec![(0.0f64, 0.0f64); k];
            let mut counts = vec![0usize; k];
            for (i, &(px, py)) in points.iter().enumerate() {
                let c = labels[i];
                new_c[c].0 += px; new_c[c].1 += py;
                counts[c] += 1;
            }
            for j in 0..k {
                if counts[j] > 0 {
                    new_c[j].0 /= counts[j] as f64;
                    new_c[j].1 /= counts[j] as f64;
                }
            }
            centroids = new_c;
        }
        (centroids, labels)
    }

    pub fn inertia(points: &[(f64, f64)], centroids: &[(f64, f64)], labels: &[usize]) -> f64 {
        points.iter().zip(labels.iter()).map(|(&(px, py), &l)| {
            let (cx, cy) = centroids[l];
            (px - cx).powi(2) + (py - cy).powi(2)
        }).sum()
    }

    pub fn silhouette_score(points: &[(f64, f64)], labels: &[usize], k: usize) -> f64 {
        let n = points.len();
        if n == 0 || k <= 1 { return 0.0; }
        let mut total = 0.0f64;
        for i in 0..n {
            let mut intra = f64::MAX;
            let mut inter = f64::MAX;
            for j in 0..k {
                let mut dist_sum = 0.0f64;
                let mut count = 0usize;
                for (m, &(px, py)) in points.iter().enumerate() {
                    if labels[m] == j && m != i {
                        dist_sum += (points[i].0 - px).hypot(points[i].1 - py);
                        count += 1;
                    }
                }
                if count == 0 { continue; }
                let avg = dist_sum / count as f64;
                if j == labels[i] { intra = intra.min(avg); }
                else { inter = inter.min(avg); }
            }
            if intra == f64::MAX { intra = 0.0; }
            if inter == f64::MAX { inter = 0.0; }
            let denom = intra.max(inter);
            if denom > 0.0 { total += (inter - intra) / denom; }
        }
        total / n as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clusters() {
        let pts = vec![(0.0,0.0),(0.1,0.1),(10.0,10.0),(10.1,10.1)];
        let (c, l) = KMeans::cluster(&pts, 2, 100);
        assert_eq!(c.len(), 2);
        assert_eq!(l.len(), 4);
    }

    #[test]
    fn inertia_decreases() {
        let pts = vec![(0.0,0.0),(1.0,1.0),(10.0,10.0),(11.0,11.0)];
        let (c1, l1) = KMeans::cluster(&pts, 2, 1);
        let i1 = KMeans::inertia(&pts, &c1, &l1);
        let (c2, l2) = KMeans::cluster(&pts, 2, 100);
        let i2 = KMeans::inertia(&pts, &c2, &l2);
        assert!(i2 <= i1);
    }

    #[test]
    fn single_point() {
        let (c, l) = KMeans::cluster(&[(5.0, 5.0)], 1, 10);
        assert_eq!(c.len(), 1);
        assert_eq!(l[0], 0);
    }

    #[test]
    fn empty() {
        let (c, l) = KMeans::cluster(&[], 3, 10);
        assert!(c.is_empty());
        assert!(l.is_empty());
    }

    #[test]
    fn labels_valid() {
        let pts = vec![(0.0,0.0),(1.0,0.0),(0.0,1.0),(10.0,10.0),(11.0,10.0),(10.0,11.0)];
        let (_, l) = KMeans::cluster(&pts, 2, 100);
        for &label in &l { assert!(label < 2); }
    }
}
