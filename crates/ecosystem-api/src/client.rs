// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service mesh client implementation
//!
//! **DEPRECATED**: This module provides a legacy implementation for communicating with
//! the Songbird service mesh. New code should use `UniversalAdapterV2` from the squirrel
//! crate for capability-based discovery instead of hardcoded primal clients.
//!
//! ## Migration Path
//!
//! ```rust,ignore
//! // ❌ OLD (deprecated)
//! use ecosystem_api::client::SongbirdClient;
//! let client = SongbirdClient::new("http://localhost:9090", None, Default::default())?;
//!
//! // ✅ NEW (capability-based)
//! use squirrel::universal_adapter_v2::UniversalAdapterV2;
//! let adapter = UniversalAdapterV2::awaken().await?;
//! let service_mesh = adapter.connect_capability("service_mesh").await?;
//! ```

use crate::error::{EcosystemError, UniversalError, UniversalResult};
use crate::traits::{RetryConfig, ServiceInfo, ServiceMeshClient, ServiceQuery};
use crate::types::{EcosystemServiceRegistration, HealthStatus, PrimalType, ServiceMeshStatus};
use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
// Removed unused serde imports
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// Songbird service mesh client
///
/// **DEPRECATED**: Use `UniversalAdapterV2` for capability-based discovery instead.
///
/// This client hardcodes the assumption that Songbird is the service mesh provider.
/// In the new architecture, service mesh is discovered dynamically at runtime.
#[deprecated(
    since = "0.2.0",
    note = "Use UniversalAdapterV2::connect_capability(\"service_mesh\") instead"
)]
pub struct SongbirdClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
    retry_config: RetryConfig,
    timeout: Duration,
}

// Backward compatibility: kept for deserialization of legacy data / existing consumers
#[expect(
    deprecated,
    reason = "backward compat: SongbirdClient for legacy consumers"
)]
impl SongbirdClient {
    /// Create a new Songbird client
    pub fn new(
        base_url: String,
        auth_token: Option<String>,
        retry_config: RetryConfig,
    ) -> Result<Self, EcosystemError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                EcosystemError::Configuration(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            client,
            base_url,
            auth_token,
            retry_config,
            timeout: Duration::from_secs(30),
        })
    }

    /// Set client timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build a request with authentication
    fn build_request(&self, method: reqwest::Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        if let Some(ref token) = self.auth_token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        request
            .header("Content-Type", "application/json")
            .timeout(self.timeout)
    }

    /// Execute a request with retries
    async fn execute_with_retries<F, T>(&self, request_fn: F) -> UniversalResult<T>
    where
        F: Fn() -> RequestBuilder + Send + Sync,
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let mut last_error = None;
        let mut delay = self.retry_config.initial_delay_ms;

        for attempt in 0..=self.retry_config.max_retries {
            if attempt > 0 {
                tracing::debug!(
                    "Retrying request, attempt {}/{}",
                    attempt,
                    self.retry_config.max_retries
                );
                sleep(Duration::from_millis(delay)).await;
                // Intentional: delay is in ms (u64), backoff uses f64; truncation acceptable for retry timing
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                {
                    let new_delay = (delay as f64) * self.retry_config.backoff_multiplier;
                    delay = u64::try_from(new_delay.round() as i64).unwrap_or(0);
                }
                delay = delay.min(self.retry_config.max_delay_ms);
            }

            let request = request_fn();

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.json::<T>().await {
                            Ok(result) => return Ok(result),
                            Err(e) => {
                                last_error = Some(UniversalError::Serialization(e.to_string()));
                            }
                        }
                    } else if status.is_client_error() {
                        // Don't retry client errors
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        return Err(UniversalError::InvalidRequest(format!(
                            "HTTP {status}: {error_text}"
                        )));
                    } else {
                        // Server error, retry
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        last_error = Some(UniversalError::Network(format!(
                            "HTTP {status}: {error_text}"
                        )));
                    }
                }
                Err(e) => {
                    last_error = Some(UniversalError::Network(e.to_string()));
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| UniversalError::Network("Max retries exceeded".to_string())))
    }
}

