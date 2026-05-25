pub struct SortMerge {
    total_merges: u64,
    total_items: u64,
}

impl SortMerge {
    pub fn new() -> Self { Self { total_merges: 0, total_items: 0 } }

    pub fn merge_two(&mut self, a: &[u64], b: &[u64]) -> Vec<u64> {
        self.total_merges += 1;
        let mut out = Vec::with_capacity(a.len() + b.len());
        let (mut i, mut j) = (0, 0);
        while i < a.len() && j < b.len() {
            if a[i] <= b[j] { out.push(a[i]); i += 1; } else { out.push(b[j]); j += 1; }
        }
        while i < a.len() { out.push(a[i]); i += 1; }
        while j < b.len() { out.push(b[j]); j += 1; }
        self.total_items += out.len() as u64;
        out
    }

    pub fn merge_k(&mut self, streams: &[Vec<u64>]) -> Vec<u64> {
        if streams.is_empty() { return vec![]; }
        self.total_merges += 1;
        let mut heap: Vec<(u64, usize, usize)> = Vec::new();
        for (si, s) in streams.iter().enumerate() {
            if !s.is_empty() { heap.push((s[0], si, 0)); }
        }
        let mut result = Vec::new();
        while !heap.is_empty() {
            let mut min_idx = 0;
            for i in 1..heap.len() {
                if heap[i].0 < heap[min_idx].0 { min_idx = i; }
            }
            let (val, si, ei) = heap.swap_remove(min_idx);
            result.push(val);
            let next = ei + 1;
            if next < streams[si].len() { heap.push((streams[si][next], si, next)); }
        }
        self.total_items += result.len() as u64;
        result
    }

    pub fn merge_intersect(&mut self, a: &[u64], b: &[u64]) -> Vec<u64> {
        self.total_merges += 1;
        let mut out = Vec::new();
        let (mut i, mut j) = (0, 0);
        while i < a.len() && j < b.len() {
            if a[i] == b[j] { out.push(a[i]); i += 1; j += 1; }
            else if a[i] < b[j] { i += 1; } else { j += 1; }
        }
        self.total_items += out.len() as u64;
        out
    }

    pub fn total_merges(&self) -> u64 { self.total_merges }
    pub fn total_items(&self) -> u64 { self.total_items }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_way() {
        let mut sm = SortMerge::new();
        let r = sm.merge_two(&[1, 3, 5], &[2, 4, 6]);
        assert_eq!(r, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn k_way() {
        let mut sm = SortMerge::new();
        let r = sm.merge_k(&[vec![1, 4], vec![2, 5], vec![3, 6]]);
        assert_eq!(r, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn intersect() {
        let mut sm = SortMerge::new();
        let r = sm.merge_intersect(&[1, 2, 3, 4], &[2, 4, 6]);
        assert_eq!(r, vec![2, 4]);
    }

    #[test]
    fn empty_streams() {
        let mut sm = SortMerge::new();
        assert!(sm.merge_k(&[]).is_empty());
        assert!(sm.merge_two(&[], &[]).is_empty());
    }

    #[test]
    fn one_empty() {
        let mut sm = SortMerge::new();
        let r = sm.merge_two(&[1, 2], &[]);
        assert_eq!(r, vec![1, 2]);
    }

    #[test]
    fn duplicates() {
        let mut sm = SortMerge::new();
        let r = sm.merge_two(&[1, 1, 2], &[1, 3]);
        assert_eq!(r, vec![1, 1, 1, 2, 3]);
    }

    #[test]
    fn stats() {
        let mut sm = SortMerge::new();
        sm.merge_two(&[1], &[2]);
        assert_eq!(sm.total_merges(), 1);
        assert_eq!(sm.total_items(), 2);
    }
}
