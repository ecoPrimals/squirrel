// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP client-side handshake for connecting to bearDog in strict mode.
//!
//! When `BEARDOG_UDS_REQUIRE_BTSP=1` is set (sporeGate LIVE, eastGate next),
//! bearDog rejects plain JSON-RPC with `-32600`. This module implements the
//! consumer-side 4-step BTSP handshake so squirrel can authenticate before
//! sending `secrets.*` requests.
//!
//! The challenge response uses LOCAL HMAC-SHA256 with the family seed — no
//! chicken-and-egg: squirrel computes HMAC locally to authenticate itself TO
//! bearDog, then sends JSON-RPC over the authenticated session.
//!
//! ## Wire Format (NDJSON — newline-delimited)
//!
//! ```text
//! 1. Send  ClientHello       { protocol: "btsp", version: 1, client_ephemeral_pub }
//! 2. Read  ServerHello       { version, server_ephemeral_pub, challenge, session_id }
//! 3. Send  ChallengeResponse { response, preferred_cipher }
//! 4. Read  HandshakeComplete { cipher, session_id }
//! ```
//!
//! Reference: `primals/songBird/crates/songbird-crypto-provider/src/btsp_client.rs`

use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, warn};

type HmacSha256 = Hmac<Sha256>;

const BTSP_VERSION: u8 = 1;
const PREFERRED_CIPHER: &str = "chacha20_poly1305";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_millis(1500);

// ── Wire types (NDJSON form) ────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ClientHello {
    protocol: &'static str,
    version: u8,
    client_ephemeral_pub: String,
}

#[derive(Debug, Deserialize)]
struct ServerHello {
    #[expect(dead_code, reason = "validated implicitly by successful parse")]
    version: u8,
    #[expect(dead_code, reason = "used by session key derivation in Phase 3")]
    server_ephemeral_pub: String,
    challenge: String,
    session_id: String,
}

#[derive(Debug, Serialize)]
struct ChallengeResponse {
    response: String,
    preferred_cipher: &'static str,
}

