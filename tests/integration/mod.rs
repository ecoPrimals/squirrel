// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration Test Framework
//!
//! This module provides a comprehensive framework for testing real primal-to-primal
//! communication and system integration. Unlike the mock-based tests, these tests
//! verify actual integration between components.
//!
//! ## Test Categories
//!
//! 1. **Primal-to-Primal Communication**: Tests direct communication between primals
//! 2. **Service Discovery Integration**: Tests capability-based discovery
//! 3. **MCP Protocol Integration**: Tests full MCP message flows
//! 4. **Authentication Flow**: Tests end-to-end auth with BearDog
//! 5. **Configuration Loading**: Tests config precedence and validation
//! 6. **Error Recovery**: Tests resilience and error handling
//! 7. **Performance Integration**: Tests under load
//!
//! ## Usage
//!
//! ```rust,ignore
//! use integration::framework::TestEnvironment;
//!
//! #[tokio::test]
//! async fn test_squirrel_to_songbird() {
//!     let env = TestEnvironment::new().await;
//!     env.start_squirrel().await;
//!     env.start_songbird().await;
//!     
//!     // Test communication
//!     let result = env.squirrel_client()
//!         .send_to_songbird("test_message")
//!         .await;
//!     
//!     assert!(result.is_ok());
//! }
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub mod framework;
pub mod fixtures;
pub mod assertions;
pub mod helpers;
pub mod universal_transport_integration;

/// Integration test environment
#[derive(Debug)]
pub struct IntegrationTestEnvironment {
    /// Test name for identification
    pub test_name: String,
    /// Test configuration
    pub config: TestConfig,
    /// Running services
    pub services: Arc<RwLock<HashMap<String, ServiceHandle>>>,
    /// Test data directory
    pub data_dir: std::path::PathBuf,
}

/// Configuration for integration tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Timeout for test operations
    pub timeout: Duration,
    /// Enable verbose logging
    pub verbose: bool,
    /// Port range for test services
    pub port_range: (u16, u16),
    /// Enable chaos testing
    pub enable_chaos: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            verbose: std::env::var("TEST_VERBOSE").is_ok(),
            port_range: (50000, 60000),
            enable_chaos: false,
        }
    }
}

/// Handle to a running test service
#[derive(Debug)]
pub struct ServiceHandle {
    /// Service name
    pub name: String,
    /// Service type (squirrel, songbird, beardog, etc.)
    pub service_type: ServiceType,
    /// Base URL for the service
    pub base_url: String,
    /// Health check endpoint
    pub health_endpoint: String,
    /// Shutdown signal
    pub shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

/// Type of primal service
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceType {
    Squirrel,
    Songbird,
    BearDog,
    BiomeOS,
    ToadStool,
    NestGate,
}

impl IntegrationTestEnvironment {
    /// Create a new test environment
    pub async fn new(test_name: impl Into<String>) -> Self {
        let test_name = test_name.into();
        let data_dir = std::env::temp_dir().join("squirrel_integration_tests").join(&test_name);
        
        // Clean up old test data
        let _ = tokio::fs::remove_dir_all(&data_dir).await;
        tokio::fs::create_dir_all(&data_dir).await
            .expect("Failed to create test data directory");
        
        Self {
            test_name,
            config: TestConfig::default(),
            services: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
        }
    }
    
    /// Start a service in the test environment
    pub async fn start_service(&self, service_type: ServiceType) -> Result<String, TestError> {
        let service_id = format!("{:?}-{}", service_type, uuid::Uuid::new_v4());
        let port = self.allocate_port().await?;
        let base_url = format!("http://localhost:{}", port);
        
        let handle = ServiceHandle {
            name: service_id.clone(),
            service_type: service_type.clone(),
            base_url: base_url.clone(),
            health_endpoint: format!("{}/health", base_url),
            shutdown_tx: None,
        };
        
        self.services.write().await.insert(service_id.clone(), handle);
        
        Ok(service_id)
    }
    
    /// Stop a service
    pub async fn stop_service(&self, service_id: &str) -> Result<(), TestError> {
        if let Some(mut service) = self.services.write().await.remove(service_id) {
            if let Some(tx) = service.shutdown_tx.take() {
                let _ = tx.send(());
            }
            Ok(())
        } else {
            Err(TestError::ServiceNotFound(service_id.to_string()))
        }
    }
    
    /// Allocate an available port for a test service
    async fn allocate_port(&self) -> Result<u16, TestError> {
        for port in self.config.port_range.0..self.config.port_range.1 {
            if self.is_port_available(port).await {
                return Ok(port);
            }
        }
        Err(TestError::NoAvailablePort)
    }
    
    /// Check if a port is available
    async fn is_port_available(&self, port: u16) -> bool {
        tokio::net::TcpListener::bind(("127.0.0.1", port)).await.is_ok()
    }
    
    /// Wait for a service to be healthy
    pub async fn wait_for_service(&self, service_id: &str) -> Result<(), TestError> {
        // LEGITIMATE SLEEP: Creating timeout future for service readiness
        let timeout = tokio::time::sleep(self.config.timeout);
        tokio::pin!(timeout);
        
        let services = self.services.read().await;
        let service = services.get(service_id)
            .ok_or_else(|| TestError::ServiceNotFound(service_id.to_string()))?;
        
        let health_url = service.health_endpoint.clone();
        drop(services);
        
        loop {
            tokio::select! {
                _ = &mut timeout => {
                    return Err(TestError::Timeout(format!("Service {} did not become healthy", service_id)));
                }
                result = self.check_health(&health_url) => {
                    if result.is_ok() {
                        return Ok(());
                    }
                    // LEGITIMATE SLEEP: Polling interval for health check
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    /// Check service health
    async fn check_health(&self, url: &str) -> Result<(), TestError> {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| TestError::HealthCheckFailed(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(TestError::HealthCheckFailed(format!("Status: {}", response.status())))
        }
    }
    
    /// Clean up the test environment
    pub async fn cleanup(&self) -> Result<(), TestError> {
        // Stop all services
        let service_ids: Vec<String> = self.services.read().await.keys().cloned().collect();
        for service_id in service_ids {
            let _ = self.stop_service(&service_id).await;
        }
        
        // Clean up test data
        tokio::fs::remove_dir_all(&self.data_dir).await
            .map_err(|e| TestError::CleanupFailed(e.to_string()))?;
        
        Ok(())
    }
}

impl Drop for IntegrationTestEnvironment {
    fn drop(&mut self) {
        // Best effort cleanup
        let _ = std::fs::remove_dir_all(&self.data_dir);
    }
}

/// Integration test errors
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("No available port in range")]
    NoAvailablePort,
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    
    #[error("Cleanup failed: {0}")]
    CleanupFailed(String),
    
    #[error("Communication error: {0}")]
    CommunicationError(String),
    
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_environment_creation() {
        let env = IntegrationTestEnvironment::new("test_env").await;
        assert_eq!(env.test_name, "test_env");
        assert!(env.data_dir.exists());
        
        env.cleanup().await.expect("Cleanup failed");
        assert!(!env.data_dir.exists());
    }
    
    #[tokio::test]
    async fn test_port_allocation() {
        let env = IntegrationTestEnvironment::new("test_ports").await;
        let port = env.allocate_port().await.expect("Failed to allocate port");
        assert!(port >= env.config.port_range.0 && port < env.config.port_range.1);
        
        env.cleanup().await.expect("Cleanup failed");
    }
}

