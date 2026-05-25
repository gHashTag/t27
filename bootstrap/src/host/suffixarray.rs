pub struct SuffixArray {
    data: Vec<u8>,
    sa: Vec<usize>,
    total_queries: u64,
}

impl SuffixArray {
    pub fn new(data: Vec<u8>) -> Self {
        let n = data.len();
        let mut sa: Vec<usize> = (0..n).collect();
        let d = data.clone();
        sa.sort_by(|&a, &b| d[a..].cmp(&d[b..]));
        Self { data: d, sa, total_queries: 0 }
    }

    pub fn suffix(&self, i: usize) -> Option<&[u8]> { self.sa.get(i).map(|&s| &self.data[s..]) }

    pub fn search(&mut self, pattern: &[u8]) -> Vec<usize> {
        self.total_queries += 1;
        let n = self.sa.len();
        let mut lo = 0usize;
        let mut hi = n;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let suffix_start = self.sa[mid];
            let suffix = &self.data[suffix_start..];
            if suffix < pattern { lo = mid + 1; } else { hi = mid; }
        }
        let start = lo;
        hi = n;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let suffix_start = self.sa[mid];
            let suffix = if suffix_start + pattern.len() <= self.data.len() { &self.data[suffix_start..suffix_start + pattern.len()] } else { &self.data[suffix_start..] };
            if suffix <= pattern { lo = mid + 1; } else { hi = mid; }
        }
        (start..lo).map(|i| self.sa[i]).collect()
    }

    pub fn lcp_array(&self) -> Vec<usize> {
        let n = self.sa.len();
        let mut rank = vec![0usize; n];
        for (i, &s) in self.sa.iter().enumerate() { rank[s] = i; }
        let mut lcp = vec![0usize; n];
        let mut k = 0usize;
        for i in 0..n {
            if rank[i] == 0 { k = 0; continue; }
            let j = self.sa[rank[i] - 1];
            while i + k < n && j + k < n && self.data[i + k] == self.data[j + k] { k += 1; }
            lcp[rank[i]] = k;
            if k > 0 { k -= 1; }
        }
        lcp
    }

    pub fn len(&self) -> usize { self.sa.len() }
    pub fn is_empty(&self) -> bool { self.sa.is_empty() }
    pub fn total_queries(&self) -> u64 { self.total_queries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted() {
        let sa = SuffixArray::new(b"banana".to_vec());
        let suffixes: Vec<&[u8]> = (0..sa.len()).map(|i| sa.suffix(i).unwrap()).collect();
        for i in 1..suffixes.len() { assert!(suffixes[i - 1] < suffixes[i]); }
    }

    #[test]
    fn search() {
        let mut sa = SuffixArray::new(b"banana".to_vec());
        let r = sa.search(b"ana");
        assert_eq!(r.len(), 2);
        assert!(r.contains(&1));
        assert!(r.contains(&3));
    }

    #[test]
    fn search_miss() {
        let mut sa = SuffixArray::new(b"banana".to_vec());
        assert!(sa.search(b"xyz").is_empty());
    }

    #[test]
    fn lcp() {
        let sa = SuffixArray::new(b"banana".to_vec());
        let lcp = sa.lcp_array();
        assert_eq!(lcp[0], 0);
        assert!(lcp.iter().sum::<usize>() > 0);
    }

    #[test]
    fn empty() { assert!(SuffixArray::new(vec![]).is_empty()); }

    #[test]
    fn stats() {
        let mut sa = SuffixArray::new(b"abc".to_vec());
        sa.search(b"a");
        assert_eq!(sa.total_queries(), 1);
    }
}
