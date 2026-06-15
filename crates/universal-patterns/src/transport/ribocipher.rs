// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Transport signal mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! riboCipher Transport Signal — client + server support (Wave 113).
//!
//! Per `RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD`, every UDS connection opens with
//! a 2-byte prefix: `[signal_byte, protocol_type]`.
//!
//! - **Client side**: [`write_ndjson_preamble`] writes `[0xEC, 0x01]` before any
//!   JSON-RPC payload.
//! - **Server side**: Consumers read the first byte and classify via
//!   [`is_ribocipher_signal`] / [`SignalResult`].
//!
//! Only Tier 1 (`0xEC`) is implemented — Tier 2 (`0xED`, mito-obfuscated) and
//! Tier 3 (`0xEE`, nuclear-sealed) require key material and are deferred until
//! cross-gate WAN transport is wired.

use tokio::io::AsyncWriteExt;

/// Clear-signal magic byte (Tier 1).
pub const CLEAR_SIGNAL: u8 = 0xEC;

/// Mito-obfuscated magic byte (Tier 2 — not yet implemented).
pub const MITO_SIGNAL: u8 = 0xED;

/// Nuclear-sealed magic byte (Tier 3 — not yet implemented).
pub const NUCLEAR_SIGNAL: u8 = 0xEE;

/// Protocol type: newline-delimited JSON-RPC 2.0.
pub const NDJSON_JSONRPC: u8 = 0x01;

/// Protocol type: BTSP binary framing.
pub const BTSP_BINARY: u8 = 0x02;

/// Protocol type: BTSP JSON-line framing.
pub const BTSP_JSON_LINE: u8 = 0x03;

/// Whether a first byte is a riboCipher signal prefix.
#[must_use]
pub const fn is_ribocipher_signal(byte: u8) -> bool {
    matches!(byte, CLEAR_SIGNAL | MITO_SIGNAL | NUCLEAR_SIGNAL)
}

/// Classification of the first byte from a connection accept loop.
pub enum SignalResult {
    /// `[0xEC, protocol_type]` — clear-tier, protocol type consumed.
    Clear(u8),
    /// `0xED` / `0xEE` — higher tier, not yet implemented.
    UnsupportedTier(u8),
    /// Not a riboCipher signal — the byte belongs to a downstream protocol.
    NotSignalled(u8),
    /// Stream closed before any data.
    Eof,
}

/// Write the riboCipher clear-signal preamble for NDJSON JSON-RPC.
///
/// Sends `[0xEC, 0x01]` — the 2-byte prefix that tells the receiving primal
/// "this is a Tier 1 clear-signal connection carrying NDJSON JSON-RPC".
///
/// Call this immediately after connecting and before any JSON-RPC payload.
pub async fn write_ndjson_preamble<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
) -> std::io::Result<()> {
    writer.write_all(&[CLEAR_SIGNAL, NDJSON_JSONRPC]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_byte_detection() {
        assert!(is_ribocipher_signal(CLEAR_SIGNAL));
        assert!(is_ribocipher_signal(MITO_SIGNAL));
        assert!(is_ribocipher_signal(NUCLEAR_SIGNAL));
        assert!(!is_ribocipher_signal(b'{'));
        assert!(!is_ribocipher_signal(0x00));
        assert!(!is_ribocipher_signal(0x16));
    }

    #[test]
    fn constants_match_standard() {
        assert_eq!(CLEAR_SIGNAL, 0xEC);
        assert_eq!(MITO_SIGNAL, 0xED);
        assert_eq!(NUCLEAR_SIGNAL, 0xEE);
        assert_eq!(NDJSON_JSONRPC, 0x01);
        assert_eq!(BTSP_BINARY, 0x02);
        assert_eq!(BTSP_JSON_LINE, 0x03);
    }

    #[tokio::test]
    async fn write_ndjson_preamble_writes_two_bytes() {
        let mut buf = Vec::new();
        write_ndjson_preamble(&mut buf).await.unwrap();
        assert_eq!(buf, vec![0xEC, 0x01]);
    }
}
