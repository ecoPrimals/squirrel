// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Type definitions for agent management
//!
//! Agent health, registration, summary, and health-check configuration
//! used by [`super::agent::AgentRegistry`].

use crate::AgentSpec;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

/// Health status of a registered agent
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentHealthStatus {
    /// Agent is healthy and available
    Healthy,
    /// Agent is functional but with degraded performance
    Degraded,
    /// Agent is experiencing issues
    Unhealthy,
    /// Agent is offline or unreachable
    Offline,
}

/// A registered agent in the routing system
#[derive(Debug, Clone)]
pub struct RegisteredAgent {
    /// Unique agent identifier
    pub id: String,
    /// Agent endpoint URL
    pub endpoint: String,
    /// Capabilities provided by this agent
    pub capabilities: Vec<String>,
    /// Maximum concurrent tasks this agent can handle
    pub max_concurrent_tasks: u32,
    /// Current load (number of active tasks)
    pub current_load: Arc<RwLock<u32>>,
    /// Average response time in milliseconds
    pub average_response_time: Arc<RwLock<f64>>,
    /// Current health status
    pub health_status: Arc<RwLock<AgentHealthStatus>>,
    /// Last time this agent was seen
    pub last_seen: Arc<RwLock<DateTime<Utc>>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl RegisteredAgent {
    /// Create a new registered agent from an agent specification
    #[must_use]
    pub fn new(spec: AgentSpec) -> Self {
        Self {
            id: spec.id,
            endpoint: spec.endpoint,
            capabilities: spec.capabilities,
            max_concurrent_tasks: spec.max_concurrent_tasks,
            current_load: Arc::new(RwLock::new(0)),
            average_response_time: Arc::new(RwLock::new(0.0)),
            health_status: Arc::new(RwLock::new(AgentHealthStatus::Healthy)),
            last_seen: Arc::new(RwLock::new(Utc::now())),
            metadata: spec.metadata,
        }
    }

    /// Check if the agent has a specific capability
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Check if the agent has all required capabilities
    #[must_use]
    pub fn has_all_capabilities(&self, required_capabilities: &[String]) -> bool {
        required_capabilities
            .iter()
            .all(|cap| self.has_capability(cap))
    }

    /// Get current load as a percentage of max capacity
    #[must_use]
    pub fn load_percentage(&self) -> f64 {
        let current_load = f64::from(
            *self
                .current_load
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let max_capacity = f64::from(self.max_concurrent_tasks);
        if max_capacity > 0.0 {
            (current_load / max_capacity) * 100.0
        } else {
            0.0
        }
    }

    /// Check if the agent is available for new tasks
    #[must_use]
    pub fn is_available(&self) -> bool {
        let health_status = self
            .health_status
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current_load = *self
            .current_load
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        matches!(
            *health_status,
            AgentHealthStatus::Healthy | AgentHealthStatus::Degraded
        ) && current_load < self.max_concurrent_tasks
    }

    /// Check if the agent is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        let health_status = self
            .health_status
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        matches!(*health_status, AgentHealthStatus::Healthy)
    }

    /// Increment current load
    pub fn increment_load(&self) {
        let mut current_load = self
            .current_load
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *current_load += 1;
    }

    /// Decrement current load
    pub fn decrement_load(&self) {
        let mut current_load = self
            .current_load
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *current_load > 0 {
            *current_load -= 1;
        }
    }

    /// Update average response time
    pub fn update_response_time(&self, response_time_ms: f64) {
        let mut avg_response_time = self
            .average_response_time
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *avg_response_time = (*avg_response_time).mul_add(0.9, response_time_ms * 0.1);
    }

    /// Update health status
    pub fn update_health_status(&self, status: AgentHealthStatus) {
        let mut health_status = self
            .health_status
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *health_status = status;
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&self) {
        let mut last_seen = self
            .last_seen
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *last_seen = Utc::now();
    }

    /// Get time since last seen
    #[must_use]
    pub fn time_since_last_seen(&self) -> chrono::Duration {
        let last_seen = *self
            .last_seen
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Utc::now() - last_seen
    }

    /// Get a summary of the agent's current state
    #[must_use]
    pub fn get_summary(&self) -> AgentSummary {
        AgentSummary {
            id: self.id.clone(),
            endpoint: self.endpoint.clone(),
            capabilities: self.capabilities.clone(),
            max_concurrent_tasks: self.max_concurrent_tasks,
            current_load: *self
                .current_load
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            load_percentage: self.load_percentage(),
            average_response_time: *self
                .average_response_time
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            health_status: self
                .health_status
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
            last_seen: *self
                .last_seen
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            time_since_last_seen: self.time_since_last_seen(),
            is_available: self.is_available(),
        }
    }
}

/// Summary of an agent's current state
#[derive(Debug, Clone)]
pub struct AgentSummary {
    /// Agent ID
    pub id: String,
    /// Agent endpoint
    pub endpoint: String,
    /// Agent capabilities
    pub capabilities: Vec<String>,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: u32,
    /// Current load
    pub current_load: u32,
    /// Load as percentage of capacity
    pub load_percentage: f64,
    /// Average response time
    pub average_response_time: f64,
    /// Health status
    pub health_status: AgentHealthStatus,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
    /// Time since last seen
    pub time_since_last_seen: chrono::Duration,
    /// Whether agent is available
    pub is_available: bool,
}

/// Configuration for agent health checking
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub check_interval: chrono::Duration,
    /// Timeout for health check requests
    pub check_timeout: chrono::Duration,
    /// Number of consecutive failures before marking as unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes to mark as healthy
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: chrono::Duration::seconds(30),
            check_timeout: chrono::Duration::seconds(5),
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

impl HealthCheckConfig {
    /// Create a new health check configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set check interval
    #[must_use]
    pub const fn with_check_interval(mut self, interval: chrono::Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Set check timeout
    #[must_use]
    pub const fn with_check_timeout(mut self, timeout: chrono::Duration) -> Self {
        self.check_timeout = timeout;
        self
    }

    /// Set failure threshold
    #[must_use]
    pub const fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set success threshold
    #[must_use]
    pub const fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }
}

/// Statistics for the agent registry
#[derive(Debug, Clone)]
pub struct AgentRegistryStats {
    /// Total number of registered agents
    pub total_agents: usize,
    /// Number of healthy agents
    pub healthy_agents: usize,
    /// Number of available agents
    pub available_agents: usize,
    /// Total capacity across all agents
    pub total_capacity: u32,
    /// Current load across all agents
    pub current_load: u32,
    /// Capacity utilization percentage
    pub capacity_utilization: f64,
}

impl std::fmt::Display for AgentHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Degraded => write!(f, "Degraded"),
            Self::Unhealthy => write!(f, "Unhealthy"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}
