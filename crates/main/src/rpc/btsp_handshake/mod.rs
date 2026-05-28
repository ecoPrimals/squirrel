// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP Phase 2 — Handshake-on-accept for UDS listeners with auto-detect.
//!
//! Implements the server-side BTSP handshake per `BTSP_PROTOCOL_STANDARD.md` v1.0.
//! When `FAMILY_ID` is set (production mode), every incoming socket connection
//! is inspected via first-byte auto-detect:
//!
//! - **`{` (0x7B)** → read the full first line. If it is a BTSP `ClientHello` (contains
//!   both `"protocol"` and `"btsp"`), the JSON-line form is accepted and the handshake
//!   runs. Otherwise the line is treated as plain JSON-RPC (unauthenticated) per PG-14
//!   (e.g. `health.liveness` probes from springs).
//! - **Any other byte** → BTSP binary framing. The 4-step challenge-response
//!   must complete before JSON-RPC frames are processed.
//!
//! The handshake crypto is delegated to the BTSP provider's `btsp.session.*` JSON-RPC
//! methods (handshake-as-a-service; typically the security primal). Squirrel does not hold the family seed.
//!
//! ## Wire format
//!
//! Handshake frames use the BTSP length-prefixed format:
//! `[4-byte BE length][JSON payload]`. After the handshake completes with
//! `BTSP_NULL` cipher, the connection reverts to newline-delimited JSON-RPC
//! (backward-compatible per standard §Wire Framing).
//!
//! ## Evolution roadmap
//!
//! - **Three-tier genetics (mito-beacon):** When `primalspring >= 0.10.0` ships,
//!   `family_seed_ref` in `btsp.session.create` evolves to `mito_beacon` fields
//!   (`family_id`, `beacon_id`, `seed`). See `DARK_FOREST_BEACON_GENETICS_STANDARD.md`.
//!   Nuclear genetics will gate AI provider routing (permission-scoped).
//! - **Phase 3 cipher negotiation:** When BearDog server-side `btsp.negotiate`
//!   is ready, extend post-handshake to select `BTSP_CHACHA20_POLY1305` /
//!   `BTSP_HMAC_PLAIN` cipher suites instead of `BTSP_NULL`.
//!
//! ## Sequence
//!
//! ```text
//! Client ──ClientHello──▶ Server
//! Client ◀──ServerHello── Server  (via BTSP provider btsp.session.create)
//! Client ──ChallengeResp─▶ Server
//! Server verifies via BTSP provider btsp.session.verify
//! Client ◀──Complete───── Server
//! ═══ Authenticated ═══
//! ```

mod btsp_handshake_wire;

pub use btsp_handshake_wire::{
    BtspError, BtspSession, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeErrorMsg,
    ServerHello,
};

use btsp_handshake_wire::{
    BTSP_VERSION, handshake_timeout, read_json_line_msg, read_message,
    read_message_with_first_byte, write_json_line, write_message,
};

#[cfg(test)]
mod btsp_handshake_tests;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, info, warn};

/// Basename stem for well-known security capability sockets under [`universal_constants::network::get_socket_dir`].
const SECURITY_CAPABILITY_SOCKET_STEM: &str = "security";

// ── Configuration ───────────────────────────────────────────────────────

/// Whether the current process requires BTSP handshake on incoming connections.
///
/// Returns `true` when `FAMILY_ID` is set to a non-default value and
/// `BIOMEOS_INSECURE` is not enabled (i.e., production mode per
/// `BTSP_PROTOCOL_STANDARD` §Security Model).
#[must_use]
pub fn is_btsp_required() -> bool {
    let config = super::unix_socket::SocketConfig::from_env();
    config
        .family_id
        .as_deref()
        .is_some_and(|v| !v.is_empty() && v != "default")
        && !config.biomeos_insecure.unwrap_or(false)
}

// ── Provider discovery ──────────────────────────────────────────────────

