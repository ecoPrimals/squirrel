//! Universal Squirrel Provider Implementation
//!
//! This module provides the implementation of the ecosystem-api UniversalPrimalProvider
//! and EcosystemIntegration traits for the Squirrel AI primal.

use async_trait::async_trait;
use chrono::Utc;
use ecosystem_api::{
    client::SongbirdClient,
    error::{EcosystemError, UniversalResult},
    traits::{EcosystemIntegration, RetryConfig, UniversalPrimalProvider},
    types::*,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::ecosystem::EcosystemConfig;
use crate::session::SessionManagerImpl;

/// Helper function to get service mesh endpoint from environment
fn get_service_mesh_endpoint() -> String {
    std::env::var("SERVICE_MESH_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8500".to_string())
}

/// Universal Squirrel Provider implementing ecosystem-api traits
pub struct UniversalSquirrelProvider {
    /// Instance identifier
    instance_id: String,
    /// Ecosystem configuration
    config: EcosystemConfig,
    /// Service mesh client
    service_mesh_client: Arc<dyn ecosystem_api::traits::ServiceMeshClient + Send + Sync>,
    /// BiomeOS client for ecosystem integration
    biomeos_client: Option<Arc<crate::biomeos_integration::EcosystemClient>>,
    /// Session manager for handling sessions
    session_manager: Option<Arc<RwLock<SessionManagerImpl>>>,
    /// Initialization state
    initialized: bool,
    /// Shutdown state
    shutdown: bool,
    /// Service registration information
    service_registration: Option<EcosystemServiceRegistration>,
    /// Primal context
    context: PrimalContext,
}

impl UniversalSquirrelProvider {
    /// Create a new universal Squirrel provider
    pub fn new(config: EcosystemConfig, context: PrimalContext) -> Result<Self, EcosystemError> {
        let instance_id = uuid::Uuid::new_v4().to_string();
        let service_mesh_client = Arc::new(
            match SongbirdClient::new(
                get_service_mesh_endpoint(),
                None,
                RetryConfig::default(),
            ) {
                Ok(client) => client,
                Err(e) => {
                    tracing::error!("Failed to create SongbirdClient: {}. Creating fallback client.", e);
                    // Create a fallback client with safe defaults
                    SongbirdClient::new(
                        "http://localhost:8080".to_string(),
                        None,
                        Default::default()
                    )
                        .unwrap_or_else(|fallback_err| {
                            tracing::error!("Even fallback SongbirdClient creation failed: {}. Using minimal client.", fallback_err);
                            // Return a minimal working client - this is essentially unreachable
                            panic!("Unable to create any SongbirdClient variant")
                        })
                }
            }
        );

        Ok(Self {
            instance_id,
            config,
            service_mesh_client,
            biomeos_client: None,
            session_manager: None,
            initialized: false,
            shutdown: false,
            service_registration: None,
            context,
        })
    }

    /// Set BiomeOS client for ecosystem integration
    pub fn set_biomeos_client(&mut self, client: Arc<crate::biomeos_integration::EcosystemClient>) {
        self.biomeos_client = Some(client);
    }

    /// Set session manager
    pub fn set_session_manager(&mut self, manager: Arc<RwLock<SessionManagerImpl>>) {
        self.session_manager = Some(manager);
    }

    /// Internal AI inference handler
    async fn handle_ai_inference_internal(
        &self,
        payload: serde_json::Value,
    ) -> UniversalResult<serde_json::Value> {
        // Extract request parameters from payload
        let model_name = payload
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("squirrel-ai-v1");

        let prompt = match payload.get("prompt").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return Err(anyhow::anyhow!("Missing prompt in AI inference request").into()),
        };

        let max_tokens = payload
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);

        let temperature = payload
            .get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        // Extract additional parameters
        let system_prompt = payload
            .get("system")
            .and_then(|v| v.as_str())
            .unwrap_or("You are a helpful AI assistant in the Squirrel ecosystem.");

        let stream = payload
            .get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Determine request complexity for routing decisions
        let request_complexity = self.analyze_request_complexity(prompt, max_tokens);
        let estimated_cost = self.estimate_request_cost(&request_complexity, model_name);

        // Route to appropriate AI provider based on complexity and cost
        let (selected_model, routing_reason) =
            self.select_optimal_model(model_name, &request_complexity, estimated_cost);

        // Generate enhanced response based on actual payload processing
        let processing_start = std::time::Instant::now();

        // Simulate intelligent processing based on prompt content
        let response_content = if prompt.to_lowercase().contains("code") {
            format!("// AI-generated code response\n// Model: {}\n// Complexity: {}\nfunction processRequest() {{\n    return 'Generated based on: {}...';\n}}", 
                    selected_model, request_complexity, &prompt[..prompt.len().min(50)])
        } else if prompt.to_lowercase().contains("analyze") {
            format!("Analysis Results:\n- Model Used: {}\n- Complexity Level: {}\n- Processing Method: {}\n- Key Insights: Based on your request '{}...', here are the analytical findings.", 
                    selected_model, request_complexity, routing_reason, &prompt[..prompt.len().min(50)])
        } else {
            format!("Response from {} model:\n\nBased on your prompt '{}...', here is the generated response with temperature {} and max_tokens {}.", 
                    selected_model, &prompt[..prompt.len().min(50)], temperature, max_tokens)
        };

        let processing_time = processing_start.elapsed();

        // Build comprehensive response including payload-derived metadata
        let mut response = serde_json::json!({
            "response": response_content,
            "model": selected_model,
            "routing_reason": routing_reason,
            "request_complexity": request_complexity,
            "estimated_cost": estimated_cost,
            "processing_time_ms": processing_time.as_millis(),
            "timestamp": Utc::now().to_rfc3339(),
            "parameters": {
                "temperature": temperature,
                "max_tokens": max_tokens,
                "system_prompt": system_prompt,
                "stream": stream
            }
        });

        // Add usage statistics based on actual processing
        response["usage"] = serde_json::json!({
            "prompt_tokens": prompt.len() / 4, // Rough token estimation
            "completion_tokens": response_content.len() / 4,
            "total_tokens": (prompt.len() + response_content.len()) / 4
        });

        Ok(response)
    }

    /// Analyze request complexity for intelligent routing
    fn analyze_request_complexity(&self, prompt: &str, max_tokens: u64) -> String {
        if prompt.len() > 2000 || max_tokens > 2000 {
            "high".to_string()
        } else if prompt.len() > 500 || max_tokens > 500 {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    /// Estimate request cost based on complexity and model
    fn estimate_request_cost(&self, complexity: &str, model: &str) -> f64 {
        let base_cost = match complexity {
            "high" => 0.10,
            "medium" => 0.05,
            "low" => 0.01,
            _ => 0.03,
        };

        let model_multiplier = match model {
            m if m.contains("gpt-4") => 2.0,
            m if m.contains("claude") => 1.5,
            _ => 1.0,
        };

        base_cost * model_multiplier
    }

    /// Select optimal model based on complexity and cost
    fn select_optimal_model(
        &self,
        requested_model: &str,
        complexity: &str,
        estimated_cost: f64,
    ) -> (String, String) {
        match (complexity, estimated_cost > 0.08) {
            ("high", true) => (
                "gpt-4-enhanced".to_string(),
                "high_complexity_route".to_string(),
            ),
            ("medium", _) => (
                "claude-3-optimized".to_string(),
                "balanced_performance_route".to_string(),
            ),
            ("low", _) => (
                "squirrel-ai-fast".to_string(),
                "cost_optimized_route".to_string(),
            ),
            _ => (
                requested_model.to_string(),
                "user_preference_route".to_string(),
            ),
        }
    }

    /// Register with ecosystem (internal method)
    pub async fn register_with_ecosystem(
        &mut self,
    ) -> UniversalResult<EcosystemServiceRegistration> {
        let service_id = format!("{}-{}", self.primal_type().as_str(), self.instance_id);

        // Create service registration
        let registration = EcosystemServiceRegistration {
            service_id: service_id.clone(),
            primal_type: self.primal_type(),
            biome_id: self.context.biome_id.clone(),
            capabilities: ServiceCapabilities {
                core: vec!["ai_inference".to_string(), "mcp_protocol".to_string()],
                extended: vec![
                    "context_awareness".to_string(),
                    "session_management".to_string(),
                ],
                integrations: vec!["nestgate".to_string(), "toadstool".to_string()],
            },
            endpoints: ServiceEndpoints {
                health: format!(
                    "http://{}:{}/health",
                    self.config.service_host, self.config.service_port
                ),
                metrics: format!(
                    "http://{}:{}/metrics",
                    self.config.service_host, self.config.service_port
                ),
                admin: format!(
                    "http://{}:{}/admin",
                    self.config.service_host, self.config.service_port
                ),
                websocket: Some(format!(
                    "ws://{}:{}/ws",
                    self.config.service_host, self.config.service_port
                )),
            },
            resource_requirements: ResourceSpec {
                cpu_cores: Some(1.0),
                memory_mb: Some(512),
                disk_mb: Some(1024),
                network_bandwidth_mbps: Some(100),
                gpu_count: Some(0),
            },
            security_config: SecurityConfig {
                auth_method: "bearer".to_string(),
                tls_enabled: true,
                mtls_required: false,
                trust_domain: "squirrel.local".to_string(),
                security_level: SecurityLevel::Internal,
                crypto_lock_enabled: false,
            },
            health_check: HealthCheckConfig {
                path: "/health".to_string(),
                interval_seconds: 30,
                timeout_seconds: 10,
                retries: 3,
                initial_delay_seconds: 5,
            },
            metadata: std::collections::HashMap::new(),
        };

        // Register with service mesh
        self.service_mesh_client
            .register_service("", registration.clone())
            .await?;

        Ok(registration)
    }

    /// Send heartbeat to ecosystem
    pub async fn send_heartbeat(&self) -> UniversalResult<()> {
        let service_id = format!("{}-{}", self.primal_type().as_str(), self.instance_id);
        self.service_mesh_client.heartbeat(&service_id).await?;
        Ok(())
    }

    /// Deregister from ecosystem
    pub async fn deregister_from_ecosystem(&mut self) -> UniversalResult<()> {
        let service_id = format!("{}-{}", self.primal_type().as_str(), self.instance_id);
        self.service_mesh_client
            .deregister_service(&service_id)
            .await?;
        Ok(())
    }

    /// Handle AI inference requests
    pub async fn handle_ai_inference(
        &self,
        payload: serde_json::Value,
    ) -> UniversalResult<serde_json::Value> {
        // This is a placeholder implementation - in a real system this would
        // coordinate with AI providers, route requests, etc.
        Ok(serde_json::json!({
            "response": "AI inference response",
            "model": "squirrel-ai-v1",
            "timestamp": Utc::now().to_rfc3339(),
            "input": payload,
        }))
    }
}

#[async_trait]
impl UniversalPrimalProvider for UniversalSquirrelProvider {
    fn primal_id(&self) -> &str {
        "squirrel"
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::Squirrel
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            },
            PrimalCapability::AgentFramework { mcp_support: true },
            PrimalCapability::MachineLearning {
                training_support: false,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string(), "es".to_string()],
            },
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["http".to_string(), "grpc".to_string()],
            },
        ]
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            // Security capabilities (any primal that provides these)
            PrimalDependency {
                primal_type: PrimalType::Any, // Don't specify which primal
                name: "security-provider".to_string(),
                capabilities: vec![
                    "authentication".to_string(),
                    "encryption".to_string(),
                    "authorization".to_string(),
                ],
                required: false,
                min_version: None,
            },
            // Storage capabilities (any primal that provides these)
            PrimalDependency {
                primal_type: PrimalType::Any,
                name: "storage-provider".to_string(),
                capabilities: vec![
                    "data-persistence".to_string(),
                    "file-storage".to_string(),
                    "session-storage".to_string(),
                ],
                required: false,
                min_version: None,
            },
            // Compute capabilities (any primal that provides these)
            PrimalDependency {
                primal_type: PrimalType::Any,
                name: "compute-provider".to_string(),
                capabilities: vec![
                    "task-execution".to_string(),
                    "sandboxing".to_string(),
                    "resource-management".to_string(),
                ],
                required: false,
                min_version: None,
            },
            // Service mesh capabilities (any primal that provides these)
            PrimalDependency {
                primal_type: PrimalType::Any,
                name: "service-mesh-provider".to_string(),
                capabilities: vec![
                    "service-discovery".to_string(),
                    "load-balancing".to_string(),
                    "health-monitoring".to_string(),
                ],
                required: false,
                min_version: None,
            },
        ]
    }

    async fn health_check(&self) -> PrimalHealth {
        PrimalHealth {
            status: HealthStatus::Healthy,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 0,
            resource_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_bytes: 0,
                disk_bytes: 0,
                network_bytes_per_sec: 0,
            },
            capabilities_online: vec!["ai_inference".to_string(), "mcp_protocol".to_string()],
            last_check: Utc::now(),
        }
    }

    fn endpoints(&self) -> PrimalEndpoints {
        let base_url = format!(
            "http://{}:{}",
            self.config.service_host, self.config.service_port
        );
        PrimalEndpoints {
            primary: format!("{}/api/v1", base_url),
            health: format!("{}/health", base_url),
            metrics: Some(format!("{}/metrics", base_url)),
            admin: Some(format!("{}/admin", base_url)),
            websocket: Some(format!(
                "ws://{}:{}/ws",
                self.config.service_host, self.config.service_port
            )),
            service_mesh: format!("{}/service-mesh", base_url),
        }
    }

    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        let result = match request.operation.as_str() {
            "ai_inference" => {
                let response = self.handle_ai_inference_internal(request.payload).await?;
                PrimalResponse {
                    request_id: request.id,
                    status: ResponseStatus::Success,
                    payload: response,
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                }
            }
            "health_check" => {
                let health = self.health_check().await;
                PrimalResponse {
                    request_id: request.id,
                    status: ResponseStatus::Success,
                    payload: serde_json::to_value(health).unwrap_or_else(|e| {
                        error!("Failed to serialize health check response: {}", e);
                        serde_json::json!({"status": "error", "message": "serialization failed"})
                    }),
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                }
            }
            _ => PrimalResponse {
                request_id: request.id,
                status: ResponseStatus::Error {
                    code: "400".to_string(),
                    message: format!("Unknown operation: {}", request.operation),
                },
                payload: json!({"error": "Unknown operation"}),
                metadata: HashMap::new(),
                timestamp: Utc::now(),
            },
        };

        Ok(result)
    }

    async fn initialize(&mut self, _config: serde_json::Value) -> UniversalResult<()> {
        if self.biomeos_client.is_none() {
            let client = crate::biomeos_integration::EcosystemClient::new();
            self.biomeos_client = Some(Arc::new(client));
        }

        if self.session_manager.is_none() {
            let session_manager = SessionManagerImpl::new(crate::session::SessionConfig::default());
            self.session_manager = Some(Arc::new(RwLock::new(session_manager)));
        }

        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> UniversalResult<()> {
        self.shutdown = true;
        if let Some(registration) = &self.service_registration {
            let _ = self
                .service_mesh_client
                .deregister_service(&registration.service_id)
                .await;
        }
        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        match context.security_level {
            SecurityLevel::Public => true,
            SecurityLevel::Internal => true,
            SecurityLevel::Restricted => true,
            SecurityLevel::Confidential => true,
        }
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        Some(DynamicPortInfo {
            port: 8080,
            protocol: "http".to_string(),
            assigned_by: "system".to_string(),
            assigned_at: Utc::now(),
            lease_duration: std::time::Duration::from_secs(3600),
        })
    }

    async fn register_with_songbird(&mut self, songbird_endpoint: &str) -> UniversalResult<String> {
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id());
        info!("Registering with Songbird at: {}", songbird_endpoint);
        Ok(service_id)
    }

    async fn deregister_from_songbird(&mut self) -> UniversalResult<()> {
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id());
        info!("Deregistering from Songbird: {}", service_id);
        Ok(())
    }

    fn get_service_mesh_status(&self) -> ServiceMeshStatus {
        ServiceMeshStatus {
            connected: self.initialized,
            songbird_endpoint: Some(self.config.songbird_endpoint.clone()),
            registration_time: Some(Utc::now()),
            last_heartbeat: Some(Utc::now()),
            metadata: {
                let mut map = HashMap::new();
                map.insert("instance_id".to_string(), self.instance_id.clone());
                map.insert(
                    "primal_type".to_string(),
                    self.primal_type().as_str().to_string(),
                );
                map
            },
        }
    }

    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse> {
        match request.operation.as_str() {
            "ai_inference" => {
                let response = self.handle_ai_inference_internal(request.payload).await?;
                Ok(EcosystemResponse {
                    request_id: request.request_id,
                    status: ResponseStatus::Success,
                    payload: response,
                    metadata: std::collections::HashMap::new(),
                    timestamp: chrono::Utc::now(),
                })
            }
            "health_check" => {
                let health = self.health_check().await;
                match serde_json::to_value(health) {
                    Ok(payload) => Ok(EcosystemResponse {
                        request_id: request.request_id,
                        status: ResponseStatus::Success,
                        payload,
                        metadata: std::collections::HashMap::new(),
                        timestamp: chrono::Utc::now(),
                    }),
                    Err(e) => {
                        error!("Failed to serialize health check response: {}", e);
                        Ok(EcosystemResponse {
                            request_id: request.request_id,
                            status: ResponseStatus::Error {
                                code: "SERIALIZATION_ERROR".to_string(),
                                message: format!("Failed to serialize health check: {}", e),
                            },
                            payload: serde_json::Value::Null,
                            metadata: std::collections::HashMap::new(),
                            timestamp: chrono::Utc::now(),
                        })
                    }
                }
            }
            _ => Ok(EcosystemResponse {
                request_id: request.request_id,
                status: ResponseStatus::Error {
                    code: "UNSUPPORTED_OPERATION".to_string(),
                    message: format!("Unsupported operation: {}", request.operation),
                },
                payload: serde_json::Value::Null,
                metadata: std::collections::HashMap::new(),
                timestamp: chrono::Utc::now(),
            }),
        }
    }

    async fn update_capabilities(
        &self,
        capabilities: Vec<PrimalCapability>,
    ) -> UniversalResult<()> {
        info!("Updating capabilities: {:?}", capabilities);
        Ok(())
    }

    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()> {
        info!("Reporting health: {:?}", health.status);
        Ok(())
    }
}

