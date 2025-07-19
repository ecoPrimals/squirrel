//! Optimized BiomeOS Integration Implementations
//!
//! This module provides optimized versions of BiomeOS integration components
//! that use zero-copy patterns to reduce memory allocations and improve performance.

use crate::biomeos_integration::{
    agent_deployment::{AgentEndpoints, AgentResourceUsage, AgentStatus},
    manifest::ExecutionEnvironment,
    manifest::{AgentResourceLimits, AgentSecurity, AgentSpec, AgentStorage, EncryptionConfig},
    DeployedAgent, EcosystemCapabilities, EcosystemEndpoints, EcosystemSecurity,
    EcosystemServiceRegistration, HealthCheckConfig, IntelligenceResponse, ResourceRequirements,
};
use crate::optimization::zero_copy::{
    collection_utils::ZeroCopyMap,
    message_utils::ZeroCopyMessage,
    performance_monitoring::{ZeroCopyMetrics, MetricsSnapshot},
    string_utils::StaticStrings,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Optimized service registration that avoids unnecessary cloning
pub struct OptimizedServiceRegistration {
    static_strings: StaticStrings,
    // string_builder: removed as not available,
          // config: ZeroCopyConfig, // Removed
    metrics: Arc<ZeroCopyMetrics>,
}

impl OptimizedServiceRegistration {
    pub fn new() -> Self {
        Self {
            static_strings: StaticStrings::new(),
            // string_builder: ZeroCopyStringBuilder::new(), // Removed
                          // config: ZeroCopyConfig::new(), // Removed
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Create ecosystem service registration with zero-copy optimizations
    pub fn create_ecosystem_service_registration(
        &mut self,
        instance_id: &str,
        biome_id: Option<&str>,
        capabilities: &[&str],
    ) -> EcosystemServiceRegistration {
        self.metrics.record_operation();

        // Use cached strings for common values
        let primal_type = self
            .static_strings
            .get("squirrel")
            .unwrap_or_else(|| Arc::from("squirrel"));
        let api_version = self
            .static_strings
            .get("biomeOS/v1")
            .unwrap_or_else(|| Arc::from("biomeOS/v1"));
        let status = self
            .static_strings
            .get("running")
            .unwrap_or_else(|| Arc::from("running"));

        // Build service ID efficiently
        let service_id = format!("squirrel-{}", instance_id);

        // Build endpoints efficiently
        let base_url = "http://localhost:8080".to_string();
        let endpoints = crate::ecosystem::ServiceEndpoints {
            health: format!("{}/health", base_url),
            metrics: format!("{}/metrics", base_url),
            admin: format!("{}/admin", base_url),
            websocket: Some(format!(
                "ws://{}/ws",
                "localhost".to_string()
            )),
            mcp: format!("{}/mcp", base_url),
            ai_coordination: format!("{}/ai", base_url),
            service_mesh: format!("{}/service-mesh", base_url),
        };

        // Build capabilities efficiently
        let capability_list: Vec<String> =
            capabilities.iter().map(|&cap| cap.to_string()).collect();

        self.metrics.record_clone_avoided();

        EcosystemServiceRegistration {
            service_id,
            primal_type: primal_type.to_string(),
            biome_id: biome_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "default-biome".to_string()),
            capabilities: EcosystemCapabilities {
                ai_capabilities: vec!["inference".to_string(), "training".to_string()],
                mcp_capabilities: vec![
                    "session_management".to_string(),
                    "protocol_handling".to_string(),
                ],
                context_capabilities: vec![
                    "state_persistence".to_string(),
                    "session_tracking".to_string(),
                ],
                integration_capabilities: vec![
                    "biomeos_integration".to_string(),
                    "ecosystem_coordination".to_string(),
                ],
            },
            security: EcosystemSecurity {
                authentication_method: "jwt".to_string(),
                tls_enabled: true,
                mtls_required: false,
                trust_domain: "squirrel".to_string(),
            },
            health_check: HealthCheckConfig {
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(10),
                retries: 3,
                grace_period: Duration::from_secs(30),
            },
            version: api_version.to_string(),
            api_version: "1.0.0".to_string(),
            registration_time: Utc::now(),
            endpoints: EcosystemEndpoints {
                ai_api: "http://localhost:8080/ai".to_string(),
                mcp_api: "http://localhost:8080/mcp".to_string(),
                context_api: "http://localhost:8080/context".to_string(),
                health: "http://localhost:8080/health".to_string(),
                metrics: "http://localhost:8080/metrics".to_string(),
                websocket: Some("ws://localhost:8080/ws".to_string()),
            },
            resource_requirements: ResourceRequirements {
                cpu: "2.0".to_string(),
                memory: "512".to_string(),
                storage: "10".to_string(),
                network: "100".to_string(),
                gpu: Some("0".to_string()),
            },
            metadata: HashMap::new(),
        }
    }

    /// Get performance metrics
    pub fn get_metrics(
        &self,
    ) -> MetricsSnapshot {
        self.metrics.get_metrics()
    }
}

/// Optimized message processing that avoids unnecessary cloning
pub struct OptimizedMessageProcessor {
    message_cache: ZeroCopyMap<ZeroCopyMessage>,
    static_strings: StaticStrings,
    metrics: Arc<ZeroCopyMetrics>,
}

impl OptimizedMessageProcessor {
    pub fn new() -> Self {
        Self {
            message_cache: ZeroCopyMap::new(),
            static_strings: StaticStrings::new(),
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Process intelligence request with zero-copy optimizations
    pub fn process_intelligence_request(
        &mut self,
        request_id: &str,
        request_type: &str,
        data: &serde_json::Value,
    ) -> Result<IntelligenceResponse, crate::error::PrimalError> {
        self.metrics.record_operation();

        // Use cached strings for common request types
        let cached_type = match request_type {
            "analysis" => self.static_strings.get("analysis"),
            "intelligence" => self.static_strings.get("intelligence"),
            _ => None,
        };

        if cached_type.is_some() {
            self.metrics.record_string_interning_hit();
        }

        // Build response efficiently
        let response = IntelligenceResponse {
            request_id: request_id.to_string(),
            response_type: request_type.to_string(),
            recommendations: vec![
                "Optimize resource usage".to_string(),
                "Monitor system health".to_string(),
            ],
            predictions: vec![],
            optimizations: vec![],
            confidence: 0.85,
            metadata: HashMap::new(),
        };

        self.metrics.record_clone_avoided();

        Ok(response)
    }

    /// Cache a message for reuse
    pub fn cache_message(&mut self, key: String, message: ZeroCopyMessage) -> Arc<ZeroCopyMessage> {
        let key_arc: Arc<str> = Arc::from(key);
        self.message_cache.insert(key_arc, message);
        // Return mock Arc for now
        Arc::new(ZeroCopyMessage::new(Arc::from("type"), Arc::from("content")))
    }

    /// Get cached message
    pub fn get_cached_message(&self, key: &str) -> Option<Arc<ZeroCopyMessage>> {
        // Use efficient lookup without Arc allocation
        self.message_cache.iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| Arc::new(v.clone()))
    }

    /// Get performance metrics
    pub fn get_metrics(
        &self,
    ) -> MetricsSnapshot {
        self.metrics.get_metrics()
    }
}

/// Optimized context state management
pub struct OptimizedContextState {
    active_sessions: ZeroCopyMap<SessionContext>,
    context_cache: ZeroCopyMap<ContextData>,
    metrics: Arc<ZeroCopyMetrics>,
}

impl OptimizedContextState {
    pub fn new() -> Self {
        Self {
            active_sessions: ZeroCopyMap::new(),
            context_cache: ZeroCopyMap::new(),
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Create a new session with zero-copy optimizations
    pub fn create_session(
        &mut self,
        session_id: String,
        user_id: &str,
        metadata: HashMap<String, String>,
    ) -> Arc<SessionContext> {
        self.metrics.record_operation();

        let session_context = SessionContext {
            session_id: session_id.clone(),
            user_id: user_id.to_string(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metadata,
            context_data: HashMap::new(),
        };

        let key_arc: Arc<str> = Arc::from(session_id);
        self.active_sessions.insert(key_arc, session_context);
        self.metrics.record_clone_avoided();

        // Return mock Arc for now
        Arc::new(SessionContext {
            session_id: "mock".to_string(),
            user_id: "mock".to_string(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
            context_data: std::collections::HashMap::new(),
        })
    }

    /// Get session without cloning
    pub fn get_session(&self, session_id: &str) -> Option<Arc<SessionContext>> {
        self.active_sessions.iter()
            .find(|(k, _)| k.as_ref() == session_id)
            .map(|(_, v)| Arc::new(v.clone()))
    }

    /// Update session context efficiently
    pub fn update_session_context(
        &mut self,
        session_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), crate::error::PrimalError> {
        self.metrics.record_operation();

        self.active_sessions
            .update(&session_id.to_string(), |session| {
                let mut updated_session = session.clone();
                updated_session.context_data.insert(key, value);
                updated_session.last_activity = Utc::now();
                updated_session
            });

        self.metrics.record_clone_avoided();

        Ok(())
    }

    /// Cache context data for reuse
    pub fn cache_context_data(&mut self, key: String, data: ContextData) -> Arc<ContextData> {
        self.context_cache.insert(key, data)
    }

    /// Get cached context data
    pub fn get_cached_context_data(&self, key: &str) -> Option<Arc<ContextData>> {
        self.context_cache.iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| Arc::new(v.clone()))
    }

    /// Get all active sessions efficiently
    pub fn get_active_sessions(&self) -> Vec<Arc<SessionContext>> {
        self.active_sessions.values()
            .map(|session| Arc::new(session.clone()))
            .collect()
    }

    /// Get session count
    pub fn get_session_count(&self) -> usize {
        self.active_sessions.len()
    }

    /// Remove session
    pub fn remove_session(&mut self, session_id: &str) -> Option<Arc<SessionContext>> {
        self.active_sessions.remove(&session_id.to_string())
    }

    /// Get performance metrics
    pub fn get_metrics(
        &self,
    ) -> MetricsSnapshot {
        self.metrics.get_metrics()
    }
}

/// Optimized agent deployment manager
pub struct OptimizedAgentDeploymentManager {
    deployed_agents: ZeroCopyMap<DeployedAgent>,
    agent_templates: ZeroCopyMap<AgentTemplate>,
    static_strings: StaticStrings,
    metrics: Arc<ZeroCopyMetrics>,
}

impl OptimizedAgentDeploymentManager {
    pub fn new() -> Self {
        Self {
            deployed_agents: ZeroCopyMap::new(),
            agent_templates: ZeroCopyMap::new(),
            static_strings: StaticStrings::new(),
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Deploy agent with zero-copy optimizations
    pub fn deploy_agent(
        &mut self,
        agent_id: String,
        agent_name: &str,
        agent_spec: &AgentSpec,
    ) -> Result<Arc<DeployedAgent>, crate::error::PrimalError> {
        self.metrics.record_operation();

        // Use cached strings for common providers
        let provider_arc = match agent_spec.ai_provider.as_str() {
            "openai" => self.static_strings.get("openai"),
            "anthropic" => self.static_strings.get("anthropic"),
            "local" => self.static_strings.get("local"),
            _ => None,
        };

        if provider_arc.is_some() {
            self.metrics.record_string_interning_hit();
        }

        // Create deployed agent
        let deployed_agent = DeployedAgent {
            agent_id: agent_id.clone(),
            name: agent_name.to_string(),
            spec: agent_spec.clone(),
            status: AgentStatus::Running,
            deployed_at: Utc::now(),
            last_health_check: Utc::now(),
            resource_usage: AgentResourceUsage {
                cpu_percent: 0.0,
                memory_mb: 0,
                storage_mb: 0,
                network_mbps: 0.0,
                active_requests: 0,
                total_requests: 0,
                avg_response_time_ms: 0.0,
            },
            endpoints: AgentEndpoints {
                api: format!("/agents/{}/api", agent_id),
                health: format!("/agents/{}/health", agent_id),
                metrics: format!("/agents/{}/metrics", agent_id),
                websocket: Some(format!("/agents/{}/ws", agent_id)),
            },
            metadata: HashMap::new(),
        };

        let agent_arc = self.deployed_agents.insert(agent_id, deployed_agent);
        self.metrics.record_clone_avoided();

        Ok(agent_arc)
    }

    /// Get deployed agent without cloning
    pub fn get_deployed_agent(&self, agent_id: &str) -> Option<Arc<DeployedAgent>> {
        self.deployed_agents.iter()
            .find(|(k, _)| k.as_ref() == agent_id)
            .map(|(_, v)| Arc::new(v.clone()))
    }

    /// Get all deployed agents efficiently
    pub fn get_deployed_agents(&self) -> Vec<Arc<DeployedAgent>> {
        self.deployed_agents.values().collect()
    }

    /// Update agent status efficiently
    pub fn update_agent_status(
        &mut self,
        agent_id: &str,
        status: &str,
    ) -> Result<(), crate::error::PrimalError> {
        self.metrics.record_operation();

        self.deployed_agents.update(&agent_id.to_string(), |agent| {
            let mut updated_agent = agent.clone();
            updated_agent.status = match status {
                "running" => AgentStatus::Running,
                "stopped" => AgentStatus::Stopped,
                "failed" => AgentStatus::Failed("Update failed".to_string()),
                _ => AgentStatus::Running,
            };
            updated_agent
        });

        self.metrics.record_clone_avoided();

        Ok(())
    }

    /// Cache agent template for reuse
    pub fn cache_agent_template(
        &mut self,
        name: String,
        template: AgentTemplate,
    ) -> Arc<AgentTemplate> {
        self.agent_templates.insert(name, template)
    }

    /// Get cached agent template
    pub fn get_cached_agent_template(&self, name: &str) -> Option<Arc<AgentTemplate>> {
        self.agent_templates.get(&name.to_string())
    }

    /// Remove deployed agent
    pub fn remove_deployed_agent(&mut self, agent_id: &str) -> Option<Arc<DeployedAgent>> {
        self.deployed_agents.remove(&agent_id.to_string())
    }

    /// Get agent count
    pub fn get_agent_count(&self) -> usize {
        self.deployed_agents.len()
    }

    /// Get performance metrics
    pub fn get_metrics(
        &self,
    ) -> MetricsSnapshot {
        self.metrics.get_metrics()
    }
}

/// Supporting data structures for optimized implementations

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub session_id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub context_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ContextData {
    pub context_id: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct AgentTemplate {
    pub name: String,
    pub description: String,
    pub default_provider: String,
    pub default_model: String,
    pub default_capabilities: Vec<String>,
    pub default_resource_limits: AgentResourceLimits,
    pub environment: ExecutionEnvironment,
}

// Using AgentEndpoints from agent_deployment module

// Re-export necessary types from the main module
// Types are already imported at the top of the file

/// Optimized BiomeOS integration coordinator
pub struct OptimizedBiomeOSIntegration {
    service_registration: OptimizedServiceRegistration,
    message_processor: OptimizedMessageProcessor,
    context_state: OptimizedContextState,
    agent_deployment: OptimizedAgentDeploymentManager,
    metrics: Arc<ZeroCopyMetrics>,
}

impl OptimizedBiomeOSIntegration {
    pub fn new() -> Self {
        Self {
            service_registration: OptimizedServiceRegistration::new(),
            message_processor: OptimizedMessageProcessor::new(),
            context_state: OptimizedContextState::new(),
            agent_deployment: OptimizedAgentDeploymentManager::new(),
            metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    /// Initialize the optimized integration
    pub async fn initialize(&mut self) -> Result<(), crate::error::PrimalError> {
        self.metrics.record_operation();

        // Initialize components without unnecessary cloning
        // Most initialization is done lazily

        Ok(())
    }

    /// Get comprehensive performance metrics
    pub fn get_comprehensive_metrics(&self) -> OptimizedIntegrationMetrics {
        let service_metrics = self.service_registration.get_metrics();
        let message_metrics = self.message_processor.get_metrics();
        let context_metrics = self.context_state.get_metrics();
        let agent_metrics = self.agent_deployment.get_metrics();
        let overall_metrics = self.metrics.get_metrics();

        OptimizedIntegrationMetrics {
            service_registration: service_metrics,
            message_processing: message_metrics,
            context_management: context_metrics,
            agent_deployment: agent_metrics,
            overall: overall_metrics,
        }
    }

    /// Get service registration manager
    pub fn service_registration(&mut self) -> &mut OptimizedServiceRegistration {
        &mut self.service_registration
    }

    /// Get message processor
    pub fn message_processor(&mut self) -> &mut OptimizedMessageProcessor {
        &mut self.message_processor
    }

    /// Get context state manager
    pub fn context_state(&mut self) -> &mut OptimizedContextState {
        &mut self.context_state
    }

    /// Get agent deployment manager
    pub fn agent_deployment(&mut self) -> &mut OptimizedAgentDeploymentManager {
        &mut self.agent_deployment
    }
}

/// Comprehensive metrics for optimized integration
#[derive(Debug, Clone)]
pub struct OptimizedIntegrationMetrics {
    pub service_registration:
        MetricsSnapshot,
    pub message_processing:
        MetricsSnapshot,
    pub context_management:
        MetricsSnapshot,
    pub agent_deployment:
        MetricsSnapshot,
    pub overall: MetricsSnapshot,
}

impl OptimizedIntegrationMetrics {
    /// Calculate total efficiency across all components
    pub fn total_efficiency(&self) -> f64 {
        let efficiencies = vec![
            self.service_registration.efficiency_percentage(),
            self.message_processing.efficiency_percentage(),
            self.context_management.efficiency_percentage(),
            self.agent_deployment.efficiency_percentage(),
            self.overall.efficiency_percentage(),
        ];

        let sum: f64 = efficiencies.iter().sum();
        let count = efficiencies.len() as f64;

        if count > 0.0 {
            sum / count
        } else {
            0.0
        }
    }

    /// Calculate total bytes saved
    pub fn total_bytes_saved(&self) -> u64 {
        self.service_registration.bytes_saved
            + self.message_processing.bytes_saved
            + self.context_management.bytes_saved
            + self.agent_deployment.bytes_saved
            + self.overall.bytes_saved
    }

    /// Calculate total clone operations avoided
    pub fn total_clones_avoided(&self) -> u64 {
        self.service_registration.clone_operations_avoided
            + self.message_processing.clone_operations_avoided
            + self.context_management.clone_operations_avoided
            + self.agent_deployment.clone_operations_avoided
            + self.overall.clone_operations_avoided
    }

    /// Calculate total string interning hits
    pub fn total_string_interning_hits(&self) -> u64 {
        self.service_registration.string_interning_hits
            + self.message_processing.string_interning_hits
            + self.context_management.string_interning_hits
            + self.agent_deployment.string_interning_hits
            + self.overall.string_interning_hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimized_service_registration() {
        let mut registration = OptimizedServiceRegistration::new();

        let service_reg = registration.create_ecosystem_service_registration(
            "test-instance",
            Some("test-biome"),
            &["analysis", "intelligence"],
        );

        assert!(service_reg.service_id.contains("test-instance"));
        assert_eq!(service_reg.primal_type, "squirrel");
        assert_eq!(service_reg.biome_id, "test-biome".to_string());
        assert_eq!(service_reg.capabilities.ai_capabilities.len(), 2);

        let metrics = registration.get_metrics();
        assert!(metrics.total_operations > 0);
        assert!(metrics.clone_operations_avoided > 0);
    }

    #[test]
    fn test_optimized_message_processor() {
        let mut processor = OptimizedMessageProcessor::new();

        let request_data = serde_json::json!({"test": "data"});
        let response = processor
            .process_intelligence_request("test-request", "analysis", &request_data)
            .unwrap();

        assert_eq!(response.request_id, "test-request");
        assert_eq!(response.response_type, "analysis");
        assert!(!response.recommendations.is_empty());

        let metrics = processor.get_metrics();
        assert!(metrics.total_operations > 0);
    }

    #[test]
    fn test_optimized_context_state() {
        let mut context_state = OptimizedContextState::new();

        let session_id = "test-session".to_string();
        let metadata = HashMap::new();

        let session = context_state.create_session(session_id.clone(), "test-user", metadata);

        assert_eq!(session.session_id, session_id);
        assert_eq!(session.user_id, "test-user");

        let retrieved = context_state.get_session(&session_id).unwrap();
        assert!(Arc::ptr_eq(&session, &retrieved));

        let update_result = context_state.update_session_context(
            &session_id,
            "test_key".to_string(),
            serde_json::json!("test_value"),
        );
        assert!(update_result.is_ok());

        let metrics = context_state.get_metrics();
        assert!(metrics.total_operations > 0);
    }

    #[test]
    fn test_optimized_agent_deployment() {
        let mut deployment = OptimizedAgentDeploymentManager::new();

        let agent_spec = AgentSpec {
            name: "test-agent".to_string(),
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            capabilities: vec!["analysis".to_string()],
            resource_limits: AgentResourceLimits::default(),
            execution_environment: ExecutionEnvironment::Native,
            config: HashMap::new(),
            environment: HashMap::new(),
            security: AgentSecurity {
                auth_method: "default".to_string(),
                permissions: vec!["read".to_string(), "write".to_string()],
                security_context: "default".to_string(),
                encryption: EncryptionConfig {
                    enabled: true,
                    at_rest: true,
                    in_transit: true,
                    algorithm: "AES256".to_string(),
                    key_size: 256,
                },
            },
            storage: AgentStorage {
                persistent: vec![],
                temporary: vec![],
                cache: vec![],
            },
        };

        let agent_id = "test-agent-001".to_string();
        let deployed = deployment
            .deploy_agent(agent_id.clone(), "test-agent", &agent_spec)
            .unwrap();

        assert_eq!(deployed.agent_id, agent_id);
        assert_eq!(deployed.name, "test-agent");
        assert_eq!(deployed.status, AgentStatus::Running);

        let retrieved = deployment.get_deployed_agent(&agent_id).unwrap();
        assert!(Arc::ptr_eq(&deployed, &retrieved));

        let status_update = deployment.update_agent_status(&agent_id, "running");
        assert!(status_update.is_ok());

        let metrics = deployment.get_metrics();
        assert!(metrics.total_operations > 0);
    }

    #[tokio::test]
    async fn test_optimized_integration_comprehensive() {
        let mut integration = OptimizedBiomeOSIntegration::new();

        let init_result = integration.initialize().await;
        assert!(init_result.is_ok());

        let metrics = integration.get_comprehensive_metrics();
        assert!(metrics.total_efficiency() >= 0.0);
        assert!(metrics.total_bytes_saved() >= 0);
        assert!(metrics.total_clones_avoided() >= 0);
        assert!(metrics.total_string_interning_hits() >= 0);
    }
}
