// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Load balancer for MCP task routing
//!
//! This module handles load balancing across multiple MCP agents, performance tracking,
//! and intelligent agent selection based on various strategies.

use super::agent::RegisteredAgent;
use super::config::LoadBalancingStrategy;
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Performance metric for tracking agent performance
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    /// When this metric was recorded
    pub timestamp: DateTime<Utc>,
    /// Response time for the operation
    pub response_time: chrono::Duration,
    /// Whether the operation was successful
    pub success: bool,
    /// Current load on the agent
    pub load: u32,
}

/// Load balancer for selecting agents based on various strategies
#[derive(Debug)]
pub struct LoadBalancer {
    /// Load balancing strategy to use
    strategy: LoadBalancingStrategy,
    /// Weights for weighted round-robin
    weights: RwLock<HashMap<String, f64>>,
    /// Performance history for agents
    performance_history: RwLock<HashMap<String, Vec<PerformanceMetric>>>,
    /// Semaphore for concurrency control
    semaphore: Arc<Semaphore>,
    /// Round-robin counter
    round_robin_counter: RwLock<usize>,
}

impl LoadBalancer {
    /// Create a new load balancer with the specified strategy
    pub fn new(strategy: LoadBalancingStrategy, max_concurrent: usize) -> Self {
        Self {
            strategy,
            weights: RwLock::new(HashMap::new()),
            performance_history: RwLock::new(HashMap::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            round_robin_counter: RwLock::new(0),
        }
    }

    /// Select an agent from the available agents using the configured strategy
    pub async fn select_agent(&self, agents: Vec<RegisteredAgent>) -> Result<RegisteredAgent> {
        if agents.is_empty() {
            return Err(Error::NoAgentAvailable);
        }

        let selected_agent = match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.select_round_robin(agents).await?,
            LoadBalancingStrategy::LeastConnections => {
                self.select_least_connections(agents).await?
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(agents).await?
            }
            LoadBalancingStrategy::ResponseTimeBased => {
                self.select_response_time_based(agents).await?
            }
            LoadBalancingStrategy::CapabilityBased => self.select_capability_based(agents).await?,
            LoadBalancingStrategy::Adaptive => self.select_adaptive(agents).await?,
        };

        debug!(
            "Selected agent '{}' using strategy {:?}",
            selected_agent.id, self.strategy
        );
        Ok(selected_agent)
    }

    /// Select agent using round-robin strategy
    async fn select_round_robin(&self, agents: Vec<RegisteredAgent>) -> Result<RegisteredAgent> {
        let mut counter = self.round_robin_counter.write();
        let index = *counter % agents.len();
        *counter = (*counter + 1) % agents.len();
        Ok(agents[index].clone())
    }

    /// Select agent with least connections
    async fn select_least_connections(
        &self,
        agents: Vec<RegisteredAgent>,
    ) -> Result<RegisteredAgent> {
        let mut best_agent = None;
        let mut min_load = u32::MAX;

        for agent in agents {
            let current_load = *agent.current_load.read();
            if current_load < min_load {
                min_load = current_load;
                best_agent = Some(agent);
            }
        }

        best_agent.ok_or(Error::NoAgentAvailable)
    }

    /// Select agent using weighted round-robin
    async fn select_weighted_round_robin(
        &self,
        agents: Vec<RegisteredAgent>,
    ) -> Result<RegisteredAgent> {
        let weights = self.weights.read();
        let mut total_weight = 0.0;
        let mut weighted_agents = Vec::new();

        for agent in agents {
            let weight = weights.get(&agent.id).copied().unwrap_or(1.0);
            total_weight += weight;
            weighted_agents.push((agent, weight));
        }

        if total_weight == 0.0 {
            return Err(Error::NoAgentAvailable);
        }

        let mut counter = self.round_robin_counter.write();
        let target = (*counter as f64 / 100.0) % total_weight;
        *counter = (*counter + 1) % (total_weight as usize * 100);

        let mut cumulative_weight = 0.0;
        for (agent, weight) in weighted_agents {
            cumulative_weight += weight;
            if target <= cumulative_weight {
                return Ok(agent);
            }
        }

        Err(Error::NoAgentAvailable)
    }

    /// Select agent based on response time
    async fn select_response_time_based(
        &self,
        agents: Vec<RegisteredAgent>,
    ) -> Result<RegisteredAgent> {
        let mut best_agent = None;
        let mut min_response_time = f64::MAX;

        for agent in agents {
            let avg_response_time = *agent.average_response_time.read();
            if avg_response_time < min_response_time {
                min_response_time = avg_response_time;
                best_agent = Some(agent);
            }
        }

        best_agent.ok_or(Error::NoAgentAvailable)
    }

    /// Select agent based on capabilities
    async fn select_capability_based(
        &self,
        agents: Vec<RegisteredAgent>,
    ) -> Result<RegisteredAgent> {
        // For now, select the agent with the most capabilities
        // In a real implementation, this would match against required capabilities
        let mut best_agent = None;
        let mut max_capabilities = 0;

        for agent in agents {
            let capability_count = agent.capabilities.len();
            if capability_count > max_capabilities {
                max_capabilities = capability_count;
                best_agent = Some(agent);
            }
        }

        best_agent.ok_or(Error::NoAgentAvailable)
    }

