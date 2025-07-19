//! Core Squirrel Primal Provider Implementation

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::biomeos_integration::EcosystemClient;
use crate::ecosystem::EcosystemManager;
use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::optimization::zero_copy::{
    string_utils::StaticStrings,
    performance_monitoring::ZeroCopyMetrics,
};
use crate::session::SessionManagerImpl;
use crate::universal::*;
use crate::universal_adapter::SquirrelUniversalAdapter;
use squirrel_mcp_config::{Config, DefaultConfigManager, EcosystemConfig};

/// # Squirrel Primal Provider
///
/// The core implementation of the Squirrel AI primal, providing intelligent AI coordination,
/// context analysis, session management, and ecosystem integration capabilities.
pub struct SquirrelPrimalProvider {
    pub(super) instance_id: String,
    pub(super) config: EcosystemConfig,
    pub(super) universal_adapter: Arc<SquirrelUniversalAdapter>,
    pub(super) ecosystem_manager: Arc<EcosystemManager>,
    pub(super) session_manager: Arc<dyn crate::session::SessionManager>,
    pub(super) metrics_collector: Arc<MetricsCollector>,
    pub(super) context: PrimalContext,
    pub(super) config_manager: DefaultConfigManager,
    pub(super) biomeos_client: Option<Arc<EcosystemClient>>,
    pub(super) port_info: Option<DynamicPortInfo>,
    pub(super) initialized: bool,
    pub(super) shutdown: bool,
    pub(super) static_strings: StaticStrings,
    pub(super) zero_copy_metrics: Arc<ZeroCopyMetrics>,
}

