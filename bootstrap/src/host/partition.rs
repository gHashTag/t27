pub struct Partition {
    total_calls: u64,
    total_swaps: u64,
}

impl Partition {
    pub fn new() -> Self { Self { total_calls: 0, total_swaps: 0 } }

    pub fn partition(&mut self, arr: &mut [u64], pivot_idx: usize) -> usize {
        self.total_calls += 1;
        let n = arr.len();
        if n == 0 { return 0; }
        let pivot_idx = pivot_idx.min(n - 1);
        arr.swap(pivot_idx, n - 1);
        let pivot = arr[n - 1];
        let mut i = 0;
        for j in 0..n - 1 {
            if arr[j] <= pivot {
                if i != j { arr.swap(i, j); self.total_swaps += 1; }
                i += 1;
            }
        }
        arr.swap(i, n - 1);
        self.total_swaps += 1;
        i
    }

    pub fn select(&mut self, arr: &mut [u64], k: usize) -> u64 {
        let mut lo = 0;
        let mut hi = arr.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let p = self.partition(&mut arr[lo..hi], mid - lo) + lo;
            if p == k { return arr[k]; }
            else if p < k { lo = p + 1; }
            else { hi = p; }
        }
        arr[k]
    }

    pub fn nth_element(&mut self, arr: &mut [u64], n: usize) -> u64 {
        self.select(arr, n.min(arr.len() - 1))
    }

    pub fn total_calls(&self) -> u64 { self.total_calls }
    pub fn total_swaps(&self) -> u64 { self.total_swaps }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut p = Partition::new();
        let mut a = [3, 1, 4, 1, 5, 9, 2, 6];
        let pi = p.partition(&mut a, 0);
        for i in 0..pi { assert!(a[i] <= a[pi]); }
        for i in pi + 1..a.len() { assert!(a[i] >= a[pi]); }
    }

    #[test]
    fn select_median() {
        let mut p = Partition::new();
        let mut a = [5, 3, 1, 4, 2];
        let m = p.nth_element(&mut a, 2);
        assert_eq!(m, 3);
    }

    #[test]
    fn select_min() {
        let mut p = Partition::new();
        let mut a = [5, 3, 1, 4, 2];
        assert_eq!(p.nth_element(&mut a, 0), 1);
    }

    #[test]
    fn select_max() {
        let mut p = Partition::new();
        let mut a = [5, 3, 1, 4, 2];
        assert_eq!(p.nth_element(&mut a, 4), 5);
    }

    #[test]
    fn single() {
        let mut p = Partition::new();
        let mut a = [42u64];
        assert_eq!(p.partition(&mut a, 0), 0);
    }

    #[test]
    fn empty() {
        let mut p = Partition::new();
        let mut a: [u64; 0] = [];
        assert_eq!(p.partition(&mut a, 0), 0);
    }

    #[test]
    fn stats() {
        let mut p = Partition::new();
        let mut a = [3, 1, 2];
        p.partition(&mut a, 1);
        assert_eq!(p.total_calls(), 1);
        assert!(p.total_swaps() > 0);
    }
}
