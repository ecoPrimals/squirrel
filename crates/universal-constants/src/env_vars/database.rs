// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Database env vars

/// Database URL
pub const URL: &str = "DATABASE_URL";
/// Database URL (dev)
pub const URL_DEV: &str = "DATABASE_URL_DEV";
/// Database URL (staging)
pub const URL_STAGING: &str = "DATABASE_URL_STAGING";
/// Database port
pub const PORT: &str = "DATABASE_PORT";
/// Database max connections
pub const MAX_CONNECTIONS: &str = "DATABASE_MAX_CONNECTIONS";
/// Database timeout (seconds)
pub const TIMEOUT_SECS: &str = "DATABASE_TIMEOUT_SECS";
/// DB max connections (alias)
pub const DB_MAX_CONNECTIONS: &str = "DB_MAX_CONNECTIONS";
/// DB timeout (alias)
pub const DB_TIMEOUT: &str = "DB_TIMEOUT";
/// Postgres port
pub const POSTGRES_PORT: &str = "POSTGRES_PORT";
