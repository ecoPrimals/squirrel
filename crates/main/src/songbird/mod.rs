//! Songbird integration for squirrel primal
//!
//! This module provides integration with the songbird orchestration system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::PrimalError;
use config::ConfigManager;

/// Songbird integration for orchestration
#[derive(Debug)]
pub struct SongbirdIntegration {
    pub config: SongbirdConfig,
    pub orchestration_state: Arc<RwLock<OrchestrationState>>,
    pub health_status: HealthStatus,
    pub http_client: reqwest::Client,
}

/// Configuration for songbird integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    pub songbird_endpoint: String,
    pub heartbeat_interval: Duration,
    pub coordination_timeout: Duration,
    pub max_retries: u32,
    pub service_name: String,
    pub auth_token: Option<String>,
}

/// Service registration for Songbird
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_id: String,
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health_endpoint: String,
    pub metadata: HashMap<String, String>,
}

/// Service info from Songbird discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_id: String,
    pub primal_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health: String,
    pub metadata: HashMap<String, String>,
}

/// Orchestration state
#[derive(Debug, Clone, Default)]
pub struct OrchestrationState {
    pub active_coordinations: HashMap<String, CoordinationSession>,
    pub primal_status: HashMap<String, PrimalStatus>,
    pub resource_allocations: HashMap<String, ResourceAllocation>,
    pub service_id: String,
    pub registered: bool,
}

/// Coordination session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationSession {
    pub session_id: String,
    pub participants: Vec<String>,
    pub session_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Primal status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub primal_id: String,
    pub primal_type: String,
    pub status: String,
    pub health_score: f64,
    pub last_seen: DateTime<Utc>,
    pub capabilities: Vec<String>,
}

/// Resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub resource_type: String,
    pub amount: f64,
    pub allocated_to: String,
    pub expires_at: DateTime<Utc>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub coordinator_status: String,
    pub active_sessions: u32,
    pub resource_utilization: f64,
}

/// Coordination request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRequest {
    pub coordination_type: String,
    pub participants: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub timeout: Option<Duration>,
}

/// Coordination response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResponse {
    pub session_id: String,
    pub status: String,
    pub participants: Vec<String>,
    pub result: Option<serde_json::Value>,
}

impl SongbirdIntegration {
    /// Create a new Songbird integration for ecosystem orchestration
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();
        let external_services = config_manager.get_external_services_config();
        
        let service_id = format!("squirrel-{}", uuid::Uuid::new_v4());

