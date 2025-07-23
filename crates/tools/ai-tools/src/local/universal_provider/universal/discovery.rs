//! Universal AI Provider Discovery Engine
//!
//! Logic for discovering capability providers across different discovery methods.

use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::config::{CapabilityDiscoveryConfig, DiscoveryMethod};
use super::types::*;
use crate::Result;

/// Capability discovery engine - finds services based on their capabilities
#[derive(Debug)]
pub struct CapabilityDiscoveryEngine {
    config: CapabilityDiscoveryConfig,
    client: reqwest::Client,
}

impl CapabilityDiscoveryEngine {
    pub async fn new(config: CapabilityDiscoveryConfig) -> Result<Self> {
        info!(
            "Initializing capability discovery engine with {} methods",
            config.methods.len()
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.query_timeout_ms))
            .build()
            .map_err(|e| crate::Error::Network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Discover all capabilities using configured methods
    pub async fn discover_capabilities(&self) -> Result<Vec<CapabilityProvider>> {
        let mut all_capabilities = Vec::new();

        for method in &self.config.methods {
            match method {
                DiscoveryMethod::Environment { prefix } => {
                    match self.discover_environment_capabilities(prefix).await {
                        Ok(mut capabilities) => all_capabilities.append(&mut capabilities),
                        Err(e) => warn!("Environment discovery failed: {}", e),
                    }
                }
                DiscoveryMethod::Network { port_range } => {
                    match self.discover_network_capabilities(port_range).await {
                        Ok(mut capabilities) => all_capabilities.append(&mut capabilities),
                        Err(e) => warn!("Network discovery failed: {}", e),
                    }
                }
                DiscoveryMethod::Process { search_paths } => {
                    match self.discover_process_capabilities(search_paths).await {
                        Ok(mut capabilities) => all_capabilities.append(&mut capabilities),
                        Err(e) => warn!("Process discovery failed: {}", e),
                    }
                }
                DiscoveryMethod::FileSystem { directory } => {
                    match self.discover_filesystem_capabilities(directory).await {
                        Ok(mut capabilities) => all_capabilities.append(&mut capabilities),
                        Err(e) => warn!("Filesystem discovery failed: {}", e),
                    }
                }
                DiscoveryMethod::Custom { name, config } => {
                    match self.discover_custom_capabilities(name, config).await {
                        Ok(mut capabilities) => all_capabilities.append(&mut capabilities),
                        Err(e) => warn!("Custom discovery '{}' failed: {}", name, e),
                    }
                }
            }
        }

        info!("Discovered {} capability providers", all_capabilities.len());
        Ok(all_capabilities)
    }

    async fn discover_environment_capabilities(
        &self,
        prefix: &str,
    ) -> Result<Vec<CapabilityProvider>> {
        let mut capabilities = Vec::new();

        // Parse environment variables for capability announcements
        // Format: CAPABILITY_[TYPE]_[INDEX]_[PROPERTY]
        // Example: CAPABILITY_TEXT_GENERATION_1_ENDPOINT=http://localhost:11434
        let mut capability_configs: HashMap<String, HashMap<String, String>> = HashMap::new();

        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let parts: Vec<&str> = key.split('_').collect();
                if parts.len() >= 4 {
                    let capability_key = format!("{}_{}", parts[1], parts[2]); // e.g., "TEXT_GENERATION_1"
                    let property = parts[3..].join("_").to_lowercase(); // e.g., "endpoint"

                    capability_configs
                        .entry(capability_key)
                        .or_insert_with(HashMap::new)
                        .insert(property, value);
                }
            }
        }

        for (capability_key, config) in capability_configs {
            if let Some(capability) = self
                .build_capability_from_env(&capability_key, &config)
                .await
            {
                capabilities.push(capability);
            }
        }

