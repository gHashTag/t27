fn cmp_swap(data: &mut [u64], i: usize, j: usize) -> bool {
    if data[i] > data[j] { data.swap(i, j); true } else { false }
}

pub struct SortNet {
    total_sorts: u64,
    total_cmps: u64,
}

impl SortNet {
    pub fn new() -> Self { Self { total_sorts: 0, total_cmps: 0 } }

    pub fn sort2(&mut self, data: &mut [u64; 2]) {
        self.total_sorts += 1;
        self.total_cmps += 1;
        cmp_swap(data, 0, 1);
    }

    pub fn sort4(&mut self, data: &mut [u64; 4]) {
        self.total_sorts += 1;
        cmp_swap(data, 0, 1); cmp_swap(data, 2, 3);
        cmp_swap(data, 0, 2); cmp_swap(data, 1, 3);
        cmp_swap(data, 1, 2);
        self.total_cmps += 5;
    }

    pub fn sort8(&mut self, data: &mut [u64; 8]) {
        self.total_sorts += 1;
        cmp_swap(data, 0, 1); cmp_swap(data, 2, 3); cmp_swap(data, 4, 5); cmp_swap(data, 6, 7);
        cmp_swap(data, 0, 2); cmp_swap(data, 1, 3); cmp_swap(data, 4, 6); cmp_swap(data, 5, 7);
        cmp_swap(data, 1, 2); cmp_swap(data, 5, 6);
        cmp_swap(data, 0, 4); cmp_swap(data, 3, 7);
        cmp_swap(data, 1, 5); cmp_swap(data, 2, 6);
        cmp_swap(data, 1, 4); cmp_swap(data, 3, 6);
        cmp_swap(data, 2, 4); cmp_swap(data, 3, 5);
        cmp_swap(data, 3, 4);
        self.total_cmps += 19;
    }

    pub fn merge2(&mut self, a: &[u64; 2], b: &[u64; 2]) -> [u64; 4] {
        self.total_sorts += 1;
        let mut out = [0u64; 4];
        let (mut ai, mut bi) = (0, 0);
        for i in 0..4 {
            if ai < 2 && (bi >= 2 || a[ai] <= b[bi]) { out[i] = a[ai]; ai += 1; } else { out[i] = b[bi]; bi += 1; }
            self.total_cmps += 1;
        }
        out
    }

    pub fn is_sorted(&self, data: &[u64]) -> bool {
        for i in 1..data.len() { if data[i] < data[i - 1] { return false; } }
        true
    }

    pub fn total_sorts(&self) -> u64 { self.total_sorts }
    pub fn total_cmps(&self) -> u64 { self.total_cmps }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort2() {
        let mut sn = SortNet::new();
        let mut d = [3u64, 1]; sn.sort2(&mut d);
        assert_eq!(d, [1, 3]);
    }

    #[test]
    fn sort4() {
        let mut sn = SortNet::new();
        let mut d = [4u64, 2, 3, 1]; sn.sort4(&mut d);
        assert!(sn.is_sorted(&d));
    }

    #[test]
    fn sort8() {
        let mut sn = SortNet::new();
        let mut d = [8u64, 6, 4, 2, 7, 5, 3, 1]; sn.sort8(&mut d);
        assert!(sn.is_sorted(&d));
    }

    #[test]
    fn sort8_already() {
        let mut sn = SortNet::new();
        let mut d = [1u64, 2, 3, 4, 5, 6, 7, 8]; sn.sort8(&mut d);
        assert!(sn.is_sorted(&d));
    }

    #[test]
    fn merge2() {
        let mut sn = SortNet::new();
        let out = sn.merge2(&[1, 3], &[2, 4]);
        assert_eq!(out, [1, 2, 3, 4]);
    }

    #[test]
    fn is_sorted() {
        let sn = SortNet::new();
        assert!(sn.is_sorted(&[1, 2, 3]));
        assert!(!sn.is_sorted(&[3, 1, 2]));
    }

    #[test]
    fn stats() {
        let mut sn = SortNet::new();
        sn.sort4(&mut [4, 2, 3, 1]);
        assert_eq!(sn.total_sorts(), 1);
        assert!(sn.total_cmps() > 0);
    }
}
