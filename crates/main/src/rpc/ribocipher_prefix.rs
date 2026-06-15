// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Transport signal mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Re-exports riboCipher constants from the canonical transport module.
//!
//! The authoritative definitions live in
//! [`universal_patterns::transport::ribocipher`]. This module re-exports them
//! so `super::ribocipher_prefix::*` continues to work within the RPC crate.

pub use universal_patterns::transport::ribocipher::*;
