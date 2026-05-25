pub struct Rle2 { total_encode: u64, total_decode: u64 }

impl Rle2 {
    pub fn new() -> Self { Self { total_encode: 0, total_decode: 0 } }

    pub fn encode(&mut self, data: &[u8]) -> Vec<(u8, usize)> {
        self.total_encode += 1;
        if data.is_empty() { return vec![]; }
        let mut out = Vec::new();
        let mut cur = data[0];
        let mut count = 1usize;
        for &b in &data[1..] {
            if b == cur && count < 255 { count += 1; }
            else { out.push((cur, count)); cur = b; count = 1; }
        }
        out.push((cur, count));
        out
    }

    pub fn decode(&mut self, rle: &[(u8, usize)]) -> Vec<u8> {
        self.total_decode += 1;
        let mut out = Vec::new();
        for &(byte, count) in rle { for _ in 0..count { out.push(byte); } }
        out
    }

    pub fn encoded_len(rle: &[(u8, usize)]) -> usize { rle.iter().map(|(_, c)| *c).sum() }

    pub fn compression_ratio(data: &[u8], rle: &[(u8, usize)]) -> f64 {
        if data.is_empty() { return 1.0; }
        (rle.len() * 2) as f64 / data.len() as f64
    }

    pub fn total_encode(&self) -> u64 { self.total_encode }
    pub fn total_decode(&self) -> u64 { self.total_decode }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() { let mut r = Rle2::new(); let d = b"aaabbbbcc"; let e = r.encode(d); assert_eq!(r.decode(&e), d.to_vec()); }
    #[test]
    fn encode_vals() { let mut r = Rle2::new(); assert_eq!(r.encode(b"aabbb"), vec![(b'a',2),(b'b',3)]); }
    #[test]
    fn empty() { let mut r = Rle2::new(); assert!(r.encode(&[]).is_empty()); assert!(r.decode(&[]).is_empty()); }
    #[test]
    fn single() { let mut r = Rle2::new(); assert_eq!(r.encode(b"aaaaa"), vec![(b'a',5)]); }
    #[test]
    fn ratio() { let mut r = Rle2::new(); let e = r.encode(b"aaaa"); assert!(Rle2::compression_ratio(b"aaaa", &e) <= 1.0); }
    #[test]
    fn stats() { let mut r = Rle2::new(); r.encode(b"aa"); r.decode(&[(b'a',2)]); assert_eq!(r.total_encode(), 1); assert_eq!(r.total_decode(), 1); }
}
