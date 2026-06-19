// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation env vars

/// Comma-separated list of federation node endpoints
pub const NODES: &str = "FEDERATION_NODES";
/// Current CPU usage for load balancing
pub const CPU_USAGE: &str = "FEDERATION_CPU_USAGE";
/// Current memory usage for load balancing
pub const MEMORY_USAGE: &str = "FEDERATION_MEMORY_USAGE";
/// Current queue length for load balancing
pub const QUEUE_LENGTH: &str = "FEDERATION_QUEUE_LENGTH";
/// Legacy alias for `SONGBIRD_FEDERATION_ENABLED` (honored as fallback)
pub const ENABLED: &str = "FEDERATION_ENABLED";
