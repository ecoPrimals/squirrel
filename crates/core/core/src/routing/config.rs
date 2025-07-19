//! Routing configuration for MCP task routing
//!
//! This module contains all configuration structures and enums for the MCP routing service,
//! including load balancing strategies, agent selection, and manual routing rules.

use crate::{PrimalType, TaskPriority};
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the MCP routing service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Task timeout duration
    pub task_timeout: Duration,
    /// Load balancing strategy to use
    pub load_balancing_strategy: LoadBalancingStrategy,
    /// Enable context persistence
    pub context_persistence: bool,
    /// Enable federation across nodes
    pub federation_enabled: bool,
    /// Enable automatic scaling
    pub scaling_enabled: bool,
    /// Enable performance monitoring
    pub performance_monitoring: bool,

    // Manual selection and override configurations
    /// Agent selection configuration
    pub agent_selection_config: AgentSelectionConfig,
    /// Agent groups configuration
    pub agent_groups: HashMap<String, AgentGroup>,
    /// Priority overrides for specific tasks
    pub priority_overrides: HashMap<String, TaskPriority>,
    /// Manual routing rules
    pub manual_routing_rules: Vec<ManualRoutingRule>,

    // Primal endpoints (fixed mappings)
    /// NestGate endpoint for storage persistence
    pub nestgate_endpoint: Option<String>,
    /// Toadstool endpoint for compute
    pub toadstool_endpoint: Option<String>,
    /// BearDog endpoint for security
    pub beardog_endpoint: Option<String>,
    /// BiomeOS endpoint for integration
    pub biomeos_endpoint: Option<String>,
}

/// Load balancing strategies available for agent selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Simple round-robin selection
    RoundRobin,
    /// Select agent with least connections
    LeastConnections,
    /// Weighted round-robin based on capacity
    WeightedRoundRobin,
    /// Select based on response time
    ResponseTimeBased,
    /// Select based on agent capabilities
    CapabilityBased,
    /// Adaptive selection based on performance
    Adaptive,
}

/// Configuration for agent selection behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSelectionConfig {
    /// Allow manual override of automatic selection
    pub allow_manual_override: bool,
    /// Default selection mode
    pub default_selection_mode: SelectionMode,
    /// Enable fallback to other agents
    pub fallback_enabled: bool,
    /// Enable sticky sessions
    pub sticky_sessions: bool,
    /// Prefer local agents over remote ones
    pub prefer_local_agents: bool,
}

/// Agent selection modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMode {
    /// Fully automatic selection
    Automatic,
    /// Manual selection only
    Manual,
    /// Hybrid approach with preferences
    Hybrid,
}

/// Configuration for agent groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGroup {
    /// Group name
    pub name: String,
    /// List of agent IDs in this group
    pub agents: Vec<String>,
    /// Priority of this group
    pub priority: u32,
    /// Selection strategy for this group
    pub selection_strategy: LoadBalancingStrategy,
    /// Maximum concurrent tasks for this group
    pub max_concurrent_tasks: Option<usize>,
    /// Failover groups if this group fails
    pub failover_groups: Vec<String>,
    /// Required capabilities for this group
    pub required_capabilities: Vec<String>,
    /// Additional tags for this group
    pub tags: HashMap<String, String>,
}

/// Manual routing rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualRoutingRule {
    /// Unique rule identifier
    pub rule_id: String,
    /// Condition that must be met
    pub condition: RoutingCondition,
    /// Action to take when condition is met
    pub action: RoutingAction,
    /// Priority of this rule (higher = more important)
    pub priority: u32,
    /// Whether this rule is enabled
    pub enabled: bool,
}

/// Conditions for routing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingCondition {
    /// Match by task type
    TaskType(String),
    /// Match by required capability
    RequiredCapability(String),
    /// Match by primal type
    PrimalType(PrimalType),
    /// Match by user request
    UserRequest(String),
    /// Match by task tag
    TaskTag(String, String),
    /// Logical AND of conditions
    And(Vec<RoutingCondition>),
    /// Logical OR of conditions
    Or(Vec<RoutingCondition>),
}

/// Actions to take when routing conditions are met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingAction {
    /// Use specific agent
    UseAgent(String),
    /// Use specific agent group
    UseAgentGroup(String),
    /// Use specific primal type
    UsePrimal(PrimalType),
    /// Set task priority
    SetPriority(TaskPriority),
    /// Reject the task with reason
    Reject(String),
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            task_timeout: Duration::seconds(30),
            load_balancing_strategy: LoadBalancingStrategy::Adaptive,
            context_persistence: true,
            federation_enabled: false,
            scaling_enabled: true,
            performance_monitoring: true,
            agent_selection_config: AgentSelectionConfig::default(),
            agent_groups: HashMap::new(),
            priority_overrides: HashMap::new(),
            manual_routing_rules: Vec::new(),
            nestgate_endpoint: None,
            toadstool_endpoint: None,
            beardog_endpoint: None,
            biomeos_endpoint: None,
        }
    }
}

