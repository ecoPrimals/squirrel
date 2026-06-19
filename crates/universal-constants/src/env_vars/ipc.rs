// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! IPC retry env vars

/// Retry base delay (ms)
pub const RETRY_BASE_DELAY_MS: &str = "IPC_RETRY_BASE_DELAY_MS";
/// Retry max attempts
pub const RETRY_MAX_ATTEMPTS: &str = "IPC_RETRY_MAX_ATTEMPTS";
/// Retry max delay (ms)
pub const RETRY_MAX_DELAY_MS: &str = "IPC_RETRY_MAX_DELAY_MS";
