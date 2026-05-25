use std::collections::BTreeMap;

pub struct Huffman;

impl Huffman {
    pub fn frequency(data: &[u8]) -> BTreeMap<u8, usize> {
        let mut freq = BTreeMap::new();
        for &b in data { *freq.entry(b).or_insert(0) += 1; }
        freq
    }

    pub fn encode(freq: &BTreeMap<u8, usize>) -> BTreeMap<u8, Vec<bool>> {
        if freq.is_empty() { return BTreeMap::new(); }
        if freq.len() == 1 {
            let mut m = BTreeMap::new();
            m.insert(*freq.keys().next().unwrap(), vec![false]);
            return m;
        }
        let mut nodes: Vec<(usize, Vec<u8>)> = freq.iter().map(|(&b, &f)| (f, vec![b])).collect();
        let mut codes: BTreeMap<u8, Vec<bool>> = freq.keys().map(|&b| (b, Vec::new())).collect();
        while nodes.len() > 1 {
            nodes.sort_by(|a, b| a.0.cmp(&b.0));
            let (f1, s1) = nodes.remove(0);
            let (f2, s2) = nodes.remove(0);
            for &b in &s1 { codes.get_mut(&b).unwrap().insert(0, false); }
            for &b in &s2 { codes.get_mut(&b).unwrap().insert(0, true); }
            let mut merged = s1;
            merged.extend(s2);
            nodes.push((f1 + f2, merged));
        }
        codes
    }

    pub fn encoded_size(codes: &BTreeMap<u8, Vec<bool>>, freq: &BTreeMap<u8, usize>) -> usize {
        freq.iter().map(|(b, f)| codes.get(b).map(|c| c.len() * f).unwrap_or(0)).sum()
    }

    pub fn compression_ratio(data_len: usize, encoded_bits: usize) -> f64 {
        if data_len == 0 { return 1.0; }
        encoded_bits as f64 / (data_len * 8) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency() { let f = Huffman::frequency(b"aab"); assert_eq!(f[&b'a'], 2); assert_eq!(f[&b'b'], 1); }

    #[test]
    fn codes_unique() {
        let f = Huffman::frequency(b"aabbc");
        let c = Huffman::encode(&f);
        assert_eq!(c.len(), 3);
        let codes: Vec<&[bool]> = c.values().map(|v| v.as_slice()).collect();
        for i in 0..codes.len() { for j in i+1..codes.len() { assert_ne!(codes[i], codes[j]); } }
    }

    #[test]
    fn prefix_free() {
        let f = Huffman::frequency(b"aaabbc");
        let c = Huffman::encode(&f);
        for (b1, c1) in &c { for (b2, c2) in &c {
            if b1 != b2 { assert!(!c2.starts_with(c1), "prefix violation"); }
        } }
    }

    #[test]
    fn single_symbol() {
        let mut f = BTreeMap::new(); f.insert(b'a', 5);
        let c = Huffman::encode(&f);
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn empty() { assert!(Huffman::encode(&BTreeMap::new()).is_empty()); }

    #[test]
    fn size() {
        let f = Huffman::frequency(b"aaabbc");
        let c = Huffman::encode(&f);
        let s = Huffman::encoded_size(&c, &f);
        assert!(s > 0);
        assert!(s < 6 * 8);
    }
}