/// Discover the BTSP provider socket for handshake delegation (`btsp.session.*`).
///
/// Routing is **capability- and env-first** (discovers by role, not primal identity):
///
/// 1. `BTSP_PROVIDER_SOCKET` — explicit path (orchestration override)
/// 2. `BTSP_CAPABILITY_SOCKET` — explicit path (capability-first; same role as tier-1 overrides elsewhere)
/// 3. `SECURITY_SOCKET` — shared with the security/crypto IPC stack
/// 4. `BEARDOG_SOCKET` — **legacy** filename compatibility only
/// 5. Manifest scan for any `btsp.*` capability with a socket path
/// 6. Well-known basename under [`universal_constants::network::get_socket_dir`] using
///    [`SECURITY_CAPABILITY_SOCKET_STEM`] for on-disk layout compatibility with the security stack
fn discover_btsp_provider() -> Result<PathBuf, BtspError> {
    use universal_constants::env_vars;

    if let Ok(path) = std::env::var(env_vars::btsp::PROVIDER_SOCKET) {
        let p = PathBuf::from(&path);
        if p.exists() {
            debug!(path = %p.display(), "BTSP provider from env");
            return Ok(p);
        }
        warn!(path = %path, "BTSP_PROVIDER_SOCKET set but socket does not exist");
    }

    if let Ok(path) = std::env::var(env_vars::btsp::CAPABILITY_SOCKET)
        && !path.is_empty()
    {
        let p = PathBuf::from(&path);
        if p.exists() {
            debug!(path = %p.display(), "BTSP provider from BTSP_CAPABILITY_SOCKET");
            return Ok(p);
        }
        warn!(path = %p.display(), "BTSP_CAPABILITY_SOCKET set but socket does not exist");
    }

    if let Ok(path) = std::env::var(env_vars::security::SOCKET)
        && !path.is_empty()
    {
        let p = PathBuf::from(path);
        if p.exists() {
            debug!(path = %p.display(), "BTSP provider from SECURITY_SOCKET");
            return Ok(p);
        }
    }

    if let Ok(path) = std::env::var(env_vars::primals::BEARDOG_SOCKET)
        && !path.is_empty()
    {
        let p = PathBuf::from(path);
        if p.exists() {
            debug!(path = %p.display(), "BTSP provider from legacy BEARDOG_SOCKET");
            return Ok(p);
        }
    }

    let manifests = universal_patterns::manifest_discovery::discover_manifests();
    for manifest in &manifests {
        if manifest.capabilities.iter().any(|c| c.starts_with("btsp.")) && manifest.socket.exists()
        {
            debug!(
                primal = %manifest.primal,
                socket = %manifest.socket.display(),
                "BTSP provider from manifest"
            );
            return Ok(manifest.socket.clone());
        }
    }

    let family_id = super::unix_socket::get_family_id();
    let socket_dir = universal_constants::network::get_socket_dir();
    let stem = SECURITY_CAPABILITY_SOCKET_STEM;
    let candidates: Vec<String> = if family_id.is_empty() || family_id == "default" {
        vec![format!("{stem}.sock")]
    } else {
        vec![format!("{stem}-{family_id}.sock"), format!("{stem}.sock")]
    };
    for name in &candidates {
        let path = socket_dir.join(name);
        if path.exists() {
            debug!(path = %path.display(), "BTSP provider from well-known path (legacy basename)");
            return Ok(path);
        }
    }

    Err(BtspError::ProviderUnavailable(
        "no BTSP provider socket found (set BTSP_PROVIDER_SOCKET, BTSP_CAPABILITY_SOCKET, SECURITY_SOCKET, or use manifest discovery)".into(),
    ))
}

// ── Server-side handshake ───────────────────────────────────────────────

/// Run the full BTSP handshake on an accepted connection (server side).
///
/// Delegates crypto to the BTSP provider via `btsp.session.create` and
/// `btsp.session.verify` JSON-RPC calls.
///
/// `peeked_first_byte`: when auto-detect has already consumed the first byte
/// of the BTSP length prefix, pass it here so the frame reader can
/// reconstruct the 4-byte header. `None` means no byte was pre-consumed.
///
/// After successful completion with `BTSP_NULL` cipher, the transport is
/// ready for standard newline-delimited JSON-RPC.
pub async fn btsp_handshake_server<S>(
    stream: &mut S,
    peeked_first_byte: Option<u8>,
) -> Result<BtspSession, BtspError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let timeout = handshake_timeout();
    // Step 1: Read ClientHello
    let client_hello: ClientHello = if let Some(byte) = peeked_first_byte {
        tokio::time::timeout(timeout, read_message_with_first_byte(stream, byte))
            .await
            .map_err(|_| BtspError::Timeout)??
    } else {
        tokio::time::timeout(timeout, read_message(stream))
            .await
            .map_err(|_| BtspError::Timeout)??
    };

    btsp_handshake_after_client_hello(stream, client_hello, false).await
}

/// Read `FAMILY_SEED` (or `BEARDOG_FAMILY_SEED` fallback) and return base64-encoded.
fn resolve_family_seed() -> Result<String, BtspError> {
    use universal_constants::env_vars;
    let raw = std::env::var(env_vars::security::FAMILY_SEED)
        .or_else(|_| std::env::var(env_vars::primals::BEARDOG_FAMILY_SEED))
        .map_err(|_| {
            BtspError::Protocol("FAMILY_SEED or BEARDOG_FAMILY_SEED must be set for BTSP".into())
        })?;
    Ok(BASE64.encode(raw.as_bytes()))
}

