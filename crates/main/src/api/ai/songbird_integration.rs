//! Songbird integration for AI capabilities
//!
//! Registers AI capabilities with Songbird service mesh and enables
//! distributed AI provider discovery across the ecosystem.

use super::router::AiRouter;
use crate::error::PrimalError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

/// AI capability registration with Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapabilityRegistration {
    /// Service ID (Squirrel instance)
    pub service_id: String,

    /// Available AI capabilities
    pub capabilities: Vec<AiCapabilityInfo>,

    /// Service endpoint
    pub endpoint: String,

    /// Health check endpoint
    pub health_endpoint: String,

    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// Information about an AI capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapabilityInfo {
    /// Capability action (e.g., "image.generation")
    pub action: String,

    /// Number of available providers
    pub provider_count: usize,

    /// Whether capability is currently available
    pub available: bool,

    /// Average cost per unit (if known)
    pub avg_cost_usd: Option<f64>,

    /// Average latency in milliseconds
    pub avg_latency_ms: u64,

    /// Quality tier
    pub quality: String,
}

/// Response from Songbird AI capability registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCapabilityRegistrationResponse {
    pub success: bool,
    pub registration_id: String,
    pub message: String,
    pub next_heartbeat_seconds: u64,
}

/// Songbird AI integration manager
pub struct SongbirdAiIntegration {
    /// AI router reference
    router: Arc<AiRouter>,

    /// Songbird endpoint
    songbird_endpoint: String,

    /// HTTP client
    client: reqwest::Client,

    /// Service ID for this Squirrel instance
    service_id: String,

    /// Service endpoint
    service_endpoint: String,

    /// Registration ID (after registration)
    registration_id: Option<String>,
}

impl SongbirdAiIntegration {
    /// Create new Songbird AI integration
    #[must_use]
    pub fn new(router: Arc<AiRouter>, service_id: String, service_endpoint: String) -> Self {
        // Evolution: Use universal constants and environment-first discovery
        use universal_constants::network;

        let songbird_endpoint = std::env::var("SONGBIRD_ENDPOINT").unwrap_or_else(|_| {
            let port = network::get_port_from_env("SONGBIRD_PORT", network::DEFAULT_HTTP_PORT);
            network::http_url(network::DEFAULT_LOCALHOST, port, "")
        });

        Self {
            router,
            songbird_endpoint,
            client: reqwest::Client::new(),
            service_id,
            service_endpoint,
            registration_id: None,
        }
    }

