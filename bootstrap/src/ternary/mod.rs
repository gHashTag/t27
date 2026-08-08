//! Ternary Runtime — Phase 3 Implementation
//!
//! All code is generated from .t27 specifications via t27c gen.
//!
//! Includes encoding, arithmetic, control flow, gates, and memory.

// Include generated code from gen/rust/base/
#[path = "../../gen/rust/base/ternary_encoding.rs"]
mod ternary_encoding;

#[path = "../../gen/rust/base/ternary_add.rs"]
mod ternary_add;

#[path = "../../gen/rust/base/ternary_memory.rs"]
mod ternary_memory;

#[path = "../../gen/rust/base/ternary_arithmetic.rs"]
mod ternary_arithmetic;

#[path = "../../gen/rust/base/ternary_control_flow.rs"]
mod ternary_control_flow;

#[path = "../../gen/rust/base/ternary_gates.rs"]
mod ternary_gates;

/// Public API for ternary operations
pub use ternary_encoding::{TernaryEncoding};

/// Encode an integer to balanced ternary
pub fn encode_trits(n: i64) -> TernaryEncoding {
    TernaryEncoding::new(n as i32)
}

/// Decode ternary to integer
pub fn decode_trits(trits: TernaryEncoding) -> i64 {
    trits.value() as i64
}

/// Parse string to TernaryEncoding (CLI helper)
/// Accepts format like "[-1, 0, 1]" or "[0, 1, -1]"
pub fn parse_trits(s: &str) -> Option<TernaryEncoding> {
    // Remove brackets and whitespace
    let cleaned = s.replace(['[', ']', ' '], "");

    // Parse comma-separated integers
    let trits: Vec<i64> = cleaned
        .split(',')
        .filter_map(|part| part.trim().parse().ok())
        .collect();

    if trits.len() < 3 || trits.len() > 64 {
        return None;
    }

    let mut value: i64 = 0;
    let mut power: i64 = 1;

    for trit in &trits {
        if !(-1..=1).contains(trit) {
            return None;
        }
        value += *trit * power;
        power *= 3;
    }

    Some(TernaryEncoding::new(value as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let encoded = encode_trits(1);
        assert_eq!(decode_trits(encoded), 1);

        let encoded = encode_trits(5);
        assert_eq!(decode_trits(encoded), 5);
    }
}
