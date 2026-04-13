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

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{debug, info, warn};

/// Maximum BTSP frame size: 16 MiB (`BTSP_PROTOCOL_STANDARD` §Wire Framing).
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// Handshake timeout per step (generous for local IPC + security provider round-trip).
const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// BTSP protocol version we speak.
const BTSP_VERSION: u32 = 1;

// ── Wire types (BTSP_PROTOCOL_STANDARD §Handshake Protocol) ─────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    pub version: u32,
    pub client_ephemeral_pub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    pub version: u32,
    pub server_ephemeral_pub: String,
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub response: String,
    pub preferred_cipher: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeComplete {
    pub cipher: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeErrorMsg {
    pub error: String,
    pub reason: String,
}

// ── Session result ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BtspSession {
    pub session_id: String,
    pub cipher: String,
}

// ── Error type ──────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum BtspError {
    #[error("BTSP I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("BTSP frame too large: {size} bytes (max {MAX_FRAME_SIZE})")]
    FrameTooLarge { size: usize },

    #[error("BTSP handshake timed out")]
    Timeout,

    #[error("BTSP handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("BTSP provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("BTSP protocol error: {0}")]
    Protocol(String),
}

// ── Frame I/O (BTSP_PROTOCOL_STANDARD §Wire Framing) ───────────────────