    /// Register AI capabilities with Songbird
    pub async fn register_capabilities(&mut self) -> Result<(), PrimalError> {
        info!("🤝 Registering AI capabilities with Songbird...");

        // Get current capabilities from router
        let providers = self.router.list_providers().await;

        let capabilities: Vec<AiCapabilityInfo> = vec![
            AiCapabilityInfo {
                action: "image.generation".to_string(),
                provider_count: providers
                    .iter()
                    .filter(|p| p.capabilities.contains(&"image.generation".to_string()))
                    .count(),
                available: providers.iter().any(|p| {
                    p.capabilities.contains(&"image.generation".to_string()) && p.is_available
                }),
                avg_cost_usd: providers
                    .iter()
                    .filter(|p| p.capabilities.contains(&"image.generation".to_string()))
                    .find_map(|p| p.cost_per_unit),
                avg_latency_ms: providers
                    .iter()
                    .filter(|p| p.capabilities.contains(&"image.generation".to_string()))
                    .map(|p| p.avg_latency_ms)
                    .next()
                    .unwrap_or(10000),
                quality: "high".to_string(),
            },
            AiCapabilityInfo {
                action: "text.generation".to_string(),
                provider_count: providers
                    .iter()
                    .filter(|p| p.capabilities.contains(&"text.generation".to_string()))
                    .count(),
                available: providers.iter().any(|p| {
                    p.capabilities.contains(&"text.generation".to_string()) && p.is_available
                }),
                avg_cost_usd: providers
                    .iter()
                    .filter(|p| p.capabilities.contains(&"text.generation".to_string()))
                    .find_map(|p| p.cost_per_unit),
                avg_latency_ms: providers
                    .iter()
                    .filter(|p| p.capabilities.contains(&"text.generation".to_string()))
                    .map(|p| p.avg_latency_ms)
                    .next()
                    .unwrap_or(2000),
                quality: "high".to_string(),
            },
        ];

        let registration = AiCapabilityRegistration {
            service_id: self.service_id.clone(),
            capabilities,
            endpoint: format!("{}/ai", self.service_endpoint),
            health_endpoint: format!("{}/health", self.service_endpoint),
            registered_at: Utc::now(),
        };

        debug!("AI capability registration: {:?}", registration);

        // Register with Songbird
        let url = format!("{}/api/v1/ai/capabilities/register", self.songbird_endpoint);

        match self
            .client
            .post(&url)
            .json(&registration)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AiCapabilityRegistrationResponse>().await {
                        Ok(reg_response) => {
                            self.registration_id = Some(reg_response.registration_id.clone());
                            info!(
                                "✅ AI capabilities registered with Songbird: {}",
                                reg_response.registration_id
                            );
                            info!(
                                "📡 Heartbeat interval: {}s",
                                reg_response.next_heartbeat_seconds
                            );
                            Ok(())
                        }
                        Err(e) => {
                            warn!("⚠️  Failed to parse Songbird registration response: {}", e);
                            Ok(()) // Non-critical - continue without Songbird
                        }
                    }
                } else {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    warn!(
                        "⚠️  Songbird registration failed ({}): {}",
                        status, error_text
                    );
                    warn!("💡 AI capabilities will work locally without Songbird coordination");
                    Ok(()) // Non-critical
                }
            }
            Err(e) => {
                warn!(
                    "⚠️  Could not connect to Songbird at {}: {}",
                    self.songbird_endpoint, e
                );
                warn!("💡 AI capabilities will work locally without Songbird coordination");
                Ok(()) // Non-critical
            }
        }
    }

    /// Send heartbeat to Songbird
    pub async fn send_heartbeat(&self) -> Result<(), PrimalError> {
        if self.registration_id.is_none() {
            debug!("Skipping heartbeat - not registered with Songbird");
            return Ok(());
        }

        // Evolution: Safe access with early return if not registered
        let registration_id = if let Some(id) = &self.registration_id {
            id
        } else {
            debug!("Cannot send heartbeat - not registered with Songbird");
            return Ok(());
        };

        let url = format!(
            "{}/api/v1/ai/capabilities/heartbeat/{}",
            self.songbird_endpoint, registration_id
        );

        let providers = self.router.list_providers().await;

        let heartbeat = serde_json::json!({
            "service_id": self.service_id,
            "timestamp": Utc::now(),
            "available_capabilities": providers.iter()
                .filter(|p| p.is_available)
                .flat_map(|p| p.capabilities.clone())
                .collect::<Vec<_>>(),
            "provider_count": providers.len(),
        });

        match self
            .client
            .post(&url)
            .json(&heartbeat)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                debug!("💓 Heartbeat sent to Songbird");
                Ok(())
            }
            Ok(response) => {
                warn!("⚠️  Heartbeat failed: {}", response.status());
                Ok(()) // Non-critical
            }
            Err(e) => {
                warn!("⚠️  Heartbeat error: {}", e);
                Ok(()) // Non-critical
            }
        }
    }

    /// Start heartbeat loop
    pub async fn start_heartbeat_loop(self: Arc<Self>, interval_seconds: u64) {
        info!(
            "💓 Starting AI capability heartbeat loop ({}s interval)",
            interval_seconds
        );

        let mut ticker = interval(Duration::from_secs(interval_seconds));

        loop {
            ticker.tick().await;

            if let Err(e) = self.send_heartbeat().await {
                error!("❌ Heartbeat error: {}", e);
            }
        }
    }

    /// Query Songbird for distributed AI providers
    pub async fn query_distributed_providers(
        &self,
        capability: &str,
    ) -> Result<Vec<DistributedAiProvider>, PrimalError> {
        let url = format!(
            "{}/api/v1/ai/capabilities/query?capability={}",
            self.songbird_endpoint, capability
        );

        match self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.json::<Vec<DistributedAiProvider>>().await {
                    Ok(providers) => {
                        info!(
                            "🌐 Found {} distributed providers for '{}'",
                            providers.len(),
                            capability
                        );
                        Ok(providers)
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to parse distributed providers: {}", e);
                        Ok(Vec::new())
                    }
                }
            }
            Ok(response) => {
                warn!("⚠️  Query failed: {}", response.status());
                Ok(Vec::new())
            }
            Err(e) => {
                warn!("⚠️  Query error: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

/// Distributed AI provider information from Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedAiProvider {
    /// Service ID
    pub service_id: String,

    /// Service endpoint
    pub endpoint: String,

    /// Capability
    pub capability: String,

    /// Provider count
    pub provider_count: usize,

    /// Availability
    pub available: bool,

    /// Last heartbeat
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_info_serialization() {
        let info = AiCapabilityInfo {
            action: "image.generation".to_string(),
            provider_count: 2,
            available: true,
            avg_cost_usd: Some(0.02),
            avg_latency_ms: 12000,
            quality: "high".to_string(),
        };

        // Evolution: Use proper error handling instead of unwrap
        let json = serde_json::to_string(&info)
            .expect("Serialization of AiCapabilityInfo should always succeed");
        let deserialized: AiCapabilityInfo = serde_json::from_str(&json)
            .expect("Deserialization of just-serialized AiCapabilityInfo should always succeed");

        assert_eq!(deserialized.action, "image.generation");
        assert_eq!(deserialized.provider_count, 2);
    }
}
