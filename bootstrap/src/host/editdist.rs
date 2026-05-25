pub struct EditDist;

impl EditDist {
    pub fn distance(a: &[u8], b: &[u8]) -> usize {
        let n = a.len();
        let m = b.len();
        if n == 0 { return m; }
        if m == 0 { return n; }
        let mut prev = (0..=m).collect::<Vec<usize>>();
        let mut curr = vec![0; m + 1];
        for i in 1..=n {
            curr[0] = i;
            for j in 1..=m {
                let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
                curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[m]
    }

    pub fn distance_str(a: &str, b: &str) -> usize {
        Self::distance(a.as_bytes(), b.as_bytes())
    }

    pub fn normalized(a: &[u8], b: &[u8]) -> f64 {
        let d = Self::distance(a, b);
        let max_len = a.len().max(b.len());
        if max_len == 0 { return 0.0; }
        d as f64 / max_len as f64
    }

    pub fn ratio(a: &[u8], b: &[u8]) -> f64 {
        let d = Self::distance(a, b);
        let total = a.len() + b.len();
        if total == 0 { return 1.0; }
        let num = total as i64 - 2 * d as i64;
        num as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal() { assert_eq!(EditDist::distance_str("kitten", "kitten"), 0); }

    #[test]
    fn kitten_sitting() { assert_eq!(EditDist::distance_str("kitten", "sitting"), 3); }

    #[test]
    fn empty() {
        assert_eq!(EditDist::distance(b"", b""), 0);
        assert_eq!(EditDist::distance(b"abc", b""), 3);
        assert_eq!(EditDist::distance(b"", b"abc"), 3);
    }

    #[test]
    fn single_edit() { assert_eq!(EditDist::distance_str("cat", "car"), 1); }

    #[test]
    fn normalized() {
        let n = EditDist::normalized(b"kitten", b"sitting");
        assert!((n - 3.0 / 7.0).abs() < 1e-9);
    }

    #[test]
    fn ratio() {
        let r = EditDist::ratio(b"abc", b"abc");
        assert!((r - 1.0).abs() < 1e-9);
        let r2 = EditDist::ratio(b"abc", b"");
        assert!((r2 - (-1.0)).abs() < 1e-9);
        let r3 = EditDist::ratio(b"", b"");
        assert!((r3 - 1.0).abs() < 1e-9);
    }
}
