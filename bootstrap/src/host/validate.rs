pub const TRITS_PER_WORD: usize = super::ternary::TRITS_PER_WORD;
pub const WORD_BITS: usize = super::ternary::WORD_BITS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiag {
    pub kind: ValidationKind,
    pub word_index: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationKind {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub total_words: usize,
    pub errors: Vec<ValidationDiag>,
    pub warnings: Vec<ValidationDiag>,
}

impl ValidationResult {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

#[derive(Debug, Clone)]
pub struct WeightValidator {
    pub check_reserved_bits: bool,
    pub check_roundtrip: bool,
    pub check_word_count_alignment: bool,
    pub words_per_neuron: usize,
}

impl WeightValidator {
    pub fn new() -> Self {
        WeightValidator {
            check_reserved_bits: true,
            check_roundtrip: true,
            check_word_count_alignment: true,
            words_per_neuron: 4,
        }
    }

    pub fn validate(&self, words: &[u64]) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if words.is_empty() {
            warnings.push(ValidationDiag {
                kind: ValidationKind::Warning,
                word_index: None,
                message: "empty word vector".to_string(),
            });
            return ValidationResult { total_words: 0, errors, warnings };
        }

        for (i, &word) in words.iter().enumerate() {
            if self.check_reserved_bits {
                let upper = word >> WORD_BITS;
                if upper != 0 {
                    errors.push(ValidationDiag {
                        kind: ValidationKind::Error,
                        word_index: Some(i),
                        message: format!("word {} has non-zero reserved bits: upper 10 bits = {:#012x}", i, upper),
                    });
                }
                for j in 0..TRITS_PER_WORD {
                    let bits = ((word >> (j * 2)) & 0b11) as u8;
                    if bits == 0b11 {
                        errors.push(ValidationDiag {
                            kind: ValidationKind::Error,
                            word_index: Some(i),
                            message: format!("word {} trit {} has invalid encoding 0b11", i, j),
                        });
                    }
                }
            }

            if self.check_roundtrip {
                let trits = super::ternary::unpack_word(word);
                let repacked = super::ternary::pack_word(&trits);
                let masked_orig = word & ((1u64 << WORD_BITS) - 1);
                if repacked != masked_orig {
                    errors.push(ValidationDiag {
                        kind: ValidationKind::Error,
                        word_index: Some(i),
                        message: format!("word {} roundtrip mismatch: orig={:#016x} masked={:#016x} repacked={:#016x}", i, word, masked_orig, repacked),
                    });
                }
            }
        }

        if self.check_word_count_alignment && self.words_per_neuron > 0 {
            let remainder = words.len() % self.words_per_neuron;
            if remainder != 0 {
                warnings.push(ValidationDiag {
                    kind: ValidationKind::Warning,
                    word_index: None,
                    message: format!("word count {} not aligned to {} words/neuron (remainder {})", words.len(), self.words_per_neuron, remainder),
                });
            }
        }

        ValidationResult { total_words: words.len(), errors, warnings }
    }
}

impl Default for WeightValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate_words(words: &[u64]) -> ValidationResult {
    WeightValidator::new().validate(words)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::ternary::{Trit, pack_word};

    fn make_valid_word() -> u64 {
        pack_word(&[Trit::N, Trit::Z, Trit::P, Trit::N, Trit::Z])
    }

    #[test]
    fn empty_words_is_warning() {
        let r = validate_words(&[]);
        assert!(r.ok());
        assert_eq!(r.warning_count(), 1);
        assert_eq!(r.total_words, 0);
    }

    #[test]
    fn single_valid_word() {
        let r = validate_words(&[make_valid_word()]);
        assert!(r.ok());
        assert_eq!(r.error_count(), 0);
        assert_eq!(r.total_words, 1);
    }

    #[test]
    fn detects_reserved_bits() {
        let bad = make_valid_word() | (1u64 << 60);
        let r = validate_words(&[bad]);
        assert!(!r.ok());
        assert!(r.errors[0].message.contains("reserved bits"));
    }

    #[test]
    fn detects_invalid_trit_11() {
        let bad = 0xFFFF_FFFF_FFFF_FFFF;
        let r = validate_words(&[bad]);
        assert!(!r.ok());
        assert!(r.errors.iter().any(|e| e.message.contains("invalid encoding 0b11")));
    }

    #[test]
    fn detects_roundtrip_mismatch() {
        let mut v = WeightValidator::new();
        v.check_reserved_bits = false;
        v.check_roundtrip = true;
        let bad = 0x00FF_0000_0000_00FF;
        let r = v.validate(&[bad]);
        assert!(!r.ok());
        assert!(r.errors[0].message.contains("roundtrip mismatch"));
    }

    #[test]
    fn alignment_warning() {
        let words = vec![make_valid_word(); 3];
        let r = validate_words(&words);
        assert!(r.ok());
        assert!(r.warnings.iter().any(|w| w.message.contains("not aligned")));
    }

    #[test]
    fn aligned_words_no_warning() {
        let words = vec![make_valid_word(); 8];
        let r = validate_words(&words);
        assert!(r.ok());
        assert!(!r.warnings.iter().any(|w| w.message.contains("not aligned")));
    }

    #[test]
    fn multiple_errors() {
        let bad1 = 0xFFFF_FFFF_FFFF_FFFF;
        let bad2 = (1u64 << 60) | make_valid_word();
        let r = validate_words(&[bad1, bad2]);
        assert!(r.error_count() > 2);
    }

    #[test]
    fn no_roundtrip_check() {
        let mut v = WeightValidator::new();
        v.check_roundtrip = false;
        v.check_reserved_bits = false;
        let r = v.validate(&[0xBAD_BAD_BAD_BAD_BAD]);
        assert!(r.ok());
    }

    #[test]
    fn all_zero_word_is_valid() {
        let r = validate_words(&[0u64]);
        assert!(r.ok());
    }

    #[test]
    fn pack_unpack_consistency() {
        let trits = vec![Trit::N, Trit::P, Trit::Z, Trit::N, Trit::N, Trit::P];
        let word = pack_word(&trits);
        let r = validate_words(&[word]);
        assert!(r.ok());
    }

    #[test]
    fn diag_has_word_index() {
        let r = validate_words(&[0xFFFF_FFFF_FFFF_FFFF]);
        assert_eq!(r.errors[0].word_index, Some(0));
    }

    #[test]
    fn result_total_words() {
        let words = vec![make_valid_word(); 7];
        let r = validate_words(&words);
        assert_eq!(r.total_words, 7);
    }

    #[test]
    fn validator_default() {
        let v = WeightValidator::default();
        assert!(v.check_reserved_bits);
        assert!(v.check_roundtrip);
    }
}
