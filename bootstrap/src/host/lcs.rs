pub struct Lcs;

impl Lcs {
    pub fn length(a: &[u8], b: &[u8]) -> usize {
        let n = a.len();
        let m = b.len();
        if n == 0 || m == 0 { return 0; }
        let mut prev = vec![0; m + 1];
        let mut curr = vec![0; m + 1];
        for i in 1..=n {
            for j in 1..=m {
                curr[j] = if a[i - 1] == b[j - 1] {
                    prev[j - 1] + 1
                } else {
                    prev[j].max(curr[j - 1])
                };
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[m]
    }

    pub fn sequence(a: &[u8], b: &[u8]) -> Vec<u8> {
        let n = a.len();
        let m = b.len();
        let mut dp = vec![vec![0; m + 1]; n + 1];
        for i in 1..=n {
            for j in 1..=m {
                dp[i][j] = if a[i - 1] == b[j - 1] {
                    dp[i - 1][j - 1] + 1
                } else {
                    dp[i - 1][j].max(dp[i][j - 1])
                };
            }
        }
        let mut result = Vec::new();
        let (mut i, mut j) = (n, m);
        while i > 0 && j > 0 {
            if a[i - 1] == b[j - 1] {
                result.push(a[i - 1]);
                i -= 1; j -= 1;
            } else if dp[i - 1][j] >= dp[i][j - 1] {
                i -= 1;
            } else {
                j -= 1;
            }
        }
        result.reverse();
        result
    }

    pub fn similarity(a: &[u8], b: &[u8]) -> f64 {
        let lcs_len = Self::length(a, b);
        let max_len = a.len().max(b.len());
        if max_len == 0 { return 1.0; }
        lcs_len as f64 / max_len as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_basic() {
        assert_eq!(Lcs::length(b"ABCBDAB", b"BDCAB"), 4);
    }

    #[test]
    fn sequence_basic() {
        let seq = Lcs::sequence(b"ABCBDAB", b"BDCAB");
        assert_eq!(seq.len(), 4);
    }

    #[test]
    fn identical() {
        assert_eq!(Lcs::length(b"abcdef", b"abcdef"), 6);
        assert_eq!(Lcs::sequence(b"abcdef", b"abcdef"), b"abcdef".to_vec());
    }

    #[test]
    fn disjoint() { assert_eq!(Lcs::length(b"abc", b"xyz"), 0); }

    #[test]
    fn empty() {
        assert_eq!(Lcs::length(b"", b""), 0);
        assert_eq!(Lcs::length(b"abc", b""), 0);
    }

    #[test]
    fn similarity() {
        let s = Lcs::similarity(b"ABCBDAB", b"BDCAB");
        assert!((s - 4.0 / 7.0).abs() < 1e-9);
    }
}
