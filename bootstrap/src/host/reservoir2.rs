pub struct Reservoir2;

impl Reservoir2 {
    pub fn sample_weighted<T: Clone>(items: &[(T, f64)], k: usize, seed: u64) -> Vec<T> {
        if k == 0 || items.is_empty() { return Vec::new(); }
        let mut state = seed;
        let mut rng = || -> f64 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (state >> 33) as f64 / (1u64 << 31) as f64
        };
        let mut heap: Vec<(f64, usize)> = Vec::with_capacity(k.min(items.len()));
        for (i, (_, w)) in items.iter().enumerate() {
            if *w <= 0.0 { continue; }
            let key = rng().powf(1.0 / w).ln();
            if heap.len() < k {
                heap.push((key, i));
                if heap.len() == k {
                    heap.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                }
            } else if key < heap[0].0 {
                heap[0] = (key, i);
                Self::sift_down(&mut heap, 0);
            }
        }
        heap.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        heap.into_iter().map(|(_, i)| items[i].0.clone()).collect()
    }

    fn sift_down(heap: &mut [(f64, usize)], mut i: usize) {
        let n = heap.len();
        loop {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            let mut largest = i;
            if left < n && heap[left].0 > heap[largest].0 { largest = left; }
            if right < n && heap[right].0 > heap[largest].0 { largest = right; }
            if largest == i { break; }
            heap.swap(i, largest);
            i = largest;
        }
    }

    pub fn sample_uniform<T: Clone>(items: &[T], k: usize, seed: u64) -> Vec<T> {
        let weighted: Vec<(T, f64)> = items.iter().cloned().map(|x| (x, 1.0)).collect();
        Self::sample_weighted(&weighted, k, seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let items: Vec<(i32, f64)> = vec![(1, 1.0), (2, 1.0), (3, 1.0), (4, 1.0), (5, 1.0)];
        let s = Reservoir2::sample_weighted(&items, 3, 42);
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn deterministic() {
        let items: Vec<(i32, f64)> = vec![(1, 1.0), (2, 1.0), (3, 1.0), (4, 1.0)];
        let s1 = Reservoir2::sample_weighted(&items, 2, 99);
        let s2 = Reservoir2::sample_weighted(&items, 2, 99);
        assert_eq!(s1, s2);
    }

    #[test]
    fn k_larger_than_n() {
        let items: Vec<(i32, f64)> = vec![(1, 1.0), (2, 1.0)];
        let s = Reservoir2::sample_weighted(&items, 5, 42);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn uniform() {
        let items = vec![10, 20, 30, 40, 50];
        let s = Reservoir2::sample_uniform(&items, 3, 42);
        assert_eq!(s.len(), 3);
        for v in &s { assert!(items.contains(v)); }
    }

    #[test]
    fn empty() { assert!(Reservoir2::sample_weighted::<i32>(&[], 3, 42).is_empty()); }
}
