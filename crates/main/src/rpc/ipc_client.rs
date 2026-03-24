// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Re-export of the autonomous IPC client from `universal-patterns`.
//!
//! Squirrel's IPC client lives in `universal-patterns` to avoid circular
//! dependencies while maintaining primal autonomy (no shared external IPC crates).

pub use universal_patterns::ipc_client::*;
