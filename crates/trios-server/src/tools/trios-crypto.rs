//! MCP tools for trios-crypto crate
//!
//! Exposes SHA-256 mining operations through the Model Context Protocol.

use anyhow::{bail, Context, Result};
use serde_json::Value;
use trios_crypto::{mine_sha256d, MiningError, MiningResult, Sha256Hash};

const SHA256_HEADER_SIZE: usize = 80;

/// Dispatch crypto tools.
pub async fn dispatch(name: &str, input: &Value) -> Option<Result<Value>> {
    match name {
        "crypto_mine_sha256" => Some(crypto_mine_sha256(input).await),
        _ => None,
    }
}

/// Mine a SHA-256 block header (80 bytes) with given difficulty target.
async fn crypto_mine_sha256(input: &Value) -> Result<Value> {
    // Get data bytes (first 76 bytes of header)
    let Some(Value::Array(data_bytes)) = input.get("data") else {
        bail!("data is required and must be an array");
    };
    if data_bytes.len() > 76 {
        bail!("data must be exactly 76 bytes");
    }

    // Get target hash
    let Some(Value::Array(target_bytes)) = input.get("target") else {
        bail!("target is required and must be an array");
    };
    if target_bytes.len() != 32 {
        bail!("target must be exactly 32 bytes");
    }

    let start_nonce = input
        .get("start_nonce")
        .and_then(|v| v.as_u64())
        .context("start_nonce is required")?;

    let max_nonce = input
        .get("max_nonce")
        .and_then(|v| v.as_u64())
        .context("max_nonce is required")?;

    // Get nonce value
    let Some(Value::Number(n)) = input.get("nonce") else {
        bail!("nonce is required");
    };
    let Some(nonce_val) = n.as_u64() else {
        bail!("nonce must be a valid u64");
    };

    // Build the 80-byte header
    let mut header = [0u8; SHA256_HEADER_SIZE];

    // Copy data bytes (first 76 bytes)
    for (i, byte) in data_bytes.iter().enumerate().take(76) {
        if let Some(b) = byte.as_u64() {
            header[i] = b as u8;
        }
    }

    // Copy nonce bytes (big-endian, last 8 bytes)
    let nonce_bytes = nonce_val.to_be_bytes();
    let nonce_offset = SHA256_HEADER_SIZE - 8;
    for (i, byte) in nonce_bytes.iter().enumerate() {
        header[nonce_offset + i] = *byte;
    }

    // Build target hash
    let mut target_hash = [0u8; 32];
    for (i, byte) in target_bytes.iter().enumerate() {
        if let Some(b) = byte.as_u64() {
            target_hash[i] = b as u8;
        }
    }

    // Call FFI function
    let result: Result<MiningResult, MiningError> = mine_sha256d(
        &header,
        &Sha256Hash { data: target_hash },
        start_nonce,
        max_nonce,
    ).await;

    match result {
        Ok(mining_result) => {
            Ok(serde_json::json!({
                "nonce": mining_result.nonce,
                "hash": hex::encode(mining_result.hash.data),
                "hashes_computed": mining_result.hashes_computed,
                "found": mining_result.found,
            }))
        }
        Err(e) => Err(anyhow::anyhow!("mining failed: {}", e)),
    }
}
