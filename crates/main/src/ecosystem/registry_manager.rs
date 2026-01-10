// # Ecosystem Registry Manager
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info};

use super::registry::config::EcosystemRegistryConfig;
use super::registry::discovery::DiscoveryOps;
// Remove HealthCheckResult import
// Keep only this direct import
use super::registry::types::{intern_registry_string, DiscoveredService, EcosystemRegistryEvent}; // Import HealthCheckResult directly
use crate::error::PrimalError;
use crate::EcosystemPrimalType; // Import from crate root // Use the existing one

/// Service registry manager with Arc<str> optimization
pub struct EcosystemRegistryManager {
    /// Service registry with Arc<str> keys
    pub service_registry: Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
    pub event_publisher: broadcast::Sender<EcosystemRegistryEvent>,
    pub http_client: reqwest::Client,
    pub config: EcosystemRegistryConfig,
}

impl EcosystemRegistryManager {
    /// Create new registry manager
    #[must_use]
    pub fn new(
        config: EcosystemRegistryConfig,
    ) -> (Self, broadcast::Receiver<EcosystemRegistryEvent>) {
        let (event_publisher, event_receiver) = broadcast::channel(1000);

        let manager = Self {
            service_registry: Arc::new(RwLock::new(HashMap::new())),
            event_publisher,
            http_client: reqwest::Client::new(),
            config,
        };

        (manager, event_receiver)
    }

    /// Initialize the registry manager
    pub async fn initialize(&self) -> Result<(), PrimalError> {
        info!("Initializing ecosystem registry manager with Arc<str> optimization");

        // Start discovery process
        self.start_discovery().await?;

        info!("✅ Ecosystem registry manager initialized successfully");
        Ok(())
    }

    /// Start discovery process with proper error handling
    pub async fn start_discovery(&self) -> Result<(), PrimalError> {
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
        ];

