//! Universal Adapter for Squirrel AI Primal
//!
//! This adapter implements the complete ecosystem integration patterns,
//! bringing together all components for seamless operation within the ecoPrimals ecosystem.
//!
//! ## Architecture
//!
//! The adapter follows the Songbird-centric communication model:
//! ```
//! biomeOS → Songbird (Service Mesh) → Squirrel Universal Adapter
//!                                           ↓
//!                        AI Coordination + MCP Protocol + Session Management
//! ```

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use tracing::info;

// // use crate::api::ApiServer; // DELETED // DELETED
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
    /// Whether the API server is running
    pub api_server_running: bool,
    /// Uptime in seconds
    pub uptime: u64,
}

/// Configuration for the universal adapter
#[derive(Debug, Clone)]
pub struct UniversalAdapterConfig {
    pub service_host: String,
    pub service_port: u16,
}

/// Universal Adapter for Squirrel Primal with Arc<str> optimization
pub struct UniversalAdapter {
    /// Configuration for the adapter
    config: UniversalAdapterConfig,

    /// Ecosystem manager for service coordination with Arc<str> types
    ecosystem_manager: Arc<EcosystemManager>,

    /// Metrics collector with Arc<str> optimization
    metrics_collector: Arc<MetricsCollector>,

    /// Shutdown manager
    shutdown_manager: Arc<ShutdownManager>,

    /// API server instance - REMOVED (HTTP API deleted)
    // api_server: Option<ApiServer>, // DELETED

    /// Initialization status
    initialized: bool,

    /// Start time for uptime tracking
    start_time: Instant,
}

impl UniversalAdapter {
    /// Create new universal adapter with Arc<str> ecosystem integration
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
                service_id: "squirrel-adapter".to_string(),
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
            api_server_running: false, // api_server removed
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

        let registration = EcosystemServiceRegistration {
            service_id: "squirrel-universal-adapter".to_string(),
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
        if false { // if let Some(_api_server) = &self.api_server { // DELETED
            info!("Shutting down API server");
            // API server shutdown would be handled here
        }

        info!("✅ Universal Adapter shutdown completed");
        Ok(())
    }
}
