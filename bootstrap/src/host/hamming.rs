pub struct Hamming;

impl Hamming {
    pub fn distance(a: u64, b: u64) -> u32 { (a ^ b).count_ones() }

    pub fn distance_bytes(a: &[u8], b: &[u8]) -> u32 {
        a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum()
    }

    pub fn encode_hamming74(data: u8) -> u8 {
        let d = [data & 1, (data >> 1) & 1, (data >> 2) & 1, (data >> 3) & 1];
        let p1 = d[0] ^ d[1] ^ d[3];
        let p2 = d[0] ^ d[2] ^ d[3];
        let p3 = d[1] ^ d[2] ^ d[3];
        p1 | (p2 << 1) | (d[0] << 2) | (p3 << 3) | (d[1] << 4) | (d[2] << 5) | (d[3] << 6)
    }

    pub fn decode_hamming74(code: u8) -> (u8, bool) {
        let bits: [u8; 7] = [
            code & 1, (code >> 1) & 1, (code >> 2) & 1, (code >> 3) & 1,
            (code >> 4) & 1, (code >> 5) & 1, (code >> 6) & 1,
        ];
        let s1 = bits[0] ^ bits[2] ^ bits[4] ^ bits[6];
        let s2 = bits[1] ^ bits[2] ^ bits[5] ^ bits[6];
        let s3 = bits[3] ^ bits[4] ^ bits[5] ^ bits[6];
        let syndrome = s1 | (s2 << 1) | (s3 << 2);
        let mut corrected = code;
        let had_error = syndrome != 0;
        if had_error && syndrome <= 7 { corrected ^= 1 << (syndrome - 1); }
        let data = ((corrected >> 2) & 1) | ((corrected >> 3) & 2) | ((corrected >> 3) & 4) | ((corrected >> 3) & 8);
        (data, had_error)
    }

    pub fn weight(v: u64) -> u32 { v.count_ones() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_zero() { assert_eq!(Hamming::distance(0, 0), 0); }

    #[test]
    fn distance_known() { assert_eq!(Hamming::distance(0b11110000, 0b10111000), 2); }

    #[test]
    fn distance_bytes() {
        assert_eq!(Hamming::distance_bytes(b"abc", b"abc"), 0);
        assert!(Hamming::distance_bytes(b"abc", b"axc") > 0);
    }

    #[test]
    fn hamming74_roundtrip() {
        for d in 0u8..16 {
            let encoded = Hamming::encode_hamming74(d);
            let (decoded, err) = Hamming::decode_hamming74(encoded);
            assert_eq!(decoded, d);
            assert!(!err);
        }
    }

    #[test]
    fn hamming74_single_error() {
        let encoded = Hamming::encode_hamming74(10);
        for bit in 0..7 {
            let corrupted = encoded ^ (1 << bit);
            let (decoded, err) = Hamming::decode_hamming74(corrupted);
            assert_eq!(decoded, 10);
            assert!(err);
        }
    }

    #[test]
    fn weight() { assert_eq!(Hamming::weight(0b10101010), 4); }
}
