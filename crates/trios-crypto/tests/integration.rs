//! Integration tests for trios-crypto
//!
//! Tests internal module interactions and validates type consistency.

use trios_crypto::{sha256, double_sha256, Sha256Hash};

#[test]
fn sha256_hash_returns_32_bytes() {
    let data = b"test data for hashing";
    let result = sha256(data);
    assert!(result.is_ok());
    let hash = result.unwrap();
    assert_eq!(hash.len(), 32);
}

#[test]
fn double_sha256_consistency_check() {
    let data = b"double hash test";
    let result = double_sha256(data);
    assert!(result.is_ok());
    let hash = result.unwrap();
    assert_eq!(hash.len(), 32);
}

#[test]
fn sha256_empty_input() {
    let data = b"";
    let result = sha256(data);
    assert!(result.is_ok());
    let hash = result.unwrap();
    assert_eq!(hash.len(), 32);
}

#[test]
fn double_sha256_empty_input() {
    let data = b"";
    let result = double_sha256(data);
    assert!(result.is_ok());
    let hash = result.unwrap();
    assert_eq!(hash.len(), 32);
}
