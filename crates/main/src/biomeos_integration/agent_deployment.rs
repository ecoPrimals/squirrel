//! # Agent Deployment for biomeOS Integration
//!
//! This module provides agent deployment capabilities that can deploy and manage
//! AI agents based on biome.yaml manifest specifications. It integrates with the
//! existing MCP protocol and provides lifecycle management for agents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::ai_intelligence::AiIntelligence;
use super::manifest::{AgentResourceLimits, AgentSpec, BiomeManifest, ExecutionEnvironment};
use super::mcp_integration::McpIntegration;
use crate::error::PrimalError;

/// Agent deployment manager for biomeOS integration
#[derive(Debug)]
pub struct AgentDeploymentManager {
    /// Deployed agents
    pub deployed_agents: Arc<RwLock<HashMap<String, DeployedAgent>>>,

    /// Agent deployment configuration
    pub config: AgentDeploymentConfig,

    /// MCP integration for agent coordination
    pub mcp_integration: Arc<McpIntegration>,

    /// AI intelligence for agent management
    pub ai_intelligence: Arc<AiIntelligence>,

    /// Deployment status
    pub deployment_status: Arc<RwLock<DeploymentStatus>>,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl AgentDeploymentManager {
    /// Create a new agent deployment manager
    pub fn new(
        config: AgentDeploymentConfig,
        mcp_integration: Arc<McpIntegration>,
        ai_intelligence: Arc<AiIntelligence>,
    ) -> Self {
        Self {
            deployed_agents: Arc::new(RwLock::new(HashMap::new())),
            config,
            mcp_integration,
            ai_intelligence,
            deployment_status: Arc::new(RwLock::new(DeploymentStatus::new())),
        }
    }

    /// Deploy agents from a biome.yaml manifest
    pub async fn deploy_from_manifest(
        &self,
        manifest: &BiomeManifest,
    ) -> Result<Vec<String>, PrimalError> {
        info!(
            "Deploying agents from biome.yaml manifest: {}",
            manifest.metadata.name
        );

        let mut deployed_agent_ids = Vec::new();

        // Validate manifest
        self.validate_manifest(manifest).await?;

        // Deploy each agent
        for agent_spec in &manifest.agents {
            match self.deploy_agent(agent_spec).await {
                Ok(agent_id) => {
                    deployed_agent_ids.push(agent_id);
                    info!("Successfully deployed agent: {}", agent_spec.name);
                }
                Err(e) => {
                    error!("Failed to deploy agent {}: {}", agent_spec.name, e);
                    return Err(e);
                }
            }
        }

        // Update deployment status
        self.update_deployment_status().await?;

        info!(
            "Successfully deployed {} agents from manifest",
            deployed_agent_ids.len()
        );
        Ok(deployed_agent_ids)
    }

    /// Deploy a single agent
    pub async fn deploy_agent(&self, spec: &AgentSpec) -> Result<String, PrimalError> {
        debug!("Deploying agent: {}", spec.name);

        // Generate unique agent ID
        let agent_id = format!("agent-{}-{}", spec.name, uuid::Uuid::new_v4());

        // Validate agent specification
        self.validate_agent_spec(spec).await?;

        // Create deployed agent
        let deployed_agent = self.create_deployed_agent(&agent_id, spec).await?;

        // Register agent with MCP integration
        self.register_agent_with_mcp(&deployed_agent).await?;

        // Start agent
        self.start_agent(&deployed_agent).await?;

        // Store deployed agent
        self.deployed_agents
            .write()
            .await
            .insert(agent_id.clone(), deployed_agent);

        debug!(
            "Agent {} deployed successfully with ID: {}",
            spec.name, agent_id
        );
        Ok(agent_id)
    }

    /// Validate the manifest
    async fn validate_manifest(&self, manifest: &BiomeManifest) -> Result<(), PrimalError> {
        // Check if we can deploy all agents
        if manifest.agents.len() > self.config.max_concurrent_agents as usize {
            return Err(PrimalError::ResourceError(format!(
                "Cannot deploy {} agents, maximum is {}",
                manifest.agents.len(),
                self.config.max_concurrent_agents
            )));
        }

        // Validate security requirements
        if self.config.security.enabled {
            for agent in &manifest.agents {
                self.validate_agent_security(agent).await?;
            }
        }

        Ok(())
    }

    /// Validate agent specification
    async fn validate_agent_spec(&self, spec: &AgentSpec) -> Result<(), PrimalError> {
        // Validate AI provider
        if self.config.security.enabled
            && !self.config.security.allowed_ai_providers.is_empty()
            && !self
                .config
                .security
                .allowed_ai_providers
                .contains(&spec.ai_provider)
        {
            return Err(PrimalError::SecurityError(format!(
                "AI provider '{}' not allowed",
                spec.ai_provider
            )));
        }

        // Validate execution environment
        if self.config.security.enabled
            && !self
                .config
                .security
                .allowed_execution_environments
                .is_empty()
            && !self
                .config
                .security
                .allowed_execution_environments
                .contains(&spec.execution_environment)
        {
            return Err(PrimalError::SecurityError(format!(
                "Execution environment '{:?}' not allowed",
                spec.execution_environment
            )));
        }

        // Validate resource limits
        if spec.resource_limits.memory_mb > self.config.default_resource_limits.memory_mb {
            return Err(PrimalError::ResourceError(format!(
                "Memory limit {} MB exceeds maximum {} MB",
                spec.resource_limits.memory_mb, self.config.default_resource_limits.memory_mb
            )));
        }

        Ok(())
    }

    /// Validate agent security
    async fn validate_agent_security(&self, spec: &AgentSpec) -> Result<(), PrimalError> {
        if self.config.security.validate_security_context {
            if spec.security.security_context.is_empty() {
                return Err(PrimalError::SecurityError(
                    "Security context must be specified".to_string(),
                ));
            }
        }

        if self.config.security.require_encryption {
            if !spec.security.encryption.at_rest || !spec.security.encryption.in_transit {
                return Err(PrimalError::SecurityError(
                    "Encryption required for both at-rest and in-transit data".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Create a deployed agent instance
    async fn create_deployed_agent(
        &self,
        agent_id: &str,
        spec: &AgentSpec,
    ) -> Result<DeployedAgent, PrimalError> {
        let endpoints = self.generate_agent_endpoints(agent_id, spec).await?;

        Ok(DeployedAgent {
            agent_id: agent_id.to_string(),
            name: spec.name.clone(),
            spec: spec.clone(),
            status: AgentStatus::Deploying,
            deployed_at: Utc::now(),
            last_health_check: Utc::now(),
            resource_usage: AgentResourceUsage::default(),
            endpoints,
            metadata: HashMap::new(),
        })
    }

    /// Generate agent endpoints
    async fn generate_agent_endpoints(
        &self,
        agent_id: &str,
        spec: &AgentSpec,
    ) -> Result<AgentEndpoints, PrimalError> {
        let base_port = 8080; // This should be dynamically allocated
        let base_url = format!("http://localhost:{}", base_port);

        Ok(AgentEndpoints {
            api: format!("{}/api/v1/agents/{}", base_url, agent_id),
            health: format!("{}/health", base_url),
            metrics: format!("{}/metrics", base_url),
            websocket: Some(format!("ws://localhost:{}/ws", base_port)),
        })
    }

    /// Register agent with MCP integration
    async fn register_agent_with_mcp(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Registering agent {} with MCP integration", agent.agent_id);

        // Register agent capabilities with MCP
        // This would integrate with the existing MCP protocol

        Ok(())
    }

    /// Start the agent
    async fn start_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Starting agent: {}", agent.agent_id);

        // Start agent based on execution environment
        match agent.spec.execution_environment {
            ExecutionEnvironment::Native => {
                self.start_native_agent(agent).await?;
            }
            ExecutionEnvironment::Wasm => {
                self.start_wasm_agent(agent).await?;
            }
            ExecutionEnvironment::Container => {
                self.start_container_agent(agent).await?;
            }
            ExecutionEnvironment::VirtualMachine => {
                self.start_vm_agent(agent).await?;
            }
        }

        Ok(())
    }

    /// Start native agent
    async fn start_native_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Starting native agent: {}", agent.agent_id);
        // Implementation for native agent startup
        Ok(())
    }

    /// Start WASM agent
    async fn start_wasm_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Starting WASM agent: {}", agent.agent_id);
        // Implementation for WASM agent startup
        Ok(())
    }

    /// Start container agent
    async fn start_container_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Starting container agent: {}", agent.agent_id);
        // Implementation for container agent startup
        Ok(())
    }

    /// Start VM agent
    async fn start_vm_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Starting VM agent: {}", agent.agent_id);
        // Implementation for VM agent startup
        Ok(())
    }

    /// Stop an agent
    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), PrimalError> {
        info!("Stopping agent: {}", agent_id);

        let mut agents = self.deployed_agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.status = AgentStatus::Stopping;

            // Stop agent based on execution environment
            match agent.spec.execution_environment {
                ExecutionEnvironment::Native => {
                    self.stop_native_agent(agent).await?;
                }
                ExecutionEnvironment::Wasm => {
                    self.stop_wasm_agent(agent).await?;
                }
                ExecutionEnvironment::Container => {
                    self.stop_container_agent(agent).await?;
                }
                ExecutionEnvironment::VirtualMachine => {
                    self.stop_vm_agent(agent).await?;
                }
            }

            agent.status = AgentStatus::Stopped;

            info!("Agent {} stopped successfully", agent_id);
        } else {
            return Err(PrimalError::NotFoundError(format!(
                "Agent {} not found",
                agent_id
            )));
        }