async fn read_frame<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Vec<u8>, BtspError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_SIZE {
        return Err(BtspError::FrameTooLarge { size: len });
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn write_frame<S: AsyncWrite + Unpin>(stream: &mut S, data: &[u8]) -> Result<(), BtspError> {
    let len =
        u32::try_from(data.len()).map_err(|_| BtspError::FrameTooLarge { size: data.len() })?;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(data).await?;
    stream.flush().await?;
    Ok(())
}

async fn read_message<S, T>(stream: &mut S) -> Result<T, BtspError>
where
    S: AsyncRead + Unpin,
    T: for<'de> Deserialize<'de>,
{
    let bytes = read_frame(stream).await?;
    serde_json::from_slice(&bytes)
        .map_err(|e| BtspError::Protocol(format!("invalid handshake message: {e}")))
}

async fn write_message<S, T>(stream: &mut S, msg: &T) -> Result<(), BtspError>
where
    S: AsyncWrite + Unpin,
    T: Serialize + Send + Sync,
{
    let bytes = serde_json::to_vec(msg)
        .map_err(|e| BtspError::Protocol(format!("serialization failed: {e}")))?;
    write_frame(stream, &bytes).await
}

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
///    legacy security socket stem for **on-disk layout compatibility**
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
    let stem = universal_constants::primal_names::BEARDOG;
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
    rand::thread_rng().fill_bytes(&mut challenge_bytes);
    let challenge_b64 = BASE64.encode(challenge_bytes);

    // Step 2: Call BTSP provider btsp.session.create
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};

    // ── Frame I/O ────────────────────────────────────────────────────

    #[tokio::test]
    async fn frame_roundtrip() {
        let (mut client, mut server) = duplex(4096);
        let payload = b"hello btsp";

        write_frame(&mut client, payload).await.expect("write");
        let got = read_frame(&mut server).await.expect("read");
        assert_eq!(got, payload);
    }

    #[tokio::test]
    async fn frame_rejects_oversized() {
        let (mut client, mut server) = duplex(64);
        let fake_len = (MAX_FRAME_SIZE as u32 + 1).to_be_bytes();
        client.write_all(&fake_len).await.expect("write len");

        let err = read_frame(&mut server).await.unwrap_err();
        assert!(matches!(err, BtspError::FrameTooLarge { .. }));
    }

    #[tokio::test]
    async fn frame_empty_payload() {
        let (mut client, mut server) = duplex(64);
        write_frame(&mut client, b"").await.expect("write");
        let got = read_frame(&mut server).await.expect("read");
        assert!(got.is_empty());
    }

    // ── Message serde ────────────────────────────────────────────────

    #[tokio::test]
    async fn message_roundtrip_client_hello() {
        let (mut client, mut server) = duplex(4096);
        let hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "dGVzdA==".into(),
        };

        write_message(&mut client, &hello).await.expect("write");
        let got: ClientHello = read_message(&mut server).await.expect("read");
        assert_eq!(got.version, 1);
        assert_eq!(got.client_ephemeral_pub, "dGVzdA==");
    }

    #[tokio::test]
    async fn message_roundtrip_server_hello() {
        let (mut client, mut server) = duplex(4096);
        let hello = ServerHello {
            version: 1,
            server_ephemeral_pub: "c2VydmVy".into(),
            challenge: "Y2hhbGxlbmdl".into(),
        };

        write_message(&mut client, &hello).await.expect("write");
        let got: ServerHello = read_message(&mut server).await.expect("read");
        assert_eq!(got.version, 1);
        assert_eq!(got.server_ephemeral_pub, "c2VydmVy");
        assert_eq!(got.challenge, "Y2hhbGxlbmdl");
    }

    #[tokio::test]
    async fn message_roundtrip_challenge_response() {
        let (mut client, mut server) = duplex(4096);
        let resp = ChallengeResponse {
            response: "aG1hYw==".into(),
            preferred_cipher: "null".into(),
        };

        write_message(&mut client, &resp).await.expect("write");
        let got: ChallengeResponse = read_message(&mut server).await.expect("read");
        assert_eq!(got.response, "aG1hYw==");
        assert_eq!(got.preferred_cipher, "null");
    }

    #[tokio::test]
    async fn message_roundtrip_handshake_complete() {
        let (mut client, mut server) = duplex(4096);
        let complete = HandshakeComplete {
            cipher: "null".into(),
            session_id: "abc123".into(),
        };

        write_message(&mut client, &complete).await.expect("write");
        let got: HandshakeComplete = read_message(&mut server).await.expect("read");
        assert_eq!(got.cipher, "null");
        assert_eq!(got.session_id, "abc123");
    }

    #[tokio::test]
    async fn message_roundtrip_error() {
        let (mut client, mut server) = duplex(4096);
        let err = HandshakeErrorMsg {
            error: "handshake_failed".into(),
            reason: "family_verification".into(),
        };

        write_message(&mut client, &err).await.expect("write");
        let got: HandshakeErrorMsg = read_message(&mut server).await.expect("read");
        assert_eq!(got.error, "handshake_failed");
        assert_eq!(got.reason, "family_verification");
    }

    #[tokio::test]
    async fn malformed_message_returns_protocol_error() {
        let (mut client, mut server) = duplex(4096);
        write_frame(&mut client, b"not json").await.expect("write");

        let err = read_message::<_, ClientHello>(&mut server)
            .await
            .unwrap_err();
        assert!(matches!(err, BtspError::Protocol(_)));
    }

    // ── is_btsp_required ─────────────────────────────────────────────

    #[test]
    fn btsp_not_required_without_family_id() {
        temp_env::with_vars(
            [
                ("SQUIRREL_FAMILY_ID", None::<&str>),
                ("BIOMEOS_FAMILY_ID", None::<&str>),
                ("FAMILY_ID", None::<&str>),
                ("BIOMEOS_INSECURE", None::<&str>),
            ],
            || {
                assert!(!is_btsp_required());
            },
        );
    }

    #[test]
    fn btsp_not_required_with_default_family() {
        temp_env::with_vars(
            [
                ("SQUIRREL_FAMILY_ID", None::<&str>),
                ("FAMILY_ID", Some("default")),
                ("BIOMEOS_INSECURE", None::<&str>),
            ],
            || {
                assert!(!is_btsp_required());
            },
        );
    }

    #[test]
    fn btsp_required_with_production_family() {
        temp_env::with_vars(
            [
                ("SQUIRREL_FAMILY_ID", None::<&str>),
                ("FAMILY_ID", Some("prod-family")),
                ("BIOMEOS_INSECURE", None::<&str>),
            ],
            || {
                assert!(is_btsp_required());
            },
        );
    }

    #[test]
    fn btsp_not_required_when_insecure() {
        temp_env::with_vars(
            [
                ("SQUIRREL_FAMILY_ID", None::<&str>),
                ("FAMILY_ID", Some("prod-family")),
                ("BIOMEOS_INSECURE", Some("1")),
            ],
            || {
                assert!(!is_btsp_required());
            },
        );
    }

    #[test]
    fn btsp_required_with_primal_specific_family() {
        temp_env::with_vars(
            [
                ("SQUIRREL_FAMILY_ID", Some("squirrel-fam")),
                ("FAMILY_ID", None::<&str>),
                ("BIOMEOS_INSECURE", None::<&str>),
            ],
            || {
                assert!(is_btsp_required());
            },
        );
    }

    // ── is_btsp_required with empty family ─────────────────────────

    #[test]
    fn btsp_not_required_with_empty_family() {
        temp_env::with_vars(
            [
                ("SQUIRREL_FAMILY_ID", None::<&str>),
                ("FAMILY_ID", Some("")),
                ("BIOMEOS_INSECURE", None::<&str>),
            ],
            || {
                assert!(!is_btsp_required());
            },
        );
    }

    // ── Multi-frame sequence ─────────────────────────────────────────

    #[tokio::test]
    async fn multiple_frames_in_sequence() {
        let (mut client, mut server) = duplex(8192);

        let hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "a2V5MQ==".into(),
        };
        let resp = ChallengeResponse {
            response: "cmVzcA==".into(),
            preferred_cipher: "null".into(),
        };

        write_message(&mut client, &hello)
            .await
            .expect("write hello");
        write_message(&mut client, &resp).await.expect("write resp");

        let got_hello: ClientHello = read_message(&mut server).await.expect("read hello");
        let got_resp: ChallengeResponse = read_message(&mut server).await.expect("read resp");

        assert_eq!(got_hello.client_ephemeral_pub, "a2V5MQ==");
        assert_eq!(got_resp.preferred_cipher, "null");
    }

    // ── Verify transport is clean after handshake frames ─────────────

    #[tokio::test]
    async fn transport_clean_after_frames() {
        let (mut client, mut server) = duplex(8192);

        let hello = ClientHello {
            version: 1,
            client_ephemeral_pub: "dGVzdA==".into(),
        };
        write_message(&mut client, &hello).await.expect("write");

        // Write raw JSON-RPC line after the frame (simulating post-handshake)
        let jsonrpc = b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"id\":1}\n";
        client.write_all(jsonrpc).await.expect("write jsonrpc");

        // Read the BTSP frame
        let _got: ClientHello = read_message(&mut server).await.expect("read hello");

        // Read the subsequent JSON-RPC line (proving the transport is clean)
        let mut line_buf = vec![0u8; jsonrpc.len()];
        server
            .read_exact(&mut line_buf)
            .await
            .expect("read jsonrpc");
        assert_eq!(&line_buf, jsonrpc);
    }

    // ── discover_btsp_provider (env precedence + filesystem; no network) ─

    #[test]
    fn discover_btsp_prefers_btsp_provider_socket() {
        let primary = tempfile::NamedTempFile::new().expect("temp file");
        let secondary = tempfile::NamedTempFile::new().expect("temp file");
        temp_env::with_vars(
            [
                (
                    "BTSP_PROVIDER_SOCKET",
                    Some(primary.path().to_str().expect("utf8 path")),
                ),
                (
                    "BTSP_CAPABILITY_SOCKET",
                    Some(secondary.path().to_str().expect("utf8 path")),
                ),
                ("SECURITY_SOCKET", None::<&str>),
                ("BEARDOG_SOCKET", None::<&str>),
            ],
            || {
                let got = discover_btsp_provider().expect("discover");
                assert_eq!(got, primary.path());
            },
        );
    }

    #[test]
    fn discover_btsp_falls_back_when_provider_path_missing() {
        let cap = tempfile::NamedTempFile::new().expect("temp file");
        temp_env::with_vars(
            [
                (
                    "BTSP_PROVIDER_SOCKET",
                    Some("/nonexistent/btsp_provider_missing.sock"),
                ),
                (
                    "BTSP_CAPABILITY_SOCKET",
                    Some(cap.path().to_str().expect("utf8 path")),
                ),
                ("SECURITY_SOCKET", None::<&str>),
                ("BEARDOG_SOCKET", None::<&str>),
            ],
            || {
                let got = discover_btsp_provider().expect("discover");
                assert_eq!(got, cap.path());
            },
        );
    }

    #[test]
    fn discover_btsp_security_socket_after_env_misses() {
        let sec = tempfile::NamedTempFile::new().expect("temp file");
        temp_env::with_vars(
            [
                (
                    "BTSP_PROVIDER_SOCKET",
                    Some("/nonexistent/btsp_provider_missing.sock"),
                ),
                (
                    "BTSP_CAPABILITY_SOCKET",
                    Some("/nonexistent/btsp_capability_missing.sock"),
                ),
                (
                    "SECURITY_SOCKET",
                    Some(sec.path().to_str().expect("utf8 path")),
                ),
                ("BEARDOG_SOCKET", None::<&str>),
            ],
            || {
                let got = discover_btsp_provider().expect("discover");
                assert_eq!(got, sec.path());
            },
        );
    }

    #[test]
    fn discover_btsp_beardog_socket_after_higher_tiers_miss() {
        let bd = tempfile::NamedTempFile::new().expect("temp file");
        temp_env::with_vars(
            [
                (
                    "BTSP_PROVIDER_SOCKET",
                    Some("/nonexistent/btsp_provider_missing.sock"),
                ),
                (
                    "BTSP_CAPABILITY_SOCKET",
                    Some("/nonexistent/btsp_capability_missing.sock"),
                ),
                (
                    "SECURITY_SOCKET",
                    Some("/nonexistent/security_missing.sock"),
                ),
                (
                    "BEARDOG_SOCKET",
                    Some(bd.path().to_str().expect("utf8 path")),
                ),
            ],
            || {
                let got = discover_btsp_provider().expect("discover");
                assert_eq!(got, bd.path());
            },
        );
    }

    #[test]
    fn maybe_handshake_returns_none_when_btsp_not_required() {
        temp_env::with_vars(
            [
                ("SQUIRREL_FAMILY_ID", None::<&str>),
                ("BIOMEOS_FAMILY_ID", None::<&str>),
                ("FAMILY_ID", None::<&str>),
                ("BIOMEOS_INSECURE", None::<&str>),
            ],
            || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime");
                let (_client, mut server) = duplex(4096);
                let out = rt.block_on(maybe_handshake(&mut server));
                assert!(out.expect("handshake").is_none());
            },
        );
    }
}
