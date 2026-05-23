pub const TRITS_PER_WORD: usize = 27;
pub const BITS_PER_TRIT: usize = 2;
pub const WORD_BITS: usize = TRITS_PER_WORD * BITS_PER_TRIT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trit {
    N = -1,
    Z = 0,
    P = 1,
}

impl Trit {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::N),
            0 => Some(Trit::Z),
            1 => Some(Trit::P),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }

    fn encode(self) -> u8 {
        match self {
            Trit::N => 0b00,
            Trit::Z => 0b01,
            Trit::P => 0b10,
        }
    }

    fn decode(bits: u8) -> Option<Self> {
        match bits & 0b11 {
            0b00 => Some(Trit::N),
            0b01 => Some(Trit::Z),
            0b10 => Some(Trit::P),
            _ => None,
        }
    }
}

pub fn pack_word(trits: &[Trit]) -> u64 {
    assert!(trits.len() <= TRITS_PER_WORD, "word overflow: {} trits", trits.len());
    let mut word: u64 = 0;
    for (i, t) in trits.iter().enumerate() {
        word |= (t.encode() as u64) << (i * BITS_PER_TRIT);
    }
    word
}

pub fn unpack_word(word: u64) -> Vec<Trit> {
    let mut trits = Vec::with_capacity(TRITS_PER_WORD);
    for i in 0..TRITS_PER_WORD {
        let bits = ((word >> (i * BITS_PER_TRIT)) & 0b11) as u8;
        trits.push(Trit::decode(bits).unwrap_or(Trit::Z));
    }
    trits
}

pub fn pack_words(all_trits: &[Trit]) -> Vec<u64> {
    all_trits.chunks(TRITS_PER_WORD).map(|chunk| pack_word(chunk)).collect()
}

pub fn unpack_words(words: &[u64]) -> Vec<Trit> {
    words.iter().flat_map(|w| unpack_word(*w)).collect()
}

pub fn parse_trit_string(s: &str) -> Option<Vec<Trit>> {
    s.split(',')
        .map(|tok| tok.trim().parse::<i8>().ok().and_then(Trit::from_i8))
        .collect()
}

pub fn format_trits(trits: &[Trit]) -> String {
    trits.iter().map(|t| t.to_i8().to_string()).collect::<Vec<_>>().join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trit_from_i8_roundtrip() {
        for v in [-1i8, 0, 1] {
            assert_eq!(Trit::from_i8(v).unwrap().to_i8(), v);
        }
    }

    #[test]
    fn trit_from_i8_rejects_invalid() {
        assert!(Trit::from_i8(2).is_none());
        assert!(Trit::from_i8(-2).is_none());
        assert!(Trit::from_i8(127).is_none());
    }

    #[test]
    fn encode_decode_roundtrip() {
        for t in [Trit::N, Trit::Z, Trit::P] {
            assert_eq!(Trit::decode(t.encode()).unwrap(), t);
        }
    }

    #[test]
    fn pack_single_trit_n() {
        assert_eq!(pack_word(&[Trit::N]), 0b00);
    }

    #[test]
    fn pack_single_trit_z() {
        assert_eq!(pack_word(&[Trit::Z]), 0b01);
    }

    #[test]
    fn pack_single_trit_p() {
        assert_eq!(pack_word(&[Trit::P]), 0b10);
    }

    #[test]
    fn pack_two_trits() {
        let w = pack_word(&[Trit::P, Trit::N]);
        assert_eq!(w, 0b00_10);
    }

    #[test]
    fn unpack_two_trits() {
        let trits = unpack_word(0b00_10);
        assert_eq!(trits[0], Trit::P);
        assert_eq!(trits[1], Trit::N);
    }

    #[test]
    fn pack_unpack_roundtrip_full_word() {
        let original: Vec<Trit> = (0..TRITS_PER_WORD)
            .map(|i| Trit::from_i8((i % 3) as i8 - 1).unwrap())
            .collect();
        let packed = pack_word(&original);
        let unpacked = unpack_word(packed);
        assert_eq!(original, unpacked);
    }

    #[test]
    fn pack_words_chunks_correctly() {
        let trits: Vec<Trit> = (0..TRITS_PER_WORD * 2 + 5)
            .map(|i| Trit::from_i8((i % 3) as i8 - 1).unwrap())
            .collect();
        let words = pack_words(&trits);
        assert_eq!(words.len(), 3);
    }

    #[test]
    fn pack_unpack_words_roundtrip() {
        let trits: Vec<Trit> = (0..TRITS_PER_WORD * 3)
            .map(|i| Trit::from_i8((i % 3) as i8 - 1).unwrap())
            .collect();
        let words = pack_words(&trits);
        let back = unpack_words(&words);
        assert_eq!(trits, back);
    }

    #[test]
    fn parse_trit_string_valid() {
        let trits = parse_trit_string("-1,0,1,0,-1").unwrap();
        assert_eq!(trits, vec![Trit::N, Trit::Z, Trit::P, Trit::Z, Trit::N]);
    }

    #[test]
    fn parse_trit_string_rejects_invalid() {
        assert!(parse_trit_string("1,2,3").is_none());
    }

    #[test]
    fn format_trits_matches_parse() {
        let original = vec![Trit::N, Trit::Z, Trit::P];
        let s = format_trits(&original);
        let parsed = parse_trit_string(&s).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn empty_pack_words() {
        let words = pack_words(&[]);
        assert!(words.is_empty());
    }

    #[test]
    fn unpack_reserved_bits_defaults_to_z() {
        let trits = unpack_word(0b11);
        assert_eq!(trits[0], Trit::Z);
    }

    #[test]
    fn trit_constants() {
        assert_eq!(TRITS_PER_WORD, 27);
        assert_eq!(BITS_PER_TRIT, 2);
        assert_eq!(WORD_BITS, 54);
    }
}
