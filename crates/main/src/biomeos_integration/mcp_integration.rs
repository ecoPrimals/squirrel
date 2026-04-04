// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # MCP Integration for biomeOS
//!
//! This module provides MCP (Machine Context Protocol) integration capabilities
//! for coordinating with the biomeOS ecosystem and other primals.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use super::{CoordinationStep, McpCoordinationRequest, McpCoordinationResponse};
use crate::error::PrimalError;

/// MCP integration for ecosystem coordination
#[derive(Debug, Clone)]
pub struct McpIntegration {
    /// The map of active coordination sessions keyed by session ID
    pub coordination_sessions: HashMap<String, CoordinationSession>,
    /// The list of MCP protocols currently active
    pub active_protocols: Vec<String>,
    /// The message routing configuration and state
    pub message_routing: MessageRouting,
    /// The tool orchestration capabilities
    pub tool_orchestration: ToolOrchestration,
    /// The resource coordination for distributed operations
    pub resource_coordination: ResourceCoordination,
    /// Whether the integration has been initialized
    pub initialized: bool,
}

/// Active coordination session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationSession {
    /// Unique identifier for the session
    pub session_id: String,
    /// List of participant identifiers in the session
    pub participants: Vec<String>,
    /// The type of coordination being performed
    pub coordination_type: String,
    /// Current status of the session
    pub status: String,
    /// When the session was started
    pub started_at: DateTime<Utc>,
    /// Timestamp of last activity in the session
    pub last_activity: DateTime<Utc>,
    /// Arbitrary coordination data shared between participants
    pub coordination_data: HashMap<String, serde_json::Value>,
}

/// Message routing for MCP protocol
#[derive(Debug, Clone)]
pub struct MessageRouting {
    /// The map of active routes keyed by route ID
    pub active_routes: HashMap<String, RouteConfig>,
    /// Queue of messages awaiting routing
    pub message_queue: Vec<PendingMessage>,
    /// Available routing strategy names
    pub routing_strategies: Vec<String>,
}

/// Tool orchestration capabilities
#[derive(Debug, Clone)]
pub struct ToolOrchestration {
    /// The map of available tools keyed by tool ID
    pub available_tools: HashMap<String, ToolDefinition>,
    /// The map of active orchestrations keyed by orchestration ID
    pub active_orchestrations: HashMap<String, ActiveOrchestration>,
    /// Supported orchestration pattern names
    pub orchestration_patterns: Vec<String>,
}

/// Resource coordination for distributed operations
#[derive(Debug, Clone)]
pub struct ResourceCoordination {
    /// The map of resource allocations keyed by allocation ID
    pub resource_allocations: HashMap<String, ResourceAllocation>,
    /// The list of coordination policies to apply
    pub coordination_policies: Vec<CoordinationPolicy>,
    /// The load balancing configuration
    pub load_balancing: LoadBalancingConfig,
}

/// Route configuration for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Unique identifier for the route
    pub route_id: String,
    /// Pattern matching message sources
    pub source_pattern: String,
    /// Pattern matching message destinations
    pub destination_pattern: String,
    /// The routing strategy to use for this route
    pub routing_strategy: String,
    /// Route priority for ordering (higher = more preferred)
    pub priority: u32,
}

/// Pending message in the routing queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMessage {
    /// Unique identifier for the message
    pub message_id: String,
    /// Source identifier for the message
    pub source: String,
    /// Destination identifier for the message
    pub destination: String,
    /// The type of message for routing decisions
    pub message_type: String,
    /// The message payload content
    pub payload: serde_json::Value,
    /// Message priority for queue ordering (higher = processed first)
    pub priority: u32,
    /// When the message was queued
    pub created_at: DateTime<Utc>,
}

/// Tool definition for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Unique identifier for the tool
    pub tool_id: String,
    /// Human-readable name of the tool
    pub tool_name: String,
    /// The category or type of the tool
    pub tool_type: String,
    /// List of capability identifiers the tool provides
    pub capabilities: Vec<String>,
    /// Resource requirements for running the tool
    pub resource_requirements: ToolResourceRequirements,
    /// Coordination patterns the tool supports
    pub coordination_patterns: Vec<String>,
}

/// Active tool orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveOrchestration {
    /// Unique identifier for the orchestration
    pub orchestration_id: String,
    /// List of tool IDs participating in the orchestration
    pub tools_involved: Vec<String>,
    /// The ordered steps of the orchestration plan
    pub orchestration_plan: Vec<OrchestrationStep>,
    /// Current status of the orchestration
    pub status: String,
    /// When the orchestration was started
    pub started_at: DateTime<Utc>,
    /// Estimated completion timestamp
    pub estimated_completion: DateTime<Utc>,
}

