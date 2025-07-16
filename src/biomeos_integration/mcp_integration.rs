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
    pub coordination_sessions: HashMap<String, CoordinationSession>,
    pub active_protocols: Vec<String>,
    pub message_routing: MessageRouting,
    pub tool_orchestration: ToolOrchestration,
    pub resource_coordination: ResourceCoordination,
    pub initialized: bool,
}

/// Active coordination session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationSession {
    pub session_id: String,
    pub participants: Vec<String>,
    pub coordination_type: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub coordination_data: HashMap<String, serde_json::Value>,
}

/// Message routing for MCP protocol
#[derive(Debug, Clone)]
pub struct MessageRouting {
    pub active_routes: HashMap<String, RouteConfig>,
    pub message_queue: Vec<PendingMessage>,
    pub routing_strategies: Vec<String>,
}

/// Tool orchestration capabilities
#[derive(Debug, Clone)]
pub struct ToolOrchestration {
    pub available_tools: HashMap<String, ToolDefinition>,
    pub active_orchestrations: HashMap<String, ActiveOrchestration>,
    pub orchestration_patterns: Vec<String>,
}

/// Resource coordination for distributed operations
#[derive(Debug, Clone)]
pub struct ResourceCoordination {
    pub resource_allocations: HashMap<String, ResourceAllocation>,
    pub coordination_policies: Vec<CoordinationPolicy>,
    pub load_balancing: LoadBalancingConfig,
}

/// Route configuration for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub route_id: String,
    pub source_pattern: String,
    pub destination_pattern: String,
    pub routing_strategy: String,
    pub priority: u32,
}

/// Pending message in the routing queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMessage {
    pub message_id: String,
    pub source: String,
    pub destination: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub priority: u32,
    pub created_at: DateTime<Utc>,
}

/// Tool definition for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub tool_id: String,
    pub tool_name: String,
    pub tool_type: String,
    pub capabilities: Vec<String>,
    pub resource_requirements: ToolResourceRequirements,
    pub coordination_patterns: Vec<String>,
}

/// Active tool orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveOrchestration {
    pub orchestration_id: String,
    pub tools_involved: Vec<String>,
    pub orchestration_plan: Vec<OrchestrationStep>,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: DateTime<Utc>,
}

/// Resource allocation for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub resource_type: String,
    pub allocated_amount: f64,
    pub allocated_to: String,
    pub allocation_time: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Coordination policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationPolicy {
    pub policy_id: String,
    pub policy_type: String,
    pub rules: Vec<PolicyRule>,
    pub priority: u32,
    pub enabled: bool,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    pub strategy: String,
    pub weights: HashMap<String, f64>,
    pub health_check_interval: std::time::Duration,
    pub failover_rules: Vec<FailoverRule>,
}

/// Tool resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResourceRequirements {
    pub cpu: f64,
    pub memory: f64,
    pub network: f64,
    pub storage: Option<f64>,
    pub gpu: Option<f64>,
}

/// Orchestration step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStep {
    pub step_id: String,
    pub tool_id: String,
    pub operation: String,
    pub inputs: HashMap<String, serde_json::Value>,
    pub dependencies: Vec<String>,
    pub timeout: std::time::Duration,
}

/// Policy rule for coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub rule_id: String,
    pub condition: String,
    pub action: String,
    pub parameters: HashMap<String, String>,
}

/// Failover rule for load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverRule {
    pub rule_id: String,
    pub trigger_condition: String,
    pub failover_target: String,
    pub recovery_strategy: String,
}

impl McpIntegration {
    /// Create a new MCP integration instance with default configuration
    ///
    /// Initializes an MCP integration with:
    /// - Empty coordination sessions HashMap
    /// - Default active protocols (mcp_2.0, ecosystem_coordination, cross_primal_messaging)
    /// - Empty message queue
    /// - Default coordination plan and load balancing configuration
    ///
    /// # Returns
    ///
    /// A new McpIntegration instance ready for ecosystem coordination
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
    pub fn is_initialized(&self) -> bool {
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
        assert!(integration
            .active_protocols
            .contains(&"mcp_2.0".to_string()));
    }

    #[tokio::test]
    async fn test_coordination_session_creation() {
        let mut integration = McpIntegration::new();
        let participants = vec!["squirrel".to_string(), "songbird".to_string()];

        let session_id = integration
            .create_coordination_session(participants.clone(), "test_coordination".to_string())
            .await
            .unwrap();
        assert!(!session_id.is_empty());

        let session = integration
            .get_coordination_session(&session_id)
            .await
            .unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().participants, participants);
    }

    #[tokio::test]
    async fn test_coordination_request_handling() {
        let integration = McpIntegration::new();
        let request = McpCoordinationRequest {
            coordination_id: "test-coord-001".to_string(),
            coordination_type: "resource_optimization".to_string(),
            participants: vec!["squirrel".to_string(), "toadstool".to_string()],
            coordination_data: HashMap::new(),
        };

        let response = integration
            .handle_coordination_request(request)
            .await
            .unwrap();
        assert_eq!(response.coordination_id, "test-coord-001");
        assert_eq!(response.status, "planned");
        assert!(!response.coordination_plan.is_empty());
    }
}
