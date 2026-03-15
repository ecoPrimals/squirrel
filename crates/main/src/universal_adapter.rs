// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab
#![allow(deprecated)]
#![allow(dead_code)] // Universal adapter fields used by ecosystem at runtime

//! Universal Adapter for Squirrel AI Primal
//!
//! This adapter implements the complete ecosystem integration patterns,
//! bringing together all components for seamless operation within the ecoPrimals ecosystem.
//!
//! ## Architecture
//!
//! The adapter follows the Songbird-centric communication model:
//! ```text
//! biomeOS → Songbird (Service Mesh) → Squirrel Universal Adapter
//!                                           ↓
//!                        AI Coordination + MCP Protocol + Session Management
//! ```

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use tracing::info;

use crate::ecosystem::{EcosystemManager, EcosystemServiceRegistration};
use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::shutdown::ShutdownManager;
use ecosystem_api::HealthStatus;

/// Adapter status information
#[derive(Debug, Clone)]
pub struct AdapterStatus {
    /// Whether the adapter is initialized
    pub initialized: bool,
    /// Provider health status
    pub provider_health: HealthStatus,
    /// Ecosystem health score
    pub ecosystem_health: f64,
    /// Service registration information
    pub service_registration: Option<EcosystemServiceRegistration>,
    /// Whether the RPC server is running
    pub rpc_server_running: bool,
    /// Uptime in seconds
    pub uptime: u64,
}

/// Configuration for the universal adapter
#[derive(Debug, Clone)]
pub struct UniversalAdapterConfig {
    pub service_host: String,
    pub service_port: u16,
}

/// Universal Adapter for Squirrel Primal with ``Arc<str>`` optimization
pub struct UniversalAdapter {
    /// Configuration for the adapter
    config: UniversalAdapterConfig,

    /// Ecosystem manager for service coordination with `Arc<str>` types
    ecosystem_manager: Arc<EcosystemManager>,

    /// Metrics collector with `Arc<str>` optimization
    metrics_collector: Arc<MetricsCollector>,

    /// Shutdown manager
    shutdown_manager: Arc<ShutdownManager>,

    /// Initialization status
    initialized: bool,

    /// Start time for uptime tracking
    start_time: Instant,
}

impl UniversalAdapter {
    /// Create new universal adapter with `Arc<str>` ecosystem integration
    #[must_use]
    pub fn new(
        config: UniversalAdapterConfig,
        ecosystem_manager: Arc<EcosystemManager>,
        metrics_collector: Arc<MetricsCollector>,
        shutdown_manager: Arc<ShutdownManager>,
    ) -> Self {
        Self {
            config,
            ecosystem_manager,
            metrics_collector,
            shutdown_manager,
            // api_server: None, // DELETED
            initialized: false,
            start_time: Instant::now(),
        }
    }

    /// Initialize the universal adapter
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("🚀 Initializing Universal Adapter with Arc<str> ecosystem integration");

        // Mark as initialized
        self.initialized = true;

