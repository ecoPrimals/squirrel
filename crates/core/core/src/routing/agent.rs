// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Agent management for MCP task routing
//!
//! This module handles agent registration, health monitoring, and lifecycle management
//! for MCP agents participating in task routing.

use crate::{AgentSpec, Error, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

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

/// Agent registry for managing registered agents
#[derive(Debug)]
pub struct AgentRegistry {
    /// Map of registered agents by ID
    agents: Arc<RwLock<HashMap<String, RegisteredAgent>>>,
    /// Health check configuration
    health_check_config: HealthCheckConfig,
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
        let current_load = f64::from(*self.current_load.read());
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
        let health_status = self.health_status.read();
        let current_load = *self.current_load.read();

        matches!(
            *health_status,
            AgentHealthStatus::Healthy | AgentHealthStatus::Degraded
        ) && current_load < self.max_concurrent_tasks
    }

    /// Check if the agent is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        let health_status = self.health_status.read();
        matches!(*health_status, AgentHealthStatus::Healthy)
    }

    /// Increment current load
    pub fn increment_load(&self) {
        let mut current_load = self.current_load.write();
        *current_load += 1;
    }

    /// Decrement current load
    pub fn decrement_load(&self) {
        let mut current_load = self.current_load.write();
        if *current_load > 0 {
            *current_load -= 1;
        }
    }

    /// Update average response time
    pub fn update_response_time(&self, response_time_ms: f64) {
        let mut avg_response_time = self.average_response_time.write();
        // Simple moving average with weight 0.1 for new values
        *avg_response_time = (*avg_response_time).mul_add(0.9, response_time_ms * 0.1);
    }

    /// Update health status
    pub fn update_health_status(&self, status: AgentHealthStatus) {
        let mut health_status = self.health_status.write();
        *health_status = status;
    }

    /// Update last seen timestamp
    pub fn update_last_seen(&self) {
        let mut last_seen = self.last_seen.write();
        *last_seen = Utc::now();
    }

    /// Get time since last seen
    #[must_use]
    pub fn time_since_last_seen(&self) -> chrono::Duration {
        let last_seen = *self.last_seen.read();
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
            current_load: *self.current_load.read(),
            load_percentage: self.load_percentage(),
            average_response_time: *self.average_response_time.read(),
            health_status: self.health_status.read().clone(),
            last_seen: *self.last_seen.read(),
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

impl AgentRegistry {
    /// Create a new agent registry
    #[must_use]
    pub fn new(health_check_config: HealthCheckConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            health_check_config,
        }
    }

    /// Register a new agent
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if registration fails.
    pub fn register_agent(&self, spec: AgentSpec) -> Result<()> {
        let agent = RegisteredAgent::new(spec);
        let agent_id = agent.id.clone();
        let capability_count = agent.capabilities.len();

        {
            let mut agents = self.agents.write();
            agents.insert(agent_id.clone(), agent);
        }

        info!(
            "Registered agent '{}' with {} capabilities",
            agent_id, capability_count
        );
        Ok(())
    }

    /// Unregister an agent
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the agent is not found.
    pub fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write();
        if agents.remove(agent_id).is_some() {
            info!("Unregistered agent '{}'", agent_id);
            Ok(())
        } else {
            Err(Error::AgentNotFound(agent_id.to_string()))
        }
    }

    /// Get an agent by ID
    #[must_use]
    pub fn get_agent(&self, agent_id: &str) -> Option<RegisteredAgent> {
        let agents = self.agents.read();
        agents.get(agent_id).cloned()
    }

    /// Get all registered agents
    #[must_use]
    pub fn get_all_agents(&self) -> Vec<RegisteredAgent> {
        let agents = self.agents.read();
        agents.values().cloned().collect()
    }

    /// Get agents with specific capabilities
    #[must_use]
    pub fn get_agents_with_capabilities(
        &self,
        required_capabilities: &[String],
    ) -> Vec<RegisteredAgent> {
        let agents = self.agents.read();
        agents
            .values()
            .filter(|agent| agent.has_all_capabilities(required_capabilities))
            .cloned()
            .collect()
    }

    /// Get available agents (healthy and not at capacity)
    #[must_use]
    pub fn get_available_agents(&self) -> Vec<RegisteredAgent> {
        let agents = self.agents.read();
        agents
            .values()
            .filter(|agent| agent.is_available())
            .cloned()
            .collect()
    }

    /// Get healthy agents
    #[must_use]
    pub fn get_healthy_agents(&self) -> Vec<RegisteredAgent> {
        let agents = self.agents.read();
        agents
            .values()
            .filter(|agent| agent.is_healthy())
            .cloned()
            .collect()
    }

    /// Update agent health status
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the agent is not found.
    pub fn update_agent_health(&self, agent_id: &str, status: AgentHealthStatus) -> Result<()> {
        let agent = {
            let agents = self.agents.read();
            agents.get(agent_id).cloned()
        };
        agent.map_or_else(
            || Err(Error::AgentNotFound(agent_id.to_string())),
            |agent| {
                agent.update_health_status(status);
                agent.update_last_seen();
                Ok(())
            },
        )
    }

    /// Update agent response time
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the agent is not found.
    pub fn update_agent_response_time(&self, agent_id: &str, response_time_ms: f64) -> Result<()> {
        let agent = {
            let agents = self.agents.read();
            agents.get(agent_id).cloned()
        };
        agent.map_or_else(
            || Err(Error::AgentNotFound(agent_id.to_string())),
            |agent| {
                agent.update_response_time(response_time_ms);
                agent.update_last_seen();
                Ok(())
            },
        )
    }

    /// Increment agent load
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the agent is not found.
    pub fn increment_agent_load(&self, agent_id: &str) -> Result<()> {
        let agent = {
            let agents = self.agents.read();
            agents.get(agent_id).cloned()
        };
        agent.map_or_else(
            || Err(Error::AgentNotFound(agent_id.to_string())),
            |agent| {
                agent.increment_load();
                Ok(())
            },
        )
    }

    /// Decrement agent load
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the agent is not found.
    pub fn decrement_agent_load(&self, agent_id: &str) -> Result<()> {
        let agent = {
            let agents = self.agents.read();
            agents.get(agent_id).cloned()
        };
        agent.map_or_else(
            || Err(Error::AgentNotFound(agent_id.to_string())),
            |agent| {
                agent.decrement_load();
                Ok(())
            },
        )
    }

    /// Get agent summaries
    pub fn get_agent_summaries(&self) -> Vec<AgentSummary> {
        let agents: Vec<RegisteredAgent> = {
            let guard = self.agents.read();
            guard.values().cloned().collect()
        };
        agents.iter().map(RegisteredAgent::get_summary).collect()
    }

    /// Get registry statistics
    #[must_use]
    pub fn get_statistics(&self) -> AgentRegistryStats {
        let agents: Vec<RegisteredAgent> = {
            let guard = self.agents.read();
            guard.values().cloned().collect()
        };
        let total_agents = agents.len();
        let healthy_agents = agents.iter().filter(|a| a.is_healthy()).count();
        let available_agents = agents.iter().filter(|a| a.is_available()).count();
        let total_capacity = agents.iter().map(|a| a.max_concurrent_tasks).sum::<u32>();
        let current_load = agents.iter().map(|a| *a.current_load.read()).sum::<u32>();

        AgentRegistryStats {
            total_agents,
            healthy_agents,
            available_agents,
            total_capacity,
            current_load,
            capacity_utilization: if total_capacity > 0 {
                (f64::from(current_load) / f64::from(total_capacity)) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Clean up stale agents (those not seen for a long time)
    pub fn cleanup_stale_agents(&self, max_age: chrono::Duration) -> Vec<String> {
        let cutoff_time = Utc::now() - max_age;
        let stale_ids: Vec<String> = {
            let agents = self.agents.read();
            agents
                .iter()
                .filter_map(|(id, agent)| {
                    let last_seen = *agent.last_seen.read();
                    (last_seen < cutoff_time).then_some(id.clone())
                })
                .collect()
        };

        let mut removed_agents = Vec::new();
        {
            let mut agents = self.agents.write();
            for id in stale_ids {
                if agents.remove(&id).is_some() {
                    removed_agents.push(id);
                }
            }
        }

        for agent_id in &removed_agents {
            warn!(
                "Removed stale agent '{}' (not seen for {:?})",
                agent_id, max_age
            );
        }

        removed_agents
    }

    /// Get health check configuration
    #[must_use]
    pub const fn get_health_check_config(&self) -> &HealthCheckConfig {
        &self.health_check_config
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgentSpec;
    use crate::Error;
    use std::collections::HashMap;

    fn sample_spec(id: &str) -> AgentSpec {
        AgentSpec {
            id: id.to_string(),
            endpoint: format!("http://{id}.local"),
            capabilities: vec!["mcp".to_string(), "tools".to_string()],
            weight: Some(1.0),
            max_concurrent_tasks: 4,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn health_check_config_default_matches_builder() {
        let d = HealthCheckConfig::default();
        let n = HealthCheckConfig::new();
        assert_eq!(d.check_interval, n.check_interval);
        assert_eq!(d.failure_threshold, 3);
        assert_eq!(d.success_threshold, 2);
        let c = HealthCheckConfig::default()
            .with_check_interval(chrono::Duration::seconds(12))
            .with_check_timeout(chrono::Duration::seconds(2))
            .with_failure_threshold(5)
            .with_success_threshold(1);
        assert_eq!(c.check_interval, chrono::Duration::seconds(12));
        assert_eq!(c.check_timeout, chrono::Duration::seconds(2));
        assert_eq!(c.failure_threshold, 5);
        assert_eq!(c.success_threshold, 1);
    }

    #[test]
    fn registered_agent_helpers_reflect_load_and_health() {
        let agent = RegisteredAgent::new(sample_spec("ag-1"));
        assert!(agent.has_capability("mcp"));
        assert!(agent.has_all_capabilities(&["mcp".to_string(), "tools".to_string()]));
        assert!(!agent.has_capability("other"));
        assert!(agent.load_percentage().abs() < f64::EPSILON);
        assert!(agent.is_available());
        assert!(agent.is_healthy());
        agent.increment_load();
        assert_eq!(*agent.current_load.read(), 1);
        assert!(agent.load_percentage() > 0.0);
        agent.update_response_time(100.0);
        assert!(*agent.average_response_time.read() > 0.0);
        agent.update_health_status(AgentHealthStatus::Unhealthy);
        assert!(!agent.is_healthy());
        assert!(!agent.is_available());
        agent.decrement_load();
        agent.decrement_load();
        assert_eq!(*agent.current_load.read(), 0);
        let s = agent.get_summary();
        assert_eq!(s.id, "ag-1");
        assert!(!s.is_available);
    }

    #[test]
    fn agent_registry_register_unregister_and_lookups() {
        let reg = AgentRegistry::new(HealthCheckConfig::default());
        reg.register_agent(sample_spec("x")).expect("register");
        assert_eq!(reg.get_agent("x").expect("get").id, "x");
        let cap: Vec<String> = vec!["mcp".to_string()];
        assert_eq!(reg.get_agents_with_capabilities(&cap).len(), 1);
        assert_eq!(reg.get_available_agents().len(), 1);
        reg.update_agent_health("x", AgentHealthStatus::Offline)
            .expect("health");
        assert_eq!(reg.get_available_agents().len(), 0);
        reg.update_agent_health("x", AgentHealthStatus::Healthy)
            .expect("health");
        assert_eq!(reg.get_available_agents().len(), 1);
        assert_eq!(reg.get_healthy_agents().len(), 1);
        reg.unregister_agent("x").expect("unreg");
        assert!(reg.get_agent("x").is_none());
    }

    #[test]
    fn agent_registry_errors_on_missing_agent() {
        let reg = AgentRegistry::new(HealthCheckConfig::default());
        assert!(matches!(
            reg.unregister_agent("nope"),
            Err(Error::AgentNotFound(_))
        ));
        assert!(matches!(
            reg.update_agent_health("nope", AgentHealthStatus::Healthy),
            Err(Error::AgentNotFound(_))
        ));
        assert!(matches!(
            reg.update_agent_response_time("nope", 1.0),
            Err(Error::AgentNotFound(_))
        ));
        assert!(matches!(
            reg.increment_agent_load("nope"),
            Err(Error::AgentNotFound(_))
        ));
        assert!(matches!(
            reg.decrement_agent_load("nope"),
            Err(Error::AgentNotFound(_))
        ));
    }

    #[test]
    fn agent_registry_statistics_and_cleanup() {
        let reg = AgentRegistry::new(HealthCheckConfig::default());
        reg.register_agent(sample_spec("a")).expect("r");
        reg.increment_agent_load("a").expect("inc");
        let stats = reg.get_statistics();
        assert_eq!(stats.total_agents, 1);
        assert_eq!(stats.current_load, 1);
        assert!(stats.capacity_utilization > 0.0);
        let summaries = reg.get_agent_summaries();
        assert_eq!(summaries.len(), 1);
        std::thread::sleep(std::time::Duration::from_millis(25));
        let removed = reg.cleanup_stale_agents(chrono::Duration::nanoseconds(1));
        assert_eq!(removed.len(), 1);
        assert!(reg.get_all_agents().is_empty());
    }

    #[test]
    fn agent_health_status_display() {
        assert_eq!(AgentHealthStatus::Healthy.to_string(), "Healthy");
        assert_eq!(AgentHealthStatus::Offline.to_string(), "Offline");
    }
}
