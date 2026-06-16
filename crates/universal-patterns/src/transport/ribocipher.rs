// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Transport signal mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! riboCipher Transport Signal — Eukaryotic Genetics Model (Wave 114).
//!
//! Per `RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD`, every UDS connection opens with
//! a 2-byte prefix: `[signal_byte, protocol_type]`.
//!
//! ## Eukaryotic Two-Stream Model
//!
//! | Stream | Signal | Purpose |
//! |--------|--------|---------|
//! | **MitoBeacon** | `0xEC` / `0xED` | Relay access, mesh, ABG transport (shared/copyable) |
//! | **Nuclear Lineage** | `0xEE` | Per-user permissions, tiered access (non-fungible) |
//!
//! BearDog owns both streams. `FAMILY_SEED` is mito-beacon material.
//!
//! - **Client side**: [`write_ndjson_preamble`] writes `[0xEC, 0x01]` (clear) and
//!   [`write_mito_ndjson_preamble`] writes `[0xED, 0x01]` (mito-obfuscated).
//! - **Server side**: Both `0xEC` and `0xED` are accepted and routed identically
//!   by protocol type. `0xEE` (nuclear) requires per-user key material and is
//!   deferred to Wave 115+.

use tokio::io::AsyncWriteExt;

/// Clear-signal magic byte (Tier 1 — MitoBeacon clear).
pub const CLEAR_SIGNAL: u8 = 0xEC;

/// Mito-obfuscated magic byte (Tier 2 — MitoBeacon obfuscated).
pub const MITO_SIGNAL: u8 = 0xED;

/// Nuclear-sealed magic byte (Tier 3 — Nuclear Lineage, per-user).
pub const NUCLEAR_SIGNAL: u8 = 0xEE;

/// Protocol type: newline-delimited JSON-RPC 2.0.
pub const NDJSON_JSONRPC: u8 = 0x01;

/// Protocol type: BTSP binary framing.
pub const BTSP_BINARY: u8 = 0x02;

/// Protocol type: BTSP JSON-line framing.
pub const BTSP_JSON_LINE: u8 = 0x03;

/// Whether a first byte is a riboCipher signal prefix (any tier).
#[must_use]
pub const fn is_ribocipher_signal(byte: u8) -> bool {
    matches!(byte, CLEAR_SIGNAL | MITO_SIGNAL | NUCLEAR_SIGNAL)
}

/// Whether a signal byte belongs to the MitoBeacon stream (shared access).
///
/// Both `0xEC` (clear) and `0xED` (mito-obfuscated) carry shared/copyable
/// relay credentials. Accept loops should handle them identically.
#[must_use]
pub const fn is_mito_beacon(byte: u8) -> bool {
    matches!(byte, CLEAR_SIGNAL | MITO_SIGNAL)
}

/// Classification of the first byte from a connection accept loop.
pub enum SignalResult {
    /// `0xEC` or `0xED` with protocol type — MitoBeacon stream (shared access).
    /// The protocol type byte has been consumed.
    MitoBeacon {
        /// The signal byte (`0xEC` clear or `0xED` mito-obfuscated).
        signal: u8,
        /// The protocol type byte (`0x01` NDJSON, `0x02` BTSP binary, etc.).
        protocol_type: u8,
    },
    /// `0xEE` — Nuclear Lineage (per-user, requires BearDog key material).
    /// Not yet implemented; accept loops should close gracefully.
    NuclearLineage,
    /// Not a riboCipher signal — the byte belongs to a downstream protocol.
    NotSignalled(u8),
    /// Stream closed before any data.
    Eof,
}

/// Write the riboCipher clear-signal preamble for NDJSON JSON-RPC.
///
/// Sends `[0xEC, 0x01]` — Tier 1 clear MitoBeacon carrying NDJSON JSON-RPC.
pub async fn write_ndjson_preamble<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
) -> std::io::Result<()> {
    writer.write_all(&[CLEAR_SIGNAL, NDJSON_JSONRPC]).await
}

/// Write the riboCipher mito-obfuscated preamble for NDJSON JSON-RPC.
///
/// Sends `[0xED, 0x01]` — Tier 2 mito-obfuscated MitoBeacon carrying NDJSON
/// JSON-RPC. Used when the connection traverses relay/mesh infrastructure
/// where the mito-beacon signal indicates shared group credential access.
pub async fn write_mito_ndjson_preamble<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
) -> std::io::Result<()> {
    writer.write_all(&[MITO_SIGNAL, NDJSON_JSONRPC]).await
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
    fn mito_beacon_classification() {
        assert!(is_mito_beacon(CLEAR_SIGNAL));
        assert!(is_mito_beacon(MITO_SIGNAL));
        assert!(!is_mito_beacon(NUCLEAR_SIGNAL));
        assert!(!is_mito_beacon(b'{'));
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
    async fn write_ndjson_preamble_writes_clear_signal() {
        let mut buf = Vec::new();
        write_ndjson_preamble(&mut buf).await.unwrap();
        assert_eq!(buf, vec![0xEC, 0x01]);
    }

    #[tokio::test]
    async fn write_mito_ndjson_preamble_writes_mito_signal() {
        let mut buf = Vec::new();
        write_mito_ndjson_preamble(&mut buf).await.unwrap();
        assert_eq!(buf, vec![0xED, 0x01]);
    }
}