#[derive(Debug, Deserialize)]
struct HandshakeComplete {
    cipher: String,
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct HandshakeError {
    #[expect(dead_code, reason = "logged but not matched on")]
    error: String,
    reason: String,
}

/// Result of a successful client-side BTSP handshake.
#[derive(Debug, Clone)]
pub struct BtspClientSession {
    /// Unique session identifier from the server.
    pub session_id: String,
    /// Negotiated cipher suite (e.g. `chacha20_poly1305` or `null`).
    pub cipher: String,
}

/// Errors from the BTSP client handshake.
#[derive(Debug, thiserror::Error)]
pub enum BtspClientError {
    /// Family seed not available in environment.
    #[error("FAMILY_SEED not available — cannot perform BTSP handshake")]
    NoFamilySeed,
    /// I/O error on the stream during handshake.
    #[error("I/O error during BTSP handshake: {0}")]
    Io(#[from] std::io::Error),
    /// Server explicitly rejected the handshake (bad family seed, etc.).
    #[error("Server rejected handshake: {0}")]
    Rejected(String),
    /// Malformed or unexpected response from server.
    #[error("Malformed server response: {0}")]
    Protocol(String),
    /// HMAC computation failed (invalid key length or RNG failure).
    #[error("HMAC computation failed")]
    Hmac,
    /// Handshake step timed out.
    #[error("BTSP handshake timed out")]
    Timeout,
}

/// Resolve the raw family seed from environment.
///
/// Checks `FAMILY_SEED` → `BEARDOG_FAMILY_SEED` → `BIOMEOS_FAMILY_SEED`.
fn resolve_family_seed_raw() -> Option<String> {
    use universal_constants::env_vars;
    std::env::var(env_vars::security::FAMILY_SEED)
        .or_else(|_| {
            std::env::var(env_vars::primals::BEARDOG_FAMILY_SEED).inspect(|_| {
                warn!("BEARDOG_FAMILY_SEED is deprecated — use FAMILY_SEED");
            })
        })
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_SEED"))
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Check whether BTSP strict mode is expected (bearDog requires handshake).
///
/// Returns `true` if `BEARDOG_UDS_REQUIRE_BTSP=1` or `BTSP_STRICT_MODE=1`.
#[must_use]
pub fn btsp_strict_mode_expected() -> bool {
    std::env::var("BEARDOG_UDS_REQUIRE_BTSP")
        .or_else(|_| std::env::var("BTSP_STRICT_MODE"))
        .is_ok_and(|v| v.trim() == "1")
}

/// Perform the client-side BTSP handshake over an NDJSON stream.
///
/// Authenticates to bearDog using the family seed from environment.
/// After success, the stream is ready for JSON-RPC traffic.
pub async fn perform_client_handshake<S>(
    stream: &mut S,
) -> Result<BtspClientSession, BtspClientError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let family_seed = resolve_family_seed_raw().ok_or(BtspClientError::NoFamilySeed)?;

    // Generate ephemeral key material (32 random bytes)
    let mut ephemeral_key = [0u8; 32];
    getrandom::fill(&mut ephemeral_key).map_err(|_| BtspClientError::Hmac)?;

    // Step 1: Send ClientHello
    let hello = ClientHello {
        protocol: "btsp",
        version: BTSP_VERSION,
        client_ephemeral_pub: BASE64_STANDARD.encode(ephemeral_key),
    };
    let hello_json = serde_json::to_string(&hello)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ClientHello: {e}")))?;
    stream
        .write_all(hello_json.as_bytes())
        .await
        .map_err(BtspClientError::Io)?;
    stream.write_all(b"\n").await.map_err(BtspClientError::Io)?;
    stream.flush().await.map_err(BtspClientError::Io)?;

    debug!("BTSP client: sent ClientHello");

    // Step 2: Read ServerHello (or HandshakeError)
    let mut buf_reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    tokio::time::timeout(HANDSHAKE_TIMEOUT, buf_reader.read_line(&mut line))
        .await
        .map_err(|_| BtspClientError::Timeout)?
        .map_err(BtspClientError::Io)?;

    if line.trim().is_empty() {
        return Err(BtspClientError::Protocol(String::from(
            "empty response from server",
        )));
    }

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(err.reason));
    }

    let server_hello: ServerHello = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse ServerHello: {e}")))?;

    debug!(
        session_id = %server_hello.session_id,
        "BTSP client: received ServerHello"
    );

    // Step 3: Compute HMAC-SHA256(family_seed, challenge) and send ChallengeResponse
    let challenge_bytes = BASE64_STANDARD
        .decode(&server_hello.challenge)
        .map_err(|e| BtspClientError::Protocol(format!("decode challenge: {e}")))?;

    let mut mac = HmacSha256::new_from_slice(family_seed.trim().as_bytes())
        .map_err(|_| BtspClientError::Hmac)?;
    mac.update(&challenge_bytes);
    let hmac_result = mac.finalize().into_bytes();

    let response = ChallengeResponse {
        response: BASE64_STANDARD.encode(hmac_result),
        preferred_cipher: PREFERRED_CIPHER,
    };
    let resp_json = serde_json::to_string(&response)
        .map_err(|e| BtspClientError::Protocol(format!("serialize ChallengeResponse: {e}")))?;

    let stream = buf_reader.into_inner();
    stream
        .write_all(resp_json.as_bytes())
        .await
        .map_err(BtspClientError::Io)?;
    stream.write_all(b"\n").await.map_err(BtspClientError::Io)?;
    stream.flush().await.map_err(BtspClientError::Io)?;

    debug!("BTSP client: sent ChallengeResponse");

    // Step 4: Read HandshakeComplete (or HandshakeError)
    let mut buf_reader = BufReader::new(&mut *stream);
    let mut line = String::new();
    tokio::time::timeout(HANDSHAKE_TIMEOUT, buf_reader.read_line(&mut line))
        .await
        .map_err(|_| BtspClientError::Timeout)?
        .map_err(BtspClientError::Io)?;

    if line.contains("\"error\"") && line.contains("\"reason\"") {
        let err: HandshakeError = serde_json::from_str(line.trim())
            .map_err(|e| BtspClientError::Protocol(format!("parse error response: {e}")))?;
        return Err(BtspClientError::Rejected(err.reason));
    }