#[async_trait]
// Backward compatibility: kept for deserialization of legacy data / existing consumers
#[expect(
    deprecated,
    reason = "backward compat: SongbirdClient ServiceMeshClient impl"
)]
impl ServiceMeshClient for SongbirdClient {
    async fn register_service(
        &self,
        _endpoint: &str,
        registration: EcosystemServiceRegistration,
    ) -> UniversalResult<String> {
        let request_fn = || {
            self.build_request(reqwest::Method::POST, "/api/v1/services/register")
                .json(&registration)
        };

        let response: ServiceRegistrationResponse = self.execute_with_retries(request_fn).await?;
        Ok(response.service_id)
    }

    async fn deregister_service(&self, service_id: &str) -> UniversalResult<()> {
        let request_fn = || {
            self.build_request(
                reqwest::Method::DELETE,
                &format!("/api/v1/services/{service_id}"),
            )
        };

        let _response: EmptyResponse = self.execute_with_retries(request_fn).await?;
        Ok(())
    }

    async fn discover_services(&self, query: ServiceQuery) -> UniversalResult<Vec<ServiceInfo>> {
        let request_fn = || {
            let mut request = self.build_request(reqwest::Method::GET, "/api/v1/services/discover");

            if let Some(ref service_type) = query.service_type {
                request = request.query(&[("type", service_type)]);
            }

            if let Some(ref primal_type) = query.primal_type {
                request = request.query(&[("primal_type", primal_type.as_str())]);
            }

            if !query.capabilities.is_empty() {
                request = request.query(&[("capabilities", &query.capabilities.join(","))]);
            }

            if let Some(ref health_status) = query.health_status {
                request = request.query(&[("health", &format!("{health_status:?}"))]);
            }

            for (key, value) in &query.metadata {
                request = request.query(&[(format!("metadata.{key}"), value)]);
            }

            request
        };

        let response: ServiceDiscoveryResponse = self.execute_with_retries(request_fn).await?;
        Ok(response.services)
    }

