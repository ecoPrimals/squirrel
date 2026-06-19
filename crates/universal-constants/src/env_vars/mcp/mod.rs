// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP protocol env vars

/// MCP environment (dev/staging/prod)
pub const ENV: &str = "MCP_ENV";
/// MCP environment (alias)
pub const ENVIRONMENT: &str = "MCP_ENVIRONMENT";
/// MCP host
pub const HOST: &str = "MCP_HOST";
/// MCP port
pub const PORT: &str = "MCP_PORT";
/// MCP server URL
pub const SERVER_URL: &str = "MCP_SERVER_URL";
/// MCP server port
pub const SERVER_PORT: &str = "MCP_SERVER_PORT";
/// MCP server host
pub const SERVER_HOST: &str = "MCP_SERVER_HOST";
/// MCP server endpoint
pub const SERVER_ENDPOINT: &str = "MCP_SERVER_ENDPOINT";
/// MCP endpoint (generic)
pub const ENDPOINT: &str = "MCP_ENDPOINT";
/// MCP timeout (ms)
pub const TIMEOUT_MS: &str = "MCP_TIMEOUT_MS";
/// MCP request timeout (ms)
pub const REQUEST_TIMEOUT_MS: &str = "MCP_REQUEST_TIMEOUT_MS";
/// MCP connection timeout (seconds)
pub const CONNECTION_TIMEOUT_SECS: &str = "MCP_CONNECTION_TIMEOUT_SECS";
/// MCP max message size
pub const MAX_MESSAGE_SIZE: &str = "MCP_MAX_MESSAGE_SIZE";
/// MCP max connections
pub const MAX_CONNECTIONS: &str = "MCP_MAX_CONNECTIONS";
/// MCP protocol version
pub const PROTOCOL_VERSION: &str = "MCP_PROTOCOL_VERSION";
/// MCP max reconnect attempts
pub const MAX_RECONNECT_ATTEMPTS: &str = "MCP_MAX_RECONNECT_ATTEMPTS";
/// MCP reconnect delay (ms)
pub const RECONNECT_DELAY_MS: &str = "MCP_RECONNECT_DELAY_MS";
/// MCP default model
pub const DEFAULT_MODEL: &str = "MCP_DEFAULT_MODEL";
/// MCP debug mode
pub const DEBUG: &str = "MCP_DEBUG";
/// MCP CORS origins
pub const CORS_ORIGINS: &str = "MCP_CORS_ORIGINS";
/// MCP heartbeat interval (seconds)
pub const HEARTBEAT_INTERVAL_SECS: &str = "MCP_HEARTBEAT_INTERVAL_SECS";
/// MCP coordination interval (seconds)
pub const COORDINATION_INTERVAL_SECS: &str = "MCP_COORDINATION_INTERVAL_SECS";
/// MCP gRPC port (legacy compat)
pub const GRPC_PORT: &str = "MCP_GRPC_PORT";

pub mod cli;
pub mod client;
