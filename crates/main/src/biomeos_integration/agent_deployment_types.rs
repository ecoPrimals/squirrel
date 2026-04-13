// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Types, configuration, and defaults for agent deployment.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::manifest::{AgentResourceLimits, AgentSpec, ExecutionEnvironment};

/// Deployed agent instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedAgent {
    /// Agent identifier
    pub agent_id: String,
    /// Agent name from specification
    pub name: String,
    /// Agent specification
    pub spec: AgentSpec,
    /// Current status
    pub status: AgentStatus,
    /// Deployment timestamp
    pub deployed_at: DateTime<Utc>,
    /// Last health check
    pub last_health_check: DateTime<Utc>,
    /// Resource usage
    pub resource_usage: AgentResourceUsage,
    /// Agent endpoints
    pub endpoints: AgentEndpoints,
    /// Agent metadata
    pub metadata: HashMap<String, String>,
}

/// Agent status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is being deployed
    Deploying,
    /// Agent is running and healthy
    Running,
    /// Agent is starting up
    Starting,
    /// Agent is stopping
    Stopping,
    /// Agent has stopped
    Stopped,
    /// Agent has failed
    Failed(String),
    /// Agent is being scaled
    Scaling,
    /// Agent is being updated
    Updating,
}

/// Agent resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Storage usage in MB
    pub storage_mb: u64,
    /// Network bandwidth usage in Mbps
    pub network_mbps: f64,
    /// Number of active requests
    pub active_requests: u32,
    /// Total requests processed
    pub total_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

/// Agent endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEndpoints {
    /// Agent API endpoint
    pub api: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: String,
    /// WebSocket endpoint (if available)
    pub websocket: Option<String>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    /// Total number of agents deployed
    pub total_agents: u32,
    /// Number of running agents
    pub running_agents: u32,
    /// Number of failed agents
    pub failed_agents: u32,
    /// Overall deployment health
    pub health: DeploymentHealth,
    /// Last deployment timestamp
    pub last_deployment: DateTime<Utc>,
    /// Deployment metrics
    pub metrics: DeploymentMetrics,
}

/// Deployment health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentHealth {
    /// All agents are healthy
    Healthy,
    /// Some agents are unhealthy but deployment is functional
    Degraded,
    /// Deployment is not functional
    Unhealthy,
    /// Deployment is in unknown state
    Unknown,
}

/// Deployment metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    /// Total deployments
    pub total_deployments: u64,
    /// Successful deployments
    pub successful_deployments: u64,
    /// Failed deployments
    pub failed_deployments: u64,
    /// Average deployment time in seconds
    pub avg_deployment_time_seconds: f64,
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization percentage
    pub cpu_percent: f64,
    /// Memory utilization percentage
    pub memory_percent: f64,
    /// Storage utilization percentage
    pub storage_percent: f64,
    /// Network utilization percentage
    pub network_percent: f64,
}

/// Configuration for agent deployment
#[derive(Debug, Clone)]
pub struct AgentDeploymentConfig {
    /// Maximum number of concurrent agents
    pub max_concurrent_agents: u32,
    /// Default resource limits
    pub default_resource_limits: AgentResourceLimits,
    /// Deployment timeout in seconds
    pub deployment_timeout_seconds: u64,
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
    /// Auto-scaling configuration
    pub auto_scaling: AutoScalingConfig,
    /// Security configuration
    pub security: DeploymentSecurityConfig,
}

/// Auto-scaling configuration for agents
#[derive(Debug, Clone)]
pub struct AutoScalingConfig {
    /// Enable auto-scaling
    pub enabled: bool,
    /// Minimum number of agents
    pub min_agents: u32,
    /// Maximum number of agents
    pub max_agents: u32,
    /// CPU utilization threshold for scaling up
    pub scale_up_cpu_threshold: f64,
    /// CPU utilization threshold for scaling down
    pub scale_down_cpu_threshold: f64,
    /// Memory utilization threshold for scaling up
    pub scale_up_memory_threshold: f64,
    /// Memory utilization threshold for scaling down
    pub scale_down_memory_threshold: f64,
}

/// Security configuration for agent deployment
#[derive(Debug, Clone)]
pub struct DeploymentSecurityConfig {
    /// Enable secure deployment
    pub enabled: bool,
    /// Security context validation
    pub validate_security_context: bool,
    /// Require encryption for agent communication
    pub require_encryption: bool,
    /// Allowed AI providers
    pub allowed_ai_providers: Vec<String>,
    /// Allowed execution environments
    pub allowed_execution_environments: Vec<ExecutionEnvironment>,
}

// ── Default implementations ──────────────────────────────────────────

impl Default for AgentDeploymentConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: 10,
            default_resource_limits: AgentResourceLimits::default(),
            deployment_timeout_seconds: 300,
            health_check_interval_seconds: 30,
            auto_scaling: AutoScalingConfig::default(),
            security: DeploymentSecurityConfig::default(),
        }
    }
}

impl Default for AutoScalingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_agents: 1,
            max_agents: 10,
            scale_up_cpu_threshold: 80.0,
            scale_down_cpu_threshold: 20.0,
            scale_up_memory_threshold: 80.0,
            scale_down_memory_threshold: 20.0,
        }
    }
}

impl Default for DeploymentSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            validate_security_context: true,
            require_encryption: true,
            allowed_ai_providers: vec![
                "openai".to_string(),
                "anthropic".to_string(),
                "local".to_string(),
            ],
            allowed_execution_environments: vec![
                ExecutionEnvironment::Wasm,
                ExecutionEnvironment::Container,
            ],
        }
    }
}

impl Default for AgentResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0,
            storage_mb: 0,
            network_mbps: 0.0,
            active_requests: 0,
            total_requests: 0,
            avg_response_time_ms: 0.0,
        }
    }
}

impl Default for DeploymentStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentStatus {
    /// Creates a new deployment status with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_agents: 0,
            running_agents: 0,
            failed_agents: 0,
            health: DeploymentHealth::Unknown,
            last_deployment: Utc::now(),
            metrics: DeploymentMetrics::default(),
        }
    }
}

impl Default for DeploymentMetrics {
    fn default() -> Self {
        Self {
            total_deployments: 0,
            successful_deployments: 0,
            failed_deployments: 0,
            avg_deployment_time_seconds: 0.0,
            resource_utilization: ResourceUtilization::default(),
        }
    }
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_percent: 0.0,
            storage_percent: 0.0,
            network_percent: 0.0,
        }
    }
}