    /// Select agent using adaptive strategy
    async fn select_adaptive(&self, agents: Vec<RegisteredAgent>) -> Result<RegisteredAgent> {
        let performance_history = self.performance_history.read();
        let mut best_agent = None;
        let mut best_score = f64::MIN;

        for agent in agents {
            let score = self.calculate_adaptive_score(&agent, &performance_history);
            if score > best_score {
                best_score = score;
                best_agent = Some(agent);
            }
        }

        best_agent.ok_or(Error::NoAgentAvailable)
    }

    /// Calculate adaptive score for an agent
    fn calculate_adaptive_score(
        &self,
        agent: &RegisteredAgent,
        performance_history: &HashMap<String, Vec<PerformanceMetric>>,
    ) -> f64 {
        let current_load = *agent.current_load.read() as f64;
        let avg_response_time = *agent.average_response_time.read();
        let max_capacity = agent.max_concurrent_tasks as f64;

        // Base score from capacity utilization (higher available capacity = better)
        let capacity_score = (max_capacity - current_load) / max_capacity;

        // Response time score (lower response time = better)
        let response_time_score = if avg_response_time > 0.0 {
            1.0 / (1.0 + avg_response_time)
        } else {
            1.0
        };

        // Historical performance score
        let performance_score = if let Some(metrics) = performance_history.get(&agent.id) {
            let recent_metrics: Vec<_> = metrics
                .iter()
                .filter(|m| m.timestamp > Utc::now() - chrono::Duration::minutes(10))
                .collect();

            if recent_metrics.is_empty() {
                0.5 // Neutral score for agents with no recent history
            } else {
                let success_rate = recent_metrics.iter().filter(|m| m.success).count() as f64
                    / recent_metrics.len() as f64;
                success_rate
            }
        } else {
            0.5 // Neutral score for agents with no history
        };

        // Weighted combination of scores
        let final_score =
            (capacity_score * 0.4) + (response_time_score * 0.3) + (performance_score * 0.3);

        debug!("Agent {} adaptive score: capacity={:.2}, response_time={:.2}, performance={:.2}, final={:.2}",
               agent.id, capacity_score, response_time_score, performance_score, final_score);

        final_score
    }

    /// Update performance metrics for an agent
    pub async fn update_performance_metrics(&self, agent_id: &str, metric: PerformanceMetric) {
        let mut performance_history = self.performance_history.write();
        let agent_metrics = performance_history.entry(agent_id.to_string()).or_default();

        agent_metrics.push(metric);

        // Keep only recent metrics (last 100 or last hour)
        let cutoff_time = Utc::now() - chrono::Duration::hours(1);
        agent_metrics.retain(|m| m.timestamp > cutoff_time);

        // Keep only the most recent 100 metrics
        if agent_metrics.len() > 100 {
            agent_metrics.drain(0..agent_metrics.len() - 100);
        }
    }

    /// Set weight for an agent (used in weighted round-robin)
    pub async fn set_agent_weight(&self, agent_id: &str, weight: f64) {
        let mut weights = self.weights.write();
        weights.insert(agent_id.to_string(), weight);
    }

    /// Get current weights for all agents
    pub async fn get_agent_weights(&self) -> HashMap<String, f64> {
        self.weights.read().clone()
    }

    /// Get performance history for an agent
    pub async fn get_agent_performance_history(&self, agent_id: &str) -> Vec<PerformanceMetric> {
        let performance_history = self.performance_history.read();
        performance_history
            .get(agent_id)
            .cloned()
            .unwrap_or_else(Vec::new)
    }

    /// Get performance statistics for an agent
    pub async fn get_agent_performance_stats(
        &self,
        agent_id: &str,
    ) -> Option<AgentPerformanceStats> {
        let performance_history = self.performance_history.read();
        let metrics = performance_history.get(agent_id)?;

        if metrics.is_empty() {
            return None;
        }

        let total_requests = metrics.len();
        let successful_requests = metrics.iter().filter(|m| m.success).count();
        let success_rate = successful_requests as f64 / total_requests as f64;

        let avg_response_time = metrics
            .iter()
            .map(|m| m.response_time.num_milliseconds() as f64)
            .sum::<f64>()
            / total_requests as f64;

        let avg_load = metrics.iter().map(|m| m.load as f64).sum::<f64>() / total_requests as f64;

        Some(AgentPerformanceStats {
            total_requests,
            successful_requests,
            success_rate,
            avg_response_time_ms: avg_response_time,
            avg_load,
        })
    }

    /// Update the load balancing strategy
    pub async fn update_strategy(&mut self, strategy: LoadBalancingStrategy) {
        info!("Load balancing strategy updated to {:?}", strategy);
        self.strategy = strategy;
    }

    /// Get current load balancing strategy
    pub fn get_strategy(&self) -> &LoadBalancingStrategy {
        &self.strategy
    }

    /// Acquire a permit for concurrent execution
    pub async fn acquire_permit(&'_ self) -> tokio::sync::SemaphorePermit<'_> {
        self.semaphore
            .acquire()
            .await
            .expect("Semaphore should not be closed")
    }

    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Clear performance history for an agent
    pub async fn clear_agent_performance_history(&self, agent_id: &str) {
        let mut performance_history = self.performance_history.write();
        performance_history.remove(agent_id);
    }

    /// Clear all performance history
    pub async fn clear_all_performance_history(&self) {
        let mut performance_history = self.performance_history.write();
        performance_history.clear();
    }
}

/// Performance statistics for an agent
#[derive(Debug, Clone)]
pub struct AgentPerformanceStats {
    /// Total number of requests processed
    pub total_requests: usize,
    /// Number of successful requests
    pub successful_requests: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Average load
    pub avg_load: f64,
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(LoadBalancingStrategy::Adaptive, 100)
    }
}
