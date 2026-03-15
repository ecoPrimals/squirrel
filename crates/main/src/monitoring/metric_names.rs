// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

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
pub use songbird::*;
pub use toadstool::*;

/// AI Intelligence component metrics
pub mod ai_intelligence {
    pub const REQUESTS_PROCESSED: &str = "requests_processed";
    pub const AVG_PROCESSING_TIME: &str = "avg_processing_time";
    pub const SUCCESS_RATE: &str = "success_rate";
    pub const MEMORY_USAGE: &str = "memory_usage";
}

/// MCP Integration component metrics
pub mod mcp_integration {
    pub const MESSAGES_SENT: &str = "messages_sent";
    pub const MESSAGES_RECEIVED: &str = "messages_received";
    pub const CONNECTION_COUNT: &str = "connection_count";
    pub const PROTOCOL_ERRORS: &str = "protocol_errors";
}

/// Context State component metrics
pub mod context_state {
    pub const ACTIVE_SESSIONS: &str = "active_sessions";
    pub const CONTEXT_SIZE: &str = "context_size";
    pub const CACHE_HIT_RATE: &str = "cache_hit_rate";
    pub const PERSISTENCE_LATENCY: &str = "persistence_latency";
}

/// Agent Deployment component metrics
pub mod agent_deployment {
    pub const DEPLOYED_AGENTS: &str = "deployed_agents";
    pub const RUNNING_AGENTS: &str = "running_agents";
    pub const FAILED_DEPLOYMENTS: &str = "failed_deployments";
    pub const DEPLOYMENT_TIME: &str = "deployment_time";
}

/// Songbird orchestration metrics
pub mod songbird {
    pub const ORCHESTRATIONS_ACTIVE: &str = "orchestrations_active";
    pub const SERVICE_DISCOVERIES: &str = "service_discoveries";
    pub const LOAD_BALANCER_REQUESTS: &str = "load_balancer_requests";
    pub const HEALTH_CHECKS: &str = "health_checks";
}

/// Toadstool compute metrics
pub mod toadstool {
    pub const COMPUTE_JOBS_QUEUED: &str = "compute_jobs_queued";
    pub const COMPUTE_JOBS_RUNNING: &str = "compute_jobs_running";
    pub const COMPUTE_JOBS_COMPLETED: &str = "compute_jobs_completed";
    pub const CPU_UTILIZATION: &str = "cpu_utilization";
}

/// `NestGate` storage metrics
pub mod nestgate {
    pub const STORAGE_OPERATIONS: &str = "storage_operations";
    pub const STORAGE_SIZE_GB: &str = "storage_size_gb";
    pub const BACKUP_OPERATIONS: &str = "backup_operations";
    pub const REPLICATION_LAG: &str = "replication_lag";
}

/// `BearDog` security metrics
pub mod beardog {
    pub const AUTHENTICATION_REQUESTS: &str = "authentication_requests";
    pub const AUTHORIZATION_CHECKS: &str = "authorization_checks";
    pub const SECURITY_VIOLATIONS: &str = "security_violations";
    pub const TOKEN_REFRESHES: &str = "token_refreshes";
}

/// Default component metrics
pub mod default {
    pub const STATUS: &str = "status";
    pub const UPTIME: &str = "uptime";
}
