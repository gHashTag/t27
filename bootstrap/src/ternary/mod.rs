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
pub fn encode_trits(n: i32) -> TernaryEncoding {
    // Delegate to generated implementation
    TernaryEncoding::new(n)
}

/// Decode ternary to integer
pub fn decode_trits(trits: TernaryEncoding) -> i32 {
    // Delegate to generated implementation
    trits.value()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let encoded = encode_trits(1);
        assert_eq!(decode_trits(encoded), 1);

        let encoded = encode_trits(5);
        let expected = [-1, 0, +1, 0, +1, 0];
        assert_eq!(decode_trits(encoded), expected[4]);
    }
}
