// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Agent registry for MCP task routing
//!
//! This module handles agent registration, health monitoring, and lifecycle management
//! for MCP agents participating in task routing.
//!
//! Type definitions live in [`super::agent_types`].

pub use super::agent_types::*;
use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::{info, warn};

/// Agent registry for managing registered agents
#[derive(Debug)]
pub struct AgentRegistry {
    /// Map of registered agents by ID
    agents: Arc<RwLock<HashMap<String, RegisteredAgent>>>,
    /// Health check configuration
    health_check_config: HealthCheckConfig,
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
    pub fn register_agent(&self, spec: crate::AgentSpec) -> Result<()> {
        let agent = RegisteredAgent::new(spec);
        let agent_id = agent.id.clone();
        let capability_count = agent.capabilities.len();

        {
            let mut agents = self
                .agents
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        let mut agents = self
            .agents
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        let agents = self
            .agents
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        agents.get(agent_id).cloned()
    }

    /// Get all registered agents
    #[must_use]
    pub fn get_all_agents(&self) -> Vec<RegisteredAgent> {
        let agents = self
            .agents
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        agents.values().cloned().collect()
    }

    /// Get agents with specific capabilities
    #[must_use]
    pub fn get_agents_with_capabilities(
        &self,
        required_capabilities: &[String],
    ) -> Vec<RegisteredAgent> {
        let agents = self
            .agents
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        agents
            .values()
            .filter(|agent| agent.has_all_capabilities(required_capabilities))
            .cloned()
            .collect()
    }

    /// Get available agents (healthy and not at capacity)
    #[must_use]
    pub fn get_available_agents(&self) -> Vec<RegisteredAgent> {
        let agents = self
            .agents
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        agents
            .values()
            .filter(|agent| agent.is_available())
            .cloned()
            .collect()
    }

    /// Get healthy agents
    #[must_use]
    pub fn get_healthy_agents(&self) -> Vec<RegisteredAgent> {
        let agents = self
            .agents
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
            let agents = self
                .agents
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
            let agents = self
                .agents
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
            let agents = self
                .agents
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
            let agents = self
                .agents
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
            let guard = self
                .agents
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.values().cloned().collect()
        };
        agents.iter().map(RegisteredAgent::get_summary).collect()
    }

    /// Get registry statistics
    #[must_use]
    pub fn get_statistics(&self) -> AgentRegistryStats {
        let agents: Vec<RegisteredAgent> = {
            let guard = self
                .agents
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.values().cloned().collect()
        };
        let total_agents = agents.len();
        let healthy_agents = agents.iter().filter(|a| a.is_healthy()).count();
        let available_agents = agents.iter().filter(|a| a.is_available()).count();
        let total_capacity = agents.iter().map(|a| a.max_concurrent_tasks).sum::<u32>();
        let current_load = agents
            .iter()
            .map(|a| {
                *a.current_load
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
            })
            .sum::<u32>();

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
        let cutoff_time = chrono::Utc::now() - max_age;
        let stale_ids: Vec<String> = {
            let agents = self
                .agents
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            agents
                .iter()
                .filter_map(|(id, agent)| {
                    let last_seen = *agent
                        .last_seen
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    (last_seen < cutoff_time).then_some(id.clone())
                })
                .collect()
        };

        let mut removed_agents = Vec::new();
        {
            let mut agents = self
                .agents
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        assert_eq!(
            *agent
                .current_load
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            1
        );
        assert!(agent.load_percentage() > 0.0);
        agent.update_response_time(100.0);
        assert!(
            *agent
                .average_response_time
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                > 0.0
        );
        agent.update_health_status(AgentHealthStatus::Unhealthy);
        assert!(!agent.is_healthy());
        assert!(!agent.is_available());
        agent.decrement_load();
        agent.decrement_load();
        assert_eq!(
            *agent
                .current_load
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            0
        );
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
