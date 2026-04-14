// bootstrap/src/jwt.rs
// JWT token generation and verification for sandbox access

use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

/// Default JWT secret (use environment variable in production)
const DEFAULT_JWT_SECRET: &[u8] = b"t27-sandbox-secret";

/// Sandbox JWT claims
#[derive(Clone, Serialize, Deserialize)]
struct SandboxClaims {
    sub: String,      // session id
    role: String,     // "sandbox"
    exp: usize,       // expiration timestamp (seconds since epoch)
    iat: usize,       // issued at timestamp
}

/// Get the JWT secret from environment or use default
fn get_jwt_secret() -> Vec<u8> {
    env::var("SANDBOX_JWT_SECRET")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_vec())
}

/// Create a JWT token for sandbox access
///
/// # Arguments
/// * `session_id` - The session ID to encode in the token
/// * `hours_until_expiry` - Number of hours until the token expires (default: 24)
///
/// # Returns
/// A JWT token string
pub fn create_sandbox_token(session_id: &str, hours_until_expiry: Option<i64>) -> Result<String> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(
            hours_until_expiry.unwrap_or(24)
        ))
        .unwrap()
        .timestamp() as usize;

    let issued_at = chrono::Utc::now().timestamp() as usize;

    let claims = SandboxClaims {
        sub: session_id.to_string(),
        role: "sandbox".to_string(),
        exp: expiration,
        iat: issued_at,
    };

    let secret = get_jwt_secret();

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret),
    )?;

    Ok(token)
}

/// Verify a sandbox JWT token and extract the session ID
///
/// # Arguments
/// * `token` - The JWT token to verify
///
/// # Returns
/// The session ID from the token
///
/// # Errors
/// Returns an error if the token is invalid, expired, or malformed
pub fn verify_sandbox_token(token: &str) -> Result<String> {
    let secret = get_jwt_secret();

    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    let token_data = decode::<SandboxClaims>(
        token,
        &DecodingKey::from_secret(&secret),
        &validation,
    )?;

    // Verify role is "sandbox"
    if token_data.claims.role != "sandbox" {
        return Err(anyhow::anyhow!("Invalid token role: expected 'sandbox'"));
    }

    Ok(token_data.claims.sub)
}

/// Extract the session ID from a token without verifying expiration
/// (useful for logging/debugging, but not for authorization)
///
/// # Arguments
/// * `token` - The JWT token to decode
///
/// # Returns
/// The session ID from the token, or an error if malformed
pub fn extract_session_id_unsafe(token: &str) -> Result<String> {
    let secret = get_jwt_secret();

    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = false; // Skip expiration check

    let token_data = decode::<SandboxClaims>(
        token,
        &DecodingKey::from_secret(&secret),
        &validation,
    )?;

    Ok(token_data.claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation_and_verification() {
        let session_id = "test_session_123";
        let token = create_sandbox_token(session_id, Some(1)).unwrap();
        let extracted = verify_sandbox_token(&token).unwrap();
        assert_eq!(extracted, session_id);
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_sandbox_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_expiry() {
        let session_id = "test_session_expiry";
        // Create token that expires in the past
        let secret = get_jwt_secret();
        let past_time = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::hours(1))
            .unwrap()
            .timestamp() as usize;

        let claims = SandboxClaims {
            sub: session_id.to_string(),
            role: "sandbox".to_string(),
            exp: past_time,
            iat: past_time,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&secret),
        ).unwrap();

        let result = verify_sandbox_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_session_id_unsafe() {
        let session_id = "test_session_unsafe";
        let token = create_sandbox_token(session_id, Some(1)).unwrap();
        let extracted = extract_session_id_unsafe(&token).unwrap();
        assert_eq!(extracted, session_id);
    }
}