        Ok(capabilities)
    }

    async fn build_capability_from_env(
        &self,
        capability_key: &str,
        config: &HashMap<String, String>,
    ) -> Option<CapabilityProvider> {
        let endpoint = config.get("endpoint")?;
        let capability_type = capability_key
            .split('_')
            .take(2)
            .collect::<Vec<_>>()
            .join("-")
            .to_lowercase();

        // Test connectivity
        if let Err(_) = self.test_endpoint(endpoint).await {
            debug!("Endpoint {} is not reachable, skipping", endpoint);
            return None;
        }

        let provider_id = Uuid::new_v4();

        Some(CapabilityProvider {
            provider_id,
            capability: AICapability {
                capability_type,
                input_formats: vec![DataFormat::JSON],
                output_formats: vec![DataFormat::JSON, DataFormat::Stream],
                processing_type: ProcessingType::Interactive,
                quality_profile: QualityProfile {
                    accuracy: config
                        .get("accuracy")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.80),
                    consistency: 0.85,
                    context_understanding: 0.80,
                    specializations: vec![],
                },
                cost_profile: CostProfile {
                    cost_per_request: config
                        .get("cost")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.001),
                    cost_per_unit: 0.0,
                    is_free: config
                        .get("cost")
                        .map(|c| c == "0.0" || c == "free")
                        .unwrap_or(false),
                    tier: CostTier::Low,
                },
            },
            interface: CapabilityInterface {
                protocol: CommunicationProtocol::HTTP,
                endpoint: self.parse_endpoint(endpoint)?,
                auth: match config.get("auth").map(|s| s.as_str()) {
                    Some("none") => AuthRequirements::None,
                    Some("bearer") => AuthRequirements::Bearer {
                        token_endpoint: None,
                    },
                    Some("apikey") => AuthRequirements::ApiKey {
                        header: "Authorization".to_string(),
                    },
                    _ => AuthRequirements::None,
                },
                message_format: MessageFormat::JSON,
            },
            performance: PerformanceProfile {
                average_latency_ms: config
                    .get("latency")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3000.0),
                p95_latency_ms: config
                    .get("p95_latency")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5000.0),
                throughput_requests_per_second: 10.0,
                availability: 0.99,
                reliability: 0.95,
            },
            resources: ResourceProfile {
                compute_intensity: ComputeIntensity::Heavy,
                memory_usage_mb: Some(4096),
                network_bandwidth_mbps: None,
                storage_requirements: Some(StorageRequirements {
                    persistent: false,
                    size_mb: 100,
                    iops_required: None,
                }),
            },
            trust_metrics: TrustMetrics {
                uptime_percentage: 100.0,
                error_rate: 0.05,
                response_consistency: 0.95,
                security_score: 1.0, // Local execution
                community_rating: None,
            },
        })
    }

    fn parse_endpoint(&self, endpoint: &str) -> Option<EndpointInfo> {
        if let Ok(url) = url::Url::parse(endpoint) {
            Some(EndpointInfo {
                address: url
                    .host_str()
                    .unwrap_or(&crate::config::DefaultEndpoints::ai_service_host())
                    .to_string(),
                port: url.port(),
                path: if url.path().is_empty() {
                    None
                } else {
                    Some(url.path().to_string())
                },
                tls: url.scheme() == "https",
            })
        } else {
            None
        }
    }

    async fn test_endpoint(&self, endpoint: &str) -> Result<()> {
        let response = self.client.get(endpoint).send().await.map_err(|e| {
            crate::Error::Network(format!("Failed to connect to {}: {}", endpoint, e))
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(crate::Error::Network(format!(
                "Endpoint {} returned status: {}",
                endpoint,
                response.status()
            )))
        }
    }

    async fn discover_network_capabilities(
        &self,
        port_range: &(u16, u16),
    ) -> Result<Vec<CapabilityProvider>> {
        let mut capabilities = Vec::new();

        // Probe known AI service ports
        let known_ai_ports = vec![11434, 8080, 8000, 5000]; // Ollama, llama.cpp, common AI ports

        for &port in &known_ai_ports {
            if port >= port_range.0 && port <= port_range.1 {
                if let Ok(capability) = self
                    .probe_network_service(
                        &crate::config::DefaultEndpoints::ai_service_host(),
                        port,
                    )
                    .await
                {
                    capabilities.push(capability);
                }
            }
        }

        Ok(capabilities)
    }

    async fn probe_network_service(&self, host: &str, port: u16) -> Result<CapabilityProvider> {
        let endpoint = format!("http://{}:{}", host, port);

        // Test basic connectivity
        self.test_endpoint(&endpoint).await?;

        let provider_id = Uuid::new_v4();

        Ok(CapabilityProvider {
            provider_id,
            capability: AICapability {
                capability_type: "text-generation".to_string(),
                input_formats: vec![DataFormat::JSON],
                output_formats: vec![DataFormat::JSON],
                processing_type: ProcessingType::Interactive,
                quality_profile: QualityProfile {
                    accuracy: 0.75,
                    consistency: 0.80,
                    context_understanding: 0.70,
                    specializations: vec!["general".to_string()],
                },
                cost_profile: CostProfile {
                    cost_per_request: 0.0,
                    cost_per_unit: 0.0,
                    is_free: true,
                    tier: CostTier::Free,
                },
            },
            interface: CapabilityInterface {
                protocol: CommunicationProtocol::HTTP,
                endpoint: EndpointInfo {
                    address: host.to_string(),
                    port: Some(port),
                    path: None,
                    tls: false,
                },
                auth: AuthRequirements::None,
                message_format: MessageFormat::JSON,
            },
            performance: PerformanceProfile {
                average_latency_ms: 2000.0,
                p95_latency_ms: 3000.0,
                throughput_requests_per_second: 5.0,
                availability: 0.95,
                reliability: 0.90,
            },
            resources: ResourceProfile {
                compute_intensity: ComputeIntensity::Medium,
                memory_usage_mb: Some(2048),
                network_bandwidth_mbps: Some(10.0),
                storage_requirements: None,
            },
            trust_metrics: TrustMetrics {
                uptime_percentage: 95.0,
                error_rate: 0.10,
                response_consistency: 0.85,
                security_score: 0.8,
                community_rating: Some(4.0),
            },
        })
    }

    async fn discover_process_capabilities(
        &self,
        _search_paths: &[std::path::PathBuf],
    ) -> Result<Vec<CapabilityProvider>> {
        // Look for AI-capable processes in search paths
        Ok(vec![]) // Simplified for now
    }

    async fn discover_filesystem_capabilities(
        &self,
        _directory: &Path,
    ) -> Result<Vec<CapabilityProvider>> {
        // Look for capability definition files
        Ok(vec![]) // Simplified for now
    }

    async fn discover_custom_capabilities(
        &self,
        _name: &str,
        _config: &serde_json::Value,
    ) -> Result<Vec<CapabilityProvider>> {
        // Custom discovery implementation
        Ok(vec![]) // Simplified for now
    }
}