/// Continue the BTSP server handshake after `ClientHello` (binary or JSON line).
async fn btsp_handshake_after_client_hello<S>(
    stream: &mut S,
    client_hello: ClientHello,
    json_line_mode: bool,
) -> Result<BtspSession, BtspError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let provider_socket = discover_btsp_provider()?;
    let timeout = handshake_timeout();
    let provider = super::ipc_client::IpcClient::new(&provider_socket)
        .with_request_timeout(timeout)
        .with_connection_timeout(timeout.min(std::time::Duration::from_secs(2)));

    if client_hello.version != BTSP_VERSION {
        let err = HandshakeErrorMsg {
            error: "handshake_failed".into(),
            reason: format!("unsupported BTSP version: {}", client_hello.version),
        };
        let _ = write_message(stream, &err).await;
        return Err(BtspError::HandshakeFailed(err.reason));
    }

    debug!(version = client_hello.version, "BTSP: received ClientHello");

    // Step 2: Call BTSP provider btsp.session.create
    let create_params = serde_json::json!({
        "family_seed": resolve_family_seed()?,
    });

    let create_result = provider
        .call("btsp.session.create", &create_params)
        .await
        .map_err(|e| BtspError::ProviderUnavailable(format!("btsp.session.create failed: {e}")))?;

    let session_id = create_result["session_id"]
        .as_str()
        .or_else(|| create_result["session_token"].as_str())
        .ok_or_else(|| BtspError::Protocol("missing session_id in create response".into()))?
        .to_string();
    let server_ephemeral_pub = create_result["server_ephemeral_pub"]
        .as_str()
        .ok_or_else(|| {
            BtspError::Protocol("missing server_ephemeral_pub in create response".into())
        })?
        .to_string();
    let challenge_b64 = create_result["challenge"]
        .as_str()
        .ok_or_else(|| BtspError::Protocol("missing challenge in create response".into()))?
        .to_string();

    // Step 3: Send ServerHello (using BearDog's challenge)
    let server_hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: server_ephemeral_pub.clone(),
        challenge: challenge_b64.clone(),
    };
    if json_line_mode {
        write_json_line(stream, &server_hello).await?;
    } else {
        write_message(stream, &server_hello).await?;
    }
    debug!("BTSP: sent ServerHello");

    // Step 4: Read ChallengeResponse
    let challenge_resp: ChallengeResponse = if json_line_mode {
        tokio::time::timeout(timeout, read_json_line_msg(stream))
            .await
            .map_err(|_| BtspError::Timeout)??
    } else {
        tokio::time::timeout(timeout, read_message(stream))
            .await
            .map_err(|_| BtspError::Timeout)??
    };

    debug!(
        preferred_cipher = %challenge_resp.preferred_cipher,
        "BTSP: received ChallengeResponse"
    );

    // Step 5: Call BTSP provider btsp.session.verify
    let verify_params = serde_json::json!({
        "session_token": session_id,
        "response": challenge_resp.response,
        "client_ephemeral_pub": client_hello.client_ephemeral_pub,
        "server_ephemeral_pub": server_ephemeral_pub,
        "challenge": challenge_b64,
    });

    let verify_result = provider
        .call("btsp.session.verify", &verify_params)
        .await
        .map_err(|e| BtspError::ProviderUnavailable(format!("btsp.session.verify failed: {e}")))?;

    if !verify_result["verified"].as_bool().unwrap_or(false) {
        let err = HandshakeErrorMsg {
            error: "handshake_failed".into(),
            reason: "family_verification".into(),
        };
        let _ = write_message(stream, &err).await;
        return Err(BtspError::HandshakeFailed(
            "family verification failed".into(),
        ));
    }

    send_complete_and_build_session(
        stream,
        json_line_mode,
        session_id,
        &verify_result,
        client_hello.client_ephemeral_pub,
    )
    .await
}

/// Step 6: send `HandshakeComplete` and build the session struct.
async fn send_complete_and_build_session<S: AsyncWrite + Unpin>(
    stream: &mut S,
    json_line_mode: bool,
    session_id: String,
    verify_result: &serde_json::Value,
    client_ephemeral_pub: String,
) -> Result<BtspSession, BtspError> {
    let cipher = "null".to_string();
    let handshake_key = verify_result["handshake_key"].as_str().map(String::from);

    let complete = HandshakeComplete {
        cipher: cipher.clone(),
        session_id: session_id.clone(),
    };
    if json_line_mode {
        write_json_line(stream, &complete).await?;
    } else {
        write_message(stream, &complete).await?;
    }

    info!(session_id = %session_id, cipher = %cipher, "BTSP handshake complete");

    Ok(BtspSession {
        session_id,
        cipher,
        handshake_key,
        client_ephemeral_pub: Some(client_ephemeral_pub),
    })
}