    async fn get_service(&self, service_id: &str) -> UniversalResult<Option<ServiceInfo>> {
        let request_fn = || {
            self.build_request(
                reqwest::Method::GET,
                &format!("/api/v1/services/{service_id}"),
            )
        };

        match self.execute_with_retries(request_fn).await {
            Ok(response) => {
                let service_response: ServiceResponse = response;
                Ok(Some(service_response.service))
            }
            Err(UniversalError::InvalidRequest(msg)) if msg.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn report_health(&self, service_id: &str, health: HealthStatus) -> UniversalResult<()> {
        let health_report = HealthReport {
            service_id: service_id.to_string(),
            status: health,
            timestamp: chrono::Utc::now(),
        };

        let request_fn = || {
            self.build_request(
                reqwest::Method::POST,
                &format!("/api/v1/services/{service_id}/health"),
            )
            .json(&health_report)
        };

        let _response: EmptyResponse = self.execute_with_retries(request_fn).await?;
        Ok(())
    }

    async fn heartbeat(&self, service_id: &str) -> UniversalResult<()> {
        let heartbeat_data = HeartbeatData {
            service_id: service_id.to_string(),
            timestamp: chrono::Utc::now(),
        };

        let request_fn = || {
            self.build_request(
                reqwest::Method::POST,
                &format!("/api/v1/services/{service_id}/heartbeat"),
            )
            .json(&heartbeat_data)
        };

        let _response: EmptyResponse = self.execute_with_retries(request_fn).await?;
        Ok(())
    }

    async fn get_mesh_status(&self) -> UniversalResult<ServiceMeshStatus> {
        let request_fn = || self.build_request(reqwest::Method::GET, "/api/v1/mesh/status");

        let response: MeshStatusResponse = self.execute_with_retries(request_fn).await?;
        Ok(response.status)
    }
}

/// Mock service mesh client for testing
#[cfg(test)]
#[derive(Clone)]
pub struct MockServiceMeshClient {
    services: std::sync::Arc<tokio::sync::RwLock<HashMap<String, ServiceInfo>>>,
    health_reports: std::sync::Arc<tokio::sync::RwLock<HashMap<String, HealthStatus>>>,
}

#[cfg(test)]
impl Default for MockServiceMeshClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MockServiceMeshClient {
    /// Create a new mock client
    pub fn new() -> Self {
        Self {
            services: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            health_reports: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Add a service to the mock registry
    pub async fn add_service(&self, service_id: String, service: ServiceInfo) {
        let mut services = self.services.write().await;
        services.insert(service_id, service);
        drop(services);
    }

    /// Get all registered services
    pub async fn get_all_services(&self) -> Vec<ServiceInfo> {
        let services = self.services.read().await;
        let result = services.values().cloned().collect();
        drop(services);
        result
    }
}

#[cfg(test)]
#[async_trait]
impl ServiceMeshClient for MockServiceMeshClient {
    async fn register_service(
        &self,
        _endpoint: &str,
        registration: EcosystemServiceRegistration,
    ) -> UniversalResult<String> {
        let service_id = registration.service_id.to_string();
        let service_info = ServiceInfo {
            id: service_id.clone(),
            name: service_id.clone(),
            service_type: registration.primal_type.as_str().to_string(),
            primal_type: registration.primal_type,
            endpoint: registration.endpoints.health.clone(),
            capabilities: registration.capabilities.core.clone(),
            health_status: "healthy".to_string(),
            metadata: registration.metadata.clone(),
        };

        self.add_service(service_id.clone(), service_info).await;
        Ok(service_id)
    }

    async fn deregister_service(&self, service_id: &str) -> UniversalResult<()> {
        let mut services = self.services.write().await;
        services.remove(service_id);
        drop(services);
        Ok(())
    }

    async fn discover_services(&self, query: ServiceQuery) -> UniversalResult<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        let mut results = Vec::new();

        for service in services.values() {
            let matches = query
                .service_type
                .as_ref()
                .is_none_or(|st| service.service_type == *st)
                && query.primal_type.is_none_or(|pt| service.primal_type == pt)
                && (query.capabilities.is_empty()
                    || query
                        .capabilities
                        .iter()
                        .all(|c| service.capabilities.contains(c)));

            if matches {
                results.push(service.clone());
            }
        }

        drop(services);
        Ok(results)
    }

    async fn get_service(&self, service_id: &str) -> UniversalResult<Option<ServiceInfo>> {
        let services = self.services.read().await;
        let result = services.get(service_id).cloned();
        drop(services);
        Ok(result)
    }

    async fn report_health(&self, service_id: &str, health: HealthStatus) -> UniversalResult<()> {
        let mut health_reports = self.health_reports.write().await;
        health_reports.insert(service_id.to_string(), health);
        drop(health_reports);
        Ok(())
    }

    async fn heartbeat(&self, service_id: &str) -> UniversalResult<()> {
        // Mock heartbeat - just record the time
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.metadata.insert(
                "last_heartbeat".to_string(),
                chrono::Utc::now().to_rfc3339(),
            );
        }
        drop(services);
        Ok(())
    }

    async fn get_mesh_status(&self) -> UniversalResult<ServiceMeshStatus> {
        Ok(ServiceMeshStatus {
            connected: true,
            service_mesh_endpoint: None, // endpoints discovered at runtime
            registration_time: Some(chrono::Utc::now()),
            last_heartbeat: Some(chrono::Utc::now()),
            metadata: HashMap::new(),
        })
    }
}

// Response types for service mesh API
#[derive(serde::Deserialize)]
struct ServiceRegistrationResponse {
    service_id: String,
}

#[derive(serde::Deserialize)]
struct ServiceDiscoveryResponse {
    services: Vec<ServiceInfo>,
}

#[derive(serde::Deserialize)]
struct ServiceResponse {
    service: ServiceInfo,
}

#[derive(serde::Deserialize)]
struct MeshStatusResponse {
    status: ServiceMeshStatus,
}

#[derive(serde::Deserialize)]
struct EmptyResponse {}

#[derive(serde::Serialize)]
struct HealthReport {
    service_id: String,
    status: HealthStatus,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize)]
struct HeartbeatData {
    service_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Service mesh client factory
pub struct ServiceMeshClientFactory;

impl ServiceMeshClientFactory {
    /// Create a new service mesh client
    // Backward compatibility: kept for deserialization of legacy data / existing consumers
    #[expect(
        deprecated,
        reason = "backward compat: ServiceMeshClientFactory for legacy consumers"
    )]
    pub fn create_client(
        base_url: String,
        auth_token: Option<String>,
        retry_config: RetryConfig,
    ) -> Result<impl ServiceMeshClient, EcosystemError> {
        SongbirdClient::new(base_url, auth_token, retry_config)
    }