/// Resource allocation for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Unique identifier for the allocation
    pub allocation_id: String,
    /// The type of resource being allocated
    pub resource_type: String,
    /// The amount of resource allocated
    pub allocated_amount: f64,
    /// Identifier of the entity the resource is allocated to
    pub allocated_to: String,
    /// When the allocation was made
    pub allocation_time: DateTime<Utc>,
    /// When the allocation expires
    pub expires_at: DateTime<Utc>,
}

/// Coordination policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationPolicy {
    /// Unique identifier for the policy
    pub policy_id: String,
    /// The type of coordination policy
    pub policy_type: String,
    /// The rules that make up this policy
    pub rules: Vec<PolicyRule>,
    /// Policy priority for conflict resolution (higher = takes precedence)
    pub priority: u32,
    /// Whether the policy is currently active
    pub enabled: bool,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// The load balancing strategy name
    pub strategy: String,
    /// Weight mapping for weighted distribution (target -> weight)
    pub weights: HashMap<String, f64>,
    /// Interval between health checks
    pub health_check_interval: std::time::Duration,
    /// Rules for failover when targets become unhealthy
    pub failover_rules: Vec<FailoverRule>,
}

/// Tool resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResourceRequirements {
    /// CPU units required
    pub cpu: f64,
    /// Memory in bytes required
    pub memory: f64,
    /// Network bandwidth units required
    pub network: f64,
    /// Optional storage in bytes required
    pub storage: Option<f64>,
    /// Optional GPU units required
    pub gpu: Option<f64>,
}

/// Orchestration step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStep {
    /// Unique identifier for the step
    pub step_id: String,
    /// The tool ID to invoke for this step
    pub tool_id: String,
    /// The operation to perform
    pub operation: String,
    /// Input parameters for the operation
    pub inputs: HashMap<String, serde_json::Value>,
    /// Step IDs that must complete before this step
    pub dependencies: Vec<String>,
    /// Maximum time allowed for the step
    pub timeout: std::time::Duration,
}

/// Policy rule for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Unique identifier for the rule
    pub rule_id: String,
    /// Condition expression that triggers the rule
    pub condition: String,
    /// Action to take when the condition is met
    pub action: String,
    /// Parameters for the action
    pub parameters: HashMap<String, String>,
}

/// Failover rule for load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverRule {
    /// Unique identifier for the failover rule
    pub rule_id: String,
    /// Condition that triggers failover
    pub trigger_condition: String,
    /// Target to fail over to when triggered
    pub failover_target: String,
    /// Strategy for recovery after failover
    pub recovery_strategy: String,
}

impl McpIntegration {
    /// Create a new MCP integration instance with default configuration
    ///
    /// Initializes an MCP integration with:
    /// - Empty coordination sessions `HashMap`
    /// - Default active protocols (`mcp_2.0`, `ecosystem_coordination`, `cross_primal_messaging`)
    /// - Empty message queue
    /// - Default coordination plan and load balancing configuration
    ///
    /// # Returns
    ///
    /// A new `McpIntegration` instance ready for ecosystem coordination
    #[must_use]
    pub fn new() -> Self {
        Self {
            coordination_sessions: HashMap::new(),
            active_protocols: vec![
                "mcp_2.0".to_string(),
                "ecosystem_coordination".to_string(),
                "cross_primal_messaging".to_string(),
            ],
            message_routing: MessageRouting::new(),
            tool_orchestration: ToolOrchestration::new(),
            resource_coordination: ResourceCoordination::new(),
            initialized: false,
        }
    }

    /// Initialize MCP integration
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing MCP integration for ecosystem coordination");

        // Initialize message routing
        self.message_routing.initialize().await?;

        // Initialize tool orchestration
        self.tool_orchestration.initialize().await?;

        // Initialize resource coordination
        self.resource_coordination.initialize().await?;

