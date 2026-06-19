// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Network env vars (flat names shared across subsystems)

/// Bind address
pub const BIND_ADDRESS: &str = "MCP_BIND_ADDRESS";
/// WebSocket port
pub const WEBSOCKET_PORT: &str = "MCP_WEBSOCKET_PORT";
/// HTTP port
pub const HTTP_PORT: &str = "MCP_HTTP_PORT";
/// Admin port
pub const ADMIN_PORT: &str = "MCP_ADMIN_PORT";
/// Metrics port
pub const METRICS_PORT: &str = "MCP_METRICS_PORT";
/// Max connections
pub const MAX_CONNECTIONS: &str = "MAX_CONNECTIONS";
/// Generic port
pub const PORT: &str = "PORT";
/// Generic bind addr
pub const BIND_ADDR: &str = "BIND_ADDR";
/// Generic bind address (non-prefixed)
pub const GENERIC_BIND_ADDRESS: &str = "BIND_ADDRESS";
/// Server bind address
pub const SERVER_BIND_ADDRESS: &str = "SERVER_BIND_ADDRESS";
/// Server port
pub const SERVER_PORT: &str = "SERVER_PORT";
/// Network host
pub const NETWORK_HOST: &str = "NETWORK_HOST";
/// Network port
pub const NETWORK_PORT: &str = "NETWORK_PORT";
/// Network connection timeout (ms)
pub const NETWORK_CONNECTION_TIMEOUT_MS: &str = "NETWORK_CONNECTION_TIMEOUT_MS";
/// Network read timeout (ms)
pub const NETWORK_READ_TIMEOUT_MS: &str = "NETWORK_READ_TIMEOUT_MS";
/// Network write timeout (ms)
pub const NETWORK_WRITE_TIMEOUT_MS: &str = "NETWORK_WRITE_TIMEOUT_MS";
/// Network max connections
pub const NETWORK_MAX_CONNECTIONS: &str = "NETWORK_MAX_CONNECTIONS";
/// Network HTTP socket
pub const NETWORK_HTTP_SOCKET: &str = "NETWORK_HTTP_SOCKET";
/// Service host
pub const SERVICE_HOST: &str = "SERVICE_HOST";
/// Service port
pub const SERVICE_PORT: &str = "SERVICE_PORT";
/// Service address
pub const SERVICE_ADDRESS: &str = "SERVICE_ADDRESS";
/// Service IP
pub const SERVICE_IP: &str = "SERVICE_IP";
/// Client IP address
pub const CLIENT_IP_ADDRESS: &str = "CLIENT_IP_ADDRESS";
/// Client user agent
pub const CLIENT_USER_AGENT: &str = "CLIENT_USER_AGENT";
/// UI host
pub const UI_HOST: &str = "UI_HOST";
/// Dev bind address
pub const DEV_BIND_ADDRESS: &str = "DEV_BIND_ADDRESS";
/// Dev server host
pub const DEV_SERVER_HOST: &str = "DEV_SERVER_HOST";
/// Storage capability endpoint (capability-first naming)
pub const STORAGE_ENDPOINT: &str = "STORAGE_ENDPOINT";
/// Storage capability port
pub const STORAGE_PORT: &str = "STORAGE_PORT";
/// Storage service port
pub const STORAGE_SERVICE_PORT: &str = "STORAGE_SERVICE_PORT";
/// Security capability endpoint (capability-first naming)
pub const SECURITY_ENDPOINT: &str = "SECURITY_ENDPOINT";
/// Security capability port
pub const SECURITY_PORT: &str = "SECURITY_PORT";
/// Security service port
pub const SECURITY_SERVICE_PORT: &str = "SECURITY_SERVICE_PORT";
/// Service mesh endpoint (capability-first naming)
pub const SERVICE_MESH_ENDPOINT: &str = "SERVICE_MESH_ENDPOINT";
/// Service mesh port
pub const SERVICE_MESH_PORT: &str = "SERVICE_MESH_PORT";