        Ok(())
    }

    /// Stop native agent
    async fn stop_native_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Stopping native agent: {}", agent.agent_id);
        // Implementation for native agent shutdown
        Ok(())
    }

    /// Stop WASM agent
    async fn stop_wasm_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Stopping WASM agent: {}", agent.agent_id);
        // Implementation for WASM agent shutdown
        Ok(())
    }

    /// Stop container agent
    async fn stop_container_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Stopping container agent: {}", agent.agent_id);
        // Implementation for container agent shutdown
        Ok(())
    }

    /// Stop VM agent
    async fn stop_vm_agent(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        debug!("Stopping VM agent: {}", agent.agent_id);
        // Implementation for VM agent shutdown
        Ok(())
    }

    /// Get agent status
    pub async fn get_agent_status(&self, agent_id: &str) -> Result<AgentStatus, PrimalError> {
        let agents = self.deployed_agents.read().await;
        if let Some(agent) = agents.get(agent_id) {
            Ok(agent.status.clone())
        } else {
            Err(PrimalError::NotFoundError(format!(
                "Agent {} not found",
                agent_id
            )))
        }
    }

    /// List all deployed agents
    pub async fn list_agents(&self) -> Vec<DeployedAgent> {
        let agents = self.deployed_agents.read().await;
        agents.values().cloned().collect()
    }

    /// Update deployment status
    async fn update_deployment_status(&self) -> Result<(), PrimalError> {
        let agents = self.deployed_agents.read().await;

        let total_agents = agents.len() as u32;
        let running_agents = agents
            .values()
            .filter(|a| matches!(a.status, AgentStatus::Running))
            .count() as u32;
        let failed_agents = agents
            .values()
            .filter(|a| matches!(a.status, AgentStatus::Failed(_)))
            .count() as u32;

        let health = if failed_agents == 0 {
            if running_agents == total_agents {
                DeploymentHealth::Healthy
            } else {
                DeploymentHealth::Degraded
            }
        } else {
            DeploymentHealth::Unhealthy
        };

        let mut status = self.deployment_status.write().await;
        status.total_agents = total_agents;
        status.running_agents = running_agents;
        status.failed_agents = failed_agents;
        status.health = health;
        status.last_deployment = Utc::now();

        Ok(())
    }

    /// Get deployment status
    pub async fn get_deployment_status(&self) -> DeploymentStatus {
        self.deployment_status.read().await.clone()
    }

    /// Perform health check on all agents
    pub async fn health_check(&self) -> Result<(), PrimalError> {
        debug!("Performing health check on all agents");

        let mut agents = self.deployed_agents.write().await;

        for agent in agents.values_mut() {
            match self.check_agent_health(agent).await {
                Ok(_) => {
                    agent.last_health_check = Utc::now();
                    if matches!(agent.status, AgentStatus::Failed(_)) {
                        agent.status = AgentStatus::Running;
                    }
                }
                Err(e) => {
                    warn!("Health check failed for agent {}: {}", agent.agent_id, e);
                    agent.status = AgentStatus::Failed(e.to_string());
                }
            }
        }

        Ok(())
    }

    /// Check health of a single agent
    async fn check_agent_health(&self, agent: &DeployedAgent) -> Result<(), PrimalError> {
        // Implementation for health check
        // This would typically make an HTTP request to the agent's health endpoint
        Ok(())
    }
}

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

