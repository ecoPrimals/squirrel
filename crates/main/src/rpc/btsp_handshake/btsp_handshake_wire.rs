// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP length-prefixed wire framing and handshake message types (`BTSP_PROTOCOL_STANDARD`).

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Maximum BTSP frame size: 16 MiB (`BTSP_PROTOCOL_STANDARD` §Wire Framing).
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// Handshake timeout per step (generous for local IPC + security provider round-trip).
pub const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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
    /// Negotiated cipher name.
    pub cipher: String,
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

    #[error("plain JSON-RPC detected on BTSP-guarded socket (auto-detect fallback)")]
    PlainJsonRpc,
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
