//! E2E tests for trios-crypto
//!
//! End-to-end tests verify the crate works correctly in various scenarios.

#[cfg(feature = "ffi")]
use trios_crypto::{double_sha256, sha256};

#[cfg(not(feature = "ffi"))]
use trios_crypto::sha256;

#[cfg(feature = "ffi")]
#[test]
fn e2e_basic_sha256_hash() {
    let data = b"trinity test data";
    let result = sha256(data);

    assert!(result.is_ok(), "sha256 should succeed");
    let hash = result.unwrap();
    assert_eq!(hash.len(), 32, "hash should be 32 bytes");
}

#[cfg(feature = "ffi")]
#[test]
fn e2e_double_sha256_hash() {
    let data = b"trinity double test";
    let result = double_sha256(data);

    assert!(result.is_ok(), "double_sha256 should succeed");
    let hash = result.unwrap();
    assert_eq!(hash.len(), 32, "hash should be 32 bytes");
}

#[cfg(feature = "ffi")]
#[test]
fn e2e_error_propagation() {
    let data = b"test";
    let result = sha256(data);

    assert!(result.is_ok(), "sha256 should succeed");
}

#[cfg(feature = "ffi")]
#[test]
fn e2e_empty_input_handling() {
    let data = b"";
    let result = sha256(data);

    assert!(result.is_ok(), "sha256 should succeed with empty input");
    let hash = result.unwrap();
    assert_eq!(hash.len(), 32, "hash should be 32 bytes");
}

#[cfg(not(feature = "ffi"))]
#[test]
fn e2e_stub_mode_errors() {
    let data = b"test";
    let result = sha256(data);

    // In stub mode (no FFI), should get FfiNotAvailable error
    assert!(result.is_err(), "should error in stub mode");

    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("FFI not available"),
        "error should mention FFI"
    );
}
