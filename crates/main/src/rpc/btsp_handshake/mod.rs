// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP Phase 2 — Handshake-on-accept for UDS listeners.
//!
//! Implements the server-side BTSP handshake per `BTSP_PROTOCOL_STANDARD.md` v1.0.
//! When `FAMILY_ID` is set (production mode), every incoming socket connection
//! must complete the 4-step challenge-response before any JSON-RPC frames are
//! processed.
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

use btsp_handshake_wire::{BTSP_VERSION, HANDSHAKE_TIMEOUT, read_message, write_message};

#[cfg(test)]
mod btsp_handshake_tests;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
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
    if let Ok(path) = std::env::var("BTSP_PROVIDER_SOCKET") {
        let p = PathBuf::from(&path);
        if p.exists() {
            debug!(path = %p.display(), "BTSP provider from env");
            return Ok(p);
        }
        warn!(path = %path, "BTSP_PROVIDER_SOCKET set but socket does not exist");
    }

    if let Ok(path) = std::env::var("BTSP_CAPABILITY_SOCKET")
        && !path.is_empty()
    {
        let p = PathBuf::from(&path);
        if p.exists() {
            debug!(path = %p.display(), "BTSP provider from BTSP_CAPABILITY_SOCKET");
            return Ok(p);
        }
        warn!(path = %p.display(), "BTSP_CAPABILITY_SOCKET set but socket does not exist");
    }

    if let Ok(path) = std::env::var("SECURITY_SOCKET")
        && !path.is_empty()
    {
        let p = PathBuf::from(path);
        if p.exists() {
            debug!(path = %p.display(), "BTSP provider from SECURITY_SOCKET");
            return Ok(p);
        }
    }

    if let Ok(path) = std::env::var("BEARDOG_SOCKET")
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
/// After successful completion with `BTSP_NULL` cipher, the transport is
/// ready for standard newline-delimited JSON-RPC.
pub async fn btsp_handshake_server<S>(stream: &mut S) -> Result<BtspSession, BtspError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let provider_socket = discover_btsp_provider()?;
    let provider = super::ipc_client::IpcClient::new(&provider_socket)
        .with_request_timeout(HANDSHAKE_TIMEOUT)
        .with_connection_timeout(std::time::Duration::from_secs(2));

    // Step 1: Read ClientHello
    let client_hello: ClientHello = tokio::time::timeout(HANDSHAKE_TIMEOUT, read_message(stream))
        .await
        .map_err(|_| BtspError::Timeout)??;

    if client_hello.version != BTSP_VERSION {
        let err = HandshakeErrorMsg {
            error: "handshake_failed".into(),
            reason: format!("unsupported BTSP version: {}", client_hello.version),
        };
        let _ = write_message(stream, &err).await;
        return Err(BtspError::HandshakeFailed(err.reason));
    }

    debug!(version = client_hello.version, "BTSP: received ClientHello");

    // Generate challenge (32 random bytes)
    let mut challenge_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut challenge_bytes);
    let challenge_b64 = BASE64.encode(challenge_bytes);

    // Step 2: Call BTSP provider btsp.session.create
    // EVOLUTION: When primalspring 0.10.0 ships, replace `family_seed_ref` with
    // mito-beacon fields from `mito_beacon_from_env()` — three-tier genetics.
    let create_params = serde_json::json!({
        "family_seed_ref": "env:FAMILY_SEED",
        "client_ephemeral_pub": client_hello.client_ephemeral_pub,
        "challenge": challenge_b64,
    });

    let create_result = provider
        .call("btsp.session.create", &create_params)
        .await
        .map_err(|e| BtspError::ProviderUnavailable(format!("btsp.session.create failed: {e}")))?;

    let session_id = create_result["session_id"]
        .as_str()
        .ok_or_else(|| BtspError::Protocol("missing session_id in create response".into()))?
        .to_string();
    let server_ephemeral_pub = create_result["server_ephemeral_pub"]
        .as_str()
        .ok_or_else(|| {
            BtspError::Protocol("missing server_ephemeral_pub in create response".into())
        })?
        .to_string();

    // Step 3: Send ServerHello
    let server_hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: server_ephemeral_pub.clone(),
        challenge: challenge_b64.clone(),
    };
    write_message(stream, &server_hello).await?;
    debug!("BTSP: sent ServerHello");

    // Step 4: Read ChallengeResponse
    let challenge_resp: ChallengeResponse =
        tokio::time::timeout(HANDSHAKE_TIMEOUT, read_message(stream))
            .await
            .map_err(|_| BtspError::Timeout)??;

    debug!(
        preferred_cipher = %challenge_resp.preferred_cipher,
        "BTSP: received ChallengeResponse"
    );

    // Step 5: Call BTSP provider btsp.session.verify
    let verify_params = serde_json::json!({
        "session_id": session_id,
        "client_response": challenge_resp.response,
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

    // Phase 2: negotiate BTSP_NULL (plaintext after authentication).
    // Full cipher negotiation (Phase 3) will use btsp.negotiate.
    let cipher = "null".to_string();

    // Step 6: Send HandshakeComplete
    let complete = HandshakeComplete {
        cipher: cipher.clone(),
        session_id: session_id.clone(),
    };
    write_message(stream, &complete).await?;

    info!(session_id = %session_id, cipher = %cipher, "BTSP handshake complete");

    Ok(BtspSession { session_id, cipher })
}

/// Conditionally run the BTSP handshake on an accepted connection.
///
/// - **Production mode** (`FAMILY_ID` set): runs the full handshake.
///   Returns `Err` if the handshake fails (connection should be dropped).
/// - **Development mode** (no `FAMILY_ID`): returns `Ok(None)` immediately.
pub async fn maybe_handshake<S>(stream: &mut S) -> Result<Option<BtspSession>, BtspError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    if !is_btsp_required() {
        return Ok(None);
    }
    btsp_handshake_server(stream).await.map(Some)
}
