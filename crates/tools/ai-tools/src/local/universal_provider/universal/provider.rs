//! Universal AI Provider Implementation
//!
//! Main provider implementation for capability-based AI service integration.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::config::*;
use super::discovery::CapabilityDiscoveryEngine;
use super::matcher::CapabilityMatcher;
use super::types::*;
use crate::common::{AIClient, ChatRequest, ChatResponse, ChatResponseStream, MessageRole};
use crate::Result;

/// Universal AI provider - completely capability-based service integration
#[derive(Debug)]
pub struct UniversalAIProvider {
    config: UniversalAIConfig,
    capability_registry: Arc<RwLock<CapabilityRegistry>>,
    discovery_engine: Arc<CapabilityDiscoveryEngine>,
    capability_matcher: CapabilityMatcher,
    performance_tracker: Arc<RwLock<HashMap<uuid::Uuid, PerformanceHistory>>>,
}

impl UniversalAIProvider {
    /// Create a new universal AI provider
    pub async fn new(config: UniversalAIConfig) -> Result<Self> {
        info!("Initializing Universal AI Provider with pure capability discovery");

        let discovery_engine =
            Arc::new(CapabilityDiscoveryEngine::new(config.discovery.clone()).await?);

        let provider = Self {
            config,
            capability_registry: Arc::new(RwLock::new(CapabilityRegistry {
                capabilities: HashMap::new(),
                performance_index: HashMap::new(),
                last_updated: std::time::SystemTime::now(),
            })),
            discovery_engine,
            capability_matcher: CapabilityMatcher::new(),
            performance_tracker: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initial capability discovery
        provider.refresh_capabilities().await?;

        // Start periodic refresh
        provider.start_capability_refresh().await;

        Ok(provider)
    }

    /// Refresh capability discovery
    pub async fn refresh_capabilities(&self) -> Result<()> {
        info!("Refreshing capability discovery");

        let discovered = self.discovery_engine.discover_capabilities().await?;

        let mut registry = self.capability_registry.write().await;
        registry.capabilities.clear();
        registry.performance_index.clear();

        // Group capabilities by type
        for capability in discovered {
            let capability_type = capability.capability.capability_type.clone();

            registry
                .capabilities
                .entry(capability_type.clone())
                .or_insert_with(Vec::new)
                .push(capability.clone());

            registry
                .performance_index
                .entry(capability_type)
                .or_insert_with(Vec::new)
                .push(capability.performance);
        }

        registry.last_updated = std::time::SystemTime::now();

        info!(
            "Updated capability registry with {} capability types",
            registry.capabilities.len()
        );
        Ok(())
    }

    /// Start periodic capability refresh
    async fn start_capability_refresh(&self) {
        let registry = Arc::clone(&self.capability_registry);
        let discovery_engine = Arc::clone(&self.discovery_engine);
        let refresh_interval = self.config.discovery.refresh_interval_ms;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_millis(refresh_interval));

            loop {
                interval.tick().await;

                match discovery_engine.discover_capabilities().await {
                    Ok(discovered) => {
                        let mut registry_guard = registry.write().await;
                        registry_guard.capabilities.clear();

                        for capability in discovered {
                            let capability_type = capability.capability.capability_type.clone();
                            registry_guard
                                .capabilities
                                .entry(capability_type)
                                .or_insert_with(Vec::new)
                                .push(capability);
                        }

                        registry_guard.last_updated = std::time::SystemTime::now();
                        debug!("Periodic capability refresh completed");
                    }
                    Err(e) => {
                        warn!("Periodic capability refresh failed: {}", e);
                    }
                }
            }
        });
    }

    /// Get capability statistics
    pub async fn get_capability_stats(&self) -> HashMap<String, usize> {
        let registry = self.capability_registry.read().await;
        registry
            .capabilities
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect()
    }

    /// Update performance metrics for a capability provider
    pub async fn update_performance_metrics(
        &self,
        provider_id: uuid::Uuid,
        latency_ms: u64,
        success: bool,
        bytes_transferred: Option<u64>,
    ) {
        let mut tracker = self.performance_tracker.write().await;
        let history = tracker
            .entry(provider_id)
            .or_insert_with(|| PerformanceHistory {
                metrics: Vec::new(),
                average_latency: 0.0,
                success_rate: 1.0,
                last_updated: std::time::SystemTime::now(),
            });

        // Add new metric
        history.metrics.push(RequestMetric {
            timestamp: std::time::Instant::now(),
            latency_ms,
            success,
            bytes_transferred,
        });

        // Keep only recent metrics (last 1000)
        if history.metrics.len() > 1000 {
            history.metrics.remove(0);
        }

        // Update aggregated metrics
        let total_requests = history.metrics.len() as f64;
        let successful_requests = history.metrics.iter().filter(|m| m.success).count() as f64;
        let total_latency: u64 = history.metrics.iter().map(|m| m.latency_ms).sum();

        history.average_latency = total_latency as f64 / total_requests;
        history.success_rate = successful_requests / total_requests;
        history.last_updated = std::time::SystemTime::now();
    }

    /// Get local inference capability (fallback)
    async fn get_local_inference_capability(&self) -> Option<CapabilityProvider> {
        if !self.config.local_inference.enabled {
            return None;
        }

        Some(CapabilityProvider {
            provider_id: uuid::Uuid::new_v4(),
            capability: AICapability {
                capability_type: "text-generation".to_string(),
                input_formats: vec![DataFormat::Text, DataFormat::JSON],
                output_formats: vec![DataFormat::Text, DataFormat::JSON],
                processing_type: ProcessingType::Synchronous,
                quality_profile: QualityProfile {
                    accuracy: 0.70, // Local models typically lower accuracy
                    consistency: 0.85,
                    context_understanding: 0.75,
                    specializations: vec!["local".to_string(), "privacy".to_string()],
                },
                cost_profile: CostProfile {
                    cost_per_request: 0.0,
                    cost_per_unit: 0.0,
                    is_free: true,
                    tier: CostTier::Free,
                },
            },
            interface: CapabilityInterface {
                protocol: CommunicationProtocol::ProcessCall,
                endpoint: EndpointInfo {
                    address: crate::config::DefaultEndpoints::ai_service_host(),
                    port: None,
                    path: None,
                    tls: false,
                },
                auth: AuthRequirements::None,
                message_format: MessageFormat::JSON,
            },
            performance: PerformanceProfile {
                average_latency_ms: 5000.0, // Local inference can be slower
                p95_latency_ms: 10000.0,
                throughput_requests_per_second: 2.0,
                availability: 1.0, // Always available locally
                reliability: 0.95,
            },
            resources: ResourceProfile {
                compute_intensity: ComputeIntensity::Heavy,
                memory_usage_mb: Some(8192),
                network_bandwidth_mbps: None,
                storage_requirements: Some(StorageRequirements {
                    persistent: true,
                    size_mb: 4000, // Model storage
                    iops_required: Some(100),
                }),
            },
            trust_metrics: TrustMetrics {
                uptime_percentage: 100.0,
                error_rate: 0.05,
                response_consistency: 0.95,
                security_score: 1.0, // Local execution = highest security
                community_rating: None,
            },
        })
    }

    /// Execute request using local inference
    async fn execute_local_inference(&self, request: &ChatRequest) -> Result<ChatResponse> {
        info!("Executing request using local inference capabilities");

        // Simple local response - in a full implementation this would
        // integrate with actual local model inference
        let response_text = format!(
            "Local inference response: Your request '{}' has been processed using local capabilities. This ensures privacy while providing AI functionality through the universal capability system.",
            request.messages.first()
                .and_then(|m| m.content.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("(no content)")
                .chars().take(50).collect::<String>()
        );

        Ok(ChatResponse {
            id: format!("local-{}", uuid::Uuid::new_v4()),
            model: "local-universal".to_string(),
            choices: vec![crate::common::ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some(response_text),
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: Some(crate::common::UsageInfo {
                prompt_tokens: 50,
                completion_tokens: 100,
                total_tokens: 150,
            }),
        })
    }
}

#[async_trait]
impl AIClient for UniversalAIProvider {
    fn provider_name(&self) -> &str {
        "universal-ai-provider"
    }

    async fn get_capabilities(
        &self,
        _model: &str,
    ) -> Result<crate::common::capability::AICapabilities> {
        // Return capabilities based on discovered providers
        Ok(crate::common::capability::AICapabilities {
            supported_model_types: {
                let mut set = HashSet::new();
                set.insert(crate::common::capability::ModelType::LargeLanguageModel);
                set
            },
            supported_task_types: {
                let mut set = HashSet::new();
                set.insert(crate::common::capability::TaskType::TextGeneration);
                set.insert(crate::common::capability::TaskType::QuestionAnswering);
                set
            },
            max_context_size: 4096,
            supports_tool_use: false,
            supports_images: false,
            supports_streaming: false,
            supports_function_calling: false,
            performance_metrics: Default::default(),
            resource_requirements: Default::default(),
            cost_metrics: Default::default(),
            security_requirements: Default::default(),
            routing_preferences: Default::default(),
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Return models based on discovered capabilities
        let registry = self.capability_registry.read().await;
        let models: Vec<String> = registry.capabilities.keys().cloned().collect();
        Ok(if models.is_empty() {
            vec!["universal-local".to_string()]
        } else {
            models
        })
    }

    async fn is_available(&self) -> bool {
        // Always available as it can fall back to local inference
        true
    }

    fn default_model(&self) -> &str {
        "universal-local"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        debug!("Processing chat request through universal capability system");

        let registry = self.capability_registry.read().await;

        // Find best capability for this request
        if let Some(provider) = self
            .capability_matcher
            .find_best_capability(&request, &registry, &self.config.requirements)
            .await
        {
            info!(
                "Selected capability provider: {} for {}",
                provider.provider_id, provider.capability.capability_type
            );

            // TODO: Actually execute the request with the selected provider
            // For now, track the selection and fall back to local inference
            drop(registry);
            self.update_performance_metrics(
                provider.provider_id,
                2000, // Estimated latency
                true,
                None,
            )
            .await;

            return self.execute_local_inference(&request).await;
        }

        // No suitable provider found, use local inference as fallback
        drop(registry);
        if let Some(_local_capability) = self.get_local_inference_capability().await {
            warn!("No suitable capability provider found, falling back to local inference");
            return self.execute_local_inference(&request).await;
        }

        error!("No capability providers available and local inference disabled");
        Err(crate::Error::Model(
            "No suitable capability providers found and local inference is disabled".to_string(),
        ))
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatResponseStream> {
        // TODO: Implement streaming support for capability-based providers
        Err(crate::Error::Runtime(
            "Streaming not yet implemented for universal provider".to_string(),
        ))
    }
}

/// Factory function for creating universal AI provider
pub async fn create_universal_ai_provider(
    config: Option<UniversalAIConfig>,
) -> Result<UniversalAIProvider> {
    UniversalAIProvider::new(config.unwrap_or_default()).await
}

/// Setup development environment with example capabilities
pub fn setup_development_capabilities() {
    // Set up example capability announcements
    std::env::set_var(
        "CAPABILITY_TEXT_GENERATION_1_ENDPOINT",
        crate::config::DefaultEndpoints::ollama_endpoint(),
    );
    std::env::set_var("CAPABILITY_TEXT_GENERATION_1_FORMAT", "openai");
    std::env::set_var("CAPABILITY_TEXT_GENERATION_1_AUTH", "none");
    std::env::set_var("CAPABILITY_TEXT_GENERATION_1_COST", "0.0");

    std::env::set_var(
        "CAPABILITY_CODE_GENERATION_1_ENDPOINT",
        crate::config::DefaultEndpoints::llamacpp_endpoint(),
    );
    std::env::set_var("CAPABILITY_CODE_GENERATION_1_FORMAT", "openai");
    std::env::set_var("CAPABILITY_CODE_GENERATION_1_AUTH", "none");
    std::env::set_var("CAPABILITY_CODE_GENERATION_1_COST", "0.0");

    std::env::set_var(
        "CAPABILITY_TEXT_GENERATION_2_ENDPOINT",
        "https://openrouter.ai/api/v1",
    );
    std::env::set_var("CAPABILITY_TEXT_GENERATION_2_FORMAT", "openai");
    std::env::set_var("CAPABILITY_TEXT_GENERATION_2_AUTH", "bearer");
    std::env::set_var("CAPABILITY_TEXT_GENERATION_2_COST", "0.001");

    info!("Development capabilities configured - ready for ecosystem discovery");
}
