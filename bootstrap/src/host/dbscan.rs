pub struct Dbscan;

impl Dbscan {
    pub fn cluster(points: &[(f64, f64)], eps: f64, min_pts: usize) -> Vec<i32> {
        let n = points.len();
        let eps2 = eps * eps;
        let mut labels = vec![-1i32; n];
        let mut cluster_id = 0i32;
        let mut visited = vec![false; n];
        for i in 0..n {
            if visited[i] { continue; }
            visited[i] = true;
            let neighbors = Self::range_query(points, i, eps2);
            if neighbors.len() < min_pts { labels[i] = -1; continue; }
            labels[i] = cluster_id;
            let mut seeds: Vec<usize> = neighbors.into_iter().filter(|&j| j != i).collect();
            let mut si = 0;
            while si < seeds.len() {
                let j = seeds[si];
                si += 1;
                if !visited[j] {
                    visited[j] = true;
                    let j_neighbors = Self::range_query(points, j, eps2);
                    if j_neighbors.len() >= min_pts { for &k in &j_neighbors { if !seeds.contains(&k) { seeds.push(k); } } }
                }
                if labels[j] == -1 { labels[j] = cluster_id; }
            }
            cluster_id += 1;
        }
        labels
    }

    fn range_query(points: &[(f64, f64)], idx: usize, eps2: f64) -> Vec<usize> {
        let (px, py) = points[idx];
        points.iter().enumerate()
            .filter(|(_, (x, y))| (x - px) * (x - px) + (y - py) * (y - py) <= eps2)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn noise_count(labels: &[i32]) -> usize { labels.iter().filter(|&&l| l == -1).count() }
    pub fn cluster_count(labels: &[i32]) -> i32 { *labels.iter().max().unwrap_or(&-1) + 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clusters() {
        let pts = vec![(0.0,0.0),(1.0,0.0),(0.0,1.0),(10.0,10.0),(11.0,10.0),(10.0,11.0)];
        let labels = Dbscan::cluster(&pts, 2.0, 2);
        assert_eq!(Dbscan::cluster_count(&labels), 2);
        assert_ne!(labels[0], labels[3]);
    }

    #[test]
    fn single_cluster() {
        let pts = vec![(0.0,0.0),(1.0,0.0),(0.0,1.0)];
        let labels = Dbscan::cluster(&pts, 2.0, 2);
        assert_eq!(Dbscan::cluster_count(&labels), 1);
    }

    #[test]
    fn all_noise() {
        let pts = vec![(0.0,0.0),(100.0,100.0),(200.0,200.0)];
        let labels = Dbscan::cluster(&pts, 1.0, 2);
        assert_eq!(Dbscan::noise_count(&labels), 3);
    }

    #[test]
    fn empty() {
        let labels = Dbscan::cluster(&[], 1.0, 2);
        assert!(labels.is_empty());
    }

    #[test]
    fn single() {
        let labels = Dbscan::cluster(&[(0.0, 0.0)], 1.0, 1);
        assert_eq!(Dbscan::cluster_count(&labels), 1);
    }
}
