// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
    #[must_use]
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
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if no agent is available or selection fails.
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
        let current_load = f64::from(*agent.current_load.read());
        let avg_response_time = *agent.average_response_time.read();
        let max_capacity = f64::from(agent.max_concurrent_tasks);

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
                recent_metrics.iter().filter(|m| m.success).count() as f64
                    / recent_metrics.len() as f64
            }
        } else {
            0.5 // Neutral score for agents with no history
        };

        // Weighted combination of scores
        let final_score =
            capacity_score.mul_add(0.4, response_time_score * 0.3) + (performance_score * 0.3);

        debug!(
            "Agent {} adaptive score: capacity={:.2}, response_time={:.2}, performance={:.2}, final={:.2}",
            agent.id, capacity_score, response_time_score, performance_score, final_score
        );

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

        let avg_load =
            metrics.iter().map(|m| f64::from(m.load)).sum::<f64>() / total_requests as f64;

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
    pub const fn get_strategy(&self) -> &LoadBalancingStrategy {
        &self.strategy
    }

    /// Acquire a permit for concurrent execution
    #[expect(
        clippy::expect_used,
        reason = "Semaphore closed is unreachable in normal operation"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::agent::RegisteredAgent;
    use crate::routing::config::LoadBalancingStrategy;
    use crate::{AgentSpec, Error};
    use chrono::Utc;
    use std::collections::HashMap;

    fn agent_with_spec(id: &str, caps: Vec<String>) -> RegisteredAgent {
        RegisteredAgent::new(AgentSpec {
            id: id.to_string(),
            endpoint: "http://localhost".to_string(),
            capabilities: caps,
            weight: None,
            max_concurrent_tasks: 10,
            metadata: HashMap::new(),
        })
    }

    #[test]
    fn new_sets_strategy_and_semaphore_capacity() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, 7);
        assert!(matches!(
            lb.get_strategy(),
            LoadBalancingStrategy::RoundRobin
        ));
        assert_eq!(lb.available_permits(), 7);
    }

    #[test]
    fn default_uses_adaptive_and_hundred_permits() {
        let lb = LoadBalancer::default();
        assert!(matches!(lb.get_strategy(), LoadBalancingStrategy::Adaptive));
        assert_eq!(lb.available_permits(), 100);
    }

    #[tokio::test]
    async fn select_agent_empty_returns_no_agent_available() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, 1);
        let err = lb.select_agent(vec![]).await.expect_err("empty");
        assert!(matches!(err, Error::NoAgentAvailable));
    }

    #[tokio::test]
    async fn round_robin_rotates_single_backend_repeatedly() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, 4);
        let a = agent_with_spec("only", vec![]);
        for _ in 0..3 {
            let picked = lb.select_agent(vec![a.clone()]).await.expect("pick");
            assert_eq!(picked.id, "only");
        }
    }

    #[tokio::test]
    async fn round_robin_cycles_two_agents() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, 4);
        let a = agent_with_spec("a", vec![]);
        let b = agent_with_spec("b", vec![]);
        let agents = vec![a, b];
        let id1 = lb.select_agent(agents.clone()).await.expect("p1").id;
        let id2 = lb.select_agent(agents.clone()).await.expect("p2").id;
        let id3 = lb.select_agent(agents.clone()).await.expect("p3").id;
        assert_ne!(id1, id2);
        assert_eq!(id1, id3);
    }

    #[tokio::test]
    async fn least_connections_prefers_lower_load() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::LeastConnections, 2);
        let low = agent_with_spec("low", vec![]);
        let high = agent_with_spec("high", vec![]);
        *high.current_load.write() = 9;
        *low.current_load.write() = 1;

        let picked = lb
            .select_agent(vec![high, low.clone()])
            .await
            .expect("pick");
        assert_eq!(picked.id, "low");
    }

    #[tokio::test]
    async fn weighted_round_robin_errors_when_all_weights_zero() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::WeightedRoundRobin, 2);
        lb.set_agent_weight("x", 0.0).await;
        lb.set_agent_weight("y", 0.0).await;
        let a = agent_with_spec("x", vec![]);
        let b = agent_with_spec("y", vec![]);
        let err = lb.select_agent(vec![a, b]).await.expect_err("weights");
        assert!(matches!(err, Error::NoAgentAvailable));
    }

    #[tokio::test]
    async fn weighted_round_robin_selects_with_default_weight_one() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::WeightedRoundRobin, 2);
        let a = agent_with_spec("w1", vec![]);
        let b = agent_with_spec("w2", vec![]);
        let picked = lb.select_agent(vec![a, b]).await.expect("weighted pick");
        assert!(picked.id == "w1" || picked.id == "w2");
    }

    #[tokio::test]
    async fn response_time_based_prefers_faster_agent() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::ResponseTimeBased, 2);
        let fast = agent_with_spec("fast", vec![]);
        let slow = agent_with_spec("slow", vec![]);
        *fast.average_response_time.write() = 10.0;
        *slow.average_response_time.write() = 500.0;

        let picked = lb
            .select_agent(vec![slow, fast.clone()])
            .await
            .expect("pick");
        assert_eq!(picked.id, "fast");
    }

    #[tokio::test]
    async fn capability_based_prefers_more_capabilities() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::CapabilityBased, 2);
        let small = agent_with_spec("small", vec!["a".to_string()]);
        let big = agent_with_spec(
            "big",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );

        let picked = lb.select_agent(vec![small, big]).await.expect("pick");
        assert_eq!(picked.id, "big");
    }

    #[tokio::test]
    async fn adaptive_prefers_lower_load_when_other_equal() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::Adaptive, 2);
        let idle = agent_with_spec("idle", vec![]);
        let busy = agent_with_spec("busy", vec![]);
        *idle.current_load.write() = 0;
        *busy.current_load.write() = 8;

        let picked = lb
            .select_agent(vec![busy, idle.clone()])
            .await
            .expect("pick");
        assert_eq!(picked.id, "idle");
    }

    #[tokio::test]
    async fn set_and_get_agent_weight_round_trip() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::WeightedRoundRobin, 1);
        lb.set_agent_weight("agent-a", 2.5).await;
        let weights = lb.get_agent_weights().await;
        let w = weights.get("agent-a").copied().expect("weight");
        assert!((w - 2.5_f64).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn update_performance_metrics_and_history() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::Adaptive, 2);
        let m = PerformanceMetric {
            timestamp: Utc::now(),
            response_time: chrono::Duration::milliseconds(12),
            success: true,
            load: 3,
        };
        lb.update_performance_metrics("agent-1", m).await;
        let hist = lb.get_agent_performance_history("agent-1").await;
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].load, 3);
    }

    #[tokio::test]
    async fn get_agent_performance_stats_computes_averages() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::Adaptive, 2);
        for (ms, ok) in [(100_i64, true), (200, true), (300, false)] {
            lb.update_performance_metrics(
                "stats-agent",
                PerformanceMetric {
                    timestamp: Utc::now(),
                    response_time: chrono::Duration::milliseconds(ms),
                    success: ok,
                    load: 2,
                },
            )
            .await;
        }
        let stats = lb
            .get_agent_performance_stats("stats-agent")
            .await
            .expect("stats");
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 2);
        let expected_rate = 2.0_f64 / 3.0_f64;
        assert!((stats.success_rate - expected_rate).abs() < 1e-12);
        let expected_avg_ms = 200.0_f64;
        assert!((stats.avg_response_time_ms - expected_avg_ms).abs() < f64::EPSILON);
        let expected_avg_load = 2.0_f64;
        assert!((stats.avg_load - expected_avg_load).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn clear_agent_performance_history_removes_series() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::Adaptive, 1);
        lb.update_performance_metrics(
            "gone",
            PerformanceMetric {
                timestamp: Utc::now(),
                response_time: chrono::Duration::milliseconds(1),
                success: true,
                load: 0,
            },
        )
        .await;
        lb.clear_agent_performance_history("gone").await;
        assert!(lb.get_agent_performance_history("gone").await.is_empty());
    }

    #[tokio::test]
    async fn clear_all_performance_history_empties_store() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::Adaptive, 1);
        lb.update_performance_metrics(
            "a",
            PerformanceMetric {
                timestamp: Utc::now(),
                response_time: chrono::Duration::milliseconds(1),
                success: true,
                load: 0,
            },
        )
        .await;
        lb.clear_all_performance_history().await;
        assert!(lb.get_agent_performance_history("a").await.is_empty());
    }

    #[tokio::test]
    async fn update_strategy_changes_reported_strategy() {
        let mut lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, 1);
        lb.update_strategy(LoadBalancingStrategy::LeastConnections)
            .await;
        assert!(matches!(
            lb.get_strategy(),
            LoadBalancingStrategy::LeastConnections
        ));
    }

    #[tokio::test]
    async fn acquire_permit_reduces_available_until_drop() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin, 2);
        assert_eq!(lb.available_permits(), 2);
        let permit = lb.acquire_permit().await;
        assert_eq!(lb.available_permits(), 1);
        drop(permit);
        assert_eq!(lb.available_permits(), 2);
    }
}
