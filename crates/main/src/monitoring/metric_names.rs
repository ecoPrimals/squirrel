// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Metric name constants for zero-copy optimization
//!
//! These constants eliminate string allocations in hot paths by using
//! static string references instead of `String::from()` or .`to_string()`

// Re-export all constants at the module root for easy access
pub use agent_deployment::*;
pub use ai_intelligence::*;
pub use beardog::*;
pub use context_state::*;
pub use default::*;
pub use mcp_integration::*;
pub use nestgate::*;
pub use orchestration::*;
pub use toadstool::*;

/// AI Intelligence component metrics
pub mod ai_intelligence {
    /// Metric name for total requests processed
    pub const REQUESTS_PROCESSED: &str = "requests_processed";
    /// Metric name for average processing time
    pub const AVG_PROCESSING_TIME: &str = "avg_processing_time";
    /// Metric name for success rate
    pub const SUCCESS_RATE: &str = "success_rate";
    /// Metric name for memory usage
    pub const MEMORY_USAGE: &str = "memory_usage";
}

/// MCP Integration component metrics
pub mod mcp_integration {
    /// Metric name for messages sent
    pub const MESSAGES_SENT: &str = "messages_sent";
    /// Metric name for messages received
    pub const MESSAGES_RECEIVED: &str = "messages_received";
    /// Metric name for connection count
    pub const CONNECTION_COUNT: &str = "connection_count";
    /// Metric name for protocol errors
    pub const PROTOCOL_ERRORS: &str = "protocol_errors";
}

/// Context State component metrics
pub mod context_state {
    /// Metric name for active sessions
    pub const ACTIVE_SESSIONS: &str = "active_sessions";
    /// Metric name for context size
    pub const CONTEXT_SIZE: &str = "context_size";
    /// Metric name for cache hit rate
    pub const CACHE_HIT_RATE: &str = "cache_hit_rate";
    /// Metric name for persistence latency
    pub const PERSISTENCE_LATENCY: &str = "persistence_latency";
}

/// Agent Deployment component metrics
pub mod agent_deployment {
    /// Metric name for deployed agents count
    pub const DEPLOYED_AGENTS: &str = "deployed_agents";
    /// Metric name for running agents count
    pub const RUNNING_AGENTS: &str = "running_agents";
    /// Metric name for failed deployments count
    pub const FAILED_DEPLOYMENTS: &str = "failed_deployments";
    /// Metric name for deployment time
    pub const DEPLOYMENT_TIME: &str = "deployment_time";
}

/// Orchestration / service mesh metrics (capability-domain, primal-agnostic)
pub mod orchestration {
    /// Metric name for active orchestrations
    pub const ORCHESTRATIONS_ACTIVE: &str = "orchestrations_active";
    /// Metric name for service discoveries
    pub const SERVICE_DISCOVERIES: &str = "service_discoveries";
    /// Metric name for load balancer requests
    pub const LOAD_BALANCER_REQUESTS: &str = "load_balancer_requests";
    /// Metric name for health checks
    pub const HEALTH_CHECKS: &str = "health_checks";
}

/// Toadstool compute metrics
pub mod toadstool {
    /// Metric name for queued compute jobs
    pub const COMPUTE_JOBS_QUEUED: &str = "compute_jobs_queued";
    /// Metric name for running compute jobs
    pub const COMPUTE_JOBS_RUNNING: &str = "compute_jobs_running";
    /// Metric name for completed compute jobs
    pub const COMPUTE_JOBS_COMPLETED: &str = "compute_jobs_completed";
    /// Metric name for CPU utilization
    pub const CPU_UTILIZATION: &str = "cpu_utilization";
}

/// `NestGate` storage metrics
pub mod nestgate {
    /// Metric name for storage operations count
    pub const STORAGE_OPERATIONS: &str = "storage_operations";
    /// Metric name for storage size in GB
    pub const STORAGE_SIZE_GB: &str = "storage_size_gb";
    /// Metric name for backup operations
    pub const BACKUP_OPERATIONS: &str = "backup_operations";
    /// Metric name for replication lag
    pub const REPLICATION_LAG: &str = "replication_lag";
}

/// `BearDog` security metrics
pub mod beardog {
    /// Metric name for authentication requests
    pub const AUTHENTICATION_REQUESTS: &str = "authentication_requests";
    /// Metric name for authorization checks
    pub const AUTHORIZATION_CHECKS: &str = "authorization_checks";
    /// Metric name for security violations
    pub const SECURITY_VIOLATIONS: &str = "security_violations";
    /// Metric name for token refreshes
    pub const TOKEN_REFRESHES: &str = "token_refreshes";
}

/// Default component metrics
pub mod default {
    /// Metric name for component status
    pub const STATUS: &str = "status";
    /// Metric name for uptime
    pub const UPTIME: &str = "uptime";
}
