// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service infrastructure configuration types.
//!
//! Service mesh, load balancing, circuit breaker, and database backend types
//! extracted from the core definitions for cohesion and file-size compliance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service mesh configuration
///
/// Consolidated from universal and unified modules — contains best features from both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Enable service mesh
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enabled: bool,

    /// Discovery endpoints (from unified — supports multiple endpoints)
    #[serde(default)]
    pub discovery_endpoints: Vec<String>,

    /// Service registry type (from universal — rich abstraction)
    #[serde(default = "crate::unified::types::defaults::default_registry_type")]
    pub registry_type: ServiceRegistryType,

    /// Maximum services to track
    #[serde(default = "crate::unified::types::defaults::default_max_services")]
    pub max_services: usize,

    /// Health check interval in seconds
    #[serde(default = "crate::unified::types::defaults::default_health_check_interval")]
    pub health_check_interval_secs: u64,

    /// Heartbeat interval in seconds (from universal — for active health checks)
    #[serde(default = "crate::unified::types::defaults::default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,

    /// Service expiration timeout in seconds (from universal — when to remove stale services)
    #[serde(default = "crate::unified::types::defaults::default_service_expiration")]
    pub service_expiration_secs: u64,

    /// Enable automatic failover (from unified)
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_failover: bool,

    /// Enable service mesh metrics (from universal)
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub metrics_enabled: bool,

    /// Service mesh namespace (from universal — for multi-tenancy)
    #[serde(default)]
    pub namespace: Option<String>,
}

/// Service registry type
///
/// Defines how services are discovered and tracked. Moved from universal module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServiceRegistryType {
    /// In-memory registry (default for development)
    InMemory,

    /// File-based registry
    File {
        /// Path to registry file
        path: String,
    },

    /// Network-based registry (e.g., Consul, etcd)
    Network {
        /// Registry endpoints
        endpoints: Vec<String>,
    },

    /// Redis-based registry
    Redis {
        /// Redis connection string
        connection_string: String,
    },
    /// Database-based registry
    Database {
        /// Database connection string
        connection_string: String,
    },

    /// Custom registry with flexible configuration
    Custom {
        /// Custom configuration key-value pairs
        config: HashMap<String, String>,
    },
}

/// Database configuration
///
/// Consolidated from core/ and environment.rs modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection string (env: `DATABASE_URL`)
    #[serde(default = "crate::unified::types::defaults::default_database_url")]
    pub connection_string: String,

    /// Maximum number of connections (env: `DB_MAX_CONNECTIONS`)
    #[serde(default = "crate::unified::types::defaults::default_max_db_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds (env: `DB_TIMEOUT`)
    #[serde(default = "crate::unified::types::defaults::default_db_timeout")]
    pub timeout_seconds: u64,

    /// Database backend type
    #[serde(default)]
    pub backend: DatabaseBackend,

    /// Enable connection pooling
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_pooling: bool,

    /// Pool size
    #[serde(default = "crate::unified::types::defaults::default_pool_size")]
    pub pool_size: u32,
}

/// Database backend options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DatabaseBackend {
    /// Content-addressed / durable storage (discover provider by capability at runtime)
    #[serde(rename = "content_addressed", alias = "nestgate")]
    ContentAddressed,

    /// PostgreSQL database
    #[serde(rename = "postgres")]
    PostgreSQL,

    /// SQLite database
    #[serde(rename = "sqlite")]
    #[default]
    SQLite,

    /// In-memory database (for testing)
    #[serde(rename = "memory")]
    Memory,
}

/// Load balancing configuration
///
/// Migrated from universal/ system — provides sophisticated load balancing strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing strategy
    #[serde(default)]
    pub strategy: LoadBalancingStrategy,

    /// Enable sticky sessions
    #[serde(default)]
    pub sticky_sessions: bool,

    /// Session affinity timeout (seconds)
    #[serde(default = "crate::unified::types::defaults::default_session_timeout")]
    pub session_timeout_secs: u64,

    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Health-based routing
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub health_based_routing: bool,

    /// Retry failed requests
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub retry_failed: bool,

    /// Maximum retries
    #[serde(default = "crate::unified::types::defaults::default_max_retries")]
    pub max_retries: u32,
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoadBalancingStrategy {
    /// Round robin distribution
    #[default]
    RoundRobin,

    /// Random selection
    Random,

    /// Least connections first
    LeastConnections,

    /// Weighted round robin
    WeightedRoundRobin,

    /// Health-based selection
    HealthBased,

    /// Response time based
    ResponseTime,

    /// Consistent hashing
    ConsistentHash,
}

/// Circuit breaker configuration (already in unified/, ensuring completeness)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enabled: bool,

    /// Failure threshold before opening circuit
    #[serde(default = "crate::unified::types::defaults::default_failure_threshold")]
    pub failure_threshold: u32,

    /// Success threshold to close circuit
    #[serde(default = "crate::unified::types::defaults::default_success_threshold")]
    pub success_threshold: u32,

    /// Timeout before attempting to close circuit (seconds)
    #[serde(default = "crate::unified::types::defaults::default_circuit_timeout")]
    pub timeout_secs: u64,

    /// Half-open state max requests
    #[serde(default = "crate::unified::types::defaults::default_half_open_requests")]
    pub half_open_max_requests: u32,
}