        // Use the modernized discovery service with Arc<str> registry
        match DiscoveryOps::discover_services(&self.service_registry, primal_types).await {
            Ok(discovered) => {
                // Publish discovery events with capabilities
                for service in discovered {
                    let event = EcosystemRegistryEvent::ServiceDiscovered {
                        service_id: service.service_id.clone(),
                        primal_type: service.primal_type,
                        endpoint: service.endpoint.clone(),
                        capabilities: service.capabilities.clone(), // Include capabilities
                    };
                    let _ = self.event_publisher.send(event);
                }
                Ok(())
            }
            Err(e) => {
                error!("Discovery failed: {}", e);
                Err(PrimalError::ServiceDiscoveryFailed(format!(
                    "Discovery error: {e}"
                )))
            }
        }
    }

    /// Get all services with Arc<str> optimization (renamed method)
    pub async fn get_discovered_services(&self) -> Vec<Arc<DiscoveredService>> {
        self.get_services().await
    }

    /// Get all services with Arc<str> optimization
    pub async fn get_services(&self) -> Vec<Arc<DiscoveredService>> {
        let service_registry = self.service_registry.read().await;
        service_registry.values().cloned().collect() // Return Arc<DiscoveredService>
    }

    /// Get active integrations (service IDs as strings)
    pub async fn get_active_integrations(&self) -> Vec<String> {
        self.get_service_ids().await
    }

    /// Get service IDs with Arc<str> to String conversion
    pub async fn get_service_ids(&self) -> Vec<String> {
        let service_registry = self.service_registry.read().await;
        service_registry
            .keys()
            .map(std::string::ToString::to_string)
            .collect() // Convert Arc<str> to String
    }

    /// Register squirrel service with the ecosystem
    pub async fn register_squirrel_service(
        &self,
        registration: crate::ecosystem::EcosystemServiceRegistration,
    ) -> Result<(), PrimalError> {
        let service = Arc::new(DiscoveredService {
            service_id: intern_registry_string(&registration.service_id),
            primal_type: registration.primal_type,
            endpoint: Arc::from(registration.endpoints.primary.as_str()), // Use endpoints.primary
            capabilities: registration
                .capabilities
                .core
                .iter()
                .chain(registration.capabilities.extended.iter())
                .chain(registration.capabilities.integrations.iter())
                .map(|cap| intern_registry_string(cap))
                .collect(),
            health_status: crate::ecosystem::registry::types::ServiceHealthStatus::Healthy,
            health_endpoint: Arc::from(format!("{}/health", registration.endpoints.primary)), // Use endpoints.primary
            discovered_at: chrono::Utc::now(),
            api_version: Arc::from("v1"),
            last_health_check: Some(chrono::Utc::now()),
            metadata: std::collections::HashMap::new(),
        });

        // Insert with Arc<str> key
        let mut registry = self.service_registry.write().await;
        registry.insert(service.service_id.clone(), service);

        info!("✅ Squirrel service registered successfully");
        Ok(())
    }

    /// Register with Songbird service mesh
    pub async fn register_with_songbird(
        &self,
        registration: crate::ecosystem::EcosystemServiceRegistration,
    ) -> Result<(), PrimalError> {
        let url = format!("{}/api/v1/services/register", self.config.songbird_endpoint);

        let response = self
            .http_client
            .post(&url)
            .json(&registration)
            .send()
            .await
            .map_err(|e| {
                PrimalError::NetworkError(format!("Failed to register with Songbird: {e}"))
            })?;

        if response.status().is_success() {
            info!("✅ Successfully registered with Songbird");
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::OperationFailed(format!(
                "Songbird registration failed: {error_text}"
            )))
        }
    }

    /// Deregister from Songbird service mesh
    pub async fn deregister_from_songbird(&self, service_id: &str) -> Result<(), PrimalError> {
        let url = format!(
            "{}/api/v1/services/deregister/{}",
            self.config.songbird_endpoint, service_id
        );

        let response = self.http_client.delete(&url).send().await.map_err(|e| {
            PrimalError::NetworkError(format!("Failed to deregister from Songbird: {e}"))
        })?;

        if response.status().is_success() {
            info!("✅ Successfully deregistered from Songbird");
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::OperationFailed(format!(
                "Songbird deregistration failed: {error_text}"
            )))
        }
    }

    /// Check if service is healthy
    pub async fn is_service_healthy(&self, _service_id: &str) -> Result<bool, PrimalError> {
        // For now, assume healthy - would integrate with actual health monitoring
        Ok(true)
    }

    /// Get services by primal type with Arc<str> compatibility (renamed method)
    pub async fn find_services_by_type(
        &self,
        primal_type: EcosystemPrimalType,
    ) -> Result<Vec<Arc<DiscoveredService>>, PrimalError> {
        Ok(self.get_services_by_type(primal_type).await)
    }

    /// Get services by primal type with Arc<str> compatibility
    pub async fn get_services_by_type(
        &self,
        primal_type: EcosystemPrimalType,
    ) -> Vec<Arc<DiscoveredService>> {
        let service_registry = self.service_registry.read().await;
        service_registry
            .values()
            .filter(|service| service.primal_type == primal_type)
            .cloned()
            .collect()
    }

    /// Call primal API with comprehensive resilience and observability
    pub async fn call_primal_api(
        &self,
        request: crate::ecosystem::PrimalApiRequest,
    ) -> Result<crate::ecosystem::PrimalApiResponse, PrimalError> {
        use crate::error_handling::safe_operations::SafeOps;
        use std::time::Duration;
        use uuid::Uuid;

        // Generate correlation ID for request tracking
        let correlation_id = Uuid::new_v4().to_string();
        let operation_start = std::time::Instant::now();

        let service_registry = self.service_registry.read().await;
        let service = service_registry
            .values()
            .find(|s| s.primal_type == request.to_primal)
            .ok_or_else(|| {
                tracing::error!(
                    correlation_id = %correlation_id,
                    request_id = %request.request_id,
                    target_primal = ?request.to_primal,
                    operation = "primal_api_service_not_found",
                    "Service not found for primal type"
                );
                PrimalError::ResourceNotFound(format!(
                    "Service not found for primal type: {:?}",
                    request.to_primal
                ))
            })?;

        let url = format!("{}/api/v1/{}", service.endpoint, request.operation);

        tracing::info!(
            correlation_id = %correlation_id,
            request_id = %request.request_id,
            target_primal = ?request.to_primal,
            service_endpoint = %service.endpoint,
            api_operation = %request.operation,
            url = %url,
            timeout_ms = request.timeout.as_millis(),
            operation = "primal_api_call_start",
            "Starting primal API call"
        );

        // Configuration for resilient API calls
        let max_retries = 3;
        let base_delay = Duration::from_millis(1000);
        let timeout = request.timeout.min(Duration::from_secs(30)); // Cap at 30s

        let mut last_error = None;

        // Retry loop with exponential backoff
        for attempt in 1..=max_retries {
            let attempt_start = std::time::Instant::now();
            let client_timeout = timeout / max_retries; // Distribute timeout across attempts

            tracing::debug!(
                correlation_id = %correlation_id,
                request_id = %request.request_id,
                attempt = attempt,
                max_retries = max_retries,
                timeout_ms = client_timeout.as_millis(),
                url = %url,
                operation = "primal_api_call_attempt",
                "Attempting primal API call"
            );

            let api_call_result = SafeOps::safe_with_timeout(
                client_timeout,
                || async {
                    self.http_client
                        .post(&url)
                        .json(&request.payload)
                        .send()
                        .await
                },
                &format!("primal_api_call_attempt_{attempt}"),
            )
            .await;

            let attempt_duration = attempt_start.elapsed();

            match api_call_result.execute_without_default() {
                Ok(Ok(response)) => {
                    let status_code = response.status().as_u16();

                    if response.status().is_success() {
                        // Success path - parse response safely
                        let parse_start = std::time::Instant::now();
                        let response_result = SafeOps::safe_with_timeout(
                            Duration::from_secs(5), // JSON parsing timeout
                            || response.json::<serde_json::Value>(),
                            "json_response_parsing",
                        )
                        .await;

                        let parse_duration = parse_start.elapsed();
                        let total_duration = operation_start.elapsed();
                        let data = response_result
                            .execute_without_default()
                            .ok()
                            .and_then(std::result::Result::ok);

                        tracing::info!(
                            correlation_id = %correlation_id,
                            request_id = %request.request_id,
                            attempt = attempt,
                            operation = "primal_api_call_success",
                            total_duration_ms = total_duration.as_millis(),
                            attempt_duration_ms = attempt_duration.as_millis(),
                            parse_duration_ms = parse_duration.as_millis(),
                            http_status = status_code,
                            has_data = data.is_some(),
                            "Primal API call completed successfully"
                        );

                        return Ok(crate::ecosystem::PrimalApiResponse {
                            request_id: request.request_id,
                            success: true,
                            data,
                            error: None,
                            headers: std::collections::HashMap::new(),
                            processing_time: total_duration,
                        });
                    } else {
                        // HTTP error - get error text safely
                        let error_text = SafeOps::safe_with_timeout(
                            Duration::from_secs(2),
                            || response.text(),
                            "error_response_parsing",
                        )
                        .await
                        .execute_without_default()
                        .and_then(|r| {
                            r.map_err(|e| {
                                crate::error_handling::safe_operations::SafeError::Network {
                                    message: format!("Request error: {e}"),
                                    endpoint: None,
                                }
                            })
                        })
                        .unwrap_or_else(|_| "Unknown HTTP error".to_string());

                        let error_msg = format!("HTTP {status_code} - {error_text}");
                        last_error = Some(error_msg.clone());

                        tracing::warn!(
                            correlation_id = %correlation_id,
                            request_id = %request.request_id,
                            attempt = attempt,
                            operation = "primal_api_call_http_error",
                            attempt_duration_ms = attempt_duration.as_millis(),
                            http_status = status_code,
                            error_text = %error_text,
                            "Primal API call failed with HTTP error"
                        );
                    }
                }
                Ok(Err(e)) => {
                    let error_msg = format!("Network error: {e}");
                    last_error = Some(error_msg.clone());

                    tracing::warn!(
                        correlation_id = %correlation_id,
                        request_id = %request.request_id,
                        attempt = attempt,
                        operation = "primal_api_call_network_error",
                        attempt_duration_ms = attempt_duration.as_millis(),
                        error = %error_msg,
                        "Primal API call failed with network error"
                    );
                }
                Err(timeout_err) => {
                    let error_msg = format!("Timeout error: {timeout_err}");
                    last_error = Some(error_msg.clone());

                    tracing::warn!(
                        correlation_id = %correlation_id,
                        request_id = %request.request_id,
                        attempt = attempt,
                        operation = "primal_api_call_timeout",
                        attempt_duration_ms = attempt_duration.as_millis(),
                        timeout_ms = client_timeout.as_millis(),
                        error = %error_msg,
                        "Primal API call timed out"
                    );
                }
            }

            // Exponential backoff between retries (except on last attempt)
            if attempt < max_retries {
                let delay = base_delay * (2_u32.pow(attempt - 1));
                tracing::debug!(
                    correlation_id = %correlation_id,
                    request_id = %request.request_id,
                    attempt = attempt,
                    delay_ms = delay.as_millis(),
                    operation = "primal_api_call_retry_delay",
                    "Waiting before retry"
                );
                tokio::time::sleep(delay).await;
            }
        }

        let total_duration = operation_start.elapsed();
        let final_error = last_error.unwrap_or_else(|| "All retry attempts failed".to_string());

        tracing::error!(
            correlation_id = %correlation_id,
            request_id = %request.request_id,
            operation = "primal_api_call_failure",
            total_duration_ms = total_duration.as_millis(),
            attempts = max_retries,
            final_error = %final_error,
            "Primal API call failed after all retry attempts"
        );

        // Return failed response instead of error to allow graceful handling
        Ok(crate::ecosystem::PrimalApiResponse {
            request_id: request.request_id,
            success: false,
            data: None,
            error: Some(final_error.into()),
            headers: std::collections::HashMap::new(),
            processing_time: total_duration,
        })
    }

    /// Start coordination between services
    pub async fn start_coordination(
        &self,
        participants: Vec<String>,
        _context: serde_json::Value,
    ) -> Result<String, PrimalError> {
        let coordination_id = uuid::Uuid::new_v4().to_string();
        info!(
            "🚀 Starting coordination {} with {} participants",
            coordination_id,
            participants.len()
        );
        Ok(coordination_id)
    }

    /// Complete coordination between services
    pub async fn complete_coordination(
        &self,
        session_id: String,
        success: bool,
    ) -> Result<(), PrimalError> {
        info!(
            "✅ Completing coordination {} with success: {}",
            session_id, success
        );
        Ok(())
    }

    /// Shutdown the registry manager
    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        info!("🔄 Shutting down ecosystem registry manager");

        // Deregister all services
        let service_ids = self.get_service_ids().await;
        for service_id in service_ids {
            let _ = self.deregister_from_songbird(&service_id).await;
        }

        info!("✅ Ecosystem registry manager shutdown completed");
        Ok(())
    }

    /// Handle health check errors with Arc<str> optimization
    async fn handle_health_check_error(&self, service_id: Arc<str>, error_text: String) {
        let event = EcosystemRegistryEvent::ServiceError {
            service_id: service_id.clone(),
            error: intern_registry_string(&error_text), // Convert String to Arc<str>
            timestamp: chrono::Utc::now(),
        };
        let _ = self.event_publisher.send(event);
    }
}