/// Send a BTSP error frame to the client so it can fail fast and retry with cleartext.
///
/// Best-effort: I/O errors are ignored since the connection will be dropped anyway.
pub async fn send_error_frame<S: AsyncWrite + Unpin>(stream: &mut S, error: &BtspError) {
    let msg = HandshakeErrorMsg {
        error: "handshake_failed".into(),
        reason: error.to_string(),
    };
    let _ = write_message(stream, &msg).await;
}

/// Conditionally run the BTSP handshake on an accepted connection.
///
/// - **Development mode** (no `FAMILY_ID`): returns `Ok(None)` immediately.
/// - **Production mode** (`FAMILY_ID` set): peeks the first byte to
///   auto-detect the protocol:
///   - `{` (0x7B) → read the full first line. If it looks like a JSON-line
///     BTSP `ClientHello` (contains both `"protocol"` and `"btsp"`), run the
///     handshake. Otherwise log a warning and return [`BtspError::PlainJsonRpc`]
///     with the line so the JSON-RPC path can proceed without authentication
///     (PG-14: health probes, composition tooling).
///   - `0x00` first byte → BTSP binary framing; runs the full handshake.
///   - Any other non-`{` byte → [`BtspError::BinaryProbe`]; connection should be
///     closed gracefully (health probe, TLS client, garbled data).
///
/// Returns `Err` if the handshake fails (connection should be dropped).
pub async fn maybe_handshake<S>(stream: &mut S) -> Result<Option<BtspSession>, BtspError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    use self::btsp_handshake_wire::MAX_FRAME_SIZE;
    use tokio::io::AsyncReadExt;

    if !is_btsp_required() {
        return Ok(None);
    }

    // Auto-detect: peek the first byte to distinguish BTSP from plain JSON-RPC.
    // BTSP frames start with a 4-byte BE length prefix (first byte is typically
    // 0x00 for small payloads). JSON-RPC starts with `{` (0x7B) or whitespace.
    let mut first = [0u8; 1];
    tokio::time::timeout(handshake_timeout(), stream.read_exact(&mut first))
        .await
        .map_err(|_| BtspError::Timeout)??;

    if first[0] == b'{' {
        let mut buf = vec![b'{'];
        while buf.len() < MAX_FRAME_SIZE {
            let mut b = [0u8; 1];
            let n = tokio::time::timeout(handshake_timeout(), stream.read(&mut b))
                .await
                .map_err(|_| BtspError::Timeout)??;
            if n == 0 {
                break;
            }
            buf.push(b[0]);
            if b[0] == b'\n' {
                break;
            }
        }
        if buf.len() > MAX_FRAME_SIZE {
            return Err(BtspError::FrameTooLarge { size: buf.len() });
        }
        let first_line = String::from_utf8(buf)
            .map_err(|e| BtspError::Protocol(format!("first line is not valid UTF-8: {e}")))?;
        let trimmed = first_line.trim();
        if trimmed.contains("\"protocol\"") && trimmed.contains("\"btsp\"") {
            let client_hello: ClientHello = serde_json::from_str(trimmed)
                .map_err(|e| BtspError::Protocol(format!("BTSP JSON-line ClientHello: {e}")))?;
            return btsp_handshake_after_client_hello(stream, client_hello, true)
                .await
                .map(Some);
        }
        warn!(
            "plain JSON-RPC client connected to BTSP-guarded socket — \
             skipping handshake (PG-14 auto-detect fallback)"
        );
        return Err(BtspError::PlainJsonRpc { first_line });
    }

    // Binary preamble: first byte is NOT `{`, so this could be a BTSP length-prefix
    // frame OR a stray binary probe (HTTP, TLS, health check, garbled data).
    // BTSP frames encode a 4-byte BE u32 length; for realistic handshake payloads
    // (JSON ClientHello ~100-500 bytes) the first byte is 0x00. Accept 0x00
    // as likely-BTSP; anything else (ASCII letters, TLS 0x16, etc.) is a probe.
    if first[0] != 0x00 {
        debug!(
            first_byte = format_args!("0x{:02x}", first[0]),
            "non-BTSP binary preamble on BTSP-guarded socket — closing gracefully"
        );
        return Err(BtspError::BinaryProbe {
            first_byte: first[0],
        });
    }

    btsp_handshake_server(stream, Some(first[0]))
        .await
        .map(Some)
}