impl Default for AgentSelectionConfig {
    fn default() -> Self {
        Self {
            allow_manual_override: true,
            default_selection_mode: SelectionMode::Hybrid,
            fallback_enabled: true,
            sticky_sessions: false,
            prefer_local_agents: true,
        }
    }
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::Adaptive
    }
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Hybrid
    }
}

impl RoutingConfig {
    /// Create a new routing configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum concurrent tasks
    pub fn with_max_concurrent_tasks(mut self, max_tasks: usize) -> Self {
        self.max_concurrent_tasks = max_tasks;
        self
    }

    /// Set task timeout
    pub fn with_task_timeout(mut self, timeout: Duration) -> Self {
        self.task_timeout = timeout;
        self
    }

    /// Set load balancing strategy
    pub fn with_load_balancing_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.load_balancing_strategy = strategy;
        self
    }

    /// Enable context persistence
    pub fn with_context_persistence(mut self, enabled: bool) -> Self {
        self.context_persistence = enabled;
        self
    }

    /// Enable federation
    pub fn with_federation(mut self, enabled: bool) -> Self {
        self.federation_enabled = enabled;
        self
    }

    /// Enable scaling
    pub fn with_scaling(mut self, enabled: bool) -> Self {
        self.scaling_enabled = enabled;
        self
    }

    /// Enable performance monitoring
    pub fn with_performance_monitoring(mut self, enabled: bool) -> Self {
        self.performance_monitoring = enabled;
        self
    }

    /// Add an agent group
    pub fn with_agent_group(mut self, name: String, group: AgentGroup) -> Self {
        self.agent_groups.insert(name, group);
        self
    }

    /// Add a manual routing rule
    pub fn with_routing_rule(mut self, rule: ManualRoutingRule) -> Self {
        self.manual_routing_rules.push(rule);
        self
    }

    /// Set primal endpoints
    pub fn with_primal_endpoints(
        mut self,
        nestgate: Option<String>,
        toadstool: Option<String>,
        beardog: Option<String>,
        biomeos: Option<String>,
    ) -> Self {
        self.nestgate_endpoint = nestgate;
        self.toadstool_endpoint = toadstool;
        self.beardog_endpoint = beardog;
        self.biomeos_endpoint = biomeos;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_concurrent_tasks == 0 {
            return Err("max_concurrent_tasks must be greater than 0".to_string());
        }

        if self.task_timeout.num_seconds() <= 0 {
            return Err("task_timeout must be positive".to_string());
        }

        // Validate agent groups
        for (name, group) in &self.agent_groups {
            if group.agents.is_empty() {
                return Err(format!("Agent group '{name}' has no agents"));
            }
        }

        // Validate manual routing rules
        for rule in &self.manual_routing_rules {
            if rule.rule_id.is_empty() {
                return Err("Manual routing rule must have a non-empty rule_id".to_string());
            }
        }

        Ok(())
    }
}

impl AgentGroup {
    /// Create a new agent group
    pub fn new(name: String) -> Self {
        Self {
            name,
            agents: Vec::new(),
            priority: 0,
            selection_strategy: LoadBalancingStrategy::default(),
            max_concurrent_tasks: None,
            failover_groups: Vec::new(),
            required_capabilities: Vec::new(),
            tags: HashMap::new(),
        }
    }

    /// Add an agent to the group
    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agents.push(agent_id);
        self
    }

    /// Set group priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set selection strategy
    pub fn with_selection_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.selection_strategy = strategy;
        self
    }

    /// Set maximum concurrent tasks
    pub fn with_max_concurrent_tasks(mut self, max_tasks: usize) -> Self {
        self.max_concurrent_tasks = Some(max_tasks);
        self
    }

    /// Add a required capability
    pub fn with_required_capability(mut self, capability: String) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }
}

impl ManualRoutingRule {
    /// Create a new manual routing rule
    pub fn new(rule_id: String, condition: RoutingCondition, action: RoutingAction) -> Self {
        Self {
            rule_id,
            condition,
            action,
            priority: 0,
            enabled: true,
        }
    }

    /// Set rule priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Enable or disable the rule
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