impl SquirrelPrimalProvider {
    /// Creates a new SquirrelPrimalProvider instance
    pub fn new(
        config: EcosystemConfig,
        context: PrimalContext,
    ) -> Result<Self, PrimalError> {
        let instance_id = Uuid::new_v4().to_string();
        let metrics_collector = Arc::new(MetricsCollector::new());

        // Create universal adapter configuration
        let adapter_config = crate::universal_adapter::AdapterConfig {
            primal_id: "squirrel".to_string(),
            instance_id: instance_id.clone(),
            registry_config: crate::ecosystem::EcosystemRegistryConfig {
                songbird_endpoint: config
                    .discovery
                    .songbird_endpoint
                    .clone()
                    .unwrap_or_else(|| "http://localhost:8080".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let universal_adapter = Arc::new(
            SquirrelUniversalAdapter::new(adapter_config)
                .map_err(|e| PrimalError::Internal(e.to_string()))?,
        );
        let ecosystem_manager = Arc::new(EcosystemManager::new(
            crate::ecosystem::EcosystemConfig::default(),
            metrics_collector.clone(),
        ));
        let session_manager = Arc::new(SessionManagerImpl::new(
            crate::session::SessionConfig::default(),
        ));
        let config_manager = DefaultConfigManager::new();

        Ok(Self {
            instance_id,
            config,
            universal_adapter,
            ecosystem_manager,
            session_manager,
            metrics_collector,
            context,
            config_manager,
            biomeos_client: None,
            port_info: None,
            initialized: false,
            shutdown: false,
            static_strings: StaticStrings::new(),
            zero_copy_metrics: Arc::new(ZeroCopyMetrics::new()),
        })
    }

    /// Sets the BiomeOS client for ecosystem integration
    pub fn set_biomeos_client(&mut self, client: Arc<EcosystemClient>) {
        self.biomeos_client = Some(client);
    }

    /// Set session manager
    pub fn set_session_manager(&mut self, manager: Arc<dyn crate::session::SessionManager>) {
        self.session_manager = manager;
    }

    /// Set dynamic port information
    pub fn set_port_info(&mut self, port_info: DynamicPortInfo) {
        self.port_info = Some(port_info);
    }

    /// Get primal information
    pub fn get_primal_info(&self) -> PrimalInfo {
        PrimalInfo {
            id: self.primal_id().to_string(),
            instance_id: self.instance_id.clone(),
            primal_type: PrimalType::AI,
            capabilities: vec![],
            version: "1.0.0".to_string(),
        }
    }

    /// Update service mesh status
    pub fn update_service_mesh_status(&mut self, _status: ServiceMeshStatus) {
        // Service mesh status is managed by the ecosystem manager
        // and retrieved dynamically via the service_mesh_status() method
        // No field assignment needed
    }

    /// Get instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get primal name
    pub fn name(&self) -> &str {
        "Squirrel AI Primal"
    }

    /// Get primal description
    pub fn description(&self) -> &str {
        "AI coordination and context analysis primal"
    }
}

// ============================================================================
// UNIVERSAL PRIMAL PROVIDER TRAIT IMPLEMENTATION
// ============================================================================

#[async_trait]
impl UniversalPrimalProvider for SquirrelPrimalProvider {
    /// Get the primal ID
    fn primal_id(&self) -> &str {
        "squirrel"
    }

    /// Get the instance ID
    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get the context
    fn context(&self) -> &PrimalContext {
        &self.context
    }

    /// Get the primal type
    fn primal_type(&self) -> PrimalType {
        PrimalType::AI
    }

    /// Get available capabilities
    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            },
            PrimalCapability::AgentFramework {
                mcp_support: true,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string(), "es".to_string()],
            },
        ]
    }

    /// Get dependencies
    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            PrimalDependency {
                primal_type: PrimalType::Storage,
                required: false,
                capabilities: vec![], 
                min_version: "1.0.0".to_string(),
                preferred_instance: None,
            },
            PrimalDependency {
                primal_type: PrimalType::Compute,
                required: false,
                capabilities: vec![],
                min_version: "1.0.0".to_string(), 
                preferred_instance: None,
            },
        ]
    }

    /// Perform health check
    async fn health_check(&self) -> PrimalHealth {
        super::health_monitoring::HealthReporting::generate_health_report(self)
    }

    /// Get service endpoints
    fn endpoints(&self) -> PrimalEndpoints {
        PrimalEndpoints {
            primary: "http://0.0.0.0:8080".to_string(),
            health: "http://0.0.0.0:8080/health".to_string(),
            metrics: "http://0.0.0.0:8080/metrics".to_string(),
            admin: "http://0.0.0.0:8080/admin".to_string(),
            websocket: Some("ws://0.0.0.0:8080/ws".to_string()),
            mcp: "http://0.0.0.0:8080/mcp".to_string(),
            ai_coordination: "http://0.0.0.0:8080/ai".to_string(),
            service_mesh: "http://0.0.0.0:8080/mesh".to_string(),
            custom: HashMap::new(),
        }
    }

    /// Handle a primal request
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        info!("Handling primal request: {}", request.operation);

        let response_payload = match request.operation.as_str() {
            "ai_inference" => {
                self.handle_ai_inference_request(request.payload).await?
            }
            "context_analysis" => {
                self.handle_context_analysis_request(request.payload).await?
            }
            "session_create" => {
                self.create_session(request.payload).await?
            }
            "session_get" => {
                self.get_session(request.payload).await?
            }
            "session_update" => {
                self.update_session(request.payload).await?
            }
            "session_delete" => {
                self.delete_session(request.payload).await?
            }
            "session_list" => {
                self.list_user_sessions(request.payload).await?
            }
            "health_check" => {
                let healthy = self.perform_health_check().await?;
                serde_json::json!({
                    "healthy": healthy,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            _ => {
                return Err(PrimalError::OperationNotSupported(
                    format!("Unknown operation: {}", request.operation)
                ));
            }
        };

        Ok(PrimalResponse {
            response_id: Uuid::new_v4(),
            request_id: request.request_id,
            success: true,
            data: serde_json::Value::Object(serde_json::Map::new()),
            payload: response_payload,
            duration: chrono::Duration::milliseconds(100),
            error_message: None,
            status: ResponseStatus::Success,
            metadata: HashMap::new(),
        })
    }

    /// Initialize the primal
    async fn initialize(&mut self, _config: serde_json::Value) -> UniversalResult<()> {
        self.initialize_ecosystem().await.map_err(|e| PrimalError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Shutdown the primal
    async fn shutdown(&mut self) -> UniversalResult<()> {
        self.shutdown_ecosystem().await.map_err(|e| PrimalError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Validate if primal can serve the context
    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        self.can_serve_context(context)
    }

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        self.dynamic_port_info()
    }

    /// Register with Songbird service mesh
    async fn register_with_songbird(&mut self, songbird_endpoint: &str) -> UniversalResult<String> {
        self.register_with_songbird(songbird_endpoint).await
    }

    /// Deregister from Songbird service mesh
    async fn deregister_from_songbird(&mut self) -> UniversalResult<()> {
        self.deregister_from_songbird().await.map_err(|e| PrimalError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Get service mesh status
    fn get_service_mesh_status(&self) -> ServiceMeshStatus {
        self.get_service_mesh_status()
    }

    /// Handle ecosystem request
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse> {
        self.handle_ecosystem_request(request).await
    }

    /// Report health to ecosystem registry
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()> {
        self.report_health(health).await
    }

    /// Update system capabilities
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>) -> UniversalResult<()> {
        self.update_capabilities(capabilities).await
    }
} 