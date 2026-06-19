// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Squirrel primal env vars

/// Override UDS socket path (`--socket` CLI equivalent)
pub const SOCKET: &str = "SQUIRREL_SOCKET";
/// Family ID for multi-instance deployments
pub const FAMILY_ID: &str = "SQUIRREL_FAMILY_ID";
/// Node identifier
pub const NODE_ID: &str = "SQUIRREL_NODE_ID";
/// TCP port for JSON-RPC
pub const PORT: &str = "SQUIRREL_PORT";
/// Alias for PORT (legacy)
pub const SERVER_PORT: &str = "SQUIRREL_SERVER_PORT";
/// TCP bind address
pub const BIND: &str = "SQUIRREL_BIND";
/// Bind address (legacy alias)
pub const BIND_ADDRESS: &str = "SQUIRREL_BIND_ADDRESS";
/// Host (legacy)
pub const HOST: &str = "SQUIRREL_HOST";
/// IPC host (legacy)
pub const IPC_HOST: &str = "SQUIRREL_IPC_HOST";
/// HTTP port
pub const HTTP_PORT: &str = "SQUIRREL_HTTP_PORT";
/// WebSocket port
pub const WEBSOCKET_PORT: &str = "SQUIRREL_WEBSOCKET_PORT";
/// gRPC port
pub const GRPC_PORT: &str = "SQUIRREL_GRPC_PORT";
/// Daemonize flag
pub const DAEMON: &str = "SQUIRREL_DAEMON";
/// Internal flag: child is already daemonized
pub const DAEMONIZED: &str = "SQUIRREL_DAEMONIZED";
/// Config file path override
pub const CONFIG: &str = "SQUIRREL_CONFIG";
/// Environment mode (dev/staging/prod)
pub const ENV: &str = "SQUIRREL_ENV";
/// Log level override
pub const LOG_LEVEL: &str = "SQUIRREL_LOG_LEVEL";
/// JSON logging format
pub const LOG_JSON: &str = "SQUIRREL_LOG_JSON";
/// Default AI provider name
pub const DEFAULT_AI_PROVIDER: &str = "SQUIRREL_DEFAULT_AI_PROVIDER";
/// AI config path
pub const AI_CONFIG: &str = "SQUIRREL_AI_CONFIG";
/// Enable AI subsystem
pub const AI_ENABLED: &str = "SQUIRREL_AI_ENABLED";
/// AI logging
pub const AI_ENABLE_LOGGING: &str = "SQUIRREL_AI_ENABLE_LOGGING";
/// AI inference timeout (seconds)
pub const AI_INFERENCE_TIMEOUT_SECS: &str = "SQUIRREL_AI_INFERENCE_TIMEOUT_SECS";
/// AI max retries
pub const AI_MAX_RETRIES: &str = "SQUIRREL_AI_MAX_RETRIES";
/// AI request timeout
pub const AI_REQUEST_TIMEOUT: &str = "SQUIRREL_AI_REQUEST_TIMEOUT";
/// MCP endpoint
pub const MCP_ENDPOINT: &str = "SQUIRREL_MCP_ENDPOINT";
/// Plugin directories
pub const PLUGIN_DIRS: &str = "SQUIRREL_PLUGIN_DIRS";
/// Plugin path
pub const PLUGIN_PATH: &str = "SQUIRREL_PLUGIN_PATH";
/// Plugin load timeout (seconds)
pub const PLUGIN_LOAD_TIMEOUT_SECS: &str = "SQUIRREL_PLUGIN_LOAD_TIMEOUT_SECS";
/// JWT secret
pub const JWT_SECRET: &str = "SQUIRREL_JWT_SECRET";
/// Trust domain for mTLS/SPIFFE
pub const TRUST_DOMAIN: &str = "SQUIRREL_TRUST_DOMAIN";
/// Rate limit whitelist
pub const RATE_LIMIT_WHITELIST: &str = "SQUIRREL_RATE_LIMIT_WHITELIST";
/// Connection timeout (seconds)
pub const CONNECTION_TIMEOUT_SECS: &str = "SQUIRREL_CONNECTION_TIMEOUT_SECS";
/// Request timeout (seconds)
pub const REQUEST_TIMEOUT_SECS: &str = "SQUIRREL_REQUEST_TIMEOUT_SECS";
/// Operation timeout (seconds)
pub const OPERATION_TIMEOUT_SECS: &str = "SQUIRREL_OPERATION_TIMEOUT_SECS";
/// Database timeout (seconds)
pub const DATABASE_TIMEOUT_SECS: &str = "SQUIRREL_DATABASE_TIMEOUT_SECS";
/// Discovery timeout (seconds)
pub const DISCOVERY_TIMEOUT_SECS: &str = "SQUIRREL_DISCOVERY_TIMEOUT_SECS";
/// Health check timeout (seconds)
pub const HEALTH_CHECK_TIMEOUT_SECS: &str = "SQUIRREL_HEALTH_CHECK_TIMEOUT_SECS";
/// Heartbeat interval (seconds)
pub const HEARTBEAT_INTERVAL_SECS: &str = "SQUIRREL_HEARTBEAT_INTERVAL_SECS";
/// Session timeout (seconds)
pub const SESSION_TIMEOUT_SECS: &str = "SQUIRREL_SESSION_TIMEOUT_SECS";
/// Registry socket for discovery
pub const REGISTRY_SOCKET: &str = "SQUIRREL_REGISTRY_SOCKET";
/// Ecosystem IPC service
pub const ECOSYSTEM_IPC_SERVICE: &str = "SQUIRREL_ECOSYSTEM_IPC_SERVICE";
/// Instance capacity
pub const INSTANCE_CAPACITY: &str = "SQUIRREL_INSTANCE_CAPACITY";
/// IPC retry base delay (ms)
pub const RETRY_BASE_DELAY_MS: &str = "SQUIRREL_RETRY_BASE_DELAY_MS";
/// IPC retry max attempts
pub const RETRY_MAX_ATTEMPTS: &str = "SQUIRREL_RETRY_MAX_ATTEMPTS";
/// IPC retry max delay (ms)
pub const RETRY_MAX_DELAY_MS: &str = "SQUIRREL_RETRY_MAX_DELAY_MS";
/// Resource: CPU
pub const RESOURCE_CPU: &str = "SQUIRREL_RESOURCE_CPU";
/// Resource: GPU
pub const RESOURCE_GPU: &str = "SQUIRREL_RESOURCE_GPU";
/// Resource: memory
pub const RESOURCE_MEMORY: &str = "SQUIRREL_RESOURCE_MEMORY";
/// Resource: network
pub const RESOURCE_NETWORK: &str = "SQUIRREL_RESOURCE_NETWORK";
/// Resource: storage
pub const RESOURCE_STORAGE: &str = "SQUIRREL_RESOURCE_STORAGE";
