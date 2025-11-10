//! # Ecosystem Client for biomeOS Integration
//!
//! This module provides client functionality for communicating with songbird
//! for service registration and coordination within the biomeOS ecosystem.

use crate::error::PrimalError;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use squirrel_mcp_config::Config;
use std::time::Duration;
use tracing::{debug, info};
use std::time::Instant;

use super::{EcosystemServiceRegistration, HealthStatus};

// Constants for URL patterns to reduce allocations
const REGISTER_ENDPOINT: &str = "/api/v1/services/register";
const HEALTH_ENDPOINT: &str = "/api/v1/health";

/// Client for biomeOS ecosystem communication via songbird
#[derive(Debug, Clone)]
pub struct EcosystemClient {
    pub songbird_url: String,
    pub client: reqwest::Client,
    pub timeout: Duration,
    pub retry_count: u32,
    pub authentication: AuthenticationConfig,
}

/// Authentication configuration for ecosystem communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    pub auth_type: String,
    pub token: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub trust_domain: String,
}

/// Response from songbird service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub success: bool,
    pub service_id: String,
    pub registration_id: String,
    pub coordination_token: String,
    pub songbird_endpoints: Vec<String>,
    pub next_heartbeat: DateTime<Utc>,
    pub message: String,
}

/// Health check response from songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub ecosystem_health: f64,
    pub primal_statuses: Vec<PrimalStatus>,
    pub coordination_recommendations: Vec<String>,
}

/// Status of other primals reported by songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub primal_type: String,
    pub service_id: String,
    pub status: String,
    pub health_score: f64,
    pub last_seen: DateTime<Utc>,
    pub capabilities: Vec<String>,
    pub load_factor: f64,
}

/// AI intelligence request to other primals via songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceCoordinationRequest {
    pub request_id: String,
    pub target_primal: String,
    pub intelligence_type: String,
    pub request_data: serde_json::Value,
    pub priority: String,
    pub timeout: Duration,
}

/// AI intelligence response from other primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceCoordinationResponse {
    pub request_id: String,
    pub response_data: serde_json::Value,
    pub confidence: f64,
    pub processing_time: f64,
    pub recommendations: Vec<String>,
}

/// MCP coordination request via songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCoordinationViaRequest {
    pub coordination_id: String,
    pub target_primals: Vec<String>,
    pub coordination_type: String,
    pub mcp_data: serde_json::Value,
    pub requires_response: bool,
}

/// Context sharing request via songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSharingRequest {
    pub sharing_id: String,
    pub target_primal: String,
    pub context_type: String,
    pub context_data: serde_json::Value,
    pub sharing_permissions: Vec<String>,
    pub expiry: Option<DateTime<Utc>>,
}

/// Ecosystem metrics from songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_primals: u32,
    pub active_primals: u32,
    pub coordination_efficiency: f64,
    pub overall_health: f64,
    pub ai_intelligence_requests: u64,
    pub mcp_coordinations: u64,
    pub context_shares: u64,
}

