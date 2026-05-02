// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP length-prefixed wire framing, handshake message types, and timeout policy
//! (`BTSP_PROTOCOL_STANDARD`).

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Maximum BTSP frame size: 16 MiB (`BTSP_PROTOCOL_STANDARD` §Wire Framing).
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// Default handshake timeout per step (ms). Configurable via `BTSP_HANDSHAKE_TIMEOUT_MS`.
const DEFAULT_HANDSHAKE_TIMEOUT_MS: u64 = 1500;

/// Handshake timeout per step: reads `BTSP_HANDSHAKE_TIMEOUT_MS` (env) or defaults to 1.5s.
///
/// The previous 5s default caused ~5s latency on guidestone runs when the BTSP
/// provider (BearDog) was unavailable — the handshake would timeout and the
/// client would have to reconnect with cleartext. 1.5s is generous for local
/// UDS IPC while keeping the failure fast enough to be invisible.
pub fn handshake_timeout() -> std::time::Duration {
    static CACHED: std::sync::OnceLock<std::time::Duration> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        let ms = std::env::var("BTSP_HANDSHAKE_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_HANDSHAKE_TIMEOUT_MS);
        std::time::Duration::from_millis(ms)
    })
}

/// BTSP protocol version we speak.
pub const BTSP_VERSION: u32 = 1;

// ── Wire types (BTSP_PROTOCOL_STANDARD §Handshake Protocol) ─────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    /// Protocol version.
    pub version: u32,
    /// Client ephemeral public key (encoding is transport-specific).
    pub client_ephemeral_pub: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    /// Protocol version.
    pub version: u32,
    /// Server ephemeral public key.
    pub server_ephemeral_pub: String,
    /// Challenge payload (e.g. base64).
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// Client response to the challenge.
    pub response: String,
    /// Preferred cipher suite name.
    pub preferred_cipher: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeComplete {
    /// Negotiated cipher.
    pub cipher: String,
    /// Session identifier.
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeErrorMsg {
    /// Error code / name.
    pub error: String,
    /// Human-readable reason.
    pub reason: String,
}

// ── Session result ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BtspSession {
    /// Established session id.
    pub session_id: String,
    /// Negotiated cipher name (Phase 2 always "null"; Phase 3 may upgrade).
    pub cipher: String,
    /// Handshake key material from the BTSP provider (base64-encoded).
    /// Present when the provider returns a `handshake_key` in `btsp.session.verify`.
    /// Used by Phase 3 `btsp.negotiate` for HKDF key derivation.
    pub handshake_key: Option<String>,
    /// Client ephemeral public key from the `ClientHello` message.
    /// Stored for Phase 3 nonce derivation context.
    pub client_ephemeral_pub: Option<String>,
}

// ── Error type ──────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum BtspError {
    #[error("BTSP I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("BTSP frame too large: {size} bytes (max {MAX_FRAME_SIZE})")]
    FrameTooLarge {
        /// Declared or computed frame size in bytes.
        size: usize,
    },

    #[error("BTSP handshake timed out")]
    Timeout,

    #[error("BTSP handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("BTSP provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("BTSP protocol error: {0}")]
    Protocol(String),

    /// First line (full UTF-8 line) was consumed; must be re-injected for the JSON-RPC handler.
    #[error("plain JSON-RPC detected on BTSP-guarded socket (auto-detect fallback)")]
    PlainJsonRpc {
        /// Complete first line, including the leading `{` and line terminator when present.
        first_line: String,
    },
}

// ── Frame I/O (BTSP_PROTOCOL_STANDARD §Wire Framing) ───────────────────

pub async fn read_frame<S: AsyncRead + Unpin>(stream: &mut S) -> Result<Vec<u8>, BtspError> {
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

/// Read a BTSP frame where the first byte of the 4-byte length prefix has
/// already been consumed (by the auto-detect peek).
pub async fn read_frame_with_first_byte<S: AsyncRead + Unpin>(
    stream: &mut S,
    first_byte: u8,
) -> Result<Vec<u8>, BtspError> {
    let mut remaining = [0u8; 3];
    stream.read_exact(&mut remaining).await?;
    let len_buf = [first_byte, remaining[0], remaining[1], remaining[2]];
    let len = u32::from_be_bytes(len_buf) as usize;
    if len > MAX_FRAME_SIZE {
        return Err(BtspError::FrameTooLarge { size: len });
    }
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

pub async fn write_frame<S: AsyncWrite + Unpin>(
    stream: &mut S,
    data: &[u8],
) -> Result<(), BtspError> {
    let len =
        u32::try_from(data.len()).map_err(|_| BtspError::FrameTooLarge { size: data.len() })?;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(data).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn read_message<S, T>(stream: &mut S) -> Result<T, BtspError>
where
    S: AsyncRead + Unpin,
    T: for<'de> Deserialize<'de>,
{
    let bytes = read_frame(stream).await?;
    serde_json::from_slice(&bytes)
        .map_err(|e| BtspError::Protocol(format!("invalid handshake message: {e}")))
}

/// Read a BTSP message where the first byte of the length prefix was already
/// consumed by auto-detect.
pub async fn read_message_with_first_byte<S, T>(
    stream: &mut S,
    first_byte: u8,
) -> Result<T, BtspError>
where
    S: AsyncRead + Unpin,
    T: for<'de> Deserialize<'de>,
{
    let bytes = read_frame_with_first_byte(stream, first_byte).await?;
    serde_json::from_slice(&bytes)
        .map_err(|e| BtspError::Protocol(format!("invalid handshake message: {e}")))
}

pub async fn write_message<S, T>(stream: &mut S, msg: &T) -> Result<(), BtspError>
where
    S: AsyncWrite + Unpin,
    T: Serialize + Send + Sync,
{
    let bytes = serde_json::to_vec(msg)
        .map_err(|e| BtspError::Protocol(format!("serialization failed: {e}")))?;
    write_frame(stream, &bytes).await
}

pub async fn write_json_line<S, T>(stream: &mut S, msg: &T) -> Result<(), BtspError>
where
    S: AsyncWrite + Unpin,
    T: Serialize + Send + Sync,
{
    let mut bytes = serde_json::to_vec(msg)
        .map_err(|e| BtspError::Protocol(format!("serialization failed: {e}")))?;
    bytes.push(b'\n');
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

pub async fn read_json_line_msg<S, T>(stream: &mut S) -> Result<T, BtspError>
where
    S: AsyncRead + Unpin,
    T: for<'de> Deserialize<'de>,
{
    let mut buf = Vec::with_capacity(4096);
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte).await?;
        if byte[0] == b'\n' {
            break;
        }
        buf.push(byte[0]);
        if buf.len() > MAX_FRAME_SIZE {
            return Err(BtspError::FrameTooLarge { size: buf.len() });
        }
    }
    serde_json::from_slice(&buf)
        .map_err(|e| BtspError::Protocol(format!("invalid JSON-line message: {e}")))
}
