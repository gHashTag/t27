//! Raw FFI declarations for zig-crypto-mining C API.

use libc::{c_int, size_t};

/// SHA-256 hash result (32 bytes).
pub type Sha256Hash = [u8; 32];

/// Mining result with nonce and hash.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MiningResult {
    /// The nonce that produced a valid hash.
    pub nonce: u64,
    /// The resulting hash.
    pub hash: Sha256Hash,
    /// Number of hashes computed.
    pub hashes_computed: u64,
    /// Whether a valid solution was found.
    pub found: bool,
}

/// DePIN work proof result.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct DepinProof {
    /// Proof data.
    pub proof: [u8; 64],
    /// Challenge nonce.
    pub challenge: u64,
    /// Validator public key hash.
    pub validator_hash: Sha256Hash,
    /// Whether proof is valid.
    pub valid: bool,
}

extern "C" {
    /// Compute SHA-256 hash of data.
    #[allow(dead_code)]
    pub fn _c_abi_crypto_sha256(data: *const u8, len: size_t, out_hash: *mut Sha256Hash) -> c_int;

    /// Mine a block header with given difficulty target.
    #[allow(dead_code)]
    pub fn _c_abi_crypto_mine_sha256d(
        header: *const u8,
        target: *const Sha256Hash,
        start_nonce: u64,
        max_nonce: u64,
        out_result: *mut MiningResult,
    ) -> c_int;

    /// Compute SHA-256 hash of data (no prefix - for compatibility).
    #[allow(dead_code)]
    pub fn crypto_sha256(data: *const u8, len: size_t, out_hash: *mut Sha256Hash) -> c_int;

    /// Mine a block header with given difficulty target (no prefix - for compatibility).
    #[allow(dead_code)]
    pub fn crypto_mine_sha256d(
        header: *const u8,
        target: *const Sha256Hash,
        start_nonce: u64,
        max_nonce: u64,
        out_result: *mut MiningResult,
    ) -> c_int;

    /// Generate a DePIN proof-of-work for given challenge.
    #[allow(dead_code)]
    pub fn crypto_depin_prove(
        challenge: u64,
        worker_id: *const u8,
        worker_id_len: size_t,
        out_proof: *mut DepinProof,
    ) -> c_int;

    /// Verify a DePIN proof-of-work.
    #[allow(dead_code)]
    pub fn crypto_depin_verify(proof: *const DepinProof) -> bool;
}