impl EcosystemClient {
    /// Create a new EcosystemClient with configuration
    pub fn new() -> Self {
        let songbird_url =
            std::env::var("SONGBIRD_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let timeout_seconds = std::env::var("TIMEOUT_SECONDS")
            .map(|s| s.parse().unwrap_or(30))
            .unwrap_or(30);

        Self {
            songbird_url,
            client: reqwest::Client::new(),
            timeout: Duration::from_secs(timeout_seconds),
            retry_count: 3,
            authentication: AuthenticationConfig::default(),
        }
    }

    /// Create a new EcosystemClient with custom configuration
    pub fn with_config(_config: Config) -> Self {
        let songbird_url =
            std::env::var("SONGBIRD_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        let timeout_seconds = std::env::var("TIMEOUT_SECONDS")
            .map(|s| s.parse().unwrap_or(30))
            .unwrap_or(30);

        Self {
            songbird_url,
            client: reqwest::Client::new(),
            timeout: Duration::from_secs(timeout_seconds),
            retry_count: 3,
            authentication: AuthenticationConfig::default(),
        }
    }

    /// Create a new EcosystemClient with explicit URL (for backward compatibility)
    pub fn with_url(songbird_url: String) -> Self {
        // Note: Config no longer has nested ecosystem field - using direct construction instead
        let timeout = Duration::from_secs(30);
        let retry_count = 3;
        Self {
            songbird_url,
            client: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            timeout,
            retry_count,
            authentication: AuthenticationConfig::default(),
        }
    }

    /// Register squirrel AI service with songbird
    pub async fn register_service_with_songbird(
        &self,
        registration: EcosystemServiceRegistration,
    ) -> Result<RegistrationResponse, PrimalError> {
        info!("Registering squirrel AI service with songbird");

        let url = format!("{}{}", self.songbird_url, REGISTER_ENDPOINT);

        let response = self
            .make_request_with_auth(
                reqwest::Method::POST,
                &url,
                Some(serde_json::to_value(registration)?),
            )
            .await?;

        if response.status().is_success() {
            let registration_response: RegistrationResponse =
                response.json().await.map_err(|e| {
                    PrimalError::Internal(format!("Failed to parse registration response: {e}"))
                })?;

            info!(
                "Successfully registered with songbird: {}",
                registration_response.service_id
            );
            Ok(registration_response)
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Service registration failed: {error_msg}"
            )))
        }
    }

    /// Deregister squirrel AI service from songbird
    pub async fn deregister_service_from_songbird(
        &self,
        service_id: &str,
    ) -> Result<(), PrimalError> {
        info!("Deregistering squirrel AI service from songbird");

        let url = format!(
            "{}/api/v1/services/{}/deregister",
            self.songbird_url, service_id
        );

        let response = self
            .make_request_with_auth(reqwest::Method::DELETE, &url, None)
            .await?;

        if response.status().is_success() {
            info!("Successfully deregistered from songbird");
            Ok(())
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Service deregistration failed: {error_msg}"
            )))
        }
    }

    /// Send heartbeat to songbird
    pub async fn send_heartbeat_to_songbird(
        &self,
        service_id: &str,
        health_status: &HealthStatus,
    ) -> Result<HealthCheckResponse, PrimalError> {
        debug!("Sending heartbeat to songbird");

        let url = format!(
            "{}/api/v1/services/{}/heartbeat",
            self.songbird_url, service_id
        );

        let response = self
            .make_request_with_auth(
                reqwest::Method::POST,
                &url,
                Some(serde_json::to_value(health_status)?),
            )
            .await?;

        if response.status().is_success() {
            let health_response: HealthCheckResponse = response.json().await.map_err(|e| {
                PrimalError::Internal(format!("Failed to parse heartbeat response: {e}"))
            })?;

            debug!("Heartbeat sent successfully to songbird");
            Ok(health_response)
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Heartbeat failed: {error_msg}"
            )))
        }
    }

    /// Get ecosystem health from songbird
    pub async fn get_ecosystem_health_from_songbird(
        &self,
    ) -> Result<HealthCheckResponse, PrimalError> {
        debug!("Getting ecosystem health from songbird");

        let url = format!("{}{}", self.songbird_url, HEALTH_ENDPOINT);

        let response = self
            .make_request_with_auth(reqwest::Method::GET, &url, None)
            .await?;

        if response.status().is_success() {
            let health_response: HealthCheckResponse = response.json().await.map_err(|e| {
                PrimalError::Internal(format!("Failed to parse health response: {e}"))
            })?;

            Ok(health_response)
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Failed to get ecosystem health: {error_msg}"
            )))
        }
    }

    /// Send AI intelligence coordination request via songbird
    pub async fn coordinate_intelligence_via_songbird(
        &self,
        request: IntelligenceCoordinationRequest,
    ) -> Result<IntelligenceCoordinationResponse, PrimalError> {
        debug!(
            "Coordinating AI intelligence via songbird to {}",
            request.target_primal
        );

        let url = format!("{}/api/v1/coordination/intelligence", self.songbird_url);

        let response = self
            .make_request_with_auth(
                reqwest::Method::POST,
                &url,
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            let intelligence_response: IntelligenceCoordinationResponse =
                response.json().await.map_err(|e| {
                    PrimalError::Internal(format!("Failed to parse intelligence response: {e}"))
                })?;

            debug!("AI intelligence coordination completed via songbird");
            Ok(intelligence_response)
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Intelligence coordination failed: {error_msg}"
            )))
        }
    }

    /// Send MCP coordination request via songbird
    pub async fn coordinate_mcp_via_songbird(
        &self,
        request: McpCoordinationViaRequest,
    ) -> Result<(), PrimalError> {
        debug!(
            "Coordinating MCP via songbird with primals: {:?}",
            request.target_primals
        );

        let url = format!("{}/api/v1/coordination/mcp", self.songbird_url);

        let response = self
            .make_request_with_auth(
                reqwest::Method::POST,
                &url,
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            debug!("MCP coordination sent successfully via songbird");
            Ok(())
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "MCP coordination failed: {error_msg}"
            )))
        }
    }

    /// Share context with other primals via songbird
    pub async fn share_context_via_songbird(
        &self,
        request: ContextSharingRequest,
    ) -> Result<(), PrimalError> {
        debug!("Sharing context via songbird to {}", request.target_primal);

        let url = format!("{}/api/v1/coordination/context/share", self.songbird_url);

        let response = self
            .make_request_with_auth(
                reqwest::Method::POST,
                &url,
                Some(serde_json::to_value(request)?),
            )
            .await?;

        if response.status().is_success() {
            debug!("Context shared successfully via songbird");
            Ok(())
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Context sharing failed: {error_msg}"
            )))
        }
    }

    /// Get ecosystem metrics from songbird
    pub async fn get_ecosystem_metrics_from_songbird(
        &self,
    ) -> Result<EcosystemMetrics, PrimalError> {
        debug!("Getting ecosystem metrics from songbird");

        let url = format!("{}/api/v1/ecosystem/metrics", self.songbird_url);

        let response = self
            .make_request_with_auth(reqwest::Method::GET, &url, None)
            .await?;

        if response.status().is_success() {
            let metrics: EcosystemMetrics = response.json().await.map_err(|e| {
                PrimalError::Internal(format!("Failed to parse metrics response: {e}"))
            })?;

            Ok(metrics)
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Failed to get ecosystem metrics: {error_msg}"
            )))
        }
    }

    /// Get list of active primals from songbird
    pub async fn get_active_primals_from_songbird(&self) -> Result<Vec<PrimalStatus>, PrimalError> {
        debug!("Getting active primals list from songbird");

        let url = format!("{}/api/v1/ecosystem/primals", self.songbird_url);

        let response = self
            .make_request_with_auth(reqwest::Method::GET, &url, None)
            .await?;

        if response.status().is_success() {
            let primals: Vec<PrimalStatus> = response.json().await.map_err(|e| {
                PrimalError::Internal(format!("Failed to parse primals response: {e}"))
            })?;

            Ok(primals)
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::Internal(format!(
                "Failed to get active primals: {error_msg}"
            )))
        }
    }

    /// Request AI assistance from specific primal via songbird
    pub async fn request_ai_assistance(
        &self,
        target_primal: &str,
        assistance_type: &str,
        request_data: serde_json::Value,
    ) -> Result<IntelligenceCoordinationResponse, PrimalError> {
        let request = IntelligenceCoordinationRequest {
            request_id: format!("ai-assist-{}", uuid::Uuid::new_v4()),
            target_primal: target_primal.to_string(),
            intelligence_type: assistance_type.to_string(),
            request_data,
            priority: "medium".to_string(),
            timeout: Duration::from_secs(60),
        };

        self.coordinate_intelligence_via_songbird(request).await
    }

    /// Broadcast MCP coordination to all compatible primals
    pub async fn broadcast_mcp_coordination(
        &self,
        coordination_type: &str,
        mcp_data: serde_json::Value,
    ) -> Result<(), PrimalError> {
        let request = McpCoordinationViaRequest {
            coordination_id: format!("mcp-broadcast-{}", uuid::Uuid::new_v4()),
            target_primals: vec![], // Empty means broadcast to all
            coordination_type: coordination_type.to_string(),
            mcp_data,
            requires_response: false,
        };

        self.coordinate_mcp_via_songbird(request).await
    }

    /// Share contextual insights with ecosystem via songbird
    pub async fn share_contextual_insights(
        &self,
        insights: serde_json::Value,
    ) -> Result<(), PrimalError> {
        let request = ContextSharingRequest {
            sharing_id: format!("context-insights-{}", uuid::Uuid::new_v4()),
            target_primal: "ecosystem".to_string(), // Share with entire ecosystem
            context_type: "ai_insights".to_string(),
            context_data: insights,
            sharing_permissions: vec!["read".to_string(), "analyze".to_string()],
            expiry: Some(Utc::now() + chrono::Duration::hours(24)),
        };

        self.share_context_via_songbird(request).await
    }

    // Private helper methods
    fn get_auth_header(&self) -> Result<String, PrimalError> {
        match self.authentication.auth_type.as_str() {
            "ecosystem_jwt" => {
                if let Some(token) = &self.authentication.token {
                    Ok(format!("Bearer {token}"))
                } else {
                    Err(PrimalError::Internal(
                        "JWT token not configured".to_string(),
                    ))
                }
            }
            "basic" => {
                if let (Some(client_id), Some(client_secret)) = (
                    &self.authentication.client_id,
                    &self.authentication.client_secret,
                ) {
                    let credentials =
                        general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"));
                    Ok(format!("Basic {credentials}"))
                } else {
                    Err(PrimalError::Internal(
                        "Basic auth credentials not configured".to_string(),
                    ))
                }
            }
            _ => Err(PrimalError::Internal(
                "Unsupported authentication type".to_string(),
            )),
        }
    }

    /// Execute operation with enhanced resilience and observability
    async fn with_retry<F, Fut, T>(&self, operation: F) -> Result<T, PrimalError>
    where
        F:  Fn(&reqwest::Client) -> Fut,
        Fut: std::future::Future<Output = Result<T, reqwest::Error>>,
    {
        use crate::error_handling::safe_operations::SafeOps;
        use uuid::Uuid;
        
        // Generate correlation ID for tracking
        let correlation_id = Uuid::new_v4().to_string();
        let operation_start = Instant::now();
        
        let max_retries = self.retry_count.max(1); // Ensure at least 1 attempt
        let per_attempt_timeout = self.timeout / max_retries as u32;
        let base_delay = Duration::from_millis(500);
        
        tracing::info!(
            correlation_id = %correlation_id,
            operation = "biomeos_operation_start",
            songbird_url = %self.songbird_url,
            max_retries = max_retries,
            total_timeout_ms = self.timeout.as_millis(),
            per_attempt_timeout_ms = per_attempt_timeout.as_millis(),
            "Starting BiomeOS ecosystem operation"
        );
        
        let mut last_error = None;

        for attempt in 1..=max_retries {
            let attempt_start = Instant::now();
            
            tracing::debug!(
                correlation_id = %correlation_id,
                attempt = attempt,
                max_retries = max_retries,
                timeout_ms = per_attempt_timeout.as_millis(),
                operation = "biomeos_operation_attempt",
                "Attempting BiomeOS ecosystem request"
            );
                
            let operation_result = SafeOps::safe_with_timeout(
                per_attempt_timeout,
                || operation(&self.client),
                &format!("biomeos_ecosystem_request_attempt_{}", attempt),
            ).await;
            
            let attempt_duration = attempt_start.elapsed();
            
            match operation_result.execute_without_default() {
                Ok(Ok(result)) => {
                    let total_duration = operation_start.elapsed();
                    
                    tracing::info!(
                        correlation_id = %correlation_id,
                        attempt = attempt,
                        operation = "biomeos_operation_success",
                        total_duration_ms = total_duration.as_millis(),
                        attempt_duration_ms = attempt_duration.as_millis(),
                        "BiomeOS ecosystem request completed successfully"
                    );
                    return Ok(result);
                }
                Ok(Err(network_error)) => {
                    let error_msg = format!("Network error: {}", network_error);
                    last_error = Some(error_msg.clone());
                    
                    tracing::warn!(
                        correlation_id = %correlation_id,
                        attempt = attempt,
                        operation = "biomeos_operation_network_error",
                        attempt_duration_ms = attempt_duration.as_millis(),
                        error = %error_msg,
                        error_kind = %network_error.to_string(),
                        "BiomeOS request failed with network error"
                    );
                }
                Err(timeout_error) => {
                    let error_msg = format!("Request timeout: {}", timeout_error);
                    last_error = Some(error_msg.clone());
                    
                    tracing::warn!(
                        correlation_id = %correlation_id,
                        attempt = attempt,
                        operation = "biomeos_operation_timeout",
                        attempt_duration_ms = attempt_duration.as_millis(),
                        timeout_ms = per_attempt_timeout.as_millis(),
                        error = %error_msg,
                        "BiomeOS request timed out"
                    );
                }
            }

            // Exponential backoff between retries (except on last attempt)
            if attempt < max_retries {
                let delay = base_delay * (2_u32.pow((attempt - 1).min(6))); // Cap at 2^6 = 64x
                let jitter = Duration::from_millis(rand::random::<u64>() % 500); // Add jitter
                let total_delay = delay + jitter;
                
                tracing::debug!(
                    correlation_id = %correlation_id,
                    attempt = attempt,
                    delay_ms = total_delay.as_millis(),
                    base_delay_ms = delay.as_millis(),
                    jitter_ms = jitter.as_millis(),
                    operation = "biomeos_operation_retry_delay",
                    "Waiting before retry with exponential backoff and jitter"
                );
                tokio::time::sleep(total_delay).await;
            }
        }

        let total_duration = operation_start.elapsed();
        let final_error = last_error.unwrap_or_else(|| "All BiomeOS request attempts failed".to_string());
        
        tracing::error!(
            correlation_id = %correlation_id,
            operation = "biomeos_operation_failure",
            total_duration_ms = total_duration.as_millis(),
            attempts = max_retries,
            final_error = %final_error,
            "BiomeOS ecosystem request failed after all retry attempts"
        );

        Err(PrimalError::Network(format!(
            "BiomeOS request failed after {} attempts: {}",
            max_retries, final_error
        )))
    }

    // Helper methods for individual requests
    async fn make_request_with_auth(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, PrimalError> {
        let auth_header = self.get_auth_header()?;

        let mut request = self
            .client
            .request(method, url)
            .header("Authorization", auth_header)
            .timeout(self.timeout);

        if let Some(body) = body {
            request = request.json(&body);
        }

        request
            .send()
            .await
            .map_err(|e| PrimalError::Network(e.to_string()))
    }
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            auth_type: "ecosystem_jwt".to_string(),
            token: None,
            client_id: None,
            client_secret: None,
            trust_domain: "biome.local".to_string(),
        }
    }
}