    let complete: HandshakeComplete = serde_json::from_str(line.trim())
        .map_err(|e| BtspClientError::Protocol(format!("parse HandshakeComplete: {e}")))?;

    debug!(
        session_id = %complete.session_id,
        cipher = %complete.cipher,
        "BTSP client: handshake COMPLETE"
    );

    Ok(BtspClientSession {
        session_id: complete.session_id,
        cipher: complete.cipher,
    })
}

/// Conditionally perform the BTSP handshake.
///
/// - If strict mode is expected AND the family seed is available, runs the
///   full handshake. Returns `Ok(Some(session))` on success.
/// - If strict mode is not expected, returns `Ok(None)` — plain JSON-RPC.
/// - If strict mode is expected but the family seed is missing, returns `Err`.
pub async fn maybe_client_handshake<S>(
    stream: &mut S,
) -> Result<Option<BtspClientSession>, BtspClientError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if !btsp_strict_mode_expected() {
        return Ok(None);
    }
    perform_client_handshake(stream).await.map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btsp_strict_mode_default_off() {
        temp_env::with_vars_unset(["BEARDOG_UDS_REQUIRE_BTSP", "BTSP_STRICT_MODE"], || {
            assert!(!btsp_strict_mode_expected());
        });
    }

    #[test]
    fn btsp_strict_mode_on_when_set() {
        temp_env::with_var("BEARDOG_UDS_REQUIRE_BTSP", Some("1"), || {
            assert!(btsp_strict_mode_expected());
        });
    }

    #[test]
    fn btsp_strict_mode_alt_env() {
        temp_env::with_vars(
            [
                ("BEARDOG_UDS_REQUIRE_BTSP", None::<&str>),
                ("BTSP_STRICT_MODE", Some("1")),
            ],
            || {
                assert!(btsp_strict_mode_expected());
            },
        );
    }

    #[test]
    fn resolve_family_seed_returns_none_when_unset() {
        temp_env::with_vars_unset(
            ["FAMILY_SEED", "BEARDOG_FAMILY_SEED", "BIOMEOS_FAMILY_SEED"],
            || {
                assert!(resolve_family_seed_raw().is_none());
            },
        );
    }

    #[test]
    fn resolve_family_seed_prefers_family_seed() {
        temp_env::with_vars(
            [
                ("FAMILY_SEED", Some("primary")),
                ("BEARDOG_FAMILY_SEED", Some("fallback")),
            ],
            || {
                assert_eq!(resolve_family_seed_raw(), Some("primary".to_string()));
            },
        );
    }

    #[test]
    fn resolve_family_seed_empty_returns_none() {
        temp_env::with_var("FAMILY_SEED", Some(""), || {
            assert!(resolve_family_seed_raw().is_none());
        });
    }

    #[test]
    fn hmac_sha256_produces_32_bytes() {
        let key = b"test-family-seed";
        let challenge = b"random-challenge-data";
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(challenge);
        let result = mac.finalize().into_bytes();
        assert_eq!(result.len(), 32);
    }

    #[test]
    fn client_hello_serializes_correctly() {
        let hello = ClientHello {
            protocol: "btsp",
            version: 1,
            client_ephemeral_pub: String::from("AAAA"),
        };
        let json = serde_json::to_string(&hello).unwrap();
        assert!(json.contains("\"protocol\":\"btsp\""));
        assert!(json.contains("\"version\":1"));
    }

    #[test]
    fn challenge_response_serializes_correctly() {
        let resp = ChallengeResponse {
            response: String::from("hmac-output"),
            preferred_cipher: PREFERRED_CIPHER,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"preferred_cipher\":\"chacha20_poly1305\""));
    }

    #[test]
    fn maybe_handshake_skips_when_not_strict() {
        temp_env::with_vars_unset(["BEARDOG_UDS_REQUIRE_BTSP", "BTSP_STRICT_MODE"], || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            let result = rt.block_on(async {
                let (mut client, _server) = tokio::io::duplex(1024);
                maybe_client_handshake(&mut client).await
            });
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        });
    }
}