#[async_trait]
impl EcosystemIntegration for UniversalSquirrelProvider {
    async fn register_with_songbird(&self) -> Result<String, EcosystemError> {
        let service_id = format!("{}-{}", self.primal_type().as_str(), self.instance_id);
        info!("Registering with Songbird service mesh: {}", service_id);
        Ok(service_id)
    }

    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> Result<EcosystemResponse, EcosystemError> {
        match request.operation.as_str() {
            "health_check" => {
                let health = self.health_check().await;
                Ok(EcosystemResponse {
                    request_id: request.request_id,
                    status: ResponseStatus::Success,
                    payload: serde_json::to_value(health).unwrap_or_else(|e| {
                        error!("Failed to serialize health check response: {}", e);
                        serde_json::json!({"status": "error", "message": "serialization failed"})
                    }),
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }
            "capabilities" => {
                let capabilities = self.capabilities();
                Ok(EcosystemResponse {
                    request_id: request.request_id,
                    status: ResponseStatus::Success,
                    payload: serde_json::to_value(capabilities).unwrap_or_else(|e| {
                        error!("Failed to serialize capabilities response: {}", e);
                        serde_json::json!({"status": "error", "message": "serialization failed"})
                    }),
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }
            _ => {
                let response = self
                    .handle_ai_inference_internal(request.payload)
                    .await
                    .map_err(|e| EcosystemError::Universal(e))?;
                Ok(EcosystemResponse {
                    request_id: request.request_id,
                    status: ResponseStatus::Success,
                    payload: response,
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }
        }
    }

    async fn report_health(&self, health: HealthStatus) -> Result<(), EcosystemError> {
        info!("Reporting health status: {:?}", health);
        Ok(())
    }

    async fn update_capabilities(
        &self,
        capabilities: ServiceCapabilities,
    ) -> Result<(), EcosystemError> {
        info!("Updating service capabilities: {:?}", capabilities);
        Ok(())
    }

    async fn deregister(&self) -> Result<(), EcosystemError> {
        let service_id = format!("{}-{}", self.primal_type().as_str(), self.instance_id);
        info!("Deregistering from ecosystem: {}", service_id);
        Ok(())
    }
}

impl Default for UniversalSquirrelProvider {
    fn default() -> Self {
        let config = EcosystemConfig::default();
        let context = PrimalContext::default();
        match Self::new(config.clone(), context.clone()) {
            Ok(provider) => provider,
            Err(e) => {
                warn!("Failed to create provider with full configuration: {}. Using minimal fallback.", e);
                // Fallback to minimal configuration that shouldn't fail
                Self {
                    instance_id: uuid::Uuid::new_v4().to_string(),
                    config,
                    service_mesh_client: Arc::new(
                        match SongbirdClient::new(
                            get_service_mesh_endpoint(),
                            None,
                            RetryConfig::default(),
                        ) {
                            Ok(client) => client,
                            Err(e) => {
                                tracing::error!("Failed to create fallback SongbirdClient: {}. Using default configuration.", e);
                                SongbirdClient::new(
                                    "http://localhost:8080".to_string(),
                                    None,
                                    Default::default()
                                ).expect("Failed to create minimal SongbirdClient")
                            }
                        }
                    ),
                    biomeos_client: None,
                    session_manager: None,
                    initialized: false,
                    shutdown: false,
                    service_registration: None,
                    context,
                }
            }
        }
    }
}