        self.initialized = true;
        info!("MCP integration initialized successfully");
        Ok(())
    }

    /// Check if MCP integration is initialized
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Coordinate MCP services across the ecosystem
    pub async fn coordinate_mcp_services(&self) -> Result<(), PrimalError> {
        debug!("Coordinating MCP services across ecosystem");

        // Process pending messages
        self.message_routing.process_pending_messages().await?;

        // Update active orchestrations
        self.tool_orchestration.update_orchestrations().await?;

        // Manage resource allocations
        self.resource_coordination.manage_allocations().await?;

        Ok(())
    }

    /// Handle coordination request
    pub async fn handle_coordination_request(
        &self,
        request: McpCoordinationRequest,
    ) -> Result<McpCoordinationResponse, PrimalError> {
        info!(
            "Handling MCP coordination request: {}",
            request.coordination_id
        );

        let coordination_plan = self.create_coordination_plan(&request).await?;
        let estimated_completion = self.estimate_completion_time(&coordination_plan).await?;

        Ok(McpCoordinationResponse {
            coordination_id: request.coordination_id,
            status: "planned".to_string(),
            coordination_plan,
            estimated_completion,
        })
    }

    /// Create coordination session
    pub async fn create_coordination_session(
        &mut self,
        participants: Vec<String>,
        coordination_type: String,
    ) -> Result<String, PrimalError> {
        let session_id = format!("coord-session-{}", uuid::Uuid::new_v4());

        let session = CoordinationSession {
            session_id: session_id.clone(),
            participants,
            coordination_type,
            status: "active".to_string(),
            started_at: Utc::now(),
            last_activity: Utc::now(),
            coordination_data: HashMap::new(),
        };

        self.coordination_sessions
            .insert(session_id.clone(), session);
        info!("Created MCP coordination session: {}", session_id);

        Ok(session_id)
    }

    /// Get coordination session
    pub async fn get_coordination_session(
        &self,
        session_id: &str,
    ) -> Result<Option<CoordinationSession>, PrimalError> {
        Ok(self.coordination_sessions.get(session_id).cloned())
    }

    /// Update coordination session
    pub async fn update_coordination_session(
        &mut self,
        session_id: &str,
        update_data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        if let Some(session) = self.coordination_sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
            for (key, value) in update_data {
                session.coordination_data.insert(key, value);
            }
        }
        Ok(())
    }

    /// Shutdown MCP integration
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down MCP integration");

        // Cleanup active sessions
        self.coordination_sessions.clear();

        // Shutdown components
        self.message_routing.shutdown().await?;
        self.tool_orchestration.shutdown().await?;
        self.resource_coordination.shutdown().await?;

        self.initialized = false;
        info!("MCP integration shut down successfully");
        Ok(())
    }

    /// Coordinate with ecosystem
    pub async fn coordinate_with_ecosystem(&self) -> Result<(), PrimalError> {
        debug!("Coordinating with ecosystem");

        // Coordinate MCP services across the ecosystem
        self.coordinate_mcp_services().await?;

        // Update coordination sessions
        self.update_coordination_sessions().await?;

        // Handle message routing
        self.handle_message_routing().await?;

        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<(), PrimalError> {
        debug!("Performing MCP integration health check");

        if !self.is_initialized() {
            return Err(PrimalError::General(
                "MCP integration not initialized".to_string(),
            ));
        }

        // Check active coordination sessions
        if self.coordination_sessions.is_empty() {
            debug!("No active coordination sessions");
        }

        // Check message routing health
        if self.message_routing.message_queue.is_empty() {
            debug!("Message queue is empty");
        }

        Ok(())
    }

    // Private helper methods
    async fn create_coordination_plan(
        &self,
        request: &McpCoordinationRequest,
    ) -> Result<Vec<CoordinationStep>, PrimalError> {
        let mut plan = Vec::new();

        // Create coordination steps based on request type
        let participants = &request.participants;
        match request.coordination_type.as_str() {
            "resource_optimization" => {
                plan.push(CoordinationStep {
                    step_id: "analyze_resources".to_string(),
                    step_type: "analysis".to_string(),
                    participants: participants.clone(),
                    estimated_duration: std::time::Duration::from_secs(30),
                    dependencies: vec![],
                });
                plan.push(CoordinationStep {
                    step_id: "optimize_allocation".to_string(),
                    step_type: "optimization".to_string(),
                    participants: participants.clone(),
                    estimated_duration: std::time::Duration::from_secs(60),
                    dependencies: vec!["analyze_resources".to_string()],
                });
            }
            "load_balancing" => {
                plan.push(CoordinationStep {
                    step_id: "assess_load".to_string(),
                    step_type: "assessment".to_string(),
                    participants: participants.clone(),
                    estimated_duration: std::time::Duration::from_secs(15),
                    dependencies: vec![],
                });
                plan.push(CoordinationStep {
                    step_id: "rebalance_load".to_string(),
                    step_type: "rebalancing".to_string(),
                    participants: participants.clone(),
                    estimated_duration: std::time::Duration::from_secs(45),
                    dependencies: vec!["assess_load".to_string()],
                });
            }
            _ => {
                plan.push(CoordinationStep {
                    step_id: "generic_coordination".to_string(),
                    step_type: "coordination".to_string(),
                    participants: participants.clone(),
                    estimated_duration: std::time::Duration::from_secs(60),
                    dependencies: vec![],
                });
            }
        }

        Ok(plan)
    }

    async fn estimate_completion_time(
        &self,
        plan: &[CoordinationStep],
    ) -> Result<DateTime<Utc>, PrimalError> {
        let total_duration: std::time::Duration =
            plan.iter().map(|step| step.estimated_duration).sum();

        Ok(Utc::now()
            + chrono::Duration::from_std(total_duration).unwrap_or(chrono::Duration::seconds(300)))
    }

    /// Update coordination sessions
    async fn update_coordination_sessions(&self) -> Result<(), PrimalError> {
        debug!("Updating coordination sessions");
        // Implementation for updating coordination sessions
        Ok(())
    }

    /// Handle message routing
    async fn handle_message_routing(&self) -> Result<(), PrimalError> {
        debug!("Handling message routing");
        // Implementation for message routing
        Ok(())
    }
}

