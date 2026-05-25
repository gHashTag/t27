use std::collections::BTreeMap;

pub struct CodeTables;

impl CodeTables {
    pub fn build_gamma(code: u64) -> Vec<bool> {
        let bin = 64 - code.leading_zeros() as usize;
        let mut bits = vec![false; bin - 1];
        for i in (0..bin).rev() { bits.push((code >> i) & 1 != 0); }
        bits
    }

    pub fn decode_gamma(bits: &[bool], pos: usize) -> (u64, usize) {
        let mut n = 0usize;
        let mut p = pos;
        while p < bits.len() && !bits[p] { n += 1; p += 1; }
        p += 1;
        let mut val = 1u64;
        for _ in 0..n {
            val = (val << 1) | if p < bits.len() && bits[p] { 1 } else { 0 };
            p += 1;
        }
        (val, p)
    }

    pub fn build_delta(code: u64) -> Vec<bool> {
        if code == 1 { return vec![true]; }
        let n = 64 - code.leading_zeros() as usize;
        let gamma_n = Self::build_gamma(n as u64);
        let mut bits = gamma_n;
        for i in (0..n - 1).rev() { bits.push((code >> i) & 1 != 0); }
        bits
    }

    pub fn varint_encode(mut val: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        loop {
            let mut b = (val & 0x7F) as u8;
            val >>= 7;
            if val > 0 { b |= 0x80; }
            bytes.push(b);
            if val == 0 { break; }
        }
        bytes
    }

    pub fn varint_decode(bytes: &[u8]) -> (u64, usize) {
        let mut val = 0u64;
        let mut shift = 0;
        for (i, &b) in bytes.iter().enumerate() {
            val |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 { return (val, i + 1); }
            shift += 7;
        }
        (val, bytes.len())
    }

    pub fn frequency_table(data: &[u8]) -> BTreeMap<u8, usize> {
        let mut freq = BTreeMap::new();
        for &b in data { *freq.entry(b).or_insert(0) += 1; }
        freq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_roundtrip() {
        for v in [1u64, 2, 3, 5, 10, 100] {
            let bits = CodeTables::build_gamma(v);
            let (decoded, _) = CodeTables::decode_gamma(&bits, 0);
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn delta_single() {
        let bits = CodeTables::build_delta(1);
        assert_eq!(bits, vec![true]);
    }

    #[test]
    fn varint_roundtrip() {
        for v in [0u64, 1, 127, 128, 300, 16384] {
            let bytes = CodeTables::varint_encode(v);
            let (decoded, _) = CodeTables::varint_decode(&bytes);
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn varint_len() {
        assert_eq!(CodeTables::varint_encode(127).len(), 1);
        assert_eq!(CodeTables::varint_encode(128).len(), 2);
    }

    #[test]
    fn frequency_table() {
        let freq = CodeTables::frequency_table(b"aaabbc");
        assert_eq!(freq[&b'a'], 3);
        assert_eq!(freq[&b'c'], 1);
    }
}