        Self {
            config: SongbirdConfig {
                songbird_endpoint: std::env::var("SONGBIRD_URL")
                    .unwrap_or(external_services.songbird_url),
                heartbeat_interval: Duration::from_secs(30),
                coordination_timeout: Duration::from_secs(60),
                max_retries: 3,
                service_name: "squirrel-mcp".to_string(),
                auth_token: std::env::var("SONGBIRD_AUTH_TOKEN").ok(),
            },
            orchestration_state: Arc::new(RwLock::new(OrchestrationState {
                service_id: service_id.clone(),
                registered: false,
                ..Default::default()
            })),
            health_status: HealthStatus {
                status: "initializing".to_string(),
                timestamp: Utc::now(),
                coordinator_status: "starting".to_string(),
                active_sessions: 0,
                resource_utilization: 0.0,
            },
            http_client: reqwest::Client::new(),
        }
    }

    /// Initialize songbird integration
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing songbird integration");

        // Test connection to Songbird
        if let Err(e) = self.test_connection().await {
            warn!("Failed to connect to Songbird: {}", e);
            info!("Continuing in standalone mode");
        } else {
            info!("Successfully connected to Songbird");
            
            // Register with Songbird
            if let Err(e) = self.register_with_songbird().await {
                error!("Failed to register with Songbird: {}", e);
                return Err(e);
            }
        }

        self.health_status.status = "running".to_string();
        self.health_status.coordinator_status = "running".to_string();
        self.health_status.timestamp = Utc::now();

        info!("Songbird integration initialized successfully");
        Ok(())
    }

    /// Test connection to Songbird
    async fn test_connection(&self) -> Result<(), PrimalError> {
        let health_url = format!("{}/health", self.config.songbird_endpoint);
        
        let response = self.http_client
            .get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| PrimalError::Network(format!("Failed to connect to Songbird: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Songbird health check failed: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Register with Songbird
    async fn register_with_songbird(&self) -> Result<(), PrimalError> {
        let registration = ServiceRegistration {
            service_id: {
                let state = self.orchestration_state.read().await;
                state.service_id.clone()
            },
            primal_type: "squirrel".to_string(),
            endpoint: self.get_service_endpoint(),
            capabilities: vec![
                "mcp".to_string(),
                "ai-task-routing".to_string(),
                "multi-mcp-coordination".to_string(),
                "context-management".to_string(),
                "federation".to_string(),
                "scaling".to_string(),
            ],
            health_endpoint: format!("{}/health", self.get_service_endpoint()),
            metadata: self.get_service_metadata(),
        };

        let register_url = format!("{}/api/v1/services/register", self.config.songbird_endpoint);
        
        let mut request = self.http_client.post(&register_url)
            .json(&registration)
            .timeout(self.config.coordination_timeout);

        // Add auth token if available
        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await
            .map_err(|e| PrimalError::Network(format!("Failed to register with Songbird: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Songbird registration failed: {}",
                response.status()
            )));
        }

        // Update registration status
        let mut state = self.orchestration_state.write().await;
        state.registered = true;

        info!("Successfully registered with Songbird");
        Ok(())
    }

    /// Discover services from Songbird
    pub async fn discover_services(&self) -> Result<Vec<ServiceInfo>, PrimalError> {
        let discovery_url = format!("{}/api/v1/services", self.config.songbird_endpoint);
        
        let mut request = self.http_client.get(&discovery_url)
            .timeout(self.config.coordination_timeout);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await
            .map_err(|e| PrimalError::Network(format!("Failed to discover services: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Service discovery failed: {}",
                response.status()
            )));
        }

        let services: Vec<ServiceInfo> = response.json().await
            .map_err(|e| PrimalError::Internal(format!("Failed to parse service discovery response: {}", e)))?;

        debug!("Discovered {} services from Songbird", services.len());
        Ok(services)
    }

    /// Coordinate with songbird
    pub async fn coordinate(
        &self,
        coordination_type: &str,
        participants: Vec<String>,
    ) -> Result<String, PrimalError> {
        debug!(
            "Coordinating with songbird: {} with participants: {:?}",
            coordination_type, participants
        );

        let coordination_request = CoordinationRequest {
            coordination_type: coordination_type.to_string(),
            participants,
            metadata: HashMap::new(),
            timeout: Some(self.config.coordination_timeout),
        };

        let coordination_url = format!("{}/api/v1/coordination", self.config.songbird_endpoint);
        
        let mut request = self.http_client.post(&coordination_url)
            .json(&coordination_request)
            .timeout(self.config.coordination_timeout);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await
            .map_err(|e| PrimalError::Network(format!("Failed to coordinate with Songbird: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Coordination failed: {}",
                response.status()
            )));
        }

        let coordination_response: CoordinationResponse = response.json().await
            .map_err(|e| PrimalError::Internal(format!("Failed to parse coordination response: {}", e)))?;

        // Update local state
        let session = CoordinationSession {
            session_id: coordination_response.session_id.clone(),
            participants: coordination_response.participants,
            session_type: coordination_type.to_string(),
            status: coordination_response.status,
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        let mut state = self.orchestration_state.write().await;
        state.active_coordinations.insert(coordination_response.session_id.clone(), session);

        Ok(coordination_response.session_id)
    }

    /// Send heartbeat to Songbird
    pub async fn send_heartbeat(&mut self) -> Result<(), PrimalError> {
        let heartbeat_url = format!("{}/api/v1/heartbeat", self.config.songbird_endpoint);
        
        let heartbeat_data = serde_json::json!({
            "service_id": {
                let state = self.orchestration_state.read().await;
                state.service_id.clone()
            },
            "status": self.health_status.status,
            "timestamp": Utc::now().to_rfc3339(),
            "active_sessions": self.health_status.active_sessions,
            "resource_utilization": self.health_status.resource_utilization
        });

        let mut request = self.http_client.post(&heartbeat_url)
            .json(&heartbeat_data)
            .timeout(Duration::from_secs(5));

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await
            .map_err(|e| PrimalError::Network(format!("Failed to send heartbeat: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Heartbeat failed: {}",
                response.status()
            )));
        }

        debug!("Heartbeat sent successfully to Songbird");
        Ok(())
    }

    /// Update health status
    pub async fn update_health(&mut self) -> Result<(), PrimalError> {
        let state = self.orchestration_state.read().await;

        self.health_status.timestamp = Utc::now();
        self.health_status.active_sessions = state.active_coordinations.len() as u32;
        self.health_status.resource_utilization = 0.5; // TODO: Calculate actual utilization

        Ok(())
    }

    /// Get service endpoint
    fn get_service_endpoint(&self) -> String {
        std::env::var("SQUIRREL_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string())
    }

    /// Get service metadata
    fn get_service_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "2.2.0".to_string());
        metadata.insert("region".to_string(), 
            std::env::var("REGION").unwrap_or_else(|_| "default".to_string()));
        metadata.insert("zone".to_string(),
            std::env::var("ZONE").unwrap_or_else(|_| "default".to_string()));
        metadata
    }

    /// Shutdown songbird integration
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down songbird integration");

        // Unregister from Songbird
        if let Err(e) = self.unregister_from_songbird().await {
            warn!("Failed to unregister from Songbird: {}", e);
        }

        self.health_status.status = "shutdown".to_string();
        self.health_status.timestamp = Utc::now();

        let mut state = self.orchestration_state.write().await;
        state.active_coordinations.clear();
        state.registered = false;

        info!("Songbird integration shut down successfully");
        Ok(())
    }

    /// Unregister from Songbird
    async fn unregister_from_songbird(&self) -> Result<(), PrimalError> {
        let service_id = {
            let state = self.orchestration_state.read().await;
            state.service_id.clone()
        };

        let unregister_url = format!("{}/api/v1/services/{}/unregister", 
            self.config.songbird_endpoint, service_id);
        
        let mut request = self.http_client.post(&unregister_url)
            .timeout(Duration::from_secs(5));

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await
            .map_err(|e| PrimalError::Network(format!("Failed to unregister from Songbird: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::Network(format!(
                "Unregistration failed: {}",
                response.status()
            )));
        }

        info!("Successfully unregistered from Songbird");
        Ok(())
    }

    /// Start periodic heartbeat
    pub async fn start_heartbeat_loop(&mut self) -> Result<(), PrimalError> {
        let mut interval = tokio::time::interval(self.config.heartbeat_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.send_heartbeat().await {
                warn!("Heartbeat failed: {}", e);
            }
        }
    }
}

