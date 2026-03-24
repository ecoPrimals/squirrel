// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Default value functions for serde and `Default` impls.

use std::path::PathBuf;

use super::service::ServiceRegistryType;
use universal_constants::network::{get_bind_address, get_service_port};

// Default value functions
pub fn default_instance_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn default_log_level() -> String {
    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
}

// Database defaults
pub fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string())
}

pub fn default_max_db_connections() -> u32 {
    std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

pub fn default_db_timeout() -> u64 {
    std::env::var("DB_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

pub fn default_pool_size() -> u32 {
    5
}

// Load balancing defaults
pub fn default_session_timeout() -> u64 {
    3600 // 1 hour
}

pub fn default_max_retries() -> u32 {
    3
}

pub fn default_failure_threshold() -> u32 {
    5
}

pub fn default_success_threshold() -> u32 {
    3
}

pub fn default_circuit_timeout() -> u64 {
    60
}

pub fn default_half_open_requests() -> u32 {
    3
}

pub fn default_work_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

pub fn default_plugin_dir() -> PathBuf {
    PathBuf::from("./plugins")
}

pub fn default_bind_address() -> String {
    std::env::var("SQUIRREL_BIND_ADDRESS").unwrap_or_else(|_| get_bind_address())
}

pub fn default_http_port() -> u16 {
    std::env::var("SQUIRREL_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| get_service_port("websocket"))
}

pub fn default_websocket_port() -> u16 {
    std::env::var("SQUIRREL_WEBSOCKET_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| get_service_port("http"))
}

pub fn default_grpc_port() -> u16 {
    std::env::var("SQUIRREL_GRPC_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| get_service_port("admin"))
}

pub fn default_max_connections() -> u32 {
    100
}

pub fn default_true() -> bool {
    true
}

pub fn default_mcp_version() -> String {
    "1.0".to_string()
}

pub fn default_max_message_size() -> usize {
    16 * 1024 * 1024 // 16 MB
}

pub fn default_buffer_size() -> usize {
    8192
}

pub fn default_compression_level() -> u32 {
    6
}

pub fn default_max_concurrent_ai_requests() -> usize {
    10
}

pub fn default_max_services() -> usize {
    1000
}

pub fn default_health_check_interval() -> u64 {
    30
}

pub fn default_heartbeat_interval() -> u64 {
    15
}

pub fn default_service_expiration() -> u64 {
    90
}

pub fn default_registry_type() -> ServiceRegistryType {
    ServiceRegistryType::InMemory
}

pub fn default_token_expiration() -> u64 {
    3600 // 1 hour
}

// Security config defaults (Nov 9, 2025 consolidation)
pub fn default_encryption_format() -> String {
    "AES256GCM".to_string()
}

pub fn default_token_expiry_minutes() -> u64 {
    60 // 1 hour in minutes
}

pub fn default_metrics_endpoint() -> String {
    "/metrics".to_string()
}

pub fn default_prometheus_port() -> u16 {
    get_service_port("metrics")
}