impl MessageRouting {
    fn new() -> Self {
        Self {
            active_routes: HashMap::new(),
            message_queue: Vec::new(),
            routing_strategies: vec![
                "priority_based".to_string(),
                "load_balanced".to_string(),
                "geographic".to_string(),
            ],
        }
    }

    async fn initialize(&mut self) -> Result<(), PrimalError> {
        debug!("Initializing message routing");
        Ok(())
    }

    async fn process_pending_messages(&self) -> Result<(), PrimalError> {
        debug!("Processing {} pending messages", self.message_queue.len());
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        debug!("Shutting down message routing");
        Ok(())
    }
}

impl ToolOrchestration {
    fn new() -> Self {
        Self {
            available_tools: HashMap::new(),
            active_orchestrations: HashMap::new(),
            orchestration_patterns: vec![
                "sequential".to_string(),
                "parallel".to_string(),
                "conditional".to_string(),
            ],
        }
    }

    async fn initialize(&mut self) -> Result<(), PrimalError> {
        debug!("Initializing tool orchestration");
        Ok(())
    }

    async fn update_orchestrations(&self) -> Result<(), PrimalError> {
        debug!(
            "Updating {} active orchestrations",
            self.active_orchestrations.len()
        );
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        debug!("Shutting down tool orchestration");
        Ok(())
    }
}

impl ResourceCoordination {
    fn new() -> Self {
        Self {
            resource_allocations: HashMap::new(),
            coordination_policies: Vec::new(),
            load_balancing: LoadBalancingConfig {
                strategy: "round_robin".to_string(),
                weights: HashMap::new(),
                health_check_interval: std::time::Duration::from_secs(
                    std::env::var("HEALTH_CHECK_INTERVAL_SECS")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(30),
                ),
                failover_rules: Vec::new(),
            },
        }
    }

    async fn initialize(&mut self) -> Result<(), PrimalError> {
        debug!("Initializing resource coordination");
        Ok(())
    }

    async fn manage_allocations(&self) -> Result<(), PrimalError> {
        debug!(
            "Managing {} resource allocations",
            self.resource_allocations.len()
        );
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        debug!("Shutting down resource coordination");
        Ok(())
    }
}

impl Default for McpIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_integration_creation() {
        let integration = McpIntegration::new();
        assert!(!integration.active_protocols.is_empty());
        assert!(
            integration
                .active_protocols
                .contains(&"mcp_2.0".to_string())
        );
    }

    #[tokio::test]
    async fn test_coordination_session_creation() {
        let mut integration = McpIntegration::new();
        let participants = vec!["ai-coordinator".to_string(), "discovery-mesh".to_string()];

        let session_id = integration
            .create_coordination_session(participants.clone(), "test_coordination".to_string())
            .await
            .expect("should succeed");
        assert!(!session_id.is_empty());

        let session = integration
            .get_coordination_session(&session_id)
            .await
            .expect("should succeed");
        assert!(session.is_some());
        assert_eq!(session.expect("should succeed").participants, participants);
    }

    #[tokio::test]
    async fn test_coordination_request_handling() {
        let integration = McpIntegration::new();
        let request = McpCoordinationRequest {
            coordination_id: "test-coord-001".to_string(),
            coordination_type: "resource_optimization".to_string(),
            participants: vec!["ai-coordinator".to_string(), "compute-provider".to_string()],
            coordination_data: HashMap::new(),
        };

        let response = integration
            .handle_coordination_request(request)
            .await
            .expect("should succeed");
        assert_eq!(response.coordination_id, "test-coord-001");
        assert_eq!(response.status, "planned");
        assert!(!response.coordination_plan.is_empty());
    }
}
