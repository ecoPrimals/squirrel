// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

// crates/mcp/src/protocol/constants.rs
//! Protocol-level constants.

/// Default key size for encryption (e.g., AES-256 needs 32 bytes).
pub const DEFAULT_ENCRYPTION_KEY_SIZE: usize = 32;

/// Default key size for signatures (e.g., HMAC-SHA256 might use 32 or 64 bytes).
pub const DEFAULT_SIGNATURE_KEY_SIZE: usize = 32; 