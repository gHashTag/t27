pub struct BoyerMoore2;

impl BoyerMoore2 {
    pub fn search(text: &[u8], pattern: &[u8]) -> Vec<usize> {
        let m = pattern.len();
        let n = text.len();
        if m == 0 || m > n { return Vec::new(); }
        let bad_char = Self::build_bad_char_table(pattern);
        let mut matches = Vec::new();
        let mut i = 0;
        while i <= n - m {
            let mut j = m;
            while j > 0 && pattern[j - 1] == text[i + j - 1] { j -= 1; }
            if j == 0 { matches.push(i); }
            let shift = if i + m < n { bad_char[text[i + m] as usize] } else { 1 };
            i += shift.max(1);
        }
        matches
    }

    fn build_bad_char_table(pattern: &[u8]) -> [usize; 256] {
        let mut table = [pattern.len() + 1; 256];
        for (i, &b) in pattern.iter().enumerate() {
            table[b as usize] = pattern.len() - i;
        }
        table
    }

    pub fn count(text: &[u8], pattern: &[u8]) -> usize { Self::search(text, pattern).len() }

    pub fn contains(text: &[u8], pattern: &[u8]) -> bool { Self::search(text, pattern).first().is_some() }

    pub fn replace(text: &[u8], pattern: &[u8], replacement: &[u8]) -> Vec<u8> {
        let matches = Self::search(text, pattern);
        if matches.is_empty() { return text.to_vec(); }
        let mut result = Vec::with_capacity(text.len());
        let mut prev = 0;
        for &pos in &matches {
            result.extend_from_slice(&text[prev..pos]);
            result.extend_from_slice(replacement);
            prev = pos + pattern.len();
        }
        result.extend_from_slice(&text[prev..]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let r = BoyerMoore2::search(b"abcabcabc", b"abc");
        assert_eq!(r, vec![0, 3, 6]);
    }

    #[test]
    fn single_char() {
        let r = BoyerMoore2::search(b"aaa", b"a");
        assert_eq!(r, vec![0, 1, 2]);
    }

    #[test]
    fn not_found() { assert!(BoyerMoore2::search(b"abcdef", b"xyz").is_empty()); }

    #[test]
    fn contains_and_count() {
        assert!(BoyerMoore2::contains(b"hello world", b"world"));
        assert!(!BoyerMoore2::contains(b"hello world", b"moon"));
        assert_eq!(BoyerMoore2::count(b"ababab", b"ab"), 3);
    }

    #[test]
    fn replace() {
        let r = BoyerMoore2::replace(b"hello world", b"world", b"earth");
        assert_eq!(r, b"hello earth");
    }

    #[test]
    fn empty_pattern() { assert!(BoyerMoore2::search(b"abc", b"").is_empty()); }
}