impl DeploymentStatus {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomeos_integration::manifest::BiomeManifestParser;

    #[tokio::test]
    async fn test_agent_deployment_creation() {
        let config = AgentDeploymentConfig::default();
        let mcp_integration = Arc::new(McpIntegration::new());
        let ai_intelligence = Arc::new(AiIntelligence::new());

        let manager = AgentDeploymentManager::new(config, mcp_integration, ai_intelligence);

        assert_eq!(manager.list_agents().await.len(), 0);
    }

    #[tokio::test]
    async fn test_manifest_deployment() {
        let config = AgentDeploymentConfig::default();
        let mcp_integration = Arc::new(McpIntegration::new());
        let ai_intelligence = Arc::new(AiIntelligence::new());

        let manager = AgentDeploymentManager::new(config, mcp_integration, ai_intelligence);

        let manifest = BiomeManifestParser::generate_template();
        let result = manager.deploy_from_manifest(&manifest).await;

        // This would pass once the full implementation is complete
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_validation() {
        let config = AgentDeploymentConfig::default();
        let mcp_integration = Arc::new(McpIntegration::new());
        let ai_intelligence = Arc::new(AiIntelligence::new());

        let manager = AgentDeploymentManager::new(config, mcp_integration, ai_intelligence);

        let manifest = BiomeManifestParser::generate_template();
        let agent_spec = &manifest.agents[0];

        let result = manager.validate_agent_spec(agent_spec).await;
        assert!(result.is_ok());
    }
}
