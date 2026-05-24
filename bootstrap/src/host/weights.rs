use super::ternary::{Trit, pack_words};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightPattern {
    AllN,
    AllZ,
    AllP,
    Alternating,
    PhiSequence,
    SeededRandom(u64),
}

pub struct WeightConfig {
    pub neurons: u32,
    pub chunks: u32,
    pub pattern: WeightPattern,
}

impl WeightConfig {
    pub fn total_trits(&self) -> usize {
        (self.neurons as usize) * (self.chunks as usize) * super::ternary::TRITS_PER_WORD
    }

    #[allow(dead_code)]
    pub fn total_words(&self) -> usize {
        (self.neurons as usize) * (self.chunks as usize)
    }
}

pub fn generate_pattern(pattern: WeightPattern, count: usize) -> Vec<Trit> {
    match pattern {
        WeightPattern::AllN => vec![Trit::N; count],
        WeightPattern::AllZ => vec![Trit::Z; count],
        WeightPattern::AllP => vec![Trit::P; count],
        WeightPattern::Alternating => (0..count).map(|i| match i % 3 {
            0 => Trit::N,
            1 => Trit::Z,
            _ => Trit::P,
        }).collect(),
        WeightPattern::PhiSequence => {
            let phi_trits = [Trit::N, Trit::Z, Trit::P, Trit::P, Trit::Z, Trit::N, Trit::P, Trit::Z, Trit::N];
            (0..count).map(|i| phi_trits[i % phi_trits.len()]).collect()
        },
        WeightPattern::SeededRandom(seed) => {
            let mut state = seed;
            (0..count).map(|_| {
                state = state.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(0x1);
                match (state >> 62) & 3 {
                    0 => Trit::N,
                    1 => Trit::Z,
                    _ => Trit::P,
                }
            }).collect()
        },
    }
}

pub fn generate_weights(config: &WeightConfig) -> Vec<u64> {
    let total = config.total_trits();
    let trits = generate_pattern(config.pattern, total);
    pack_words(&trits)
}

pub fn pattern_name(pattern: WeightPattern) -> &'static str {
    match pattern {
        WeightPattern::AllN => "all-n",
        WeightPattern::AllZ => "all-z",
        WeightPattern::AllP => "all-p",
        WeightPattern::Alternating => "alternating",
        WeightPattern::PhiSequence => "phi-sequence",
        WeightPattern::SeededRandom(_) => "seeded-random",
    }
}

pub fn parse_pattern(s: &str) -> Option<WeightPattern> {
    match s {
        "all-n" => Some(WeightPattern::AllN),
        "all-z" => Some(WeightPattern::AllZ),
        "all-p" => Some(WeightPattern::AllP),
        "alternating" => Some(WeightPattern::Alternating),
        "phi-sequence" => Some(WeightPattern::PhiSequence),
        other if other.starts_with("seeded-random") => {
            let seed_str = other.strip_prefix("seeded-random:")?;
            let seed = seed_str.parse::<u64>().ok()?;
            Some(WeightPattern::SeededRandom(seed))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::ternary::TRITS_PER_WORD;

    #[test]
    fn all_n_pattern() {
        let trits = generate_pattern(WeightPattern::AllN, 10);
        assert!(trits.iter().all(|t| *t == Trit::N));
    }

    #[test]
    fn all_z_pattern() {
        let trits = generate_pattern(WeightPattern::AllZ, 10);
        assert!(trits.iter().all(|t| *t == Trit::Z));
    }

    #[test]
    fn all_p_pattern() {
        let trits = generate_pattern(WeightPattern::AllP, 10);
        assert!(trits.iter().all(|t| *t == Trit::P));
    }

    #[test]
    fn alternating_pattern_cycle() {
        let trits = generate_pattern(WeightPattern::Alternating, 6);
        assert_eq!(trits, vec![Trit::N, Trit::Z, Trit::P, Trit::N, Trit::Z, Trit::P]);
    }

    #[test]
    fn phi_sequence_repeats() {
        let trits = generate_pattern(WeightPattern::PhiSequence, 18);
        assert_eq!(&trits[0..9], &trits[9..18]);
    }

    #[test]
    fn seeded_random_deterministic() {
        let t1 = generate_pattern(WeightPattern::SeededRandom(42), 100);
        let t2 = generate_pattern(WeightPattern::SeededRandom(42), 100);
        assert_eq!(t1, t2);
    }

    #[test]
    fn seeded_random_different_seeds_differ() {
        let t1 = generate_pattern(WeightPattern::SeededRandom(1), 100);
        let t2 = generate_pattern(WeightPattern::SeededRandom(2), 100);
        assert_ne!(t1, t2);
    }

    #[test]
    fn generate_weights_word_count() {
        let config = WeightConfig { neurons: 2, chunks: 4, pattern: WeightPattern::AllP };
        assert_eq!(generate_weights(&config).len(), 8);
    }

    #[test]
    fn generate_weights_total_trits() {
        let config = WeightConfig { neurons: 3, chunks: 2, pattern: WeightPattern::AllZ };
        assert_eq!(config.total_trits(), 3 * 2 * TRITS_PER_WORD);
    }

    #[test]
    fn pattern_name_roundtrip() {
        assert_eq!(pattern_name(WeightPattern::AllN), "all-n");
        assert_eq!(pattern_name(WeightPattern::AllZ), "all-z");
        assert_eq!(pattern_name(WeightPattern::AllP), "all-p");
        assert_eq!(pattern_name(WeightPattern::Alternating), "alternating");
        assert_eq!(pattern_name(WeightPattern::PhiSequence), "phi-sequence");
    }

    #[test]
    fn parse_pattern_valid() {
        assert_eq!(parse_pattern("all-n"), Some(WeightPattern::AllN));
        assert_eq!(parse_pattern("all-z"), Some(WeightPattern::AllZ));
        assert_eq!(parse_pattern("alternating"), Some(WeightPattern::Alternating));
    }

    #[test]
    fn parse_pattern_invalid() {
        assert_eq!(parse_pattern("bogus"), None);
    }

    #[test]
    fn parse_pattern_seeded() {
        assert_eq!(parse_pattern("seeded-random:12345"), Some(WeightPattern::SeededRandom(12345)));
    }

    #[test]
    fn all_n_packs_to_zero() {
        let config = WeightConfig { neurons: 1, chunks: 1, pattern: WeightPattern::AllN };
        let words = generate_weights(&config);
        assert_eq!(words[0], 0u64);
    }

    #[test]
    fn all_z_packs_to_known() {
        let config = WeightConfig { neurons: 1, chunks: 1, pattern: WeightPattern::AllZ };
        let words = generate_weights(&config);
        let expected: u64 = (0..27).fold(0u64, |acc, i| acc | (0b01u64 << (i * 2)));
        assert_eq!(words[0], expected);
    }

    #[test]
    fn empty_config() {
        let config = WeightConfig { neurons: 0, chunks: 0, pattern: WeightPattern::AllN };
        assert_eq!(config.total_trits(), 0);
        assert_eq!(generate_weights(&config).len(), 0);
    }
}
