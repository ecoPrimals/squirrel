// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Timeout env vars

/// Connection timeout (generic)
pub const CONNECTION: &str = "MCP_CONNECTION_TIMEOUT";
/// Request timeout (generic)
pub const REQUEST: &str = "REQUEST_TIMEOUT";
/// Operation timeout
pub const OPERATION: &str = "OPERATION_TIMEOUT";
/// Database timeout
pub const DATABASE: &str = "DATABASE_TIMEOUT";
/// Heartbeat interval (service mesh)
pub const HEARTBEAT_INTERVAL: &str = "SERVICE_MESH_HEARTBEAT_INTERVAL";
/// Initial delay (service mesh)
pub const INITIAL_DELAY: &str = "SERVICE_MESH_INITIAL_DELAY_MS";