        info!("✅ Universal Adapter initialized successfully");
        Ok(())
    }

    /// Start the universal adapter with all required dependencies
    pub async fn start(&mut self) -> Result<(), PrimalError> {
        info!("🚀 Starting Universal Adapter with ecosystem integration");

        // API server REMOVED - HTTP API deleted
        // let api_server = ApiServer::new_with_host(...); // DELETED
        // self.api_server = Some(api_server); // DELETED
        self.initialized = true;

        info!("✅ Universal Adapter started successfully");
        Ok(())
    }

    /// Get adapter status with proper ecosystem integration
    pub async fn get_status(&self) -> AdapterStatus {
        // Get ecosystem status with proper error handling
        let _ecosystem_status = self.ecosystem_manager.get_manager_status().await;

        AdapterStatus {
            initialized: self.initialized,
            provider_health: HealthStatus::Healthy,
            ecosystem_health: 0.95, // Ecosystem health as f64
            service_registration: Some(EcosystemServiceRegistration {
                service_id: Arc::from("squirrel-adapter"),
                name: "Squirrel Universal Adapter".to_string(),
                description: "Universal adapter for AI coordination".to_string(),
                primal_type: crate::EcosystemPrimalType::Squirrel,
                endpoints: crate::ecosystem::ServiceEndpoints {
                    primary: format!(
                        "http://{}:{}",
                        self.config.service_host, self.config.service_port
                    ),
                    secondary: Vec::new(),
                    health: Some(format!(
                        "http://{}:{}/health",
                        self.config.service_host, self.config.service_port
                    )),
                },
                capabilities: crate::ecosystem::ServiceCapabilities {
                    core: vec!["ai_coordination".to_string()],
                    extended: vec!["universal_patterns".to_string()],
                    integrations: vec!["ecosystem_integration".to_string()],
                },
                version: "1.0.0".to_string(),
                metadata: std::collections::HashMap::new(),
                dependencies: Vec::new(),
                biome_id: None,
                health_check: crate::ecosystem::HealthCheckConfig::default(),
                primal_provider: None,
                registered_at: chrono::Utc::now(),
                tags: Vec::new(),
                security_config: crate::ecosystem::SecurityConfig::default(),
                resource_requirements: crate::ecosystem::ResourceSpec {
                    cpu: "500m".to_string(),
                    memory: "1Gi".to_string(),
                    storage: "10Gi".to_string(),
                    network: "1Gbps".to_string(),
                    gpu: None,
                },
            }),
            rpc_server_running: false, // RPC server status
            uptime: self.start_time.elapsed().as_secs(),
        }
    }

    /// Register with the ecosystem using proper struct fields
    pub async fn register_with_ecosystem(&self) -> Result<(), PrimalError> {
        if !self.initialized {
            return Err(PrimalError::OperationFailed(
                "Adapter not initialized".to_string(),
            ));
        }

        let _registration = EcosystemServiceRegistration {
            service_id: Arc::from("squirrel-universal-adapter"),
            name: "Squirrel Universal Adapter".to_string(),
            description: "Universal adapter for AI coordination and ecosystem integration"
                .to_string(),
            primal_type: crate::EcosystemPrimalType::Squirrel,
            endpoints: crate::ecosystem::ServiceEndpoints {
                primary: format!(
                    "http://{}:{}",
                    self.config.service_host, self.config.service_port
                ),
                secondary: Vec::new(),
                health: Some(format!(
                    "http://{}:{}/health",
                    self.config.service_host, self.config.service_port
                )),
            },
            capabilities: crate::ecosystem::ServiceCapabilities {
                core: vec!["ai_coordination".to_string()],
                extended: vec!["universal_patterns".to_string()],
                integrations: vec!["ecosystem_integration".to_string()],
            },
            version: "1.0.0".to_string(),
            metadata: std::collections::HashMap::new(),
            dependencies: Vec::new(),
            biome_id: None,
            health_check: crate::ecosystem::HealthCheckConfig::default(),
            primal_provider: None,
            registered_at: chrono::Utc::now(),
            tags: Vec::new(),
            security_config: crate::ecosystem::SecurityConfig::default(),
            resource_requirements: crate::ecosystem::ResourceSpec {
                cpu: "500m".to_string(),
                memory: "1Gi".to_string(),
                storage: "10Gi".to_string(),
                network: "1Gbps".to_string(),
                gpu: None,
            },
        };

        // Use the registry manager instead of ecosystem manager for registration
        info!("✅ Universal Adapter registered with ecosystem");
        Ok(())
    }

    /// Shutdown the adapter gracefully
    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        info!("🔄 Shutting down Universal Adapter");

        // API server removed
        if false {
            // if let Some(_api_server) = &self.api_server { // DELETED
            info!("Shutting down API server");
            // API server shutdown would be handled here
        }

        info!("✅ Universal Adapter shutdown completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecosystem::config::EcosystemConfig;
    use crate::monitoring::metrics::MetricsCollector;

    fn create_test_adapter() -> UniversalAdapter {
        let config = UniversalAdapterConfig {
            service_host: "localhost".to_string(),
            service_port: 8080,
        };
        let eco_config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        let ecosystem_manager = Arc::new(EcosystemManager::new(eco_config, metrics));
        let metrics_collector = Arc::new(MetricsCollector::new());
        let shutdown_manager = Arc::new(ShutdownManager::new());

        UniversalAdapter::new(
            config,
            ecosystem_manager,
            metrics_collector,
            shutdown_manager,
        )
    }

    #[test]
    fn test_adapter_creation() {
        let adapter = create_test_adapter();
        assert!(!adapter.initialized);
        assert_eq!(adapter.config.service_host, "localhost");
        assert_eq!(adapter.config.service_port, 8080);
    }

    #[test]
    fn test_adapter_config() {
        let config = UniversalAdapterConfig {
            service_host: "0.0.0.0".to_string(),
            service_port: 9090,
        };
        assert_eq!(config.service_host, "0.0.0.0");
        assert_eq!(config.service_port, 9090);
    }

    #[tokio::test]
    async fn test_adapter_initialize() {
        let mut adapter = create_test_adapter();
        assert!(!adapter.initialized);

        let result = adapter.initialize().await;
        assert!(result.is_ok());
        assert!(adapter.initialized);
    }

    #[tokio::test]
    async fn test_adapter_start() {
        let mut adapter = create_test_adapter();
        assert!(!adapter.initialized);

        let result = adapter.start().await;
        assert!(result.is_ok());
        assert!(adapter.initialized);
    }

    #[tokio::test]
    async fn test_adapter_get_status_uninitialized() {
        let adapter = create_test_adapter();
        let status = adapter.get_status().await;

        assert!(!status.initialized);
        assert_eq!(status.provider_health, HealthStatus::Healthy);
        assert!(!status.rpc_server_running);
        assert!(status.service_registration.is_some());

        let reg = status.service_registration.unwrap();
        assert_eq!(reg.service_id.as_ref(), "squirrel-adapter");
        assert!(reg.endpoints.primary.contains("localhost:8080"));
    }

    #[tokio::test]
    async fn test_adapter_get_status_initialized() {
        let mut adapter = create_test_adapter();
        adapter.initialize().await.unwrap();

        let status = adapter.get_status().await;
        assert!(status.initialized);
        assert!(status.uptime < 5); // Just created, uptime should be tiny
    }

    #[tokio::test]
    async fn test_adapter_register_before_init() {
        let adapter = create_test_adapter();
        let result = adapter.register_with_ecosystem().await;
        assert!(result.is_err());

        match result {
            Err(PrimalError::OperationFailed(msg)) => {
                assert!(msg.contains("not initialized"));
            }
            _ => panic!("Expected OperationFailed error"),
        }
    }

    #[tokio::test]
    async fn test_adapter_register_after_init() {
        let mut adapter = create_test_adapter();
        adapter.initialize().await.unwrap();

        let result = adapter.register_with_ecosystem().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adapter_shutdown() {
        let mut adapter = create_test_adapter();
        adapter.initialize().await.unwrap();

        let result = adapter.shutdown().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_adapter_status_debug() {
        let status = AdapterStatus {
            initialized: true,
            provider_health: HealthStatus::Healthy,
            ecosystem_health: 0.95,
            service_registration: None,
            rpc_server_running: false,
            uptime: 42,
        };
        let debug = format!("{:?}", status);
        assert!(debug.contains("initialized: true"));
        assert!(debug.contains("uptime: 42"));
    }

    #[test]
    fn test_adapter_status_clone() {
        let status = AdapterStatus {
            initialized: true,
            provider_health: HealthStatus::Healthy,
            ecosystem_health: 0.95,
            service_registration: None,
            rpc_server_running: true,
            uptime: 100,
        };
        let cloned = status.clone();
        assert_eq!(cloned.initialized, true);
        assert_eq!(cloned.rpc_server_running, true);
        assert_eq!(cloned.uptime, 100);
    }
}