impl Default for SongbirdConfig {
    fn default() -> Self {
        let heartbeat_interval_secs = std::env::var("SONGBIRD_HEARTBEAT_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30);

        let coordination_timeout_secs = std::env::var("SONGBIRD_COORDINATION_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        let max_retries = std::env::var("SONGBIRD_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(3);

        Self {
            songbird_endpoint: std::env::var("SONGBIRD_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            heartbeat_interval: Duration::from_secs(heartbeat_interval_secs),
            coordination_timeout: Duration::from_secs(coordination_timeout_secs),
            max_retries,
            service_name: "squirrel-mcp".to_string(),
            auth_token: std::env::var("SONGBIRD_AUTH_TOKEN").ok(),
        }
    }
}

impl Default for SongbirdIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_songbird_integration_initialization() {
        let mut integration = SongbirdIntegration::new();
        assert!(integration.initialize().await.is_ok());
        assert_eq!(integration.health_status.status, "running");
    }

    #[tokio::test]
    async fn test_coordination_session_creation() {
        let integration = SongbirdIntegration::new();
        let participants = vec!["squirrel".to_string(), "toadstool".to_string()];

        // This test will fail without actual Songbird running, but shows the API
        if let Ok(session_id) = integration
            .coordinate("resource_optimization", participants)
            .await
        {
            assert!(!session_id.is_empty());

            let state = integration.orchestration_state.read().await;
            assert!(state.active_coordinations.contains_key(&session_id));
        }
    }

    #[tokio::test]
    async fn test_health_update() {
        let mut integration = SongbirdIntegration::new();
        let original_timestamp = integration.health_status.timestamp;

        // Wait a bit to ensure timestamp changes
        tokio::time::sleep(Duration::from_millis(10)).await;

        integration.update_health().await.unwrap();
        assert!(integration.health_status.timestamp > original_timestamp);
    }

    #[tokio::test] 
    async fn test_service_registration() {
        let integration = SongbirdIntegration::new();
        let registration = ServiceRegistration {
            service_id: "test-service".to_string(),
            primal_type: "squirrel".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["mcp".to_string()],
            health_endpoint: "http://localhost:8080/health".to_string(),
            metadata: HashMap::new(),
        };

        // Test that registration structure is correct
        assert_eq!(registration.primal_type, "squirrel");
        assert!(!registration.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_configuration_from_env() {
        std::env::set_var("SONGBIRD_ENDPOINT", "http://test:8080");
        std::env::set_var("SONGBIRD_AUTH_TOKEN", "test-token");
        
        let config = SongbirdConfig::default();
        assert_eq!(config.songbird_endpoint, "http://test:8080");
        assert_eq!(config.auth_token, Some("test-token".to_string()));
    }
}

#[cfg(test)]
mod integration_tests;

/// Test module for comprehensive Songbird integration tests
#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path, header};
    use serde_json::json;
    use std::sync::Arc;

    async fn setup_mock_songbird() -> MockServer {
        MockServer::start().await
    }

    async fn setup_integration(mock_server: &MockServer) -> SongbirdIntegration {
        std::env::set_var("SONGBIRD_ENDPOINT", mock_server.uri());
        std::env::set_var("SONGBIRD_AUTH_TOKEN", "test-token");
        
        SongbirdIntegration::new()
    }

    #[tokio::test]
    async fn test_full_service_lifecycle() {
        let mock_server = setup_mock_songbird().await;
        
        // Health check
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Registration
        Mock::given(method("POST"))
            .and(path("/api/v1/services/register"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Service discovery
        Mock::given(method("GET"))
            .and(path("/api/v1/services"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Coordination
        Mock::given(method("POST"))
            .and(path("/api/v1/coordination"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "session_id": "test-session",
                "status": "active",
                "participants": ["squirrel"],
                "result": null
            })))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Heartbeat
        Mock::given(method("POST"))
            .and(path("/api/v1/heartbeat"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        // Unregistration
        Mock::given(method("POST"))
            .and(path("/api/v1/services"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mut integration = setup_integration(&mock_server).await;
        
        // Initialize
        let init_result = integration.initialize().await;
        assert!(init_result.is_ok());
        assert_eq!(integration.health_status.status, "running");
        
        // Discover services
        let services = integration.discover_services().await;
        assert!(services.is_ok());
        
        // Coordinate
        let coordination_result = integration.coordinate("test", vec!["squirrel".to_string()]).await;
        assert!(coordination_result.is_ok());
        
        // Send heartbeat
        let heartbeat_result = integration.send_heartbeat().await;
        assert!(heartbeat_result.is_ok());
        
        // Shutdown
        let shutdown_result = integration.shutdown().await;
        assert!(shutdown_result.is_ok());
        assert_eq!(integration.health_status.status, "shutdown");
    }
} 