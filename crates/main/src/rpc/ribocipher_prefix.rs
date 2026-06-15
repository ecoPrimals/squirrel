// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Transport signal mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! riboCipher Transport Signal — clear-tier prefix detection (Wave 113).
//!
//! Implements server-side Tier 1 (clear signal) detection per the
//! `RIBOCIPHER_TRANSPORT_SIGNAL_STANDARD`. The client sends `[0xEC, protocol_type]`
//! before any payload. This module strips that prefix so downstream handlers
//! receive a clean byte stream.
//!
//! Only Tier 1 (`0xEC`) is implemented — Tier 2 (`0xED`, mito-obfuscated) and
//! Tier 3 (`0xEE`, nuclear-sealed) require key material and are deferred until
//! cross-gate WAN transport is wired.

/// Clear-signal magic byte (Tier 1).
pub const CLEAR_SIGNAL: u8 = 0xEC;

/// Mito-obfuscated magic byte (Tier 2 — not yet implemented).
pub const MITO_SIGNAL: u8 = 0xED;

/// Nuclear-sealed magic byte (Tier 3 — not yet implemented).
pub const NUCLEAR_SIGNAL: u8 = 0xEE;

// Protocol type constants per riboCipher standard §Protocol Type Table.
pub const NDJSON_JSONRPC: u8 = 0x01;
pub const BTSP_BINARY: u8 = 0x02;
pub const BTSP_JSON_LINE: u8 = 0x03;

/// Whether a first byte is a riboCipher signal prefix.
#[must_use]
pub const fn is_ribocipher_signal(byte: u8) -> bool {
    matches!(byte, CLEAR_SIGNAL | MITO_SIGNAL | NUCLEAR_SIGNAL)
}

/// Classify a first byte from the accept loop.
///
/// Returns `Some(())` for riboCipher signal bytes, `None` for anything else
/// (raw JSON `{`, BTSP binary `0x00`, or unknown).
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
}