    /// Create a mock client for testing
    #[cfg(test)]
    pub fn create_mock_client() -> impl ServiceMeshClient {
        MockServiceMeshClient::new()
    }
}

/// Health monitor for tracking service health
pub struct HealthMonitor {
    client: Box<dyn ServiceMeshClient + Send + Sync>,
    service_id: String,
    interval: Duration,
}

impl HealthMonitor {
    /// Create a new health monitor
    #[must_use]
    pub fn new(
        client: Box<dyn ServiceMeshClient + Send + Sync>,
        service_id: String,
        interval: Duration,
    ) -> Self {
        Self {
            client,
            service_id,
            interval,
        }
    }

    /// Start health monitoring
    pub async fn start_monitoring(&self) -> UniversalResult<()> {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;

            match self.client.heartbeat(&self.service_id).await {
                Ok(()) => {
                    tracing::debug!(
                        "Heartbeat sent successfully for service: {}",
                        self.service_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to send heartbeat for service {}: {}",
                        self.service_id,
                        e
                    );
                }
            }
        }
    }

    /// Report health status
    pub async fn report_health(&self, health: HealthStatus) -> UniversalResult<()> {
        self.client.report_health(&self.service_id, health).await
    }
}

/// Service discovery helper
pub struct ServiceDiscovery {
    client: Box<dyn ServiceMeshClient + Send + Sync>,
}

impl ServiceDiscovery {
    /// Create a new service discovery helper
    #[must_use]
    pub fn new(client: Box<dyn ServiceMeshClient + Send + Sync>) -> Self {
        Self { client }
    }

    /// Find services by primal type
    pub async fn find_by_primal_type(
        &self,
        primal_type: PrimalType,
    ) -> UniversalResult<Vec<ServiceInfo>> {
        let query = ServiceQuery {
            primal_type: Some(primal_type),
            ..Default::default()
        };

        self.client.discover_services(query).await
    }

    /// Find services by capability
    pub async fn find_by_capability(&self, capability: &str) -> UniversalResult<Vec<ServiceInfo>> {
        let query = ServiceQuery {
            capabilities: vec![capability.to_string()],
            ..Default::default()
        };

        self.client.discover_services(query).await
    }

    /// Find healthy services
    pub async fn find_healthy_services(&self) -> UniversalResult<Vec<ServiceInfo>> {
        let query = ServiceQuery {
            health_status: Some(HealthStatus::Healthy),
            ..Default::default()
        };

        self.client.discover_services(query).await
    }

    /// Find services by metadata filter
    pub async fn find_by_metadata(
        &self,
        metadata: HashMap<String, String>,
    ) -> UniversalResult<Vec<ServiceInfo>> {
        let query = ServiceQuery {
            metadata,
            ..Default::default()
        };

        self.client.discover_services(query).await
    }
}

#[cfg(all(test, feature = "http-api"))]
mod tests {
    use super::*;
    use std::time::Duration;

    #[allow(deprecated)]
    #[test]
    fn test_songbird_client_new() {
        let result = SongbirdClient::new(
            "http://localhost:9999".to_string(),
            None,
            RetryConfig::default(),
        );
        assert!(result.is_ok());
        let client = result.unwrap();
        let _with_timeout = client.with_timeout(Duration::from_secs(10));
    }

    #[allow(deprecated)]
    #[test]
    fn test_songbird_client_with_auth() {
        let result = SongbirdClient::new(
            "http://localhost:9999".to_string(),
            Some("test-token".to_string()),
            RetryConfig::default(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_service_mesh_client_factory_create_client() {
        let result = ServiceMeshClientFactory::create_client(
            "http://localhost:9999".to_string(),
            None,
            RetryConfig::default(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_health_monitor_new() {
        let mock = MockServiceMeshClient::new();
        let client: Box<dyn ServiceMeshClient + Send + Sync> = Box::new(mock);
        let _monitor =
            HealthMonitor::new(client, "test-service".to_string(), Duration::from_secs(30));
    }

    #[test]
    fn test_service_discovery_new() {
        let mock = MockServiceMeshClient::new();
        let client: Box<dyn ServiceMeshClient + Send + Sync> = Box::new(mock);
        let _discovery = ServiceDiscovery::new(client);
    }
}
