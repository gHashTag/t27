//! # trios-crypto
//!
//! Safe Rust wrapper around [zig-crypto-mining](https://github.com/gHashTag/zig-crypto-mining),
//! providing Bitcoin mining primitives and DePIN proof-of-work.
//!
//! ## Example
//!
//! ```ignore
//! use trios_crypto::{sha256, double_sha256};
//!
//! let hash = sha256(b"hello world");
//! println!("SHA-256: {:?}", hash);
//! ```

mod ffi;

pub use ffi::{DepinProof, MiningResult, Sha256Hash};

/// Compute SHA-256 hash of data.
pub fn sha256(data: &[u8]) -> Result<Sha256Hash, String> {
    let mut hash = [0u8; 32];
    let rc = unsafe { ffi::crypto_sha256(data.as_ptr(), data.len(), &mut hash) };
    if rc == 0 {
        Ok(hash)
    } else {
        Err(format!("sha256 failed with code {rc}"))
    }
}

/// Compute double SHA-256 (Bitcoin standard hash).
pub fn double_sha256(data: &[u8]) -> Result<Sha256Hash, String> {
    let mut hash = [0u8; 32];
    let rc = unsafe { ffi::crypto_double_sha256(data.as_ptr(), data.len(), &mut hash) };
    if rc == 0 {
        Ok(hash)
    } else {
        Err(format!("double_sha256 failed with code {rc}"))
    }
}

/// Mine a block header using SHA-256d.
///
/// - `header`: 80-byte block header
/// - `target`: difficulty threshold hash
/// - `start_nonce`: beginning of nonce range
/// - `max_nonce`: end of nonce range
pub fn mine_sha256d(
    header: &[u8],
    target: &Sha256Hash,
    start_nonce: u64,
    max_nonce: u64,
) -> Result<MiningResult, String> {
    if header.len() != 80 {
        return Err("block header must be exactly 80 bytes".into());
    }
    let mut result = MiningResult {
        nonce: 0,
        hash: [0u8; 32],
        hashes_computed: 0,
        found: false,
    };
    let rc = unsafe {
        ffi::crypto_mine_sha256d(
            header.as_ptr(),
            target,
            start_nonce,
            max_nonce,
            &mut result,
        )
    };
    if rc == 0 {
        Ok(result)
    } else {
        Err(format!("mine_sha256d failed with code {rc}"))
    }
}

/// Generate a DePIN proof-of-work for a given challenge.
pub fn depin_prove(challenge: u64, worker_id: &[u8]) -> Result<DepinProof, String> {
    let mut proof = DepinProof {
        proof: [0u8; 64],
        challenge,
        validator_hash: [0u8; 32],
        valid: false,
    };
    let rc = unsafe {
        ffi::crypto_depin_prove(challenge, worker_id.as_ptr(), worker_id.len(), &mut proof)
    };
    if rc == 0 {
        Ok(proof)
    } else {
        Err(format!("depin_prove failed with code {rc}"))
    }
}

/// Verify a DePIN proof-of-work.
pub fn depin_verify(proof: &DepinProof) -> bool {
    unsafe { ffi::crypto_depin_verify(proof) }
}

/// Get estimated hashrate for current hardware (MH/s).
pub fn estimate_hashrate() -> f64 {
    unsafe { ffi::crypto_estimate_hashrate() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires zig-crypto-mining vendor submodule"]
    fn sha256_hello_world() {
        let hash = sha256(b"hello world").unwrap();
        // Known SHA-256 of "hello world"
        let expected: [u8; 32] = [
            0xb9, 0x4d, 0x27, 0xb9, 0x93, 0x4d, 0x3e, 0x08,
            0xa5, 0x2e, 0x52, 0xd7, 0xda, 0x7d, 0xab, 0xfa,
            0xc4, 0x84, 0xef, 0xe3, 0x7a, 0x53, 0x80, 0xee,
            0x90, 0x88, 0xf7, 0xac, 0xe2, 0xef, 0xcd, 0xe9,
        ];
        assert_eq!(hash, expected);
    }
}
