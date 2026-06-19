// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Performance tuning env vars

/// Batch processor size
pub const BATCH_PROCESSOR_SIZE: &str = "PERF_BATCH_PROCESSOR_SIZE";
/// FS buffer size
pub const FS_BUFFER_SIZE: &str = "PERF_FS_BUFFER_SIZE";
/// Max plugin ID length
pub const MAX_PLUGIN_ID_LENGTH: &str = "PERF_MAX_PLUGIN_ID_LENGTH";
/// Session timeout (seconds)
pub const SESSION_TIMEOUT_SECONDS: &str = "PERF_SESSION_TIMEOUT_SECONDS";
/// String pool capacity
pub const STRING_POOL_CAPACITY: &str = "PERF_STRING_POOL_CAPACITY";
