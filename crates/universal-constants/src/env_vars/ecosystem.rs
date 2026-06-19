// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem orchestration env vars

/// biomeOS family ID
pub const BIOMEOS_FAMILY_ID: &str = "BIOMEOS_FAMILY_ID";
/// Generic family ID (lowest priority)
pub const FAMILY_ID: &str = "FAMILY_ID";
/// biomeOS socket path (legacy)
pub const BIOMEOS_SOCKET_PATH: &str = "BIOMEOS_SOCKET_PATH";
/// biomeOS socket (discovery)
pub const BIOMEOS_SOCKET: &str = "BIOMEOS_SOCKET";
/// Test biomeOS optimized port
pub const TEST_BIOMEOS_OPT_PORT: &str = "TEST_BIOMEOS_OPT_PORT";
/// biomeOS insecure mode flag
pub const BIOMEOS_INSECURE: &str = "BIOMEOS_INSECURE";
/// biomeOS endpoint
pub const BIOMEOS_ENDPOINT: &str = "BIOMEOS_ENDPOINT";
/// biomeOS port
pub const BIOMEOS_PORT: &str = "BIOMEOS_PORT";
/// biomeOS UI endpoint
pub const BIOMEOS_UI_ENDPOINT: &str = "BIOMEOS_UI_ENDPOINT";
/// biomeOS websocket URL
pub const BIOMEOS_WEBSOCKET_URL: &str = "BIOMEOS_WEBSOCKET_URL";
/// Ecosystem endpoint (capability-first)
pub const ECOSYSTEM_ENDPOINT: &str = "ECOSYSTEM_ENDPOINT";
/// Ecosystem port
pub const ECOSYSTEM_PORT: &str = "ECOSYSTEM_PORT";
/// Ecosystem orchestrator socket
pub const ECOSYSTEM_ORCHESTRATOR_SOCKET: &str = "ECOSYSTEM_ORCHESTRATOR_SOCKET";
/// Ecosystem service mesh endpoint
pub const ECOSYSTEM_SERVICE_MESH_ENDPOINT: &str = "ECOSYSTEM_SERVICE_MESH_ENDPOINT";
/// Ecosystem service timeout (ms)
pub const ECOSYSTEM_SERVICE_TIMEOUT_MS: &str = "ECOSYSTEM_SERVICE_TIMEOUT_MS";
/// Ecosystem websocket URL (capability-first)
pub const ECOSYSTEM_WEBSOCKET_URL: &str = "ECOSYSTEM_WEBSOCKET_URL";
/// Ecosystem router service ID
pub const ECOSYSTEM_ROUTER_SERVICE_ID: &str = "ECOSYSTEM_ROUTER_SERVICE_ID";
/// Neural API socket
pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";
/// Node ID
pub const NODE_ID: &str = "NODE_ID";
/// Biome ID
pub const BIOME_ID: &str = "BIOME_ID";
/// biomeOS socket directory override
pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";
/// Transport endpoint (JSON-encoded, sourDough `TransportEndpoint` format).
/// When set, the primal binds to this endpoint instead of self-selecting transport.
pub const TRANSPORT_ENDPOINT: &str = "TRANSPORT_ENDPOINT";