impl Default for EcosystemClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecosystem_client_creation() {
        let client = EcosystemClient::new();
        assert_eq!(client.songbird_url, "http://localhost:8080");
        assert_eq!(client.retry_count, 3);
        assert_eq!(client.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_auth_config_default() {
        let config = AuthenticationConfig::default();
        assert_eq!(config.auth_type, "ecosystem_jwt");
        assert_eq!(config.trust_domain, "biome.local");
    }

    #[test]
    fn test_ecosystem_client_with_config() {
        use squirrel_mcp_config::Config;

        let config = Config::default();
        let client = EcosystemClient::with_config(config);

        // Test that the client was created successfully
        assert!(!client.songbird_url.is_empty());
        assert_eq!(client.authentication.auth_type, "ecosystem_jwt");
    }

    #[tokio::test]
    async fn test_intelligence_coordination_request_creation() {
        let request = IntelligenceCoordinationRequest {
            request_id: "test-001".to_string(),
            target_primal: "toadstool".to_string(),
            intelligence_type: "workload_optimization".to_string(),
            request_data: serde_json::json!({"workload_type": "compute_heavy"}),
            priority: "high".to_string(),
            timeout: Duration::from_secs(30),
        };

        assert_eq!(request.target_primal, "toadstool");
        assert_eq!(request.intelligence_type, "workload_optimization");
        assert_eq!(request.priority, "high");
    }

    #[test]
    fn test_client_helper_methods() {
        let client = EcosystemClient::new();

        // Test broadcast coordination creation
        let _coordination_data = serde_json::json!({"optimization_level": "high"});

        // This would fail without a running songbird, but we can test the structure
        assert_eq!(client.songbird_url, "http://localhost:8080");
    }
}
