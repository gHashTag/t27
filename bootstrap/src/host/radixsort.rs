pub struct RadixSort {
    total_sorts: u64,
    total_passes: u64,
}

impl RadixSort {
    pub fn new() -> Self { Self { total_sorts: 0, total_passes: 0 } }

    pub fn sort_u64(&mut self, data: &mut [u64]) {
        self.total_sorts += 1;
        let n = data.len();
        if n <= 1 { return; }
        let mut temp = vec![0u64; n];
        for shift in (0..64).step_by(8) {
            self.total_passes += 1;
            let mut count = [0usize; 256];
            for &v in data.iter() { count[((v >> shift) & 0xFF) as usize] += 1; }
            let mut total = 0usize;
            for c in count.iter_mut() { let old = *c; *c = total; total += old; }
            for &v in data.iter() {
                let bucket = ((v >> shift) & 0xFF) as usize;
                temp[count[bucket]] = v;
                count[bucket] += 1;
            }
            data.copy_from_slice(&temp);
        }
    }

    pub fn sort_by_key<T: Clone>(&mut self, data: &mut [T], key_fn: impl Fn(&T) -> u64) {
        self.total_sorts += 1;
        let n = data.len();
        if n <= 1 { return; }
        let mut indexed: Vec<(u64, usize)> = data.iter().enumerate().map(|(i, v)| (key_fn(v), i)).collect();
        self.sort_u64_raw(&mut indexed);
        let sorted: Vec<T> = indexed.iter().map(|&(_, i)| data[i].clone()).collect();
        data.clone_from_slice(&sorted);
    }

    fn sort_u64_raw(&mut self, data: &mut [(u64, usize)]) {
        let n = data.len();
        let mut temp = vec![(0u64, 0usize); n];
        for shift in (0..64).step_by(8) {
            self.total_passes += 1;
            let mut count = [0usize; 256];
            for &(v, _) in data.iter() { count[((v >> shift) & 0xFF) as usize] += 1; }
            let mut total = 0usize;
            for c in count.iter_mut() { let old = *c; *c = total; total += old; }
            for &(v, ref i) in data.iter() {
                let bucket = ((v >> shift) & 0xFF) as usize;
                temp[count[bucket]] = (v, *i);
                count[bucket] += 1;
            }
            data.copy_from_slice(&temp);
        }
    }

    pub fn is_sorted_u64(data: &[u64]) -> bool {
        for i in 1..data.len() { if data[i] < data[i - 1] { return false; } }
        true
    }

    pub fn total_sorts(&self) -> u64 { self.total_sorts }
    pub fn total_passes(&self) -> u64 { self.total_passes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_u64() {
        let mut rs = RadixSort::new();
        let mut d = vec![5u64, 3, 1, 4, 2];
        rs.sort_u64(&mut d);
        assert!(RadixSort::is_sorted_u64(&d));
    }

    #[test]
    fn sort_already() {
        let mut rs = RadixSort::new();
        let mut d = vec![1u64, 2, 3];
        rs.sort_u64(&mut d);
        assert_eq!(d, vec![1, 2, 3]);
    }

    #[test]
    fn sort_reverse() {
        let mut rs = RadixSort::new();
        let mut d = vec![5u64, 4, 3, 2, 1];
        rs.sort_u64(&mut d);
        assert!(RadixSort::is_sorted_u64(&d));
    }

    #[test]
    fn sort_duplicates() {
        let mut rs = RadixSort::new();
        let mut d = vec![3u64, 1, 3, 2, 1];
        rs.sort_u64(&mut d);
        assert!(RadixSort::is_sorted_u64(&d));
    }

    #[test]
    fn sort_large() {
        let mut rs = RadixSort::new();
        let mut d: Vec<u64> = (0..1000u64).rev().collect();
        rs.sort_u64(&mut d);
        assert!(RadixSort::is_sorted_u64(&d));
    }

    #[test]
    fn sort_by_key() {
        let mut rs = RadixSort::new();
        let mut d = vec![100u32, 300, 200];
        rs.sort_by_key(&mut d, |v| *v as u64);
        assert_eq!(d, vec![100, 200, 300]);
    }

    #[test]
    fn empty() {
        let mut rs = RadixSort::new();
        let mut d: Vec<u64> = vec![];
        rs.sort_u64(&mut d);
        assert!(d.is_empty());
    }

    #[test]
    fn stats() {
        let mut rs = RadixSort::new();
        rs.sort_u64(&mut [3u64, 1, 2]);
        assert_eq!(rs.total_sorts(), 1);
        assert!(rs.total_passes() > 0);
    }
}
